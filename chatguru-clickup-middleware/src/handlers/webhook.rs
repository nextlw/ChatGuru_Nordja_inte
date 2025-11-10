/// Webhook Handler: Recebe payload do ChatGuru e adiciona à fila
///
/// Arquitetura Unificada Event-Driven:
/// 1. Webhook ACK imediato (<100ms)
/// 2. Adiciona mensagem à fila (MessageQueueService)
/// 3. Callback processa automaticamente quando:
///    - 8 mensagens acumuladas OU
///    - 180 segundos transcorridos
/// 4. Callback envia batch para Pub/Sub
/// 5. Worker processa de forma assíncrona
///
/// Benefícios:
/// - Arquitetura consistente e centralizada
/// - Rate limiting automático via batching + Pub/Sub
/// - Retry e dead-letter queues gerenciados pelo GCP
/// - Nenhuma lógica de negócio no webhook
/// - Uma única rota de processamento via callback

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;
use uuid;

use chatguru_clickup_middleware::utils::AppError;
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

/// Handler principal do webhook
/// Retorna Success imediatamente após enviar para Pub/Sub
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<Value>, AppError> {
    let start_time = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string()[..8].to_string(); // ID único para tracking
    
    log_info(&format!(
        "🔍 WEBHOOK INICIADO - RequestID: {} | Endpoint: {} | Method: {}",
        request_id, "/webhooks/chatguru", "POST"
    ));

    // Extrair body como bytes
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    log_info(&format!(
        "📦 BODY EXTRAÍDO - RequestID: {} | Size: {} bytes",
        request_id, body_bytes.len()
    ));

    // Validar UTF-8
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in request body: {}", e)))?;

    // Parsear JSON para extrair chat_id
    let payload: Value = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;

    log_info(&format!(
        "✅ JSON PARSEADO - RequestID: {} | Success",
        request_id
    ));

    // Extrair chat_id do payload
    let chat_id = payload
        .get("chat_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extrair informações adicionais para logging
    let sender_name = payload
        .get("sender_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    let message_type = payload
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("text");
    
    let has_media = payload
        .get("media_url")
        .and_then(|v| v.as_str())
        .map(|url| !url.is_empty())
        .unwrap_or(false);

    // Extrair texto da mensagem (truncado para logs)
    let message_text = payload
        .get("texto_mensagem")
        .and_then(|v| v.as_str())
        .map(|text| {
            if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text.to_string()
            }
        })
        .unwrap_or_default();

    // Verificar se é PDF duplicado (pode ter descrição vazia)
    let is_pdf = payload
        .get("media_url")
        .and_then(|v| v.as_str())
        .map(|url| url.to_lowercase().contains(".pdf"))
        .unwrap_or(false);

    let pdf_info = if is_pdf {
        " | ⚠️ PDF_DETECTED"
    } else {
        ""
    };

    // Log detalhado do webhook recebido
    log_info(&format!(
        "📥 WEBHOOK RECEBIDO - RequestID: {} | ChatID: {} | Sender: {} | Type: {} | Media: {} | Size: {} bytes{} | Text: \"{}\"",
        request_id, chat_id, sender_name, message_type,
        if has_media { "Sim" } else { "Não" },
        body_str.len(),
        pdf_info,
        message_text
    ));

    log_info(&format!(
        "📬 ADICIONANDO À FILA - RequestID: {} | ChatID: {} | Queue size: estimating...",
        request_id, chat_id
    ));

    // Adicionar à fila (processa automaticamente quando atingir 5 msgs ou 100s via callback)
    state.message_queue.enqueue(chat_id.clone(), payload).await;

    let processing_time = start_time.elapsed().as_millis() as u64;
    
    log_info(&format!(
        "✅ WEBHOOK CONCLUÍDO - RequestID: {} | ChatID: {} | Processing time: {}ms | Status: 200",
        request_id, chat_id, processing_time
    ));

    // ACK imediato (compatível com legado)
    Ok(Json(json!({
        "message": "Success",
        "request_id": request_id,
        "chat_id": chat_id,
        "processing_time_ms": processing_time
    })))
}
