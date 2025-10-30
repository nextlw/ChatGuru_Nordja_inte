//! Serviço de IA usando async-openai
//!
//! Este crate fornece uma interface unificada para serviços de IA da OpenAI:
//! - Classificação de atividades (GPT-4o-mini)
//! - Transcrição de áudio (Whisper)
//! - Descrição de imagens (Vision)
//! - Extração de texto de PDFs (GPT-4 Vision)
//! - Embeddings (text-embedding-3-small)
//!
//! Usa a biblioteca oficial async-openai para tipagem forte e manutenção simplificada.

use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioInput, AudioResponseFormat, ChatCompletionRequestMessage,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs, CreateTranscriptionRequestArgs,
        EmbeddingInput, ImageDetail, ImageUrl, ResponseFormat,
        ChatCompletionRequestUserMessageContentPart,
    },
    Client,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use lopdf::Document;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Resultado do processamento de mídia com anotação separada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaProcessingResult {
    /// Conteúdo extraído/transcrito (para classificação AI)
    pub extracted_content: String,
    /// Anotação formatada para enviar ao ChatGuru (opcional)
    pub annotation: Option<String>,
}

/// Erros do serviço de IA
#[derive(Debug)]
pub enum IaServiceError {
    OpenAIError(String),
    DownloadError(String),
    ParseError(String),
    ConfigError(String),
    PdfError(String),
}

impl fmt::Display for IaServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IaServiceError::OpenAIError(msg) => write!(f, "OpenAI error: {}", msg),
            IaServiceError::DownloadError(msg) => write!(f, "Download error: {}", msg),
            IaServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            IaServiceError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            IaServiceError::PdfError(msg) => write!(f, "PDF error: {}", msg),
        }
    }
}

impl Error for IaServiceError {}

pub type IaResult<T> = Result<T, IaServiceError>;

/// Classificação de atividades retornada pela IA
/// Mantém compatibilidade com OpenAIClassification do código legado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityClassification {
    pub is_activity: bool,
    pub reason: String,
    #[serde(default)]
    pub tipo_atividade: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub sub_categoria: Option<String>,
    #[serde(default)]
    pub subtasks: Vec<String>,
    #[serde(default)]
    pub status_back_office: Option<String>,
    #[serde(default)]
    pub campanha: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub space_name: Option<String>,
    #[serde(default)]
    pub folder_name: Option<String>,
    #[serde(default)]
    pub list_name: Option<String>,
    #[serde(default)]
    pub info_1: Option<String>,
    #[serde(default)]
    pub info_2: Option<String>,
}

/// Configuração do serviço de IA
#[derive(Clone)]
pub struct IaServiceConfig {
    /// API key da OpenAI
    pub api_key: String,
    /// Modelo para chat/classificação (padrão: gpt-4o-mini)
    pub chat_model: String,
    /// Modelo para Vision/PDFs (padrão: gpt-4o)
    pub vision_model: String,
    /// Modelo para embeddings (padrão: text-embedding-3-small)
    pub embedding_model: String,
    /// Temperatura para classificação (padrão: 0.1)
    pub temperature: f32,
    /// Max tokens para respostas (padrão: 500)
    pub max_tokens: u16,
    /// Timeout para downloads de mídia em segundos (padrão: 10)
    pub download_timeout_secs: u64,
}

impl IaServiceConfig {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            chat_model: "gpt-4o-mini".to_string(),
            vision_model: "gpt-4o".to_string(), // gpt-4o suporta Vision e PDFs
            embedding_model: "text-embedding-3-small".to_string(),
            temperature: 0.26,
            max_tokens: 500,
            download_timeout_secs: 10,
        }
    }

    pub fn with_chat_model(mut self, model: impl Into<String>) -> Self {
        self.chat_model = model.into();
        self
    }

    pub fn with_vision_model(mut self, model: impl Into<String>) -> Self {
        self.vision_model = model.into();
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_max_tokens(mut self, tokens: u16) -> Self {
        self.max_tokens = tokens;
        self
    }
}

/// Serviço principal de IA
#[derive(Clone)]
pub struct IaService {
    client: Client<OpenAIConfig>,
    config: IaServiceConfig,
    http_client: reqwest::Client,
}

