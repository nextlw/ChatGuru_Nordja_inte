/// Vertex AI Service: Integração completa com Google Cloud Vertex AI
///
/// IMPLEMENTAÇÃO COMPLETA:
/// ✅ FASE 1: Estruturas de dados compatíveis com API oficial
/// ✅ FASE 2: Autenticação OAuth2/ADC com cache de tokens
/// ✅ FASE 3: Processamento de mídia (download + base64)
/// 
/// TODO Próximas fases:
/// - Fase 4: Cliente HTTP (chamadas à API)
/// - Fase 5: Service principal (integração completa)

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, SystemTime};
use reqwest::header::HeaderMap;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use tracing::{info, debug};
use crate::utils::error::AppError;

// ==================== FASE 2: AUTENTICAÇÃO ====================

/// Token OAuth2 com cache e expiração
#[derive(Debug, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_at: SystemTime,
}

impl CachedToken {
    pub fn new(access_token: String, expires_in_seconds: u64) -> Self {
        let expires_at = SystemTime::now() + Duration::from_secs(expires_in_seconds.saturating_sub(60)); // 1 minuto de margem
        Self {
            access_token,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() >= self.expires_at
    }
}

/// Autenticador Vertex AI usando Google Cloud Metadata
#[derive(Debug, Clone)]
pub struct VertexAuthenticator {
    project_id: String,
    location: String,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
    http_client: reqwest::Client,
}

impl VertexAuthenticator {
    /// Cria novo autenticador usando Google Cloud Metadata Server
    pub async fn new_with_adc(project_id: String, location: String) -> Result<Self, AppError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| AppError::VertexError(format!("Falha criação cliente HTTP: {}", e)))?;

        debug!("Autenticador Vertex AI criado para projeto: {}, localização: {}", project_id, location);

        Ok(Self {
            project_id,
            location,
            cached_token: Arc::new(RwLock::new(None)),
            http_client,
        })
    }

    /// Obtém token válido (com cache automático)
    pub async fn get_access_token(&self) -> Result<String, AppError> {
        // Verifica cache
        {
            let cached = self.cached_token.read().await;
            if let Some(ref token) = *cached {
                if !token.is_expired() {
                    return Ok(token.access_token.clone());
                }
            }
        }

        // Cache expirado ou vazio - renovar token
        let new_token = self.refresh_token().await?;
        
        // Atualiza cache
        {
            let mut cached = self.cached_token.write().await;
            *cached = Some(new_token.clone());
        }

        Ok(new_token.access_token)
    }

    /// Renova token via Google Cloud Metadata Server
    async fn refresh_token(&self) -> Result<CachedToken, AppError> {
        debug!("Renovando token de acesso via Metadata Server");

        // URL do Google Cloud Metadata Server para obter token de acesso
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";
        
        let response = self.http_client
            .get(metadata_url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha consulta Metadata Server: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::VertexError(format!(
                "Metadata Server retornou HTTP {}",
                response.status()
            )));
        }

        let token_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha parsing token response: {}", e)))?;

        let access_token = token_response
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::VertexError("Token de acesso não encontrado na resposta".to_string()))?;

        let expires_in = token_response
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .unwrap_or(3600); // Padrão 1 hora se não especificado

        debug!("Token obtido com sucesso, expira em {} segundos", expires_in);

        Ok(CachedToken::new(access_token.to_string(), expires_in))
    }
    /// Headers HTTP com autenticação
    pub async fn auth_headers(&self) -> Result<HeaderMap, AppError> {
        let token = self.get_access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse()
                .map_err(|e| AppError::VertexError(format!("Header inválido: {}", e)))?
        );
        headers.insert(
            "Content-Type",
            "application/json".parse()
                .map_err(|e| AppError::VertexError(format!("Content-Type inválido: {}", e)))?
        );
        Ok(headers)
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn location(&self) -> &str {
        &self.location
    }
}

// ==================== FASE 3: PROCESSAMENTO DE MÍDIA ====================

/// Tipos MIME suportados pelo Vertex AI
#[derive(Debug, Clone, PartialEq)]
pub enum SupportedMimeType {
    // Imagens
    ImageJpeg,
    ImagePng,
    ImageWebp,
    ImageGif,
    
    // Vídeos
    VideoMp4,
    VideoQuicktime,
    VideoMpeg,
    VideoWebm,
    
    // Áudio
    AudioMp3,
    AudioMpeg,
    AudioWav,
    AudioAac,
    
