use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::services::secrets::SecretManagerService;
use crate::services::prompts::AiPromptConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Serviço de fallback usando OpenAI API (como o sistema legado)
#[derive(Clone)]
pub struct OpenAIService {
    client: Client,
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIClassification {
    pub is_activity: bool,
    pub reason: String,
    pub tipo_atividade: Option<String>,
    pub category: Option<String>,
    pub sub_categoria: Option<String>,
    pub subtasks: Vec<String>,
    pub status_back_office: Option<String>,
    pub campanha: Option<String>,     // Novo campo opcional
    pub description: Option<String>,  // Mudou para opcional
    pub space_name: Option<String>,   // Novo campo
    pub folder_name: Option<String>,  // Novo campo
    pub list_name: Option<String>,    // Novo campo
    pub info_1: Option<String>,
    pub info_2: Option<String>,
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

        // Cliente HTTP com timeouts otimizados
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(20)) // Timeout total: 20s (Whisper + classificação)
            .connect_timeout(std::time::Duration::from_secs(5)) // Conexão: 5s
            .build()
            .unwrap_or_else(|_| Client::new());

        Some(Self {
            client,
            api_key: key,
        })
    }
    
    /// Classifica atividade usando OpenAI (fallback) com prompt configurável
    pub async fn classify_activity_fallback(&self, context: &str) -> AppResult<OpenAIClassification> {
        log_info("Using OpenAI fallback for classification");

        let url = "https://api.openai.com/v1/chat/completions";

        // Carregar prompt do YAML (mesma fonte que o Vertex AI)
        let prompt_config = AiPromptConfig::load_default()
            .unwrap_or_else(|e| {
                log_error(&format!("Failed to load AI prompt config, using fallback: {}", e));
                self.get_fallback_config()
            });

        // Gerar prompt usando a mesma lógica do Vertex AI
        let full_prompt = prompt_config.generate_prompt(context);

        let request_body = json!({
            "model": "gpt-4o-mini", // Modelo mais recente e barato
            "messages": [
                {"role": "user", "content": full_prompt}
            ],
            "temperature": 0.1,
            "max_tokens": 500,
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
                "OpenAI classification: is_activity={}, category={:?}, subcategory={:?}, reason={}",
                classification.is_activity,
                classification.category,
                classification.sub_categoria,
                classification.reason
            ));
            
            Ok(classification)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("OpenAI API error: {}", error_text));
            Err(AppError::InternalError(format!("OpenAI API error: {}", error_text)))
        }
    }

    /// Baixa áudio de uma URL com timeout de 10s
    pub async fn download_audio(&self, url: &str) -> AppResult<Vec<u8>> {
        log_info(&format!("Downloading audio from: {}", url));

        // Cliente separado com timeout mais curto para download
        let download_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // Download: 10s máximo
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        let response = download_client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to download audio: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::InternalError(format!(
                "Failed to download audio: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read audio bytes: {}", e)))?;

        log_info(&format!("Audio downloaded: {} bytes", bytes.len()));
        Ok(bytes.to_vec())
    }

    /// Transcreve áudio usando OpenAI Whisper API
    pub async fn transcribe_audio(&self, audio_bytes: &[u8], file_extension: &str) -> AppResult<String> {
        log_info("Using OpenAI Whisper for audio transcription");

        let url = "https://api.openai.com/v1/audio/transcriptions";

        // Determinar MIME type baseado na extensão
        let mime_type = match file_extension.to_lowercase().as_str() {
            "ogg" => "audio/ogg",
            "mp3" => "audio/mpeg",
            "m4a" => "audio/mp4",
            "wav" => "audio/wav",
            "webm" => "audio/webm",
            _ => "audio/mpeg", // Default
        };

        let filename = format!("audio.{}", file_extension);

        // Criar multipart form com o arquivo de áudio
        let part = reqwest::multipart::Part::bytes(audio_bytes.to_vec())
            .file_name(filename)
            .mime_str(mime_type)
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

    /// Baixa imagem de uma URL com timeout de 10s
    pub async fn download_image(&self, url: &str) -> AppResult<Vec<u8>> {
        log_info(&format!("Downloading image from: {}", url));

        // Cliente separado com timeout mais curto para download
        let download_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10)) // Download: 10s máximo
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        let response = download_client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                log_error(&format!("Failed to download image: {}", e));
                AppError::InternalError(format!("Download failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            log_error(&format!("Image download failed with status: {}", status));
            return Err(AppError::InternalError(format!("HTTP {}", status)));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| {
                log_error(&format!("Failed to read image bytes: {}", e));
                AppError::InternalError(format!("Read failed: {}", e))
            })?
            .to_vec();

        log_info(&format!("Image downloaded: {} bytes", bytes.len()));
        Ok(bytes)
    }

    /// Descreve imagem usando OpenAI Vision (GPT-4 Vision)
    pub async fn describe_image(&self, image_bytes: &[u8]) -> AppResult<String> {
        use base64::{Engine as _, engine::general_purpose};

        log_info("Describing image with OpenAI Vision");

        let url = "https://api.openai.com/v1/chat/completions";

        // Converter imagem para base64
        let image_base64 = general_purpose::STANDARD.encode(image_bytes);

        let request_body = json!({
            "model": "gpt-4o-mini",
            "messages": [
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
                                "url": format!("data:image/jpeg;base64,{}", image_base64)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 500
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

            let description = json_response
                .get("choices")
                .and_then(|c| c.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|msg| msg.get("content"))
                .and_then(|content| content.as_str())
                .ok_or_else(|| AppError::InternalError("Invalid vision response format".to_string()))?
                .to_string();

            log_info(&format!("Image description completed: {} characters", description.len()));
            Ok(description)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("OpenAI Vision API error: {}", error_text));
            Err(AppError::InternalError(format!("Vision API error: {}", error_text)))
        }
    }

    /// Retorna configuração de fallback caso o YAML não possa ser carregado
    fn get_fallback_config(&self) -> AiPromptConfig {
        use std::collections::HashMap;
        use crate::services::prompts::{ActivityType, StatusOption};

        AiPromptConfig {
            system_role: "Você é um assistente especializado em classificar solicitações e mapear campos para o sistema ClickUp.".to_string(),
            task_description: "Classifique se é uma atividade de trabalho válida e determine os campos apropriados.".to_string(),
            categories: vec![
                "Agendamentos".to_string(),
                "Compras".to_string(),
                "Documentos".to_string(),
                "Lazer".to_string(),
                "Logística".to_string(),
                "Viagens".to_string(),
            ],
            activity_types: vec![
                ActivityType {
                    name: "Rotineira".to_string(),
                    description: "tarefas recorrentes e do dia a dia".to_string(),
                    id: "64f034f3-c5db-46e5-80e5-f515f11e2131".to_string(),
                },
                ActivityType {
                    name: "Especifica".to_string(),
                    description: "tarefas pontuais com propósito específico".to_string(),
                    id: "e85a4dc7-82d8-4f63-89ee-462232f50f31".to_string(),
                },
            ],
            status_options: vec![
                StatusOption {
                    name: "Executar".to_string(),
                    id: "7889796f-033f-450d-97dd-6fee2a44f1b1".to_string(),
                },
            ],
            category_mappings: HashMap::new(),
            subcategory_mappings: HashMap::new(),
            subcategory_examples: HashMap::new(),
            rules: vec![
                "Sempre escolha valores EXATOS das listas fornecidas".to_string(),
                "Se não houver certeza, classifique como false".to_string(),
            ],
            response_format: r#"Responda APENAS com JSON válido:
{
  "is_activity": true/false,
  "reason": "título curto e profissional (máximo 5 palavras)",
  "tipo_atividade": "tipo da atividade",
  "category": "categoria",
  "sub_categoria": "subcategoria ou null",
  "subtasks": [],
  "status_back_office": "status"
}"#.to_string(),
            field_ids: None,
            cliente_solicitante_mappings: HashMap::new(),
        }
    }
}