impl IaService {
    /// Cria novo serviço de IA
    pub fn new(config: IaServiceConfig) -> IaResult<Self> {
        let openai_config = OpenAIConfig::new().with_api_key(&config.api_key);
        let client = Client::with_config(openai_config);

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.download_timeout_secs))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .map_err(|e| IaServiceError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        tracing::info!("✅ IaService inicializado com modelo: {}", config.chat_model);

        Ok(Self {
            client,
            config,
            http_client,
        })
    }

    /// Classifica atividade usando prompt estruturado
    ///
    /// O prompt deve ser pré-formatado externamente (usando AiPromptConfig)
    /// e já conter todas as instruções necessárias
    pub async fn classify_activity(&self, prompt: &str) -> IaResult<ActivityClassification> {
        tracing::info!("🔍 Iniciando classificação de atividade");

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.chat_model)
            .messages(vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()
                    .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build message: {}", e)))?,
            )])
            .temperature(self.config.temperature)
            .max_tokens(self.config.max_tokens)
            .response_format(ResponseFormat::JsonObject)
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| IaServiceError::OpenAIError(format!("API call failed: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| IaServiceError::ParseError("No content in response".to_string()))?;

        tracing::debug!("📋 Response JSON: {}", content);

        let classification: ActivityClassification = serde_json::from_str(content).map_err(|e| {
            IaServiceError::ParseError(format!("Failed to parse JSON: {}. Content: {}", e, content))
        })?;

        tracing::info!(
            "✅ Classificação: is_activity={}, category={:?}, subcategory={:?}",
            classification.is_activity,
            classification.category,
            classification.sub_categoria
        );

        Ok(classification)
    }

    /// Transcreve áudio usando Whisper
    ///
    /// # Argumentos
    /// * `audio_bytes` - Bytes do arquivo de áudio
    /// * `filename` - Nome do arquivo com extensão (ex: "audio.ogg", "recording.mp3")
    pub async fn transcribe_audio(&self, audio_bytes: &[u8], filename: &str) -> IaResult<String> {
        tracing::info!("🎤 Transcrevendo áudio com Whisper: {}", filename);

        // AudioInput para async-openai v0.27 usa from_vec_u8
        let audio_input = AudioInput::from_vec_u8(filename.to_string(), audio_bytes.to_vec());

        let request = CreateTranscriptionRequestArgs::default()
            .file(audio_input)
            .model("whisper-1")
            .language("pt")
            .response_format(AudioResponseFormat::Json)
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build transcription request: {}", e)))?;

        let response = self
            .client
            .audio()
            .transcribe(request)
            .await
            .map_err(|e| IaServiceError::OpenAIError(format!("Transcription failed: {}", e)))?;

        tracing::info!("✅ Transcrição completada: {} chars", response.text.len());

        Ok(response.text)
    }

    /// Descreve imagem usando Vision (GPT-4o-mini)
    ///
    /// # Argumentos
    /// * `image_bytes` - Bytes da imagem (JPEG, PNG, WebP, GIF)
    pub async fn describe_image(&self, image_bytes: &[u8]) -> IaResult<String> {
        tracing::info!("🖼️ Descrevendo imagem com Vision");

        let image_base64 = STANDARD.encode(image_bytes);
        let data_url = format!("data:image/jpeg;base64,{}", image_base64);

        // Para async-openai v0.27, construir mensagem multimodal
        use async_openai::types::{ChatCompletionRequestMessageContentPartText, ChatCompletionRequestMessageContentPartImage};

        let text_part = ChatCompletionRequestUserMessageContentPart::Text(
            ChatCompletionRequestMessageContentPartText {
                text: "Descreva detalhadamente esta imagem em português do Brasil. Foque em elementos relevantes para contexto de atendimento ao cliente ou solicitação de serviços. Inclua: o que está visível na imagem, texto que apareça na imagem (se houver), e contexto ou situação representada. Seja objetivo e claro.".to_string(),
            }
        );

        let image_part = ChatCompletionRequestUserMessageContentPart::ImageUrl(
            ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl {
                    url: data_url,
                    detail: Some(ImageDetail::Auto),
                },
            }
        );

        let message = ChatCompletionRequestUserMessageArgs::default()
            .content(ChatCompletionRequestUserMessageContent::Array(vec![text_part, image_part]))
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build vision message: {}", e)))?;

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.chat_model)
            .messages(vec![ChatCompletionRequestMessage::User(message)])
            .max_tokens(self.config.max_tokens)
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build vision request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| IaServiceError::OpenAIError(format!("Vision API call failed: {}", e)))?;

        let description = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| IaServiceError::ParseError("No description in response".to_string()))?
            .to_string();

        tracing::info!("✅ Descrição da imagem: {} chars", description.len());

        Ok(description)
    }

    /// Processa imagem com anotação separada (retorna descrição para classificação + anotação)
    ///
    /// # Argumentos
    /// * `image_bytes` - Bytes da imagem
    ///
    /// # Retorna
    /// `MediaProcessingResult` com descrição e anotação formatada
    pub async fn process_image_with_annotation(&self, image_bytes: &[u8]) -> IaResult<MediaProcessingResult> {
        // Gera descrição da imagem
        let description = self.describe_image(image_bytes).await?;

        // Formata anotação para ChatGuru
        let annotation = format!(
            "🖼️ **Imagem Processada**\n\n\
            {}\n\n\
            ℹ️ A descrição da imagem foi gerada e será usada para classificação da atividade.",
            description
        );

        Ok(MediaProcessingResult {
            extracted_content: description.clone(),
            annotation: Some(annotation),
        })
    }

    /// Processa áudio com anotação separada (retorna transcrição para classificação + anotação)
    ///
    /// # Argumentos
    /// * `audio_bytes` - Bytes do áudio
    /// * `filename` - Nome do arquivo (para extensão)
    ///
    /// # Retorna
    /// `MediaProcessingResult` com transcrição e anotação formatada
    pub async fn process_audio_with_annotation(&self, audio_bytes: &[u8], filename: &str) -> IaResult<MediaProcessingResult> {
        // Transcreve áudio
        let transcription = self.transcribe_audio(audio_bytes, filename).await?;

        // Formata anotação para ChatGuru
        let annotation = format!(
            "🎵 **Áudio Transcrito**\n\n\
            \"{}\"\n\n\
            ℹ️ A transcrição do áudio foi gerada e será usada para classificação da atividade.",
            transcription
        );

        Ok(MediaProcessingResult {
            extracted_content: transcription,
            annotation: Some(annotation),
        })
    }

    /// Gera embeddings para texto
    ///
    /// # Argumentos
    /// * `text` - Texto para gerar embedding (max 8191 tokens)
    pub async fn get_embedding(&self, text: &str) -> IaResult<Vec<f32>> {
        tracing::info!("📊 Gerando embedding");

        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.config.embedding_model)
            .input(EmbeddingInput::String(text.to_string()))
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build embedding request: {}", e)))?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| IaServiceError::OpenAIError(format!("Embedding API call failed: {}", e)))?;

        let embedding = response
            .data
            .first()
            .ok_or_else(|| IaServiceError::ParseError("No embedding in response".to_string()))?
            .embedding
            .clone();

        tracing::info!("✅ Embedding gerado: {} dimensões", embedding.len());

        Ok(embedding)
    }

    /// Baixa áudio de uma URL
    ///
    /// # Argumentos
    /// * `url` - URL do arquivo de áudio
    pub async fn download_audio(&self, url: &str) -> IaResult<Vec<u8>> {
        tracing::info!("⬇️ Baixando áudio de: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IaServiceError::DownloadError(format!(
                "HTTP {} while downloading audio",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Failed to read bytes: {}", e)))?
            .to_vec();

        tracing::info!("✅ Áudio baixado: {} bytes", bytes.len());

        Ok(bytes)
    }

    /// Baixa imagem de uma URL
    ///
    /// # Argumentos
    /// * `url` - URL da imagem
    pub async fn download_image(&self, url: &str) -> IaResult<Vec<u8>> {
        tracing::info!("⬇️ Baixando imagem de: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IaServiceError::DownloadError(format!(
                "HTTP {} while downloading image",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Failed to read bytes: {}", e)))?
            .to_vec();

        tracing::info!("✅ Imagem baixada: {} bytes", bytes.len());

        Ok(bytes)
    }

    /// Extrai texto de PDF usando lopdf (processamento local)
    ///
    /// # Argumentos
    /// * `pdf_bytes` - Bytes do arquivo PDF
    fn extract_pdf_text_local(pdf_bytes: &[u8]) -> IaResult<String> {
        tracing::info!("📄 Extraindo texto do PDF localmente: {} bytes", pdf_bytes.len());

        // Carregar PDF com lopdf
        let document = Document::load_mem(pdf_bytes)
            .map_err(|e| IaServiceError::PdfError(format!("Failed to load PDF: {}", e)))?;

        let mut extracted_text = String::new();
        let pages = document.get_pages();

        // Iterar pelas páginas e extrair texto
        for (page_num, _page_id) in pages.iter() {
            if let Ok(text) = document.extract_text(&[*page_num]) {
                if !extracted_text.is_empty() {
                    extracted_text.push_str("\n\n");
                }
                extracted_text.push_str(&format!("--- Página {} ---\n", page_num));
                extracted_text.push_str(&text);
            }
        }

        if extracted_text.is_empty() {
            tracing::warn!("⚠️ Nenhum texto extraído do PDF (pode ser PDF de imagens/escaneado)");
            return Err(IaServiceError::PdfError(
                "PDF não contém texto extraível (pode ser PDF escaneado/imagem)".to_string()
            ));
        }

        tracing::info!("✅ Texto extraído do PDF: {} caracteres", extracted_text.len());

        Ok(extracted_text)
    }

    /// Processa PDF extraindo texto localmente e analisando com GPT-4
    ///
    /// # Argumentos
    /// * `pdf_bytes` - Bytes do arquivo PDF
    ///
    /// # Retorna
    /// Texto extraído do PDF (apenas o texto bruto para classificação)
    pub async fn process_pdf(&self, pdf_bytes: &[u8]) -> IaResult<String> {
        // Extrai texto localmente
        let extracted_text = Self::extract_pdf_text_local(pdf_bytes)?;

        tracing::info!("✅ PDF processado: {} caracteres extraídos", extracted_text.len());

        Ok(extracted_text)
    }

    /// Descreve PDF usando extração local + GPT-4 (descrição resumida para anotações)
    ///
    /// # Argumentos
    /// * `pdf_bytes` - Bytes do arquivo PDF
    ///
    /// # Retorna
    /// Descrição resumida do conteúdo (para enviar como anotação ao ChatGuru)
    pub async fn describe_pdf(&self, pdf_bytes: &[u8]) -> IaResult<String> {
        // Extrai texto localmente
        let extracted_text = Self::extract_pdf_text_local(pdf_bytes)?;

        tracing::info!("📄 Gerando descrição do PDF com GPT-4: {} caracteres extraídos", extracted_text.len());

        // Truncar texto se for muito longo (GPT-4 tem limite de tokens)
        let text_for_analysis = if extracted_text.len() > 8000 {
            format!("{}...\n\n[Texto truncado por tamanho]", &extracted_text[..8000])
        } else {
            extracted_text.clone()
        };

        // Gerar descrição resumida com GPT-4
        let prompt = format!(
            "Analise o seguinte texto extraído de um documento PDF e crie uma descrição resumida em português do Brasil.\n\n\
            Foque em:\n\
            - Tipo de documento (cotação, nota fiscal, contrato, relatório, etc.)\n\
            - Assunto principal\n\
            - Informações mais relevantes (valores, datas, nomes importantes)\n\n\
            Seja conciso (máximo 4 frases).\n\n\
            TEXTO DO PDF:\n{}\n\n\
            DESCRIÇÃO:",
            text_for_analysis
        );

        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(ChatCompletionRequestUserMessageContent::Text(prompt))
                .build()
                .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build message: {}", e)))?
        )];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.chat_model) // Usa chat_model (gpt-4o-mini) para análise
            .messages(messages)
            .temperature(0.3)
            .max_tokens(400u32)
            .build()
            .map_err(|e| IaServiceError::OpenAIError(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| IaServiceError::OpenAIError(format!("PDF description failed: {}", e)))?;

        let description = response
            .choices
            .get(0)
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| IaServiceError::OpenAIError("No description in response".to_string()))?
            .clone();

        tracing::info!("✅ PDF descrito: {} caracteres", description.len());

        Ok(description)
    }

    /// Processa PDF com anotação separada (retorna texto extraído + descrição para anotação)
    ///
    /// # Argumentos
    /// * `pdf_bytes` - Bytes do arquivo PDF
    ///
    /// # Retorna
    /// `MediaProcessingResult` com texto extraído e anotação formatada
    pub async fn process_pdf_with_annotation(&self, pdf_bytes: &[u8]) -> IaResult<MediaProcessingResult> {
        // Extrai texto localmente
        let extracted_text = Self::extract_pdf_text_local(pdf_bytes)?;

        // Gera descrição para anotação
        let description = self.describe_pdf(pdf_bytes).await?;

        // Formata anotação para ChatGuru
        let annotation = format!(
            "📄 **Documento PDF Processado**\n\n\
            {}\n\n\
            ℹ️ O texto completo foi extraído e será usado para classificação da atividade.",
            description
        );

        Ok(MediaProcessingResult {
            extracted_content: extracted_text,
            annotation: Some(annotation),
        })
    }

    /// Processa mídia (áudio, imagem ou PDF) automaticamente
    ///
    /// # Argumentos
    /// * `media_url` - URL da mídia
    /// * `media_type` - Tipo MIME (ex: "audio/ogg", "image/jpeg", "application/pdf")
    pub async fn process_media(&self, media_url: &str, media_type: &str) -> IaResult<String> {
        tracing::info!("📎 Processando mídia: {} ({})", media_url, media_type);

        if media_type.contains("audio") {
            let audio_bytes = self.download_audio(media_url).await?;
            let extension = media_url
                .split('.')
                .last()
                .and_then(|ext| ext.split('?').next())
                .unwrap_or("ogg");
            let filename = format!("audio.{}", extension);
            self.transcribe_audio(&audio_bytes, &filename).await
        } else if media_type.contains("image") {
            let image_bytes = self.download_image(media_url).await?;
            self.describe_image(&image_bytes).await
        } else if media_type.contains("pdf") || media_type.contains("application/pdf") {
            let pdf_bytes = self.download_file(media_url, "PDF").await?;
            self.process_pdf(&pdf_bytes).await
        } else {
            Err(IaServiceError::ConfigError(format!(
                "Tipo de mídia não suportado: {}",
                media_type
            )))
        }
    }

    /// Baixa arquivo genérico de uma URL
    ///
    /// # Argumentos
    /// * `url` - URL do arquivo
    /// * `file_type` - Tipo do arquivo para log (ex: "PDF", "Áudio")
    pub async fn download_file(&self, url: &str, file_type: &str) -> IaResult<Vec<u8>> {
        tracing::info!("⬇️ Baixando {} de: {}", file_type, url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Download failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IaServiceError::DownloadError(format!(
                "HTTP {} while downloading {}",
                response.status(),
                file_type
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| IaServiceError::DownloadError(format!("Failed to read bytes: {}", e)))?
            .to_vec();

        tracing::info!("✅ {} baixado: {} bytes", file_type, bytes.len());

        Ok(bytes)
    }

    /// Obtém informações sobre a configuração atual
    pub fn get_config(&self) -> &IaServiceConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = IaServiceConfig::new("test-key".to_string())
            .with_chat_model("gpt-4")
            .with_temperature(0.5)
            .with_max_tokens(1000);

        assert_eq!(config.chat_model, "gpt-4");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 1000);
        assert_eq!(config.api_key, "test-key");
    }

    #[test]
    fn test_classification_serialization() {
        let classification = ActivityClassification {
            is_activity: true,
            reason: "Teste".to_string(),
            tipo_atividade: Some("Rotineira".to_string()),
            category: Some("Logística".to_string()),
            sub_categoria: Some("Corrida de motoboy".to_string()),
            subtasks: vec![],
            status_back_office: Some("Executar".to_string()),
            campanha: None,
            description: Some("Descrição teste".to_string()),
            space_name: None,
            folder_name: None,
            list_name: None,
            info_1: None,
            info_2: None,
        };

        let json = serde_json::to_string(&classification).unwrap();
        let deserialized: ActivityClassification = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.is_activity, true);
        assert_eq!(deserialized.category, Some("Logística".to_string()));
    }

    #[test]
    fn test_classification_deserialization_with_defaults() {
        // Testa que campos opcionais tem defaults corretos
        let json = r#"{"is_activity": true, "reason": "Test"}"#;
        let classification: ActivityClassification = serde_json::from_str(json).unwrap();

        assert_eq!(classification.is_activity, true);
        assert_eq!(classification.reason, "Test");
        assert_eq!(classification.subtasks, Vec::<String>::new());
        assert_eq!(classification.category, None);
    }
}
