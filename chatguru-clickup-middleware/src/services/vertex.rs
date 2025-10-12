/// Vertex AI Service: Processamento assíncrono de mídia (áudio/imagem)
///
/// Arquitetura:
/// 1. Worker detecta media_url/media_type no payload
/// 2. Publica requisição em Pub/Sub topic "media-processing-requests"
/// 3. Cloud Function processa com Vertex AI (Gemini Pro)
/// 4. Resultado volta via "media-processing-results"
/// 5. Worker aguarda resultado com timeout (30s default)

use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use google_cloud_pubsub::client::{Client as PubSubClient, ClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;

/// Request para processamento de mídia
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaProcessingRequest {
    pub correlation_id: String,
    pub media_url: String,
    pub media_type: String,
    pub chat_id: Option<String>,
    pub timestamp: String,
}

/// Resultado do processamento
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaProcessingResult {
    pub correlation_id: String,
    pub result: String,
    pub media_type: String,
    pub error: Option<String>,
}

/// Serviço Vertex AI para processamento de mídia
#[derive(Clone)]
pub struct VertexAIService {
    client: Client,
    pubsub_client: Option<Arc<PubSubClient>>,
    project_id: String,
    location: String,
    topic_name: String,
}

impl VertexAIService {
    /// Cria nova instância do VertexAIService
    pub async fn new(project_id: String, topic_name: String) -> AppResult<Self> {
        log_info(&format!("Initializing Vertex AI Service for project: {}", project_id));

        // Configurar cliente Pub/Sub
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to configure Pub/Sub client: {}", e)))?;

        let pubsub_client = PubSubClient::new(config)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create Pub/Sub client: {}", e)))?;

        let location = std::env::var("VERTEX_AI_LOCATION")
            .unwrap_or_else(|_| "us-central1".to_string());

        log_info(&format!("Vertex AI Service configured for location: {}", location));

        Ok(Self {
            client: Client::new(),
            pubsub_client: Some(Arc::new(pubsub_client)),
            project_id,
            location,
            topic_name,
        })
    }

    /// Processa mídia de forma assíncrona (publica requisição no Pub/Sub)
    /// Retorna correlation_id para rastrear resultado
    pub async fn process_media_async(
        &self,
        media_url: &str,
        media_type: &str,
        chat_id: Option<String>,
    ) -> AppResult<String> {
        let correlation_id = Uuid::new_v4().to_string();

        log_info(&format!(
            "📤 Enviando requisição de processamento de mídia: {} (type: {})",
            correlation_id, media_type
        ));

        // Criar payload da requisição
        let request = MediaProcessingRequest {
            correlation_id: correlation_id.clone(),
            media_url: media_url.to_string(),
            media_type: media_type.to_string(),
            chat_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Serializar e publicar no Pub/Sub
        self.publish_request(&request).await?;

        log_info(&format!("✅ Requisição publicada: {}", correlation_id));

        Ok(correlation_id)
    }

    /// Publica requisição no tópico Pub/Sub
    async fn publish_request(&self, request: &MediaProcessingRequest) -> AppResult<()> {
        let pubsub_client = self.pubsub_client.as_ref()
            .ok_or_else(|| AppError::InternalError("Pub/Sub client not initialized".to_string()))?;

        let topic = pubsub_client.topic(&self.topic_name);

        // Verificar se tópico existe
        if !topic.exists(None).await
            .map_err(|e| AppError::InternalError(format!("Failed to check topic existence: {}", e)))? {
            return Err(AppError::InternalError(format!(
                "Topic '{}' does not exist",
                self.topic_name
            )));
        }

        // Criar publisher
        let publisher = topic.new_publisher(None);

        // Serializar mensagem
        let msg_bytes = serde_json::to_vec(&request)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize request: {}", e)))?;

        // Criar mensagem Pub/Sub
        let msg = PubsubMessage {
            data: msg_bytes.into(),
            ..Default::default()
        };

        // Publicar
        let awaiter = publisher.publish(msg).await;
        awaiter.get().await
            .map_err(|e| AppError::InternalError(format!("Failed to publish message: {}", e)))?;

        log_info(&format!("📨 Mensagem publicada no tópico '{}'", self.topic_name));

        Ok(())
    }

    /// Determina se o tipo de mídia é suportado
    pub fn is_supported_media_type(media_type: &str) -> bool {
        let mt = media_type.to_lowercase();
        mt.contains("audio")
            || mt.contains("voice")
            || mt.contains("image")
            || mt.contains("photo")
            || mt.contains("png")
            || mt.contains("jpg")
            || mt.contains("jpeg")
    }

    /// Retorna tipo de processamento (audio ou image)
    pub fn get_processing_type(media_type: &str) -> &str {
        let mt = media_type.to_lowercase();
        if mt.contains("audio") || mt.contains("voice") {
            "audio"
        } else if mt.contains("image") || mt.contains("photo") || mt.contains("png") || mt.contains("jpg") || mt.contains("jpeg") {
            "image"
        } else {
            "unknown"
        }
    }

    /// Obtém o project_id configurado
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Obtém a location configurada
    pub fn location(&self) -> &str {
        &self.location
    }

    /// Obtém o nome do tópico configurado
    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Obtém referência ao cliente HTTP
    pub fn http_client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_media_types() {
        assert!(VertexAIService::is_supported_media_type("audio/ogg"));
        assert!(VertexAIService::is_supported_media_type("audio/mpeg"));
        assert!(VertexAIService::is_supported_media_type("image/png"));
        assert!(VertexAIService::is_supported_media_type("image/jpeg"));
        assert!(!VertexAIService::is_supported_media_type("video/mp4"));
    }

    #[test]
    fn test_processing_type() {
        assert_eq!(VertexAIService::get_processing_type("audio/ogg"), "audio");
        assert_eq!(VertexAIService::get_processing_type("image/png"), "image");
        assert_eq!(VertexAIService::get_processing_type("video/mp4"), "unknown");
    }
}
