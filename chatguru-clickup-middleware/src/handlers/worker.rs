/// Worker Handler: Processa mensagens do Pub/Sub
///
/// Arquitetura:
/// 1. Recebe payload RAW do Pub/Sub via HTTP POST
/// 2. Processa com OpenAI para classificação
/// 3. Se for atividade, cria tarefa no ClickUp
/// 4. Envia anotação de volta ao ChatGuru
///
/// Este endpoint é chamado automaticamente pelo Cloud Tasks
/// Headers esperados:
/// - X-CloudTasks-TaskName: Nome da task
/// - X-CloudTasks-QueueName: Nome da fila

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use chatguru_clickup_middleware::models::WebhookPayload;
use chatguru_clickup_middleware::utils::{AppResult, AppError};
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;
use chatguru_clickup_middleware::services::openai::OpenAIService;
use chatguru_clickup_middleware::services::chatguru::ChatGuruApiService;

/// Handler do worker
/// Retorna 200 OK se processado com sucesso
/// Retorna 4xx se erro não recuperável (não faz retry)
/// Retorna 5xx se erro recuperável (Pub/Sub faz retry)
pub async fn handle_worker(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let start_time = Instant::now();
    log_request_received("/worker/process", "POST");

    // Extrair body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            log_error(&format!("Failed to read request body: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid request body"}))
            ));
        }
    };

    let body_str = match String::from_utf8(body_bytes.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            log_error(&format!("Invalid UTF-8: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid UTF-8"}))
            ));
        }
    };

    // Parsear envelope do Pub/Sub
    let envelope: Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("Invalid JSON: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid JSON"}))
            ));
        }
    };

    // Extrair payload RAW do envelope
    let raw_payload_str = match envelope.get("raw_payload").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            log_error("Missing raw_payload in envelope");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing raw_payload"}))
            ));
        }
    };

    // Parsear payload do ChatGuru
    let payload: WebhookPayload = match serde_json::from_str(raw_payload_str) {
        Ok(p) => p,
        Err(e) => {
            log_error(&format!("Failed to parse ChatGuru payload: {}", e));
            // Erro não recuperável - não fazer retry
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid ChatGuru payload"}))
            ));
        }
    };

    // Processar mensagem
    match process_message(&state, &payload).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            log_request_processed("/worker/process", 200, processing_time);
            Ok(Json(result))
        }
        Err(e) => {
            log_error(&format!("Worker processing error: {}", e));
            // Erro recuperável - Pub/Sub vai fazer retry
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()}))
            ))
        }
    }
}

/// Processa uma mensagem do ChatGuru
async fn process_message(state: &Arc<AppState>, payload: &WebhookPayload) -> AppResult<Value> {
    // Filtrar eventos que não devem ser processados
    if let WebhookPayload::EventType(event_payload) = payload {
        if event_payload.event_type == "annotation.added" {
            log_info("⏭️  Ignorando evento annotation.added (gerado pelo sistema)");
            return Ok(json!({
                "status": "skipped",
                "reason": "annotation.added event"
            }));
        }
    }

    // Extrair dados básicos
    let nome = extract_nome_from_payload(payload);
    let message = extract_message_from_payload(payload);
    let phone = extract_phone_from_payload(payload);
    let chat_id = extract_chat_id_from_payload(payload);

    log_info(&format!(
        "💬 Processando mensagem de {}: {}",
        if !nome.is_empty() { nome.clone() } else { "Desconhecido".to_string() },
        message
    ));

    // Classificar com OpenAI
    let openai_service = match OpenAIService::new(None).await {
        Some(service) => service,
        None => {
            return Err(AppError::InternalError("Failed to initialize OpenAI service".to_string()));
        }
    };

    let context = format!(
        "Campanha: WhatsApp\nOrigem: whatsapp\nNome: {}\nMensagem: {}\nTelefone: {}",
        nome, message, phone.as_deref().unwrap_or("N/A")
    );

    let classification = match openai_service.classify_activity_fallback(&context).await {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("❌ Erro na classificação OpenAI: {}", e));
            return Err(AppError::InternalError(format!("OpenAI classification failed: {}", e)));
        }
    };

    let annotation = format!("Tarefa: {}", classification.reason);
    let is_activity = classification.is_activity;

    if is_activity {
        log_info(&format!("✅ Atividade identificada: {}", classification.reason));

        // Criar tarefa no ClickUp
        let task_result = create_clickup_task(state, payload, &classification, &nome, &message).await?;

        // Enviar anotação ao ChatGuru
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!("⚠️  Falha ao enviar anotação ao ChatGuru: {}", e));
            // Não falhar o processamento se anotação falhar
        }

        Ok(json!({
            "status": "processed",
            "is_activity": true,
            "task_id": task_result.get("id"),
            "annotation": annotation
        }))
    } else {
        log_info(&format!("❌ Não é atividade: {}", classification.reason));

        // Apenas enviar anotação
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!("⚠️  Falha ao enviar anotação ao ChatGuru: {}", e));
        }

        Ok(json!({
            "status": "processed",
            "is_activity": false,
            "annotation": annotation
        }))
    }
}

