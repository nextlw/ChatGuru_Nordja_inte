/// Webhook Handler: Recebe payload do ChatGuru e envia RAW para Pub/Sub
///
/// Arquitetura Event-Driven:
/// 1. Webhook ACK imediato (<100ms)
/// 2. Envia payload RAW completo para Pub/Sub
/// 3. Worker processa de forma assíncrona
///
/// Benefícios:
/// - Rate limiting automático via Pub/Sub
/// - Retry e dead-letter queues gerenciados pelo GCP
/// - Escalabilidade horizontal
/// - Nenhuma lógica de negócio no webhook

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use chatguru_clickup_middleware::utils::{AppResult, AppError};
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

    // Validar JSON básico (não parsear estrutura complexa)
    let _: Value = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;

    log_info(&format!("📥 Webhook payload recebido ({} bytes)", body_str.len()));

    // Enviar RAW para Pub/Sub de forma assíncrona
    let state_clone = Arc::clone(&state);
    let body_clone = body_str.clone();

    tokio::spawn(async move {
        if let Err(e) = send_raw_to_pubsub(&state_clone, &body_clone).await {
            log_error(&format!("❌ Erro ao enviar para Pub/Sub: {}", e));
        } else {
            log_info("✅ Payload enviado para Pub/Sub com sucesso");
        }
    });

    let processing_time = start_time.elapsed().as_millis() as u64;
    log_request_processed("/webhooks/chatguru", 200, processing_time);

    // ACK imediato (compatível com legado)
    Ok(Json(json!({
        "message": "Success"
    })))
}

/// Envia payload RAW para Pub/Sub
async fn send_raw_to_pubsub(state: &Arc<AppState>, raw_payload: &str) -> AppResult<()> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;

    // Configurar cliente Pub/Sub
    let config = ClientConfig::default().with_auth().await
        .map_err(|e| AppError::InternalError(format!("Failed to configure Pub/Sub client: {}", e)))?;

    let client = Client::new(config).await
        .map_err(|e| AppError::InternalError(format!("Failed to create Pub/Sub client: {}", e)))?;

    // Obter nome do tópico
    let default_topic = "chatguru-webhook-raw".to_string();
    let topic_name = state.settings.gcp.pubsub_topic
        .as_ref()
        .unwrap_or(&default_topic);

    let topic = client.topic(topic_name);

    // Verificar se tópico existe
    if !topic.exists(None).await
        .map_err(|e| AppError::InternalError(format!("Failed to check topic existence: {}", e)))? {
        return Err(AppError::InternalError(format!("Topic '{}' does not exist", topic_name)));
    }

    // Criar publisher
    let publisher = topic.new_publisher(None);

    // Preparar mensagem com timestamp
    let envelope = json!({
        "raw_payload": raw_payload,
        "received_at": chrono::Utc::now().to_rfc3339(),
        "source": "chatguru-webhook"
    });

    let msg_bytes = serde_json::to_vec(&envelope)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize message: {}", e)))?;

    // Criar mensagem Pub/Sub
    let msg = PubsubMessage {
        data: msg_bytes.into(),
        ..Default::default()
    };

    // Publicar mensagem
    let awaiter = publisher.publish(msg).await;
    awaiter.get().await
        .map_err(|e| AppError::InternalError(format!("Failed to publish message: {}", e)))?;

    log_info(&format!("📤 Mensagem publicada no tópico '{}'", topic_name));

    Ok(())
}
