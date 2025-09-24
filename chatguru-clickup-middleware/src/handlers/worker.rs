use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{error, info, warn};
use crate::{
    models::webhook_payload::WebhookPayload,
    services::{
        cloud_tasks::TaskPayload,
        VertexAIService, 
        ChatGuruApiService,
    },
    utils::logging::*,
    AppState,
};

pub async fn process_task_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TaskPayload<WebhookPayload>>,
) -> impl IntoResponse {
    let task_name = headers
        .get("X-CloudTasks-TaskName")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    let queue_name = headers
        .get("X-CloudTasks-QueueName")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    let retry_count = headers
        .get("X-CloudTasks-TaskRetryCount")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    
    let execution_count = headers
        .get("X-CloudTasks-TaskExecutionCount")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    info!(
        "Processing Cloud Task: task_name={}, queue={}, retry={}, execution={}, created_at={}",
        task_name, queue_name, retry_count, execution_count, payload.created_at
    );

    if retry_count > 5 {
        warn!(
            "Task {} has been retried {} times. Consider dead-lettering.",
            task_name, retry_count
        );
    }

    let start_time = std::time::Instant::now();

    // Processar o webhook usando a mesma lógica do handler principal
    match process_chatguru_event(
        &payload.data,
        &state,
    )
    .await
    {
                Ok(result) => {
            let duration = start_time.elapsed();
            info!(
                "Cloud Task {} processed successfully in {:?}. Result: {:?}",
                task_name, duration, result
            );

            // Publicar evento de processamento completado - Fase 2
            if let Some(pubsub) = &state.pubsub_events {
                let pubsub_clone = pubsub.clone();
                let payload_clone = payload.data.clone();
                let result_json = json!({
                    "task_name": task_name,
                    "processing_time_ms": duration.as_millis(),
                    "result": result,
                    "retry_count": retry_count,
                    "execution_count": execution_count
                });
                tokio::spawn(async move {
                    if let Err(e) = pubsub_clone.publish_processing_completed(&payload_clone, &result_json).await {
                        error!("Failed to publish processing_completed event: {}", e);
                    }
                });
            }

            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "task_name": task_name,
                    "processing_time_ms": duration.as_millis(),
                    "result": result
                }))
            )
        }
        Err(e) => {
            let duration = start_time.elapsed();
            error!(
                "Cloud Task {} failed after {:?}: {:?}",
                task_name, duration, e
            );

            // Publicar erro crítico se não é retryable ou muitas tentativas - Fase 2
            if let Some(pubsub) = &state.pubsub_events {
                let pubsub_clone = pubsub.clone();
                let payload_clone = payload.data.clone();
                let error_msg = e.to_string();
                let task_name_clone = task_name.to_string();
                tokio::spawn(async move {
                    let context = format!("worker_task_processing:{}", task_name_clone);
                    if let Err(publish_err) = pubsub_clone.publish_critical_error(
                        &context,
                        &error_msg,
                        Some(&payload_clone)
                    ).await {
                        error!("Failed to publish critical_error event: {}", publish_err);
                    }
                });
            }

            let is_retryable = match e.to_string().as_str() {
                s if s.contains("rate limit") => true,
                s if s.contains("timeout") => true,
                s if s.contains("connection") => true,
                s if s.contains("temporary") => true,
                _ => retry_count < 3,
            };

            if is_retryable {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "status": "error",
                        "task_name": task_name,
                        "error": e.to_string(),
                        "retry_count": retry_count,
                        "will_retry": true
                    }))
                )
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "status": "permanent_failure",
                        "task_name": task_name,
                        "error": e.to_string(),
                        "retry_count": retry_count,
                        "will_retry": false
                    }))
                )
            }
        }
    }
}

