use axum::{
    extract::{Request, State},
    http::HeaderMap,
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use crate::models::WebhookPayload;
use crate::services::{VertexAIService, ChatGuruApiService};
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::AppState;

pub async fn handle_webhook_flexible(
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
    if state.settings.chatguru.validate_signature {
        if let Some(ref secret) = state.settings.chatguru.webhook_secret {
            verify_webhook_signature(&headers, &body_bytes, secret)?;
        }
    }

    // Parse flexível do payload
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in request body: {}", e)))?;
    
    // Tentar parsear como WebhookPayload (aceita múltiplos formatos)
    let webhook_payload: WebhookPayload = match serde_json::from_str(&body_str) {
        Ok(payload) => payload,
        Err(e) => {
            log_validation_error("payload", &format!("Invalid JSON: {}", e));
            
            // Se falhar, tentar parsear como JSON genérico
            let raw_json: Value = serde_json::from_str(&body_str)
                .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;
            
            // Log do tipo de payload recebido para debug
            log_info(&format!("Received raw JSON payload: {:?}", raw_json));
            
            return Err(AppError::ValidationError(format!("Could not parse webhook payload: {}", e)));
        }
    };
    
    // Log do tipo de payload detectado
    match &webhook_payload {
        WebhookPayload::ChatGuru(_) => log_info("Detected ChatGuru payload format"),
        WebhookPayload::EventType(_) => log_info("Detected EventType payload format"),
        WebhookPayload::Generic(_) => log_info("Detected Generic payload format"),
    }

    // Clonar estado e payload para processamento assíncrono
    let state_clone = Arc::clone(&state);
    let webhook_payload_clone = webhook_payload.clone();
    
    // Processar em background (não bloqueia a resposta)
    tokio::spawn(async move {
        if let Err(e) = process_webhook_with_ai(&state_clone, &webhook_payload_clone).await {
            log_error(&format!("Background webhook processing error: {}", e));
        }
    });
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    log_request_processed("/webhooks/chatguru", 200, processing_time);
    
    // SEMPRE retorna Success imediatamente (como sistema legado)
    Ok(Json(json!({
        "message": "Success"
    })))
}

async fn process_webhook_with_ai(state: &Arc<AppState>, payload: &WebhookPayload) -> AppResult<()> {
    // Extrair telefone do payload
    let phone = extract_phone_from_payload(payload);
    
    // Verificar se IA está habilitada
    if let Some(ai_config) = &state.settings.ai {
        if ai_config.enabled {
            // Usar Vertex AI (único provider no Google Cloud)
            let vertex_service = VertexAIService::new(state.settings.gcp.project_id.clone()).await?;
            let classification = vertex_service.classify_activity(payload).await?;
            
            // Construir anotação
            let annotation = vertex_service.build_chatguru_annotation(&classification);
            
            // Enviar anotação de volta para o ChatGuru (se configurado)
            if let Some(chatguru_token) = &state.settings.chatguru.api_token {
                let api_endpoint = state.settings.chatguru.api_endpoint.as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "https://s15.chatguru.app/api/v1".to_string());
                let account_id = state.settings.chatguru.account_id.as_ref()
                    .map(|s| s.clone())
                    .unwrap_or_else(|| "625584ce6fdcb7bda7d94aa8".to_string());
                
                let chatguru_service = ChatGuruApiService::new(
                    chatguru_token.clone(),
                    api_endpoint,
                    account_id
                );
                
                // Enviar anotação
                if let Some(phone_number) = phone {
                    if let Err(e) = chatguru_service.send_annotation(&phone_number, &annotation).await {
                        log_error(&format!("Failed to send ChatGuru annotation: {}", e));
                        // Não falha o processamento se a anotação falhar
                    }
                }
            }
            
            // Se for uma atividade válida, criar tarefa no ClickUp
            if classification.is_activity {
                log_info("Activity classified as valid - Creating ClickUp task");
                state.clickup.process_webhook_payload(payload).await?;
            } else {
                log_info(&format!("Activity classified as invalid: {}", classification.reason));
            }
        } else {
            // IA desabilitada, processa normalmente
            log_info("AI is disabled - Processing webhook normally");
            state.clickup.process_webhook_payload(payload).await?;
        }
    } else {
        // Sem configuração de IA, processa normalmente (cria tarefa sempre)
        log_info("No AI configuration - Processing webhook normally");
        state.clickup.process_webhook_payload(payload).await?;
    }
    
    Ok(())
}

fn extract_phone_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => Some(p.celular.clone()),
        WebhookPayload::EventType(p) => p.data.phone.clone(),
        WebhookPayload::Generic(p) => p.celular.clone(),
    }
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