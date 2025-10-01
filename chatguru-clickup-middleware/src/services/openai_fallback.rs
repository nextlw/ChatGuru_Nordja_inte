use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::services::secret_manager::SecretManagerService;
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
    /// Cria uma nova instância do OpenAIService
    /// Busca a API key através do SecretManagerService
    pub async fn new(api_key: Option<String>) -> Option<Self> {
        let key = if let Some(k) = api_key {
            k
        } else {
            // Buscar através do SecretManagerService
            match SecretManagerService::new().await {
                Ok(secret_mgr) => {
                    match secret_mgr.get_openai_api_key().await {
                        Ok(k) => k,
                        Err(e) => {
                            log_error(&format!("Failed to get OpenAI API key: {}", e));
                            return None;
                        }
                    }
                }
                Err(e) => {
                    log_error(&format!("Failed to initialize SecretManagerService: {}", e));
                    return None;
                }
            }
        };

        log_info("OpenAI service initialized successfully");

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

    /// Transcreve áudio usando OpenAI Whisper API
    pub async fn transcribe_audio(&self, audio_bytes: &[u8]) -> AppResult<String> {
        log_info("Using OpenAI Whisper for audio transcription");

        let url = "https://api.openai.com/v1/audio/transcriptions";

        // Criar multipart form com o arquivo de áudio
        let part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
            .file_name("audio.mp3")
            .mime_str("audio/mpeg")
            .map_err(|e| AppError::InternalError(format!("Failed to create audio part: {}", e)))?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", "whisper-1")
            .text("language", "pt")
            .text("response_format", "text");

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if response.status().is_success() {
            let transcription = response.text().await?;
            log_info(&format!("Whisper transcription completed: {} chars", transcription.len()));
            Ok(transcription)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("Whisper API error: {}", error_text));
            Err(AppError::InternalError(format!("Whisper API error: {}", error_text)))
        }
    }

    /// Gera embeddings para um texto usando OpenAI
    pub async fn get_embedding(&self, text: &str) -> AppResult<Vec<f32>> {
        log_info("Generating embedding with OpenAI");

        let url = "https://api.openai.com/v1/embeddings";

        let request_body = json!({
            "model": "text-embedding-3-small", // Modelo mais barato e rápido
            "input": text,
            "encoding_format": "float"
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

            let embedding = json_response
                .get("data")
                .and_then(|d| d.get(0))
                .and_then(|item| item.get("embedding"))
                .and_then(|e| e.as_array())
                .ok_or_else(|| AppError::InternalError("Invalid embedding response format".to_string()))?
                .iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect::<Vec<f32>>();

            log_info(&format!("Embedding generated: {} dimensions", embedding.len()));
            Ok(embedding)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("OpenAI Embedding API error: {}", error_text));
            Err(AppError::InternalError(format!("Embedding API error: {}", error_text)))
        }
    }
}