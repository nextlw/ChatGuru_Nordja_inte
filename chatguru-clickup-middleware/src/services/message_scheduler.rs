use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::utils::logging::*;
use crate::models::WebhookPayload;
use crate::services::{VertexAIService, ChatGuruApiService, ClickUpService};
use crate::config::Settings;

/// Message queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub chat_id: String,
    pub phone: String,
    pub nome: String,
    pub message: String,
    pub annotation: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
    pub info_1: Option<String>,  // Solicitante
    pub info_2: Option<String>,  // Conta Cliente
}

/// Conversation state for a contact
#[derive(Debug, Clone)]
pub struct ConversationState {
    pub chat_id: String,
    pub phone: String,
    pub nome: String,
    pub messages: Vec<QueuedMessage>,
    pub last_processed: DateTime<Utc>,
    pub active: bool,
}

/// Message scheduler similar to legacy system's APScheduler
#[derive(Clone)]
pub struct MessageScheduler {
    /// Conversations indexed by chat_id
    conversations: Arc<RwLock<HashMap<String, ConversationState>>>,
    /// Interval in seconds (legacy uses 100 seconds)
    interval_seconds: u64,
    /// Running state
    running: Arc<RwLock<bool>>,
    /// Settings for services
    settings: Option<Settings>,
    /// ClickUp service
    clickup: Option<ClickUpService>,
}

