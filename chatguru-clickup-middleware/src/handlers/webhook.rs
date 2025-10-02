use axum::{
    extract::{Request, State},
    http::HeaderMap,
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use chatguru_clickup_middleware::models::WebhookPayload;
use chatguru_clickup_middleware::utils::{AppResult, AppError};
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

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
    
    // Log do tipo de payload detectado e verificar mídia
    match &webhook_payload {
        WebhookPayload::ChatGuru(p) => {
            log_info("Detected ChatGuru payload format");
            // Debug: verificar se há mídia anexada
            if p.media_url.is_some() || p.media_type.is_some() {
                log_info(&format!("📎 Mídia detectada no payload - URL: {:?}, Tipo: {:?}", 
                    p.media_url, p.media_type));
            } else if p.texto_mensagem == "audio" || p.texto_mensagem.contains("áudio") {
                log_warning(&format!("⚠️ Mensagem parece ser áudio mas sem media_url/media_type. Payload completo: {:?}", 
                    serde_json::to_string(&p).unwrap_or_default()));
            }
        },
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
    // COMPORTAMENTO DO LEGADO:
    // 1. Agrupa mensagem primeiro
    // 2. Scheduler processa depois (a cada 100 segundos)
    // 3. AI classifica no scheduler
    // 4. Cria tarefa apenas se for atividade
    
    // Filtrar eventos que não devem ser processados
    if let WebhookPayload::EventType(event_payload) = payload {
        // Ignorar eventos de annotation.added - estes são criados pelo próprio sistema
        if event_payload.event_type == "annotation.added" {
            log_info("Ignoring annotation.added event - this is a system-generated event");
            return Ok(());
        }
    }
    
    // Extrair dados básicos
    let _chat_id = extract_chat_id_from_payload(payload);  // Usado internamente no scheduler
    let _phone = extract_phone_from_payload(payload);      // Usado internamente no scheduler
    let nome = extract_nome_from_payload(payload);
    let message = extract_message_from_payload(payload);
    
    // Verificar se há mídia anexada (áudio ou imagem)
    let media_info = extract_media_from_payload(payload);
    if let Some((media_url, media_type)) = &media_info {
        log_info(&format!(
            "Mídia {} recebida de {}: {}",
            media_type,
            if !nome.is_empty() { nome.clone() } else { "Não Disponível".to_string() },
            media_url
        ));
    }
    
    // Log como o legado
    log_info(&format!(
        "Mensagem de {} agrupada recebida: {}",
        if !nome.is_empty() { nome.clone() } else { "Não Disponível".to_string() },
        message
    ));
    
    // Adicionar ao scheduler para processamento posterior (COMO O LEGADO)
    state.scheduler.queue_message(payload, None).await;
    
    // NÃO processar imediatamente - o scheduler fará isso
    log_info("Message queued for processing by scheduler");
    
    Ok(())
}

fn extract_phone_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => Some(p.celular.clone()),
        WebhookPayload::EventType(p) => p.data.phone.clone(),
        WebhookPayload::Generic(p) => p.celular.clone(),
    }
}

fn extract_chat_id_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => p.chat_id.clone(),
        WebhookPayload::EventType(_) => None,  // EventType não tem chat_id
        WebhookPayload::Generic(_) => None,    // Generic também não tem
    }
}

fn extract_message_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => p.texto_mensagem.clone(),
        WebhookPayload::EventType(p) => {
            // EventData não tem campo message, usar annotation ou task_title
            p.data.annotation.clone()
                .or(p.data.task_title.clone())
                .or(p.data.lead_name.clone())
                .unwrap_or_default()
        },
        WebhookPayload::Generic(p) => p.mensagem.clone().unwrap_or_default(),
    }
}

fn extract_nome_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => p.nome.clone(),
        WebhookPayload::EventType(p) => p.data.lead_name.clone().unwrap_or_default(),
        WebhookPayload::Generic(p) => p.nome.clone().unwrap_or_default(),
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

fn extract_media_from_payload(payload: &WebhookPayload) -> Option<(String, String)> {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            if let (Some(url), Some(media_type)) = (&p.media_url, &p.media_type) {
                Some((url.clone(), media_type.clone()))
            } else {
                None
            }
        },
        _ => None,
    }
}