"""
Vertex AI Media Processor - Cloud Function

Arquitetura Híbrida com Fallback OpenAI:
1. Tenta processar com Vertex AI Gemini (primário)
2. Se Vertex AI falha → fallback para OpenAI Whisper/Vision
3. Publica resultado no Pub/Sub

Lógica:
1. Download da mídia da URL
2. Processamento com Vertex AI (transcrição de áudio ou descrição de imagem)
3. Se Vertex AI falha → OpenAI Whisper (áudio) ou Vision (imagem)
4. Publicação do resultado no Pub/Sub

Triggered by: Pub/Sub topic "media-processing-requests"
Publishes to: Pub/Sub topic "media-processing-results"
"""

import base64
import json
import requests
from google.cloud import pubsub_v1
from google.cloud import secretmanager
import vertexai
from vertexai.generative_models import GenerativeModel, Part
from openai import OpenAI
import logging
import os

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Configuration
PROJECT_ID = "buzzlightear"
LOCATION = "us-central1"

# Initialize Vertex AI
vertexai.init(project=PROJECT_ID, location=LOCATION)

# Initialize Pub/Sub publisher
publisher = pubsub_v1.PublisherClient()
RESULTS_TOPIC = f"projects/{PROJECT_ID}/topics/media-processing-results"

# OpenAI client (inicializado sob demanda com API key do Secret Manager)
_openai_client = None


def get_openai_client() -> OpenAI:
    """
    Retorna cliente OpenAI, buscando API key do Secret Manager
    Cache do cliente em variável global para reusar entre invocações
    """
    global _openai_client

    if _openai_client is not None:
        return _openai_client

    try:
        # Buscar OpenAI API key do Secret Manager
        client = secretmanager.SecretManagerServiceClient()
        secret_name = f"projects/{PROJECT_ID}/secrets/openai-api-key/versions/latest"

        logger.info(f"Fetching OpenAI API key from Secret Manager: {secret_name}")
        response = client.access_secret_version(request={"name": secret_name})
        api_key = response.payload.data.decode('UTF-8')

        # Inicializar cliente OpenAI
        _openai_client = OpenAI(api_key=api_key)
        logger.info("✅ OpenAI client initialized successfully")

        return _openai_client

    except Exception as e:
        logger.error(f"❌ Failed to initialize OpenAI client: {str(e)}")
        raise


def publish_result(result_data: dict):
    """
    Publica resultado no tópico Pub/Sub
    """
    try:
        message_json = json.dumps(result_data)
        message_bytes = message_json.encode('utf-8')

        future = publisher.publish(RESULTS_TOPIC, message_bytes)
        future.result()  # Wait for publish to complete

        logger.info(f"Result published to {RESULTS_TOPIC}")
    except Exception as e:
        logger.error(f"Error publishing result: {str(e)}")
        raise