    // Documentos
    ApplicationPdf,
}

impl SupportedMimeType {
    pub fn from_str(mime_str: &str) -> Option<Self> {
        match mime_str.to_lowercase().as_str() {
            "image/jpeg" | "image/jpg" => Some(Self::ImageJpeg),
            "image/png" => Some(Self::ImagePng),
            "image/webp" => Some(Self::ImageWebp),
            "image/gif" => Some(Self::ImageGif),
            "video/mp4" => Some(Self::VideoMp4),
            "video/quicktime" | "video/mov" => Some(Self::VideoQuicktime),
            "video/mpeg" => Some(Self::VideoMpeg),
            "video/webm" => Some(Self::VideoWebm),
            "audio/mp3" | "audio/mpeg" => Some(Self::AudioMp3),
            "audio/wav" => Some(Self::AudioWav),
            "audio/aac" => Some(Self::AudioAac),
            "application/pdf" => Some(Self::ApplicationPdf),
            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Self::ImageJpeg => "image/jpeg",
            Self::ImagePng => "image/png",
            Self::ImageWebp => "image/webp",
            Self::ImageGif => "image/gif",
            Self::VideoMp4 => "video/mp4",
            Self::VideoQuicktime => "video/quicktime",
            Self::VideoMpeg => "video/mpeg",
            Self::VideoWebm => "video/webm",
            Self::AudioMp3 | Self::AudioMpeg => "audio/mpeg",
            Self::AudioWav => "audio/wav",
            Self::AudioAac => "audio/aac",
            Self::ApplicationPdf => "application/pdf",
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(self, Self::ImageJpeg | Self::ImagePng | Self::ImageWebp | Self::ImageGif)
    }

    pub fn is_video(&self) -> bool {
        matches!(self, Self::VideoMp4 | Self::VideoQuicktime | Self::VideoMpeg | Self::VideoWebm)
    }

    pub fn is_audio(&self) -> bool {
        matches!(self, Self::AudioMp3 | Self::AudioMpeg | Self::AudioWav | Self::AudioAac)
    }

    pub fn is_document(&self) -> bool {
        matches!(self, Self::ApplicationPdf)
    }
}

/// Processador de mídia para Vertex AI
#[derive(Debug, Clone)]
pub struct MediaProcessor {
    http_client: reqwest::Client,
}

impl MediaProcessor {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ChatGuru-ClickUp-Middleware/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { http_client }
    }

    /// Download e conversão para Base64 com validação MIME
    pub async fn download_and_encode(&self, url: &str) -> Result<(String, SupportedMimeType), AppError> {
        // Download
        let response = self.http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha download: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::VertexError(format!(
                "HTTP {} ao baixar mídia: {}", 
                response.status(), 
                url
            )));
        }

        // Validação MIME
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");

        let mime_type = SupportedMimeType::from_str(content_type)
            .ok_or_else(|| AppError::VertexError(format!(
                "Tipo MIME não suportado: {}", 
                content_type
            )))?;

        // Download conteúdo
        let content_bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha leitura conteúdo: {}", e)))?;

        // Validação tamanho (10MB limite)
        if content_bytes.len() > 10 * 1024 * 1024 {
            return Err(AppError::VertexError(format!(
                "Arquivo muito grande: {} bytes (máximo 10MB)", 
                content_bytes.len()
            )));
        }

        // Conversão Base64
        let base64_data = STANDARD.encode(&content_bytes);

        Ok((base64_data, mime_type))
    }

    /// Cria Part para Vertex AI com mídia
    pub async fn create_media_part(&self, url: &str) -> Result<Part, AppError> {
        let (base64_data, mime_type) = self.download_and_encode(url).await?;
        
        Ok(Part::InlineData(InlineData {
            mime_type: mime_type.to_string().to_string(),
            data: base64_data,
        }))
    }

    /// Valida se URL é acessível
    pub async fn validate_url(&self, url: &str) -> Result<SupportedMimeType, AppError> {
        let response = self.http_client
            .head(url)
            .send()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha validação URL: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::VertexError(format!(
                "URL inacessível: HTTP {}", 
                response.status()
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("application/octet-stream");

        SupportedMimeType::from_str(content_type)
            .ok_or_else(|| AppError::VertexError(format!(
                "Tipo MIME não suportado: {}", 
                content_type
            )))
    }
}

// ==================== FASE 1: ESTRUTURAS DE DADOS ====================

/// Dados inline para mídia (imagem, vídeo, áudio, PDF)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String, // Base64
}