/// Replica a lógica de processamento do MessageScheduler
pub async fn process_chatguru_event(
    payload: &WebhookPayload,
    state: &Arc<AppState>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let clickup = &state.clickup;
    let settings = &state.settings;
    
    // Extrair dados do payload como no MessageScheduler
    let (chat_id, phone, nome, message, info_1, info_2) = match payload {
        WebhookPayload::ChatGuru(p) => {
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

    log_info(&format!("Processing message from {}: {}", nome, message));

    // 1. Classificar com AI
    let mut annotation = "Tarefa: Não é uma atividade".to_string();
    let mut should_create_task = false;
    let mut ai_classification = None;
    
    if let Some(ai_config) = &settings.ai {
        if ai_config.enabled {
            // Reconstruir payload básico para classificação
            let classification_payload = WebhookPayload::ChatGuru(crate::models::ChatGuruPayload {
                campanha_id: String::new(),
                campanha_nome: String::new(),
                origem: String::new(),
                email: phone.clone(),
                nome: nome.clone(),
                tags: vec![],
                texto_mensagem: message.clone(),
                campos_personalizados: HashMap::new(),
                bot_context: None,
                responsavel_nome: None,
                responsavel_email: None,
                link_chat: String::new(),
                celular: phone.clone(),
                phone_id: Some("62558780e2923cc4705beee1".to_string()),
                chat_id: Some(chat_id.clone()),
                chat_created: None,
            });
            
            // Classificar com Vertex AI
            if let Ok(mut vertex_service) = VertexAIService::new_with_clickup(
                settings.gcp.project_id.clone(),
                Some(settings.clickup.token.clone()),
                Some(settings.clickup.list_id.clone())
            ).await {
                if let Ok(classification) = vertex_service.classify_activity(&classification_payload).await {
                    annotation = vertex_service.build_chatguru_annotation(&classification);
                    should_create_task = classification.is_activity;
                    
                    if classification.is_activity {
                        log_info(&format!("Activity identified for {}: {}", 
                            nome, classification.reason));
                        ai_classification = Some(classification);
                    }
                }
            }
        }
    }
    
    // 2. Criar tarefa no ClickUp se for atividade
    if should_create_task {
        let mut campos_personalizados = HashMap::new();
        
        // Adicionar Info_1 e Info_2 se existirem
        if let Some(ref info_1) = info_1 {
            campos_personalizados.insert("Info_1".to_string(), serde_json::json!(info_1));
        }
        if let Some(ref info_2) = info_2 {
            campos_personalizados.insert("Info_2".to_string(), serde_json::json!(info_2));
        }
        
        let task_payload = WebhookPayload::ChatGuru(crate::models::ChatGuruPayload {
            campanha_id: String::new(),
            campanha_nome: "ChatGuru".to_string(),
            origem: "scheduler".to_string(),
            email: phone.clone(),
            nome: nome.clone(),
            tags: vec![],
            texto_mensagem: message.clone(),
            campos_personalizados,
            bot_context: None,
            responsavel_nome: None,
            responsavel_email: None,
            link_chat: String::new(),
            celular: phone.clone(),
            phone_id: Some("62558780e2923cc4705beee1".to_string()),
            chat_id: Some(chat_id.clone()),
            chat_created: None,
        });
        
        // Criar tarefa no ClickUp
        match clickup.process_webhook_payload_with_ai(&task_payload, ai_classification.as_ref()).await {
            Ok(task_response) => {
                log_info(&format!("ClickUp task created for {}", nome));
                
                // Publicar evento de task criada - Fase 2
                if let Some(pubsub) = &state.pubsub_events {
                    let pubsub_clone = pubsub.clone();
                    let payload_clone = payload.clone();
                    let task_response_clone = task_response.clone();
                    tokio::spawn(async move {
                        if let Err(e) = pubsub_clone.publish_task_created(&payload_clone, &task_response_clone).await {
                            error!("Failed to publish task_created event: {}", e);
                        }
                    });
                }
                
                // Registrar tarefa criada no ConversationTracker
                if let Some(task_id) = task_response.get("id").and_then(|id| id.as_str()) {
                    let task_name = task_response.get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Tarefa")
                        .to_string();
                    
                    if let Ok(vertex_service) = VertexAIService::new_with_clickup(
                        settings.gcp.project_id.clone(),
                        Some(settings.clickup.token.clone()),
                        Some(settings.clickup.list_id.clone())
                    ).await {
                        vertex_service.register_created_task(
                            &phone,
                            task_id.to_string(),
                            task_name
                        ).await;
                    }
                }
                
                // Enviar confirmação "Ok" se temos um chat_id real
                let has_real_chat_id = !chat_id.starts_with("generic_") && 
                                       !chat_id.starts_with("event_") &&
                                       !chat_id.chars().all(|c| c.is_numeric() || c == '+');
                
                if has_real_chat_id {
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
                        
                        if let Err(e) = chatguru_service.send_confirmation_message(
                            &phone,
                            Some("62558780e2923cc4705beee1"),
                            "Ok ✅"
                        ).await {
                            log_error(&format!("Failed to send confirmation: {}", e));
                        }
                    }
                }
            },
            Err(e) => {
                log_error(&format!("Failed to create ClickUp task: {}", e));
                return Err(format!("ClickUp task creation failed: {}", e).into());
            }
        }
    }
    
    // 3. Enviar anotação de volta ao ChatGuru
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
        
        if let Err(e) = chatguru_service.add_annotation(
            &chat_id,
            &phone,
            &annotation
        ).await {
            log_error(&format!("Failed to add annotation: {}", e));
            return Err(format!("Annotation failed: {}", e).into());
        } else {
            log_info(&format!("Message sent successfully: {}", annotation));
            log_info(&format!("Response sent and state updated for {}", nome));
            
            // Publicar evento de anotação processada - Fase 2
            if let Some(pubsub) = &state.pubsub_events {
                let pubsub_clone = pubsub.clone();
                let chat_id_clone = chat_id.clone();
                let annotation_data = json!({
                    "chat_id": chat_id,
                    "phone": phone,
                    "annotation": annotation,
                    "nome": nome,
                    "should_create_task": should_create_task,
                    "ai_classification": ai_classification
                });
                tokio::spawn(async move {
                    if let Err(e) = pubsub_clone.publish_annotation_processed(&chat_id_clone, &annotation_data).await {
                        error!("Failed to publish annotation_processed event: {}", e);
                    }
                });
            }
        }
    }
    
    Ok(format!("Successfully processed message from {}", nome))
}

pub async fn worker_health_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "worker": "cloud_tasks",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    )
}