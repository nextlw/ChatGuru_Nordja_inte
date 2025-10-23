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

    // Adicionar à fila (processa quando atingir 5 msgs ou 100s)
    let state_clone = Arc::clone(&state);
    let payload_clone = payload.clone();
    let chat_id_clone = chat_id.clone();

    tokio::spawn(async move {
        match state_clone.message_queue.enqueue(chat_id_clone.clone(), payload_clone).await {
            Some(messages) => {
                // Fila pronta - processar batch
                log_info(&format!(
                    "🚀 Chat '{}': Fila pronta com {} mensagens - enviando para processamento",
                    chat_id_clone,
                    messages.len()
                ));
                
                // Enviar batch para Pub/Sub
                if let Err(e) = send_batch_to_pubsub(&state_clone, chat_id_clone, messages).await {
                    log_error(&format!("❌ Erro ao enviar batch para Pub/Sub: {}", e));
                }
            }
            None => {
                // Ainda aguardando mais mensagens
                log_info(&format!("⏳ Chat '{}': Aguardando mais mensagens...", chat_id_clone));
            }
        }
    });

    let processing_time = start_time.elapsed().as_millis() as u64;
    log_request_processed("/webhooks/chatguru", 200, processing_time);

    // ACK imediato (compatível com legado)
    Ok(Json(json!({
        "message": "Success"
    })))
}

/// Envia batch de mensagens agregadas para Pub/Sub
async fn send_batch_to_pubsub(
    state: &Arc<AppState>,
    chat_id: String,
    messages: Vec<chatguru_clickup_middleware::services::QueuedMessage>,
) -> AppResult<()> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use chatguru_clickup_middleware::services::MessageQueueService;

    // 1. Agregar mensagens em um único payload (usa PRIMEIRA mensagem como base)
    let aggregated_payload = MessageQueueService::process_batch_sync(chat_id.clone(), messages)
        .map_err(|e| AppError::InternalError(format!("Failed to aggregate messages: {}", e)))?;

    log_info(&format!(
        "📦 Payload agregado para chat '{}' criado com sucesso",
        chat_id
    ));

    // 2. Configurar cliente Pub/Sub (usa metadata server do GCP em Cloud Run)
    let config = ClientConfig::default().with_auth().await
        .map_err(|e| AppError::InternalError(format!("Failed to configure Pub/Sub client: {}", e)))?;

    let client = Client::new(config).await
        .map_err(|e| AppError::InternalError(format!("Failed to create Pub/Sub client: {}", e)))?;

    // 3. Obter nome do tópico
    let default_topic = "chatguru-webhook-raw".to_string();
    let topic_name = state.settings.gcp.pubsub_topic
        .as_ref()
        .unwrap_or(&default_topic);

    let topic = client.topic(topic_name);

    // 4. Verificar se tópico existe
    if !topic.exists(None).await
        .map_err(|e| AppError::InternalError(format!("Failed to check topic existence: {}", e)))? {
        return Err(AppError::InternalError(format!("Topic '{}' does not exist", topic_name)));
    }

    // 5. Criar publisher
    let publisher = topic.new_publisher(None);

    // 6. Preparar envelope com payload agregado (formato compatível com worker)
    let envelope = json!({
        "raw_payload": serde_json::to_string(&aggregated_payload)
            .map_err(|e| AppError::InternalError(format!("Failed to serialize aggregated payload: {}", e)))?,
        "received_at": chrono::Utc::now().to_rfc3339(),
        "source": "chatguru-webhook-queue",
        "chat_id": chat_id,
        "is_batch": true
    });

    let msg_bytes = serde_json::to_vec(&envelope)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize envelope: {}", e)))?;

    // 7. Criar mensagem Pub/Sub
    let msg = PubsubMessage {
        data: msg_bytes.into(),
        ..Default::default()
    };

    // 8. Publicar mensagem
    let awaiter = publisher.publish(msg).await;
    awaiter.get().await
        .map_err(|e| AppError::InternalError(format!("Failed to publish message: {}", e)))?;

    log_info(&format!(
        "📤 Payload agregado publicado no tópico '{}' (chat: {})",
        topic_name,
        chat_id
    ));

    Ok(())
}
