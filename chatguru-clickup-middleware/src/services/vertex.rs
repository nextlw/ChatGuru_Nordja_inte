/// Vertex AI Service: Estruturas de dados conforme API oficial
///
/// FASE 1 CONCLUÍDA: Estruturas compatíveis com API Vertex AI
/// - VertexAIRequest, Content, Part, InlineData, GenerationConfig
/// - VertexAIResponse, Candidate, SafetyRating, UsageMetadata
/// - Métodos auxiliares e construtores
/// 
/// TODO Próximas fases:
/// - Fase 2: Autenticação (Google ADC + OAuth2)
/// - Fase 3: Processamento de mídia (download + base64)
/// - Fase 4: Cliente HTTP (chamadas à API)
/// - Fase 5: Service principal (integração completa)

use serde::{Deserialize, Serialize};

// ============================================================================
// ESTRUTURAS OFICIAIS DA API VERTEX AI (Fase 1 - ✅ Implementado)
// ============================================================================

/// Estrutura principal da requisição para Vertex AI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VertexAIRequest {
    pub contents: Vec<Content>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(rename = "safetySettings", skip_serializing_if = "Option::is_none")]
    pub safety_settings: Option<Vec<SafetySetting>>,
}

/// Conteúdo da mensagem (texto + mídia)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: String, // "user" ou "model"
    pub parts: Vec<Part>,
}

/// Parte do conteúdo: texto OU dados inline
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
    InlineData { 
        #[serde(rename = "inlineData")]
        inline_data: InlineData 
    },
}

/// Dados de mídia em base64
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String, // base64
}

/// Configurações de geração do modelo
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(rename = "topP", skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "topK", skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(rename = "candidateCount", skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<u32>,
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Configurações de segurança
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

/// Resposta completa da API Vertex AI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VertexAIResponse {
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata", skip_serializing_if = "Option::is_none")]
    pub usage_metadata: Option<UsageMetadata>,
}

/// Candidato de resposta
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason", skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    #[serde(rename = "safetyRatings", skip_serializing_if = "Option::is_none")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

/// Avaliação de segurança
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

/// Metadados de uso de tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: u32,
}

// ============================================================================
// IMPLEMENTAÇÕES DE HELPER METHODS (Fase 1 - ✅ Implementado)
// ============================================================================

impl VertexAIRequest {
    /// Cria requisição apenas com texto
    pub fn new_text_request(text: String, config: Option<GenerationConfig>) -> Self {
        Self {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part::Text { text }],
            }],
            generation_config: config,
            safety_settings: None,
        }
    }

    /// Cria requisição multimodal (texto + mídia)
    pub fn new_multimodal_request(
        text: String,
        mime_type: String,
        media_data: String,
        config: Option<GenerationConfig>,
    ) -> Self {
        Self {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![
                    Part::Text { text },
                    Part::InlineData {
                        inline_data: InlineData {
                            mime_type,
                            data: media_data,
                        },
                    },
                ],
            }],
            generation_config: config,
            safety_settings: None,
        }
    }
}

impl GenerationConfig {
    /// Configuração para análise de mídia
    pub fn default_media_analysis() -> Self {
        Self {
            temperature: Some(0.4),
            max_output_tokens: Some(1000),
            top_p: Some(0.8),
            top_k: Some(40),
            candidate_count: Some(1),
            stop_sequences: None,
        }
    }

    /// Configuração para classificação de texto
    pub fn default_text_classification() -> Self {
        Self {
            temperature: Some(0.2),
            max_output_tokens: Some(500),
            top_p: Some(0.9),
            top_k: Some(20),
            candidate_count: Some(1),
            stop_sequences: None,
        }
    }
}

// ============================================================================
// VERTEX AI SERVICE (Preparado para próximas fases)
// ============================================================================

/// Serviço Vertex AI para chamadas diretas à API
#[derive(Clone)]
pub struct VertexAIService {
    project_id: String,
    location: String,
    model_name: String,
    http_client: reqwest::Client,
}