/// Parte do conteúdo (texto ou mídia)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text(String),
    InlineData(InlineData),
}

/// Conteúdo da mensagem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// Configuração de geração
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "topP")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "topK")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxOutputTokens")]
    pub max_output_tokens: Option<i32>,
}

/// Request para Vertex AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAIRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
}

/// Rating de segurança
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Metadados de uso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: i32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: i32,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: i32,
}

/// Candidato de resposta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: String,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Vec<SafetyRating>,
}

/// Response do Vertex AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAIResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: UsageMetadata,
}

// ==================== IMPLEMENTAÇÕES E MÉTODOS AUXILIARES ====================

impl VertexAIRequest {
    /// Cria request com texto simples
    pub fn new_text(prompt: &str) -> Self {
        Self {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part::Text(prompt.to_string())],
            }],
            generation_config: Some(GenerationConfig::default()),
        }
    }

    /// Cria request multimodal (texto + mídia)
    pub fn new_multimodal(prompt: &str, media_parts: Vec<Part>) -> Self {
        let mut parts = vec![Part::Text(prompt.to_string())];
        parts.extend(media_parts);

        Self {
            contents: vec![Content {
                role: "user".to_string(),
                parts,
            }],
            generation_config: Some(GenerationConfig::default()),
        }
    }

    /// Adiciona mídia ao request
    pub fn add_media(&mut self, media_part: Part) {
        if let Some(content) = self.contents.get_mut(0) {
            content.parts.push(media_part);
        }
    }
}

impl GenerationConfig {
    pub fn new() -> Self {
        Self {
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            max_output_tokens: Some(1024),
        }
    }

    pub fn conservative() -> Self {
        Self {
            temperature: Some(0.2),
            top_p: Some(0.8),
            top_k: Some(20),
            max_output_tokens: Some(1024),
        }
    }

    pub fn creative() -> Self {
        Self {
            temperature: Some(0.9),
            top_p: Some(0.95),
            top_k: Some(50),
            max_output_tokens: Some(2048),
        }
    }
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexAIResponse {
    /// Extrai texto da primeira resposta
    pub fn get_text(&self) -> Option<String> {
        self.candidates
            .first()?
            .content
            .parts
            .iter()
            .find_map(|part| match part {
                Part::Text(text) => Some(text.clone()),
                _ => None,
            })
    }

    /// Verifica se resposta foi bloqueada por segurança
    pub fn is_blocked(&self) -> bool {
        self.candidates
            .first()
            .map(|c| c.finish_reason == "SAFETY")
            .unwrap_or(false)
    }

    /// Obtém razão de finalização
    pub fn finish_reason(&self) -> Option<&str> {
        self.candidates
            .first()
            .map(|c| c.finish_reason.as_str())
    }
}

// ==================== FASE 4/5: SERVICE PRINCIPAL ====================

/// Service principal do Vertex AI
#[derive(Debug, Clone)]
pub struct VertexAIService {
    authenticator: VertexAuthenticator,
    media_processor: MediaProcessor,
    http_client: reqwest::Client,
    model_name: String,
}

impl VertexAIService {
    /// Cria novo service com ADC
    pub async fn new(project_id: String, location: String, model: Option<String>) -> Result<Self, AppError> {
        let authenticator = VertexAuthenticator::new_with_adc(project_id, location).await?;
        let media_processor = MediaProcessor::new();
        
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("ChatGuru-ClickUp-Middleware/1.0")
            .build()
            .map_err(|e| AppError::VertexError(format!("Falha criação cliente HTTP: {}", e)))?;

        let model_name = model.unwrap_or_else(|| "gemini-1.5-flash".to_string());

        Ok(Self {
            authenticator,
            media_processor,
            http_client,
            model_name,
        })
    }

    /// Processa texto simples
    pub async fn process_text(&self, prompt: &str) -> Result<String, AppError> {
        let request = VertexAIRequest::new_text(prompt);
        let response = self.send_request(request).await?;
        
        response.get_text()
            .ok_or_else(|| AppError::VertexError("Resposta vazia do Vertex AI".to_string()))
    }

