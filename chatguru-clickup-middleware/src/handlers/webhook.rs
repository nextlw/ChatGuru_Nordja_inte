/// Webhook Handler: Recebe payload do ChatGuru e adiciona à fila
///
/// Arquitetura Unificada Event-Driven:
/// 1. Webhook ACK imediato (<100ms)
/// 2. Adiciona mensagem à fila (MessageQueueService)
/// 3. Callback processa automaticamente quando:
///    - 5 mensagens acumuladas OU
///    - 100 segundos transcorridos
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
    log_request_received("/webhooks/chatguru", "POST");

    // Extrair body como bytes
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    // Validar UTF-8
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in request body: {}", e)))?;

    // Parsear JSON para extrair chat_id
    let payload: Value = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;

    log_info(&format!("📥 Webhook payload recebido ({} bytes)", body_str.len()));

    // Extrair chat_id do payload
    let chat_id = payload
        .get("chat_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    log_info(&format!("📬 Adicionando mensagem do chat '{}' à fila", chat_id));

    // Adicionar à fila (processa automaticamente quando atingir 5 msgs ou 100s via callback)
    state.message_queue.enqueue(chat_id.clone(), payload).await;

    let processing_time = start_time.elapsed().as_millis() as u64;
    log_request_processed("/webhooks/chatguru", 200, processing_time);

    // ACK imediato (compatível com legado)
    Ok(Json(json!({
        "message": "Success"
    })))
}