impl VertexAIService {
    /// Construtor do service
    pub fn new(project_id: String, location: String) -> Self {
        Self {
            project_id,
            location,
            model_name: "gemini-1.5-pro-002".to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    // TODO: Phase 3 - Implementar download e conversão de mídia
    pub async fn process_media(
        &self,
        _media_url: &str,
        _prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implementar na Fase 3: Media Processing")
    }

    // TODO: Phase 4 - Implementar chamadas HTTP à API Vertex AI
    pub async fn generate_content(
        &self,
        _request: &VertexAIRequest,
    ) -> Result<VertexAIResponse, Box<dyn std::error::Error + Send + Sync>> {
        todo!("Implementar na Fase 4: HTTP Client")
    }

    // TODO: Phase 4 - URL builder para API Vertex AI
    pub fn build_api_url(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project_id, self.location, self.model_name
        )
    }

    // Método utilitário para validar tipos de mídia suportados
    pub fn is_supported_media_type(media_type: &str) -> bool {
        matches!(
            media_type,
            "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "video/mp4" | "video/avi" | "video/quicktime"
        )
    }
}

// ============================================================================
// TESTES ABRANGENTES (Fase 1 - ✅ Implementado)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_ai_request_creation() {
        let request = VertexAIRequest::new_text_request(
            "Analise esta imagem".to_string(),
            Some(GenerationConfig::default_media_analysis()),
        );

        assert_eq!(request.contents.len(), 1);
        assert_eq!(request.contents[0].role, "user");
        assert_eq!(request.contents[0].parts.len(), 1);

        if let Part::Text { text } = &request.contents[0].parts[0] {
            assert_eq!(text, "Analise esta imagem");
        } else {
            panic!("Expected text part");
        }
    }

    #[test]
    fn test_multimodal_request_creation() {
        let request = VertexAIRequest::new_multimodal_request(
            "Describa esta imagem".to_string(),
            "image/jpeg".to_string(),
            "base64data".to_string(),
            Some(GenerationConfig::default_media_analysis()),
        );

        assert_eq!(request.contents[0].parts.len(), 2);

        if let Part::Text { text } = &request.contents[0].parts[0] {
            assert_eq!(text, "Describa esta imagem");
        } else {
            panic!("Expected text part");
        }

        if let Part::InlineData { inline_data } = &request.contents[0].parts[1] {
            assert_eq!(inline_data.mime_type, "image/jpeg");
            assert_eq!(inline_data.data, "base64data");
        } else {
            panic!("Expected inline data part");
        }
    }

    #[test]
    fn test_generation_config_defaults() {
        let media_config = GenerationConfig::default_media_analysis();
        assert_eq!(media_config.temperature, Some(0.4));
        assert_eq!(media_config.max_output_tokens, Some(1000));

        let text_config = GenerationConfig::default_text_classification();
        assert_eq!(text_config.temperature, Some(0.2));
        assert_eq!(text_config.max_output_tokens, Some(500));
    }

    #[test]
    fn test_vertex_ai_response_serialization() {
        let json = r#"{
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [
                            {
                                "text": "Esta é uma resposta de teste"
                            }
                        ]
                    },
                    "finishReason": "STOP",
                    "safetyRatings": []
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 20,
                "totalTokenCount": 30
            }
        }"#;

        let _: VertexAIResponse = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_service_creation() {
        let service = VertexAIService::new(
            "test-project".to_string(),
            "us-central1".to_string(),
        );

        assert!(VertexAIService::is_supported_media_type("image/jpeg"));
        assert!(VertexAIService::is_supported_media_type("image/png"));
        assert!(VertexAIService::is_supported_media_type("video/mp4"));
        assert!(!VertexAIService::is_supported_media_type("text/plain"));
    }

    #[test]
    fn test_build_api_url() {
        let service = VertexAIService::new(
            "test-project".to_string(),
            "us-central1".to_string(),
        );

        let url = service.build_api_url();
        assert!(url.contains("aiplatform.googleapis.com"));
        assert!(url.contains("generateContent"));
        assert!(url.contains("test-project"));
        assert!(url.contains("us-central1"));
    }

    #[test]
    fn test_request_serialization_format() {
        let request = VertexAIRequest::new_text_request(
            "Test message".to_string(),
            Some(GenerationConfig::default_text_classification()),
        );

        let json = serde_json::to_string(&request).unwrap();
        
        // Verificar que usa camelCase nos campos
        assert!(json.contains("generationConfig"));
        assert!(json.contains("maxOutputTokens"));
        assert!(!json.contains("generation_config"));
        assert!(!json.contains("max_output_tokens"));
    }
}