impl MessageScheduler {
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            interval_seconds,
            running: Arc::new(RwLock::new(false)),
            settings: None,
            clickup: None,
        }
    }
    
    /// Configure services (must be called after creation)
    pub fn configure(&mut self, settings: Settings, clickup: ClickUpService) {
        self.settings = Some(settings);
        self.clickup = Some(clickup);
    }
    
    /// Add a message to the queue (called from webhook handler)
    pub async fn queue_message(&self, payload: &WebhookPayload, annotation: Option<String>) {
        let (chat_id, phone, nome, message, info_1, info_2) = match payload {
            WebhookPayload::ChatGuru(p) => {
                // Extrair Info_1 e Info_2 dos campos personalizados
                let info_1 = p.campos_personalizados.get("Info_1")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let info_2 = p.campos_personalizados.get("Info_2")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                (
                    p.chat_id.clone().unwrap_or_else(|| p.celular.clone()),
                    p.celular.clone(),
                    p.nome.clone(),
                    p.texto_mensagem.clone(),
                    info_1,
                    info_2,
                )
            },
            WebhookPayload::EventType(p) => {
                let phone = p.data.phone.clone().unwrap_or_default();
                (
                    format!("event_{}", p.id),
                    phone.clone(),
                    p.data.lead_name.clone().unwrap_or_default(),
                    p.data.annotation.clone().unwrap_or_default(),
                    None,
                    None,
                )
            },
            WebhookPayload::Generic(p) => {
                let phone = p.celular.clone().unwrap_or_default();
                (
                    format!("generic_{}", phone),
                    phone.clone(),
                    p.nome.clone().unwrap_or_default(),
                    p.mensagem.clone().unwrap_or_default(),
                    None,
                    None,
                )
            }
        };
        
        let queued_message = QueuedMessage {
            chat_id: chat_id.clone(),
            phone: phone.clone(),
            nome: nome.clone(),
            message,
            annotation,
            timestamp: Utc::now(),
            processed: false,
            info_1,
            info_2,
        };
        
        let mut conversations = self.conversations.write().await;
        
        // Legacy behavior: group messages per contact
        log_info(&format!("Mensagem de {} agrupada recebida: {}", nome, queued_message.message));
        
        let entry = conversations
            .entry(chat_id.clone())
            .or_insert_with(|| ConversationState {
                chat_id: chat_id.clone(),
                phone,
                nome,
                messages: Vec::new(),
                last_processed: Utc::now(),
                active: true,
            });
        
        entry.messages.push(queued_message);
        
        // Processar imediatamente se tem muitas mensagens ou passou tempo suficiente
        let should_process = entry.messages.len() >= 3 || 
            (Utc::now() - entry.last_processed).num_seconds() > 30;
            
        if should_process {
            log_info(&format!("Triggering immediate processing for {} (messages: {})", 
                entry.nome, entry.messages.len()));
            // Marcar para processamento imediato
            entry.last_processed = Utc::now() - chrono::Duration::seconds(self.interval_seconds as i64 + 1);
        }
    }
    
    /// Start the scheduler (similar to APScheduler job)
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            log_warning("Scheduler already running");
            return;
        }
        *running = true;
        drop(running);
        
        let scheduler = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(scheduler.interval_seconds));
            
            // Log similar to legacy: Added job "verificar_e_enviar_mensagens" to job store "default"
            log_info("Added job \"verificar_e_enviar_mensagens\" to job store \"default\"");
            
            // Processar imediatamente na primeira vez (não esperar o intervalo)
            tokio::time::sleep(Duration::from_secs(2)).await; // Pequeno delay para coletar mensagens iniciais
            scheduler.verificar_e_enviar_mensagens().await;
            
            loop {
                interval.tick().await;
                
                let running = scheduler.running.read().await;
                if !*running {
                    break;
                }
                drop(running);
                
                // Execute the job
                scheduler.verificar_e_enviar_mensagens().await;
            }
            
            log_info("Removed job verificar_e_enviar_mensagens");
        });
    }
    
    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        log_info("Scheduler stopped");
    }
    
    /// Main processing function (matches legacy function name)
    async fn verificar_e_enviar_mensagens(&self) {
        let conversations = self.conversations.read().await;
        
        // Get list of contacts for logging (matches legacy format)
        let contact_names: Vec<String> = conversations
            .values()
            .filter(|c| c.active && !c.messages.is_empty())
            .map(|c| c.nome.clone())
            .collect();
        
        if !contact_names.is_empty() {
            log_info(&format!(
                "Executando verificar_e_enviar_mensagens para {}",
                contact_names.join(", ")
            ));
        }
        
        // Clone to avoid holding the lock (prevents the legacy bug)
        let conversations_snapshot: Vec<ConversationState> = conversations
            .values()
            .cloned()
            .collect();
        
        drop(conversations);
        
        // Process each conversation
        for conversation in conversations_snapshot {
            if !conversation.active || conversation.messages.is_empty() {
                continue;
            }
            
            // Check if enough time has passed (legacy waits for interval)
            let time_since_last = Utc::now() - conversation.last_processed;
            if time_since_last.num_seconds() < (self.interval_seconds as i64) {
                log_info(&format!(
                    "Aguardando mais mensagens ou intervalo para {}",
                    conversation.nome
                ));
                continue;
            }
            
            // Process messages for this contact
            self.process_conversation(&conversation).await;
            
            // Update state
            let mut conversations = self.conversations.write().await;
            if let Some(conv) = conversations.get_mut(&conversation.chat_id) {
                conv.last_processed = Utc::now();
                
                // Mark messages as processed
                for msg in &mut conv.messages {
                    msg.processed = true;
                }
                
                // Remove processed messages (keep last few for context)
                conv.messages.retain(|m| !m.processed || 
                    (Utc::now() - m.timestamp).num_seconds() < 300);
                
                // If no more messages, mark as inactive
                if conv.messages.is_empty() {
                    conv.active = false;
                    log_info(&format!("Fim de verificar_e_enviar_mensagens para {}", conv.nome));
                }
            }
        }
    }
    
    /// Process messages for a conversation (COMO O LEGADO)
    async fn process_conversation(&self, conversation: &ConversationState) {
        // Só processar se temos configurações
        let settings = match &self.settings {
            Some(s) => s,
            None => {
                log_error("Scheduler not configured with settings");
                return;
            }
        };
        
        let clickup = match &self.clickup {
            Some(c) => c,
            None => {
                log_error("Scheduler not configured with ClickUp service");
                return;
            }
        };
        
        for message in &conversation.messages {
            if message.processed {
                continue;
            }
            
            // AQUI É ONDE O LEGADO PROCESSA COM AI
            // 1. Classificar com AI
            let mut annotation = "Tarefa: Não é uma atividade".to_string();
            let mut should_create_task = false;
            let mut ai_classification = None;
            
            if let Some(ai_config) = &settings.ai {
                if ai_config.enabled {
                    // Reconstruir payload básico para classificação
                    let payload = WebhookPayload::ChatGuru(crate::models::ChatGuruPayload {
                        campanha_id: String::new(),
                        campanha_nome: String::new(),
                        origem: String::new(),
                        email: conversation.phone.clone(),
                        nome: conversation.nome.clone(),
                        tags: vec![],
                        texto_mensagem: message.message.clone(),
                        campos_personalizados: HashMap::new(),
                        bot_context: None,
                        responsavel_nome: None,
                        responsavel_email: None,
                        link_chat: String::new(),
                        celular: conversation.phone.clone(),
                        phone_id: Some("62558780e2923cc4705beee1".to_string()),
                        chat_id: Some(conversation.chat_id.clone()),
                        chat_created: None,
                    });
                    
                    // Classificar com Vertex AI com suporte a campos dinâmicos do ClickUp
                    if let Ok(mut vertex_service) = VertexAIService::new_with_clickup(
                        settings.gcp.project_id.clone(),
                        Some(settings.clickup.token.clone()),
                        Some(settings.clickup.list_id.clone())
                    ).await {
                        if let Ok(classification) = vertex_service.classify_activity(&payload).await {
                            annotation = vertex_service.build_chatguru_annotation(&classification);
                            should_create_task = classification.is_activity;
                            
                            // Log como o legado
                            if classification.is_activity {
                                log_info(&format!("Atividade identificada para {}: {}", 
                                    conversation.nome, classification.reason));
                                
                                // Guardar classificação para usar ao criar tarefa
                                ai_classification = Some(classification);
                            }
                        }
                    }
                }
            }
            
            // 2. Criar tarefa no ClickUp se for atividade
            if should_create_task {
                // Criar payload para ClickUp IDÊNTICO AO LEGADO
                // Mas preservar campos personalizados Info_1 e Info_2 se existirem
                let mut campos_personalizados = HashMap::new();
                
                // Adicionar Info_1 e Info_2 se existirem na mensagem
                if let Some(ref info_1) = message.info_1 {
                    campos_personalizados.insert("Info_1".to_string(), serde_json::json!(info_1));
                }
                if let Some(ref info_2) = message.info_2 {
                    campos_personalizados.insert("Info_2".to_string(), serde_json::json!(info_2));
                }
                
                let payload = WebhookPayload::ChatGuru(crate::models::ChatGuruPayload {
                    campanha_id: String::new(),
                    campanha_nome: "ChatGuru".to_string(),  // Sempre ChatGuru como no legado
                    origem: "scheduler".to_string(),  // Origem sempre scheduler no legado
                    email: conversation.phone.clone(),  // Email = celular no legado
                    nome: conversation.nome.clone(),
                    tags: vec![],  // Legado não usa tags
                    texto_mensagem: message.message.clone(),
                    campos_personalizados,  // Preservar campos Info_1 e Info_2 se vierem
                    bot_context: None,  // Legado não usa bot_context
                    responsavel_nome: None,  // Legado não define responsável
                    responsavel_email: None,
                    link_chat: String::new(),  // Legado não inclui link do chat na descrição
                    celular: conversation.phone.clone(),
                    phone_id: Some("62558780e2923cc4705beee1".to_string()),
                    chat_id: Some(conversation.chat_id.clone()),
                    chat_created: None,
                });
                
                
                // Tentar criar tarefa no ClickUp com classificação AI
                match clickup.process_webhook_payload_with_ai(&payload, ai_classification.as_ref()).await {
                    Ok(task_response) => {
                        log_info(&format!("ClickUp task created for {}", conversation.nome));
                        
                        // Registrar tarefa criada no ConversationTracker para futuras referências
                        // Vamos criar um novo vertex_service localmente se necessário
                        if let Some(task_id) = task_response.get("id").and_then(|id| id.as_str()) {
                            let task_name = task_response.get("name")
                                .and_then(|n| n.as_str())
                                .unwrap_or("Tarefa")
                                .to_string();
                            
                            // Criar novo vertex_service para registrar a tarefa
                            if let Ok(vertex_service) = VertexAIService::new_with_clickup(
                                settings.gcp.project_id.clone(),
                                Some(settings.clickup.token.clone()),
                                Some(settings.clickup.list_id.clone())
                            ).await {
                                vertex_service.register_created_task(
                                    &conversation.phone,
                                    task_id.to_string(),
                                    task_name
                                ).await;
                            }
                        }
                        
                        // ENVIAR "Ok" DE CONFIRMAÇÃO AO USUÁRIO
                        if let Some(chatguru_token) = &settings.chatguru.api_token {
                            let api_endpoint = settings.chatguru.api_endpoint.as_ref()
                                .map(|s| s.clone())
                                .unwrap_or_else(|| "https://s15.chatguru.app".to_string());
                            let account_id = settings.chatguru.account_id.as_ref()
                                .map(|s| s.clone())
                                .unwrap_or_else(|| "625584ce6fdcb7bda7d94aa8".to_string());
                            
                            let chatguru_service = ChatGuruApiService::new(
                                chatguru_token.clone(),
                                api_endpoint,
                                account_id
                            );
                            
                            // Enviar mensagem de confirmação
                            if let Err(e) = chatguru_service.send_confirmation_message(
                                &conversation.phone,
                                Some("62558780e2923cc4705beee1"),
                                "Ok ✅"
                            ).await {
                                log_error(&format!("Failed to send confirmation: {}", e));
                            }
                        }
                    },
                    Err(e) => {
                        log_error(&format!("Failed to create ClickUp task: {}", e));
                    }
                }
            }
            
            // 3. Enviar anotação de volta ao ChatGuru (COMO O LEGADO FAZ!)
            if let Some(chatguru_token) = &settings.chatguru.api_token {
                let api_endpoint = settings.chatguru.api_endpoint.as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "https://s15.chatguru.app".to_string());
                let account_id = settings.chatguru.account_id.as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "625584ce6fdcb7bda7d94aa8".to_string());
                
                let chatguru_service = ChatGuruApiService::new(
                    chatguru_token.clone(),
                    api_endpoint,
                    account_id
                );
                
                // Adicionar anotação ao chat
                if let Err(e) = chatguru_service.add_annotation(
                    &conversation.chat_id,
                    &conversation.phone,
                    &annotation
                ).await {
                    log_error(&format!("Failed to add annotation: {}", e));
                } else {
                    log_info(&format!("Mensagem enviada com sucesso: {}", annotation));
                    log_info(&format!("Resposta enviada e estado atualizado para {}", conversation.nome));
                }
            }
        }
    }
    
    /// Get current queue status
    pub async fn get_status(&self) -> HashMap<String, usize> {
        let conversations = self.conversations.read().await;
        let mut status = HashMap::new();
        
        for (chat_id, conv) in conversations.iter() {
            let pending_count = conv.messages.iter().filter(|m| !m.processed).count();
            if pending_count > 0 {
                status.insert(format!("{} ({})", conv.nome, chat_id), pending_count);
            }
        }
        
        status
    }
}