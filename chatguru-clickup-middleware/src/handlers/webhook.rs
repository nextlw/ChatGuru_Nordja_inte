use axum::{
    extract::{Request, State},
    http::HeaderMap,
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use crate::models::ChatGuruEvent;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::AppState;

pub async fn handle_chatguru_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Json<Value>, AppError> {
    let start_time = Instant::now();
    log_request_received("/webhooks/chatguru", "POST");

    // Extrair o body da request
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    // Verificar assinatura do webhook (se configurado)
    if let Some(ref secret) = state.settings.chatguru.webhook_secret {
        verify_webhook_signature(&headers, &body_bytes, secret)?;
    }

    // Parse do evento ChatGuru
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in request body: {}", e)))?;
    
    let chatguru_event: ChatGuruEvent = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;

    // Validar o evento
    validate_chatguru_event(&chatguru_event)?;

    // Processar o evento
    let result = process_chatguru_event(&state, &chatguru_event).await;
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(response) => {
            log_request_processed("/webhooks/chatguru", 200, processing_time);
            Ok(Json(response))
        },
        Err(e) => {
            log_request_processed("/webhooks/chatguru", 500, processing_time);
            log_error(&format!("Webhook processing error: {}", e));
            Err(e)
        }
    }
}

async fn process_chatguru_event(state: &AppState, event: &ChatGuruEvent) -> AppResult<Value> {
    let mut response = json!({
        "success": true,
        "campanha_id": event.campanha_id.clone(),
        "campanha_nome": event.campanha_nome.clone(),
        "nome_contato": event.nome.clone(),
        "processed_at": chrono::Utc::now().to_rfc3339()
    });

    // Processar task no ClickUp (criar nova ou atualizar existente)
    let _clickup_task = match state.clickup.process_clickup_task(event).await {
        Ok(task) => {
            // Determinar se foi criação ou atualização baseado na resposta
            let action = if task.get("date_created") == task.get("date_updated") {
                "created"
            } else {
                "updated"
            };
            
            response["clickup_task_processed"] = json!(true);
            response["clickup_task_action"] = json!(action);
            response["clickup_task_id"] = task.get("id").unwrap_or(&json!(null)).clone();
            
            log_info(&format!("ClickUp task {} - ID: {}",
                action,
                task.get("id").and_then(|v| v.as_str()).unwrap_or("unknown")
            ));
            
            Some(task)
        },
        Err(e) => {
            log_clickup_api_error("process_task", None, &e.to_string());
            response["clickup_task_processed"] = json!(false);
            response["clickup_error"] = json!(e.to_string());
            None
        }
    };

    // PubSub removido temporariamente para simplificar o deployment
    response["pubsub_enabled"] = json!(false);
    
    // Determinar se houve algum erro crítico
    if !response["clickup_task_processed"].as_bool().unwrap_or(false) {
        response["success"] = json!(false);
        response["message"] = json!("Event processed with errors - ClickUp task processing failed");
        return Err(AppError::InternalError("ClickUp integration failed".to_string()));
    } else {
        let action = response["clickup_task_action"].as_str().unwrap_or("processed");
        response["message"] = json!(format!("Event processed successfully - task {}", action));
    }

    Ok(response)
}

fn verify_webhook_signature(
    headers: &HeaderMap,
    body: &[u8],
    secret: &str,
) -> AppResult<()> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use hex;

    let signature_header = headers
        .get("X-ChatGuru-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ValidationError("Missing X-ChatGuru-Signature header".to_string()))?;

    // Remove o prefixo "sha256=" se presente
    let signature = signature_header.strip_prefix("sha256=").unwrap_or(signature_header);

    // Calcular HMAC
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|e| AppError::ValidationError(format!("Invalid secret key: {}", e)))?;
    
    mac.update(body);
    let expected = hex::encode(mac.finalize().into_bytes());

    // Comparação segura
    if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
        log_validation_error("webhook_signature", "Invalid signature");
        return Err(AppError::ValidationError("Invalid webhook signature".to_string()));
    }

    Ok(())
}

// Comparação de tempo constante para evitar timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

fn validate_chatguru_event(event: &ChatGuruEvent) -> AppResult<()> {
    // Validar campos obrigatórios do ChatGuru
    if event.campanha_id.is_empty() {
        log_validation_error("campanha_id", "Campaign ID cannot be empty");
        return Err(AppError::ValidationError("Campaign ID cannot be empty".to_string()));
    }

    if event.nome.is_empty() {
        log_validation_error("nome", "Contact name cannot be empty");
        return Err(AppError::ValidationError("Contact name cannot be empty".to_string()));
    }

    if event.celular.is_empty() {
        log_validation_error("celular", "Phone number cannot be empty");
        return Err(AppError::ValidationError("Phone number cannot be empty".to_string()));
    }

    Ok(())
}
