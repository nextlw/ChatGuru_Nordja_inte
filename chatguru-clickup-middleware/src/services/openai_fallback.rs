use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Serviço de fallback usando OpenAI API (como o sistema legado)
#[derive(Clone)]
pub struct OpenAIService {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIClassification {
    pub is_activity: bool,
    pub reason: String,
    pub confidence: f32,
    pub category: Option<String>,
}

impl OpenAIService {
    pub fn new(api_key: Option<String>) -> Option<Self> {
        // Usar API key da env var ou config
        let key = api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok())?;
        
        Some(Self {
            client: Client::new(),
            api_key: key,
        })
    }
    
    /// Classifica atividade usando OpenAI (fallback)
    pub async fn classify_activity_fallback(&self, context: &str) -> AppResult<OpenAIClassification> {
        log_info("Using OpenAI fallback for classification");
        
        let url = "https://api.openai.com/v1/chat/completions";
        
        // Prompt similar ao que o sistema legado usava
        let system_prompt = r#"
Você é um assistente que classifica mensagens de WhatsApp para identificar se representam atividades de trabalho válidas.

Atividades válidas incluem:
- Pedidos de produtos ou serviços
- Solicitações de orçamento
- Requisições de reparo ou manutenção
- Compras ou encomendas
- Qualquer solicitação que necessite ação da empresa

NÃO são atividades:
- Saudações (oi, olá, bom dia, etc.)
- Agradecimentos
- Confirmações simples (ok, certo, sim)
- Perguntas genéricas sem pedido específico
- Conversas casuais

Responda SEMPRE em JSON com este formato:
{
  "is_activity": true ou false,
  "reason": "explicação breve",
  "confidence": 0.0 a 1.0,
  "category": "pedido|orcamento|reparo|consulta|outro" (opcional)
}
"#;
        
        let user_prompt = format!("Classifique esta mensagem:\n{}", context);
        
        let request_body = json!({
            "model": "gpt-3.5-turbo", // Mais barato e rápido
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.1,
            "max_tokens": 150,
            "response_format": { "type": "json_object" }
        });
        
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let json_response: Value = response.json().await?;
            
            // Extrair resposta do modelo
            let content = json_response
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|c| c.as_str())
                .ok_or_else(|| AppError::InternalError("Invalid OpenAI response format".to_string()))?;
            
            // Parse do JSON retornado
            let classification: OpenAIClassification = serde_json::from_str(content)
                .map_err(|e| AppError::InternalError(format!("Failed to parse OpenAI response: {}", e)))?;
            
            log_info(&format!(
                "OpenAI classification: is_activity={}, confidence={}, reason={}", 
                classification.is_activity,
                classification.confidence,
                classification.reason
            ));
            
            Ok(classification)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("OpenAI API error: {}", error_text));
            Err(AppError::InternalError(format!("OpenAI API error: {}", error_text)))
        }
    }
}