"""
Vertex AI Media Processor - Cloud Function

Este Cloud Function replica a mesma lógica do fallback OpenAI no worker Rust,
mas usando Vertex AI Gemini SDK para processar áudio e imagem.

Lógica idêntica ao fallback:
1. Download da mídia da URL
2. Processamento com Vertex AI (transcrição de áudio ou descrição de imagem)
3. Publicação do resultado no Pub/Sub

Triggered by: Pub/Sub topic "media-processing-requests"
Publishes to: Pub/Sub topic "media-processing-results"
"""

import base64
import json
import requests
from google.cloud import pubsub_v1
import vertexai
from vertexai.generative_models import GenerativeModel, Part
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

        # Process based on media type (mesma lógica do fallback)
        result_text = None
        error = None

        try:
            if processing_type == 'audio':
                logger.info("🔄 Transcrevendo áudio com Vertex AI Gemini")
                result_text = transcribe_audio_with_vertex(media_bytes, media_type, media_url)
                logger.info(f"✅ Transcrição Vertex AI concluída: {result_text}")
            else:
                logger.info("🔄 Descrevendo imagem com Vertex AI Gemini")
                result_text = describe_image_with_vertex(media_bytes, media_type)
                logger.info(f"✅ Descrição Vertex AI concluída: {result_text}")

        except Exception as e:
            error = str(e)
            logger.error(f"❌ Erro ao processar mídia com Vertex AI: {error}")

        # Publish result
        result_payload = {
            'correlation_id': correlation_id,
            'result': result_text or "",
            'media_type': processing_type,
            'error': error
        }

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

        # Initialize model (Gemini 1.5 Flash suporta áudio)
        model = GenerativeModel("gemini-1.5-flash")

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

        # Initialize model (Gemini 1.5 Flash suporta imagem)
        model = GenerativeModel("gemini-1.5-flash")

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