def process_media(data, context):
    """
    Processa mensagem do Pub/Sub contendo mídia (áudio ou imagem)
    Lógica idêntica ao fallback OpenAI do worker Rust

    Args:
        data: The Pub/Sub message data (dict with 'data' field)
        context: The Pub/Sub message context
    """
    try:
        # Decode Pub/Sub message
        pubsub_message = base64.b64decode(data["data"]).decode('utf-8')
        payload = json.loads(pubsub_message)

        logger.info(f"Processing media: {payload.get('correlation_id')}")

        media_url = payload.get('media_url')
        media_type = payload.get('media_type', '')
        correlation_id = payload.get('correlation_id')

        if not media_url or not correlation_id:
            raise ValueError("Missing media_url or correlation_id")

        # Detectar tipo de processamento (áudio ou imagem)
        processing_type = 'audio' if 'audio' in media_type.lower() else 'image'

        logger.info(f"📎 Mídia detectada ({processing_type}: {media_type}), URL: {media_url}")

        # Download media (mesma lógica do OpenAI fallback no worker)
        logger.info(f"Downloading {processing_type} from: {media_url}")

        try:
            # Timeout de 30s para download (mesmo do fallback)
            media_response = requests.get(media_url, timeout=30)
            media_response.raise_for_status()
            media_bytes = media_response.content

            logger.info(f"Downloaded {len(media_bytes)} bytes")

        except Exception as e:
            error_msg = f"Failed to download media: {str(e)}"
            logger.error(f"❌ {error_msg}")

            # Publicar erro
            publish_result({
                'correlation_id': correlation_id,
                'result': "",
                'media_type': processing_type,
                'error': error_msg
            })
            return

        # Process based on media type (HYBRID: Vertex AI → OpenAI fallback)
        result_text = None
        error = None
        service_used = None

        # ESTRATÉGIA 1: Tentar Vertex AI primeiro
        try:
            if processing_type == 'audio':
                logger.info("🚀 Tentando transcrever áudio com Vertex AI Gemini")
                result_text = transcribe_audio_with_vertex(media_bytes, media_type, media_url)
                service_used = "vertex_ai"
                logger.info(f"✅ Transcrição Vertex AI concluída: {result_text}")
            else:
                logger.info("🚀 Tentando descrever imagem com Vertex AI Gemini")
                result_text = describe_image_with_vertex(media_bytes, media_type)
                service_used = "vertex_ai"
                logger.info(f"✅ Descrição Vertex AI concluída: {result_text}")

        except Exception as vertex_error:
            logger.warning(f"⚠️ Vertex AI falhou: {str(vertex_error)}")
            logger.info("🔄 Fazendo fallback para OpenAI...")

            # ESTRATÉGIA 2: Fallback para OpenAI se Vertex AI falha
            try:
                if processing_type == 'audio':
                    logger.info("🔄 Transcrevendo áudio com OpenAI Whisper (fallback)")
                    result_text = transcribe_audio_with_openai(media_bytes, media_type, media_url)
                    service_used = "openai_whisper"
                    logger.info(f"✅ Transcrição OpenAI Whisper concluída: {result_text}")
                else:
                    logger.info("🔄 Descrevendo imagem com OpenAI Vision (fallback)")
                    result_text = describe_image_with_openai(media_bytes, media_type)
                    service_used = "openai_vision"
                    logger.info(f"✅ Descrição OpenAI Vision concluída: {result_text}")

            except Exception as openai_error:
                # Ambos falharam - registrar erro
                error = f"Vertex AI: {str(vertex_error)} | OpenAI: {str(openai_error)}"
                logger.error(f"❌ FALHA TOTAL: Vertex AI e OpenAI falharam. {error}")

        # Publish result
        result_payload = {
            'correlation_id': correlation_id,
            'result': result_text or "",
            'media_type': processing_type,
            'service_used': service_used,  # "vertex_ai", "openai_whisper", "openai_vision", ou None
            'error': error
        }

        # Log do serviço usado
        if service_used:
            logger.info(f"📊 Mídia processada com sucesso usando: {service_used}")
        elif error:
            logger.error(f"❌ Nenhum serviço conseguiu processar a mídia")

        logger.info(f"Publishing result for: {correlation_id}")
        publish_result(result_payload)
        logger.info(f"✅ Result published successfully")

    except Exception as e:
        logger.error(f"Fatal error in process_media: {str(e)}", exc_info=True)


