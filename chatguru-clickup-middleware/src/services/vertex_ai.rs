use crate::models::WebhookPayload;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::services::context_cache::ContextCache;
use crate::services::openai_fallback::OpenAIService;
use crate::services::conversation_tracker::{ConversationTracker, TaskAction};
use crate::services::ai_prompt_loader::AiPromptConfig;
use crate::services::clickup_fields_fetcher::{ClickUpFieldsFetcher, FieldMappings};
use std::fs;
use serde_yaml;
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
    // Novos campos para mapeamento inteligente
    pub cliente_solicitante_id: Option<String>,  // ID da opção no dropdown
    pub tipo_atividade: Option<String>,  // Rotineira, Urgente, etc.
    pub sub_categoria: Option<String>,  // Sub categoria selecionada
    pub status_back_office: Option<String>,  // Status inicial
}

#[derive(Clone)]
pub struct VertexAIService {
    client: Client,
    project_id: String,
    access_token: Option<String>,
    cache: ContextCache,  // Cache inteligente para economizar
    openai_fallback: Option<OpenAIService>,  // Fallback para OpenAI
    conversation_tracker: ConversationTracker,  // Rastreador de contexto para evitar duplicatas
    prompt_config: AiPromptConfig,  // Configuração do prompt carregada do YAML
    fields_fetcher: Option<ClickUpFieldsFetcher>,  // Busca campos dinâmicos do ClickUp
    cached_field_mappings: Option<FieldMappings>,  // Cache dos mapeamentos
}

impl VertexAIService {
    /// Cria nova instância usando as credenciais padrão do Google Cloud
    /// Isso é mais eficiente pois usa as credenciais já configuradas no ambiente
    #[allow(dead_code)]
    pub async fn new(project_id: String) -> AppResult<Self> {
        Self::new_with_clickup(project_id, None, None).await
    }
    