/// Cria tarefa no ClickUp
async fn create_clickup_task(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    classification: &chatguru_clickup_middleware::services::openai::OpenAIClassification,
    nome: &str,
    message: &str,
) -> AppResult<Value> {
    let clickup = &state.clickup;

    // Montar dados da tarefa
    let task_title = format!("[Campanha] {}", nome);
    let task_description = format!(
        "**Campanha:** WhatsApp\n**Origem:** whatsapp\n\n**Cliente:** {}\n**Telefone:** {}\n\n**Mensagem:**\n{}\n\n**Classificação:** {}\n**Categoria:** {}",
        nome,
        extract_phone_from_payload(payload).as_deref().unwrap_or("N/A"),
        message,
        classification.reason,
        classification.category.as_deref().unwrap_or("Não especificada")
    );

    let task_data = json!({
        "name": task_title,
        "description": task_description,
        "status": "pendente",
        "priority": 3,  // Prioridade padrão
    });

    match clickup.create_task_from_json(&task_data).await {
        Ok(response) => {
            if let Some(task_id) = response.get("id").and_then(|v| v.as_str()) {
                log_info(&format!("✅ Tarefa criada no ClickUp: {}", task_id));
            }
            Ok(response)
        }
        Err(e) => {
            log_error(&format!("❌ Erro ao criar tarefa no ClickUp: {}", e));
            Err(AppError::InternalError(format!("Failed to create ClickUp task: {}", e)))
        }
    }
}

/// Envia anotação de volta ao ChatGuru
async fn send_annotation_to_chatguru(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    annotation: &str,
) -> AppResult<()> {
    let api_token = state.settings.chatguru.api_token.clone()
        .unwrap_or_else(|| "default_token".to_string());
    let api_endpoint = state.settings.chatguru.api_endpoint.clone()
        .unwrap_or_else(|| "https://s15.chatguru.app/api/v1".to_string());
    let account_id = state.settings.chatguru.account_id.clone()
        .unwrap_or_else(|| "default_account".to_string());

    let chatguru_service = ChatGuruApiService::new(api_token, api_endpoint, account_id);

    let chat_id = extract_chat_id_from_payload(payload);
    let phone = extract_phone_from_payload(payload);

    if let Some(chat_id) = chat_id {
        let phone_str = phone.as_deref().unwrap_or("");
        chatguru_service.add_annotation(&chat_id, phone_str, annotation).await?;
        log_info("📝 Anotação enviada ao ChatGuru");
    }

    Ok(())
}

// ============================================================================
// Funções auxiliares de extração de dados
// ============================================================================

fn extract_nome_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            if p.nome.is_empty() {
                "Desconhecido".to_string()
            } else {
                p.nome.clone()
            }
        },
        WebhookPayload::EventType(p) => p.data.lead_name.clone().unwrap_or_else(|| "Desconhecido".to_string()),
        WebhookPayload::Generic(p) => p.nome.clone().unwrap_or_else(|| "Desconhecido".to_string()),
    }
}

fn extract_message_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => p.texto_mensagem.clone(),
        WebhookPayload::EventType(p) => p.data.annotation.clone().unwrap_or_default(),
        WebhookPayload::Generic(p) => p.mensagem.clone().unwrap_or_default(),
    }
}

fn extract_phone_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => Some(p.celular.clone()),
        WebhookPayload::EventType(p) => p.data.phone.clone(),
        WebhookPayload::Generic(_) => None,
    }
}

fn extract_chat_id_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => p.chat_id.clone(),
        WebhookPayload::EventType(_) => None,  // EventType não tem chat_id direto
        WebhookPayload::Generic(_) => None,
    }
}