def transcribe_audio_with_vertex(audio_bytes: bytes, mime_type: str, media_url: str) -> str:
    """
    Transcreve áudio usando Vertex AI Gemini SDK
    Lógica idêntica ao OpenAI Whisper do fallback, mas usando Vertex AI

    Args:
        audio_bytes: Bytes do áudio (já baixado)
        mime_type: Tipo MIME original
        media_url: URL original (para detectar extensão)
    """
    try:
        # Detectar extensão do arquivo da URL (mesma lógica do fallback OpenAI)
        extension = media_url.split('.')[-1].split('?')[0].lower()

        # Normalizar mime_type baseado na extensão (mesma lógica do fallback)
        extension_map = {
            'ogg': 'audio/ogg',
            'oga': 'audio/ogg',
            'mp3': 'audio/mpeg',
            'mpeg': 'audio/mpeg',
            'wav': 'audio/wav',
            'webm': 'audio/webm',
            'm4a': 'audio/mp4',
            'mp4': 'audio/mp4'
        }

        normalized_mime = extension_map.get(extension)
        if not normalized_mime:
            # Se não encontrou extensão conhecida, tentar normalizar pelo mime_type original
            if 'ogg' in mime_type.lower():
                normalized_mime = "audio/ogg"
            elif 'mp3' in mime_type.lower() or 'mpeg' in mime_type.lower():
                normalized_mime = "audio/mpeg"
            elif 'wav' in mime_type.lower():
                normalized_mime = "audio/wav"
            elif 'webm' in mime_type.lower():
                normalized_mime = "audio/webm"
            else:
                normalized_mime = "audio/mpeg"  # default

        logger.info(f"Transcribing audio with Vertex AI Gemini (mime_type: {normalized_mime}, size: {len(audio_bytes)} bytes)")

        # Initialize model (Gemini 2.0 Flash suporta áudio e é o modelo estável atual)
        model = GenerativeModel("gemini-2.0-flash-001")

        # Create audio part
        audio_part = Part.from_data(data=audio_bytes, mime_type=normalized_mime)

        # Generate content (prompt igual ao usado no fallback OpenAI)
        response = model.generate_content(
            [
                "Transcreva este áudio em português do Brasil. Retorne apenas o texto transcrito, sem adicionar comentários ou explicações.",
                audio_part
            ],
            generation_config={
                "temperature": 0.1,
                "top_p": 0.95,
                "top_k": 40,
                "max_output_tokens": 2048,
            }
        )

        if not response.text:
            raise Exception("Empty transcription returned from Vertex AI")

        transcription = response.text.strip()
        logger.info(f"Audio transcription completed: {len(transcription)} characters")
        return transcription

    except Exception as e:
        logger.error(f"Error in transcribe_audio_with_vertex: {str(e)}")
        raise


def describe_image_with_vertex(image_bytes: bytes, mime_type: str) -> str:
    """
    Descreve imagem usando Vertex AI Gemini SDK
    Lógica idêntica ao OpenAI Vision do fallback, mas usando Vertex AI

    Args:
        image_bytes: Bytes da imagem (já baixada)
        mime_type: Tipo MIME original
    """
    try:
        # Normalizar mime_type (mesma lógica do fallback)
        if 'png' in mime_type.lower():
            normalized_mime = "image/png"
        elif 'jpeg' in mime_type.lower() or 'jpg' in mime_type.lower():
            normalized_mime = "image/jpeg"
        elif 'webp' in mime_type.lower():
            normalized_mime = "image/webp"
        else:
            normalized_mime = "image/jpeg"  # default

        logger.info(f"Describing image with Vertex AI Gemini (mime_type: {normalized_mime}, size: {len(image_bytes)} bytes)")

        # Initialize model (Gemini 2.0 Flash suporta imagem e é o modelo estável atual)
        model = GenerativeModel("gemini-2.0-flash-001")

        # Create image part
        image_part = Part.from_data(data=image_bytes, mime_type=normalized_mime)

        # Generate content (prompt idêntico ao usado no fallback OpenAI Vision)
        response = model.generate_content(
            [
                """Descreva detalhadamente esta imagem em português do Brasil. Foque em elementos relevantes para contexto de atendimento ao cliente ou solicitação de serviços. Inclua: o que está visível na imagem, texto que apareça na imagem (se houver), e contexto ou situação representada. Seja objetivo e claro.""",
                image_part
            ],
            generation_config={
                "temperature": 0.4,
                "top_p": 1.0,
                "top_k": 32,
                "max_output_tokens": 2048,
            }
        )

        if not response.text:
            raise Exception("Empty description returned from Vertex AI")

        description = response.text.strip()
        logger.info(f"Image description completed: {len(description)} characters")
        return description

    except Exception as e:
        logger.error(f"Error in describe_image_with_vertex: {str(e)}")
        raise