    /// Cria nova instância com suporte para buscar campos do ClickUp dinamicamente
    pub async fn new_with_clickup(
        project_id: String, 
        clickup_token: Option<String>,
        clickup_list_id: Option<String>
    ) -> AppResult<Self> {
        // Obter access token OAuth2 usando o metadata service do Google Cloud
        // Isso funciona automaticamente no Cloud Run com a conta de serviço
        let access_token = Self::get_access_token().await.ok();
        
        if access_token.is_none() {
            log_warning("Failed to get OAuth2 access token from metadata service. Vertex AI will not be available.");
        } else {
            log_info("Successfully obtained OAuth2 access token for Vertex AI");
        }
        
        // Configurar OpenAI como fallback
        let openai_fallback = OpenAIService::new(None);
        
        if access_token.is_none() && openai_fallback.is_none() {
            log_warning("Neither Vertex AI nor OpenAI are configured. AI classification will be disabled.");
        }
        
        // Carregar configuração do prompt
        let prompt_config = AiPromptConfig::load_default()
            .unwrap_or_else(|e| {
                log_warning(&format!("Failed to load AI prompt config: {}. Using default.", e));
                // Criar uma configuração padrão mínima se falhar
                AiPromptConfig {
                    system_role: "Você é um assistente de classificação.".to_string(),
                    task_description: "Classifique a mensagem.".to_string(),
                    categories: vec![],
                    activity_types: vec![],
                    status_options: vec![],
                    category_mappings: std::collections::HashMap::new(),
                    subcategory_examples: std::collections::HashMap::new(),
                    rules: vec![],
                    response_format: "Responda em JSON.".to_string(),
                }
            });
        
        // Configurar fetcher do ClickUp se tokens foram fornecidos
        let fields_fetcher = if let (Some(token), Some(list_id)) = (clickup_token, clickup_list_id) {
            Some(ClickUpFieldsFetcher::new(token, list_id))
        } else {
            None
        };
        
        Ok(Self {
            client: Client::new(),
            project_id,
            access_token,
            cache: ContextCache::new(),
            openai_fallback,
            conversation_tracker: ConversationTracker::new(),
            prompt_config,
            fields_fetcher,
            cached_field_mappings: None,
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
    pub async fn classify_activity(&mut self, payload: &WebhookPayload) -> AppResult<ActivityClassification> {
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
                cliente_solicitante_id: None,
                tipo_atividade: None,
                sub_categoria: None,
                status_back_office: None,
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
                cliente_solicitante_id: None,
                tipo_atividade: None,
                sub_categoria: None,
                status_back_office: None,
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
    async fn call_vertex_ai(&mut self, context: &str) -> AppResult<Value> {
        // Usar apenas OAuth2 com Vertex AI
        let token = self.access_token.as_ref()
            .ok_or_else(|| AppError::ConfigError("No OAuth2 access token available for Vertex AI".to_string()))?;
        
        // Usar us-central1 onde o modelo está disponível
        let vertex_region = "us-central1";
        let model_name = "gemini-2.0-flash-001";  // Modelo mais recente do Gemini
        
        // Endpoint do Vertex AI com OAuth2
        let url = format!(
            "https://{}-{}/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            vertex_region, VERTEX_AI_BASE, self.project_id, vertex_region, model_name
        );
        
        // Gerar prompt dinâmico com campos atualizados do ClickUp
        let prompt = if let Some(ref fetcher) = self.fields_fetcher {
            // Buscar categorias dinâmicas do ClickUp
            let categories = fetcher.get_available_categories().await.ok();
            
            // Buscar tipos de atividade dinâmicos
            let activity_types = if let Ok(fields) = fetcher.get_custom_fields().await {
                fields.iter()
                    .find(|f| f.name == "Tipo de Atividade")
                    .and_then(|f| f.type_config.as_ref())
                    .and_then(|tc| tc.options.as_ref())
                    .map(|opts| opts.iter()
                        .map(|o| {
                            // Mapear descrições conhecidas
                            let desc = match o.name.as_str() {
                                "Rotineira" => "tarefas recorrentes e do dia a dia",
                                "Especifica" => "tarefas pontuais com propósito específico",
                                "Dedicada" => "tarefas que demandam dedicação especial",
                                _ => "tarefa",
                            };
                            (o.name.clone(), desc.to_string())
                        })
                        .collect::<Vec<_>>()
                    )
            } else {
                None
            };
            
            // Buscar status dinâmicos
            let status_options = if let Ok(fields) = fetcher.get_custom_fields().await {
                fields.iter()
                    .find(|f| f.name == "Status Back Office")
                    .and_then(|f| f.type_config.as_ref())
                    .and_then(|tc| tc.options.as_ref())
                    .map(|opts| opts.iter()
                        .map(|o| o.name.clone())
                        .collect::<Vec<_>>()
                    )
            } else {
                None
            };
            
            // Atualizar cache de mapeamentos
            if let Ok(mappings) = fetcher.get_all_field_mappings().await {
                self.cached_field_mappings = Some(mappings);
            }
            
            // Gerar prompt com campos dinâmicos
            let mut full_prompt = self.prompt_config.generate_prompt_with_dynamic_fields(
                context,
                categories,
                activity_types,
                status_options
            );
            
            // Adicionar subcategorias disponíveis (limitado para não ficar muito grande)
            if let Ok(subcategories) = fetcher.get_available_subcategories().await {
                if !subcategories.is_empty() {
                    full_prompt.push_str("\nSUBCATEGORIAS DISPONÍVEIS (exemplos):\n");
                    for (i, sub) in subcategories.iter().enumerate() {
                        if i < 15 { // Limitar para não deixar o prompt muito grande
                            full_prompt.push_str(&format!("- {}\n", sub));
                        }
                    }
                    if subcategories.len() > 15 {
                        full_prompt.push_str(&format!("... e mais {} opções relacionadas às categorias\n", subcategories.len() - 15));
                    }
                    full_prompt.push_str("\nIMPORTANTE: A subcategoria deve sempre estar relacionada à categoria principal escolhida.\n");
                }
            }
            
            log_info("Using dynamic prompt with updated ClickUp fields");
            full_prompt
        } else {
            // Sem fetcher, tentar usar arquivo estático atualizado
            if let Ok(static_fields) = self.load_static_fields() {
                log_info("Using static fields from clickup_fields_static.yaml");
                
                // Gerar prompt com campos estáticos
                let categories = Some(static_fields.categories);
                let activity_types = Some(static_fields.activity_types
                    .into_iter()
                    .map(|at| (at.name, at.description))
                    .collect());
                let status_options = Some(static_fields.status_options);
                
                self.prompt_config.generate_prompt_with_dynamic_fields(
                    context,
                    categories,
                    activity_types,
                    status_options
                )
            } else {
                // Fallback final: usar prompt estático do YAML
                log_info("Using fallback static prompt from YAML");
                self.prompt_config.generate_prompt(context)
            }
        };
        
        // Formato correto para generateContent endpoint
        let request_body = json!({
            "contents": [{
                "role": "user",  // Obrigatório para Gemini 2.0
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

        // OTIMIZAÇÃO FASE 1: Timeout adaptativo baseado no tamanho do texto
        // Escala: 10s base + crescimento proporcional até 60s
        // - Até 500 chars: 15s
        // - Até 1000 chars: 20s  
        // - Até 2000 chars: 30s
        // - Até 4000 chars: 45s
        // - Acima 4000 chars: 60s
        let text_length = prompt.len();
        let timeout_seconds = std::cmp::min(
            60, // Máximo 60 segundos (1 minuto)
            10 + (text_length / 250) * 3 // Escala mais gradual: 3s para cada 250 chars
        ) as u64;
        
        log_info(&format!("⏱️ Vertex AI timeout adaptativo: {}s para {} caracteres", 
            timeout_seconds, text_length));
        
        // Configurar requisição com OAuth2 e timeout
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    log_warning(&format!("Vertex AI timeout after {}s", timeout_seconds));
                    AppError::InternalError(format!("Vertex AI timeout após {}s", timeout_seconds))
                } else {
                    AppError::InternalError(format!("Vertex AI request error: {}", e))
                }
            })?;

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

        // Limpar resposta que pode vir com markdown
        let cleaned_response = text_response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        
        // Parse do JSON na resposta limpa
        let classification: Value = serde_json::from_str(cleaned_response)
            .map_err(|e| AppError::InternalError(format!("Failed to parse classification from response: {} - Response was: {}", e, cleaned_response)))?;

        // Criar estrutura completa com mapeamento de campos
        Ok(ActivityClassification {
            is_activity: classification.get("is_activity")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            activity_type: classification.get("tipo_atividade")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            category: classification.get("category")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            subtasks: classification.get("subtasks")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect())
                .unwrap_or_else(Vec::new),
            priority: None,
            reason: classification.get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("Sem motivo especificado")
                .to_string(),
            // Novos campos mapeados
            cliente_solicitante_id: None, // Será mapeado depois com Info_1
            tipo_atividade: classification.get("tipo_atividade")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            sub_categoria: classification.get("sub_categoria")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            status_back_office: classification.get("status_back_office")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Constrói anotação profissional e detalhada para enviar ao ChatGuru
    pub fn build_chatguru_annotation(&self, classification: &ActivityClassification) -> String {
        if classification.is_activity {
            // Extrair título profissional da razão
            let titulo = self.extract_professional_title(&classification.reason);
            
            let mut annotation = format!("Tarefa: Atividade Identificada: {}", titulo);
            
            // Tipo de atividade é OBRIGATÓRIO para atividades válidas
            if let Some(ref tipo) = classification.tipo_atividade {
                annotation.push_str(&format!("\nTipo de Atividade: {}", tipo));
            } else {
                // Fallback para tipo padrão se não foi identificado
                annotation.push_str("\nTipo de Atividade: Rotineira");
            }
            
            // Categoria é OBRIGATÓRIA para atividades válidas
            if let Some(ref category) = classification.category {
                annotation.push_str(&format!("\nCategoria: {}", category));
            } else {
                // Fallback para categoria genérica
                annotation.push_str("\nCategoria: Atividades Corporativas");
            }
            
            // Subcategoria detalhada quando disponível
            if let Some(ref sub_categoria) = classification.sub_categoria {
                if !sub_categoria.is_empty() && sub_categoria != "null" {
                    annotation.push_str(&format!("\nSub Categoria: {}", sub_categoria));
                }
            }
            
            // Adicionar subtarefas se houver (apresentar de forma profissional)
            if !classification.subtasks.is_empty() {
                annotation.push_str("\nSubtarefas: (se aplicável)");
                for subtask in &classification.subtasks {
                    annotation.push_str(&format!("\n- {}", subtask));
                }
            }
            
            // Status do back office quando relevante
            if let Some(ref status) = classification.status_back_office {
                if !status.is_empty() {
                    annotation.push_str(&format!("\nStatus Back Office: {}", status));
                }
            }
            
            annotation
        } else {
            format!("Tarefa: Não é uma atividade de trabalho - {}", classification.reason)
        }
    }
    
    /// Extrai um título profissional da razão/descrição da atividade
    fn extract_professional_title(&self, reason: &str) -> String {
        // Remover prefixos comuns e deixar apenas a essência da atividade
        let clean_reason = reason
            .replace("A mensagem contém", "")
            .replace("O usuário solicitou", "")
            .replace("A solicitação é sobre", "")
            .replace("Trata-se de", "")
            .replace("É uma solicitação de", "")
            .replace("um pedido específico de", "")
            .replace("uma solicitação para", "")
            .replace("solicita", "")
            .trim()
            .to_string();
        
        // Capitalizar primeira letra e formatar profissionalmente
        if clean_reason.is_empty() {
            "Atividade Profissional".to_string()
        } else {
            // Capitalizar primeira letra
            let mut chars = clean_reason.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let capitalized = first.to_uppercase().collect::<String>() + chars.as_str();
                    // Limitar tamanho e garantir que termine bem
                    if capitalized.len() > 80 {
                        format!("{}...", &capitalized[..77])
                    } else {
                        capitalized
                    }
                }
            }
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
                cliente_solicitante_id: None,
                tipo_atividade: None,
                sub_categoria: None,
                status_back_office: None,
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
                cliente_solicitante_id: None,
                tipo_atividade: None,
                sub_categoria: None,
                status_back_office: None,
            })
        }
    }
    
    /// Analisa contexto da conversa para decidir se deve atualizar tarefa
    #[allow(dead_code)]
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
    
    /// Carrega campos estáticos do arquivo YAML atualizado pelo script
    fn load_static_fields(&self) -> AppResult<StaticFields> {
        let path = "config/clickup_fields_static.yaml";
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read static fields: {}", e)))?;
        
        let fields: StaticFields = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse static fields: {}", e)))?;
        
        Ok(fields)
    }
}

#[derive(Debug, Deserialize)]
struct StaticFields {
    categories: Vec<String>,
    activity_types: Vec<StaticActivityType>,
    status_options: Vec<String>,
    _subcategories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StaticActivityType {
    name: String,
    description: String,
}