use crate::models::WebhookPayload;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::services::context_cache::ContextCache;
use crate::services::openai_fallback::OpenAIService;
use crate::services::conversation_tracker::{ConversationTracker, TaskAction};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// Vertex AI base endpoint
const VERTEX_AI_BASE: &str = "aiplatform.googleapis.com/v1";

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
    cache: ContextCache,  // Cache inteligente para economizar
    openai_fallback: Option<OpenAIService>,  // Fallback para OpenAI
    conversation_tracker: ConversationTracker,  // Rastreador de contexto
}

impl VertexAIService {
    /// Cria nova instância usando as credenciais padrão do Google Cloud
    /// Isso é mais eficiente pois usa as credenciais já configuradas no ambiente
    pub async fn new(project_id: String) -> AppResult<Self> {
        // Tentar obter token do Vertex AI (pode falhar)
        let access_token = Self::get_access_token().await.ok();
        
        // Configurar OpenAI como fallback
        let openai_fallback = OpenAIService::new(None);
        
        if access_token.is_none() && openai_fallback.is_none() {
            log_warning("Neither Vertex AI nor OpenAI are configured. AI classification will be disabled.");
        }
        
        Ok(Self {
            client: Client::new(),
            project_id,
            access_token,
            cache: ContextCache::new(),
            openai_fallback,
            conversation_tracker: ConversationTracker::new(),
        })
    }

    /// Obtém o access token usando o metadata service do Google Cloud
    /// Isso funciona automaticamente no Cloud Run/Compute Engine
    async fn get_access_token() -> AppResult<String> {
        // No Cloud Run, use o metadata service para obter o token
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token?scopes=https://www.googleapis.com/auth/cloud-platform";
        
        let client = Client::new();
        let response = client
            .get(metadata_url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to contact metadata service: {}", e)))?;
        
        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(AppError::ConfigError(format!("Metadata service error: {}", error)));
        }
        
        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
        }
        
        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to parse token response: {}", e)))?;
        
        Ok(token_response.access_token)
    }

    /// Classifica se o payload representa uma atividade válida usando Vertex AI
    pub async fn classify_activity(&self, payload: &WebhookPayload) -> AppResult<ActivityClassification> {
        let context = self.extract_context(payload);
        
        // Incrementar contador de requisições
        self.cache.increment_request_count().await;
        
        // 1. Verificar cache primeiro
        if let Some((is_activity, reason)) = self.cache.get_cached_classification(&context).await {
            log_info(&format!("Cache HIT: is_activity={}, reason={}", is_activity, reason));
            return Ok(ActivityClassification {
                is_activity,
                reason,
                activity_type: None,
                category: None,
                subtasks: Vec::new(),
                priority: None,
            });
        }
        
        // 2. Tentar classificar por padrões aprendidos
        if let Some((is_activity, reason)) = self.cache.classify_by_pattern(&context).await {
            log_info(&format!("Pattern MATCH: is_activity={}, reason={}", is_activity, reason));
            
            // Armazenar no cache para futuras consultas
            self.cache.store_classification(&context, is_activity, &reason, 0.85).await;
            
            return Ok(ActivityClassification {
                is_activity,
                reason,
                activity_type: None,
                category: None,
                subtasks: Vec::new(),
                priority: None,
            });
        }
        
        // 3. Se não encontrou no cache/padrões, chamar AI (Vertex ou OpenAI)
        log_info(&format!("Cache MISS - Calling AI for: {}", context));
        self.cache.increment_ai_calls().await;
        
        // Tentar Vertex AI primeiro, se falhar usar OpenAI
        let classification = match self.call_vertex_ai(&context).await {
            Ok(response) => {
                match self.parse_vertex_response(&response) {
                    Ok(cls) => cls,
                    Err(e) => {
                        log_warning(&format!("Vertex AI parse error: {}, trying OpenAI fallback", e));
                        self.call_openai_fallback(&context).await?
                    }
                }
            },
            Err(e) => {
                log_warning(&format!("Vertex AI failed: {}, using OpenAI fallback", e));
                self.call_openai_fallback(&context).await?
            }
        };
        
        // 4. Armazenar resultado no cache para economizar futuras chamadas
        self.cache.store_classification(
            &context, 
            classification.is_activity, 
            &classification.reason,
            0.95  // Alta confiança para respostas do AI
        ).await;
        
        // 5. Log estatísticas do cache
        let stats = self.cache.get_stats().await;
        log_info(&stats);
        
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
        
        // IMPORTANTE: Gemini só está disponível em us-central1, não em southamerica-east1
        // Usar sempre us-central1 para Vertex AI, independente de onde o Cloud Run está
        let vertex_region = "us-central1";
        
        // Endpoint do Vertex AI para Gemini (deve usar us-central1)
        // IMPORTANTE: Usar gemini-pro que está disponível para todos os projetos
        let url = format!(
            "https://{}-{}/projects/{}/locations/{}/publishers/google/models/gemini-pro:generateContent",
            vertex_region, VERTEX_AI_BASE, self.project_id, vertex_region
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
        
        // Formato correto para generateContent endpoint
        let request_body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
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
        // Estrutura de resposta do generateContent endpoint
        let text_response = response
            .get("candidates")
            .and_then(|c| c.get(0))
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.get(0))
            .and_then(|part| part.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| AppError::InternalError("Invalid Vertex AI response format".to_string()))?;

        // Parse do JSON na resposta
        let classification: Value = serde_json::from_str(text_response)
            .map_err(|e| AppError::InternalError(format!("Failed to parse classification from response: {} - Response was: {}", e, text_response)))?;

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
    
    /// Chama OpenAI como fallback
    async fn call_openai_fallback(&self, context: &str) -> AppResult<ActivityClassification> {
        if let Some(ref openai) = self.openai_fallback {
            let result = openai.classify_activity_fallback(context).await?;
            
            Ok(ActivityClassification {
                is_activity: result.is_activity,
                reason: result.reason,
                activity_type: result.category.clone(),
                category: result.category,
                subtasks: Vec::new(),
                priority: None,
            })
        } else {
            // Se nem OpenAI está configurado, usar classificação básica
            log_warning("No AI service available, using basic classification");
            
            let context_lower = context.to_lowercase();
            let is_activity = context_lower.contains("preciso") ||
                             context_lower.contains("quero") ||
                             context_lower.contains("pedido") ||
                             context_lower.contains("orçamento") ||
                             context_lower.contains("comprar") ||
                             context_lower.contains("tarefa") ||
                             context_lower.contains("agendar") ||
                             context_lower.contains("reunião") ||
                             context_lower.contains("criar") ||
                             context_lower.contains("relatório") ||
                             context_lower.contains("urgente");
            
            Ok(ActivityClassification {
                is_activity,
                reason: if is_activity { 
                    "Palavras-chave de atividade detectadas".to_string() 
                } else { 
                    "Sem indicadores de atividade".to_string() 
                },
                activity_type: None,
                category: None,
                subtasks: Vec::new(),
                priority: None,
            })
        }
    }
    
    /// Analisa contexto da conversa para decidir se deve atualizar tarefa
    pub async fn analyze_conversation_context(
        &self,
        phone: &str,
        message: &str,
        is_activity: bool,
    ) -> TaskAction {
        self.conversation_tracker.analyze_context(phone, message, is_activity).await
    }
    
    /// Registra tarefa criada no tracker
    pub async fn register_created_task(&self, phone: &str, task_id: String, title: String) {
        self.conversation_tracker.register_task(phone, task_id, title).await;
    }
}