# ============================================================================
# OPENAI FALLBACK FUNCTIONS
# ============================================================================

def transcribe_audio_with_openai(audio_bytes: bytes, mime_type: str, media_url: str) -> str:
    """
    Transcreve áudio usando OpenAI Whisper API (fallback quando Vertex AI falha)
    Lógica idêntica ao src/services/openai.rs:173-219

    Args:
        audio_bytes: Bytes do áudio (já baixado)
        mime_type: Tipo MIME original
        media_url: URL original (para detectar extensão)
    """
    try:
        # Detectar extensão do arquivo da URL
        extension = media_url.split('.')[-1].split('?')[0].lower()

        # Normalizar mime_type baseado na extensão (mesma lógica do Rust)
        extension_map = {
            'ogg': 'audio/ogg',
            'oga': 'audio/ogg',
            'mp3': 'audio/mpeg',
            'mpeg': 'audio/mpeg',
            'wav': 'audio/wav',
            'webm': 'audio/webm',
            'm4a': 'audio/mp4',
            'mp4': 'audio/mp4'
        }

        normalized_mime = extension_map.get(extension, 'audio/mpeg')

        logger.info(f"Transcribing audio with OpenAI Whisper (mime_type: {normalized_mime}, size: {len(audio_bytes)} bytes)")

        # Criar arquivo temporário em memória (Whisper API precisa de file-like object)
        import io
        audio_file = io.BytesIO(audio_bytes)
        audio_file.name = f"audio.{extension}"

        # Chamar Whisper API
        client = get_openai_client()
        response = client.audio.transcriptions.create(
            model="whisper-1",
            file=audio_file,
            language="pt",
            response_format="text"
        )

        transcription = response.strip()
        logger.info(f"OpenAI Whisper transcription completed: {len(transcription)} characters")
        return transcription

    except Exception as e:
        logger.error(f"Error in transcribe_audio_with_openai: {str(e)}")
        raise


def describe_image_with_openai(image_bytes: bytes, mime_type: str) -> str:
    """
    Descreve imagem usando OpenAI Vision (GPT-4o-mini) - fallback quando Vertex AI falha
    Lógica idêntica ao src/services/openai.rs:302-362

    Args:
        image_bytes: Bytes da imagem (já baixada)
        mime_type: Tipo MIME original
    """
    try:
        # Normalizar mime_type (mesma lógica do Rust)
        if 'png' in mime_type.lower():
            normalized_mime = "image/png"
        elif 'jpeg' in mime_type.lower() or 'jpg' in mime_type.lower():
            normalized_mime = "image/jpeg"
        elif 'webp' in mime_type.lower():
            normalized_mime = "image/webp"
        else:
            normalized_mime = "image/jpeg"  # default

        logger.info(f"Describing image with OpenAI Vision (mime_type: {normalized_mime}, size: {len(image_bytes)} bytes)")

        # Converter imagem para base64 (OpenAI Vision precisa de base64)
        import base64
        image_base64 = base64.b64encode(image_bytes).decode('utf-8')

        # Chamar OpenAI Vision API
        client = get_openai_client()
        response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=[
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Descreva detalhadamente esta imagem em português do Brasil. Foque em elementos relevantes para contexto de atendimento ao cliente ou solicitação de serviços. Inclua: o que está visível na imagem, texto que apareça na imagem (se houver), e contexto ou situação representada. Seja objetivo e claro."
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": f"data:{normalized_mime};base64,{image_base64}"
                            }
                        }
                    ]
                }
            ],
            max_tokens=500
        )

        description = response.choices[0].message.content.strip()
        logger.info(f"OpenAI Vision description completed: {len(description)} characters")
        return description

    except Exception as e:
        logger.error(f"Error in describe_image_with_openai: {str(e)}")
        raise
