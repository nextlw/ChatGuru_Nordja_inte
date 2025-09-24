use crate::models::WebhookPayload;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Vertex AI endpoint para Gemini no Google Cloud
const VERTEX_AI_ENDPOINT: &str = "https://us-central1-aiplatform.googleapis.com/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityClassification {
    pub is_activity: bool,
    pub activity_type: Option<String>,
    pub category: Option<String>,
    pub subtasks: Vec<String>,
    pub priority: Option<String>,
    pub reason: String,
}

#[derive(Clone)]
pub struct VertexAIService {
    client: Client,
    project_id: String,
    access_token: Option<String>,
}

impl VertexAIService {
    /// Cria nova instância usando as credenciais padrão do Google Cloud
    /// Isso é mais eficiente pois usa as credenciais já configuradas no ambiente
    pub async fn new(project_id: String) -> AppResult<Self> {
        // No Google Cloud Run, as credenciais são automaticamente disponíveis
        let access_token = Self::get_access_token().await?;
        
        Ok(Self {
            client: Client::new(),
            project_id,
            access_token: Some(access_token),
        })
    }

    /// Obtém o access token usando as credenciais padrão do Google Cloud
    async fn get_access_token() -> AppResult<String> {
        // Usa a biblioteca google-cloud-auth para obter o token
        // Isso funciona automaticamente no Cloud Run com a service account padrão
        use google_cloud_auth::token::DefaultTokenSourceProvider;
        use google_cloud_auth::project::Config;
        use google_cloud_token::TokenSourceProvider;
        
        let config = Config::default()
            .with_scopes(&["https://www.googleapis.com/auth/cloud-platform"]);
        
        let provider = DefaultTokenSourceProvider::new(config).await
            .map_err(|e| AppError::ConfigError(format!("Failed to create token provider: {}", e)))?;
        
        let token_source = provider.token_source();
        
        let token = token_source.token().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get access token: {}", e)))?;
        
        Ok(token)
    }

    /// Classifica se o payload representa uma atividade válida usando Vertex AI
    pub async fn classify_activity(&self, payload: &WebhookPayload) -> AppResult<ActivityClassification> {
        let context = self.extract_context(payload);
        
        log_info(&format!("Classifying activity with Vertex AI for context: {}", context));
        
        // Usar Gemini 1.5 Flash através do Vertex AI (mais barato e rápido)
        let response = self.call_vertex_ai(&context).await?;
        let classification = self.parse_vertex_response(&response)?;
        
        log_info(&format!("Activity classification result: is_activity={}, reason={}", 
            classification.is_activity, classification.reason));
        
        Ok(classification)
    }

    fn extract_context(&self, payload: &WebhookPayload) -> String {
        match payload {
            WebhookPayload::ChatGuru(p) => {
                format!(
                    "Campanha: {}\nOrigem: {}\nNome: {}\nMensagem: {}\nTags: {:?}",
                    p.campanha_nome, p.origem, p.nome, p.texto_mensagem, p.tags
                )
            },
            WebhookPayload::EventType(p) => {
                format!(
                    "Tipo de Evento: {}\nDados: {:?}",
                    p.event_type, p.data
                )
            },
            WebhookPayload::Generic(p) => {
                format!(
                    "Nome: {:?}\nMensagem: {:?}\nDados extras: {:?}",
                    p.nome, p.mensagem, p.extra
                )
            }
        }
    }

    /// Chama o Vertex AI para classificação
    async fn call_vertex_ai(&self, context: &str) -> AppResult<Value> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| AppError::ConfigError("No access token available".to_string()))?;
        
        // Endpoint do Vertex AI para Gemini
        let url = format!(
            "{}/projects/{}/locations/us-central1/publishers/google/models/gemini-1.5-flash:predict",
            VERTEX_AI_ENDPOINT, self.project_id
        );
        
        // Prompt otimizado para classificação rápida
        let prompt = format!(
            r#"Classifique se esta mensagem representa uma atividade de trabalho válida.

MENSAGEM:
{}

Responda APENAS com JSON:
{{
    "is_activity": true/false,
    "reason": "explicação curta"
}}"#,
            context
        );
        
        let request_body = json!({
            "instances": [{
                "content": prompt
            }],
            "parameters": {
                "temperature": 0.1,  // Baixa temperatura para respostas consistentes
                "maxOutputTokens": 256,  // Menos tokens = mais rápido e barato
                "topP": 0.8,
                "topK": 10
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            let json_response = response.json::<Value>().await?;
            Ok(json_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("Vertex AI error: Status {} - {}", status, error_text));
            Err(AppError::InternalError(format!("Vertex AI error: {}", error_text)))
        }
    }

    fn parse_vertex_response(&self, response: &Value) -> AppResult<ActivityClassification> {
        // Estrutura de resposta do Vertex AI
        let predictions = response.get("predictions")
            .and_then(|p| p.get(0))
            .and_then(|pred| pred.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AppError::InternalError("Invalid Vertex AI response format".to_string()))?;

        // Parse do JSON na resposta
        let classification: Value = serde_json::from_str(predictions)
            .map_err(|e| AppError::InternalError(format!("Failed to parse classification: {}", e)))?;

        // Criar estrutura simplificada
        Ok(ActivityClassification {
            is_activity: classification.get("is_activity")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            activity_type: None,  // Simplificado para performance
            category: None,
            subtasks: Vec::new(),
            priority: None,
            reason: classification.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Sem motivo especificado")
                .to_string(),
        })
    }

    /// Constrói anotação para enviar de volta ao ChatGuru
    pub fn build_chatguru_annotation(&self, classification: &ActivityClassification) -> String {
        if classification.is_activity {
            format!("Tarefa: **Atividade Identificada** - {}", classification.reason)
        } else {
            format!("Tarefa: Não é uma atividade - {}", classification.reason)
        }
    }
}