    /// Processa texto + mídia
    pub async fn process_multimodal(&self, prompt: &str, media_urls: &[String]) -> Result<String, AppError> {
        // Processa mídia
        let mut media_parts = Vec::new();
        for url in media_urls {
            let media_part = self.media_processor.create_media_part(url).await?;
            media_parts.push(media_part);
        }

        // Cria request multimodal
        let request = VertexAIRequest::new_multimodal(prompt, media_parts);
        let response = self.send_request(request).await?;
        
        response.get_text()
            .ok_or_else(|| AppError::VertexError("Resposta vazia do Vertex AI".to_string()))
    }

    /// Valida URLs de mídia antes do processamento
    pub async fn validate_media_urls(&self, urls: &[String]) -> Result<Vec<SupportedMimeType>, AppError> {
        let mut mime_types = Vec::new();
        for url in urls {
            let mime_type = self.media_processor.validate_url(url).await?;
            mime_types.push(mime_type);
        }
        Ok(mime_types)
    }

    /// Envia request para API
    async fn send_request(&self, request: VertexAIRequest) -> Result<VertexAIResponse, AppError> {
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.authenticator.location(),
            self.authenticator.project_id(),
            self.authenticator.location(),
            self.model_name
        );

        let headers = self.authenticator.auth_headers().await?;
        
        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha envio request: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::VertexError(format!(
                "API retornou HTTP {}: {}", 
                status, 
                error_text
            )));
        }

        let vertex_response: VertexAIResponse = response
            .json()
            .await
            .map_err(|e| AppError::VertexError(format!("Falha parsing resposta: {}", e)))?;

        // Verifica se resposta foi bloqueada
        if vertex_response.is_blocked() {
            return Err(AppError::VertexError(
                "Resposta bloqueada por filtros de segurança".to_string()
            ));
        }

        Ok(vertex_response)
    }

    /// Teste de conectividade com Vertex AI
    pub async fn test_connection(&self) -> Result<(), AppError> {
        info!("Testando conectividade com Vertex AI");
        
        // Teste simples com um prompt mínimo
        let test_prompt = "Responda apenas 'OK' para confirmar conectividade.";
        let result = self.process_text(test_prompt).await?;
        
        info!("Teste de conectividade bem-sucedido. Resposta: {}", result);
        Ok(())
    }

    /// Obtém informações do modelo
    pub fn model_info(&self) -> (&str, &str, &str) {
        (
            &self.model_name,
            self.authenticator.project_id(),
            self.authenticator.location()
        )
    }
}

// ==================== TESTES UNITÁRIOS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_mime_type_detection() {
        assert_eq!(
            SupportedMimeType::from_str("image/jpeg"),
            Some(SupportedMimeType::ImageJpeg)
        );
        assert_eq!(
            SupportedMimeType::from_str("video/mp4"),
            Some(SupportedMimeType::VideoMp4)
        );
        assert_eq!(
            SupportedMimeType::from_str("application/unknown"),
            None
        );
    }

    #[test]
    fn test_mime_type_categories() {
        let image = SupportedMimeType::ImageJpeg;
        let video = SupportedMimeType::VideoMp4;
        let audio = SupportedMimeType::AudioMp3;
        let doc = SupportedMimeType::ApplicationPdf;

        assert!(image.is_image());
        assert!(!image.is_video());

        assert!(video.is_video());
        assert!(!video.is_audio());

        assert!(audio.is_audio());
        assert!(!audio.is_document());

        assert!(doc.is_document());
        assert!(!doc.is_image());
    }

    #[test]
    fn test_cached_token_expiration() {
        let token = CachedToken::new("test_token".to_string(), 0);
        // Token com 0 segundos deve estar expirado
        assert!(token.is_expired());

        let token = CachedToken::new("test_token".to_string(), 3600);
        // Token com 1 hora deve estar válido
        assert!(!token.is_expired());
    }

    #[test]
    fn test_vertex_request_creation() {
        let request = VertexAIRequest::new_text("Test prompt");
        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].role, "user");
        assert_eq!(request.contents[0].parts.len(), 1);
        
        match &request.contents[0].parts[0] {
            Part::Text(text) => assert_eq!(text, "Test prompt"),
            _ => panic!("Expected text part"),
        }
    }

    #[test]
    fn test_generation_config_presets() {
        let conservative = GenerationConfig::conservative();
        assert_eq!(conservative.temperature, Some(0.2));

        let creative = GenerationConfig::creative();
        assert_eq!(creative.temperature, Some(0.9));

        let default = GenerationConfig::default();
        assert_eq!(default.temperature, Some(0.7));
    }
}
