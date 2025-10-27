/// Handler para receber eventos de webhooks do ClickUp
///
/// Arquitetura:
/// 1. ClickUp envia evento HTTP POST para /webhooks/clickup
/// 2. Handler valida assinatura (segurança)
/// 3. Publica evento no Pub/Sub (desacoplamento)
/// 4. Worker processa assincronamente
///
/// Benefícios:
/// - Processamento em tempo real de eventos ClickUp
/// - Retry automático via Pub/Sub
/// - Escalabilidade (múltiplos subscribers)
/// - Auditoria completa de eventos

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
    http::HeaderMap,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;

use chatguru_clickup_middleware::utils::AppError;
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;
use clickup::webhooks::WebhookPayload;

/// Handler principal para webhooks do ClickUp
///
/// Recebe eventos do ClickUp, valida assinatura e envia para Pub/Sub.
///
/// # Headers esperados
/// - `X-Signature`: Assinatura HMAC-SHA256 do payload
/// - `Content-Type`: application/json
///
/// # Response
/// - 200 OK: Evento recebido e publicado com sucesso
/// - 401 Unauthorized: Assinatura inválida
/// - 400 Bad Request: Payload inválido
pub async fn handle_clickup_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Json<Value>, AppError> {
    let start_time = Instant::now();
    log_request_received("/webhooks/clickup", "POST");

    // Extrair body como bytes (necessário para validar assinatura)
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    // ========== SEGURANÇA: Validar assinatura ==========
    // IMPORTANTE: Em produção, SEMPRE valide a assinatura!
    let signature = headers
        .get("X-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ValidationError("Missing X-Signature header".to_string()))?;

    // Obter secret do webhook (de env var ou config)
    let webhook_secret = std::env::var("CLICKUP_WEBHOOK_SECRET")
        .map_err(|_| AppError::ConfigError("CLICKUP_WEBHOOK_SECRET não configurado".to_string()))?;

    // Validar assinatura
    if !WebhookPayload::verify_signature(signature, &webhook_secret, &body_bytes) {
        log_warning("❌ Assinatura inválida do webhook ClickUp!");
        return Err(AppError::ValidationError("Invalid webhook signature".to_string()));
    }

    log_info("✅ Assinatura do webhook validada");

    // Parsear payload
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8: {}", e)))?;

    let payload: WebhookPayload = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON: {}", e)))?;

    log_info(&format!(
        "📥 Evento ClickUp recebido: {:?} (webhook_id: {})",
        payload.event, payload.webhook_id
    ));

    // Publicar no Pub/Sub para processamento assíncrono
    match publish_to_pubsub(&state, &payload).await {
        Ok(_) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            log_request_processed("/webhooks/clickup", 200, processing_time);

            Ok(Json(json!({
                "status": "success",
                "message": "Event published to Pub/Sub"
            })))
        }
        Err(e) => {
            log_error(&format!("❌ Erro ao publicar evento no Pub/Sub: {}", e));
            Err(AppError::InternalError(format!("Failed to publish event: {}", e)))
        }
    }
}

/// Publica evento do ClickUp no Pub/Sub
async fn publish_to_pubsub(
    state: &AppState,
    payload: &WebhookPayload,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;

    // Configurar cliente Pub/Sub
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    // Obter nome do tópico (pode ser configurável)
    let topic_name = state.settings.gcp.clickup_webhook_topic
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("clickup-webhook-events");

    let topic = client.topic(topic_name);

    // Verificar se tópico existe
    if !topic.exists(None).await? {
        return Err(format!("Topic '{}' does not exist", topic_name).into());
    }

    // Criar publisher
    let publisher = topic.new_publisher(None);

    // Preparar envelope
    let envelope = json!({
        "webhook_id": payload.webhook_id,
        "event": payload.event,
        "task_id": payload.task_id,
        "list_id": payload.list_id,
        "folder_id": payload.folder_id,
        "space_id": payload.space_id,
        "data": payload.data,
        "received_at": chrono::Utc::now().to_rfc3339(),
        "source": "clickup-webhook"
    });

    let msg_bytes = serde_json::to_vec(&envelope)?;

    // Criar mensagem Pub/Sub
    let msg = PubsubMessage {
        data: msg_bytes.into(),
        ..Default::default()
    };

    // Publicar com retry (máximo 3 tentativas)
    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 100;

    for attempt in 1..=MAX_RETRIES {
        match publisher.publish(msg.clone()).await.get().await {
            Ok(_) => {
                tracing::info!(
                    "✅ Evento ClickUp publicado no Pub/Sub - tópico: '{}', evento: {:?}",
                    topic_name, payload.event
                );
                return Ok(());
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                    tracing::warn!(
                        "⚠️ Tentativa {}/{} falhou. Retry em {}ms...",
                        attempt, MAX_RETRIES, backoff_ms
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    unreachable!()
}

/// Handler para gerenciar webhooks (admin/debug)
///
/// GET /clickup/webhooks - Lista webhooks registrados
pub async fn list_registered_webhooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    use clickup::webhooks::WebhookManager;

    log_request_received("/clickup/webhooks", "GET");

    // Obter token e workspace_id
    let api_token = state.settings.clickup.token.clone();
    let workspace_id = state.settings.clickup.workspace_id
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("CLICKUP_WORKSPACE_ID não configurado".to_string()))?
        .clone();

    // Criar WebhookManager
    let manager = WebhookManager::from_token(api_token, workspace_id)
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create WebhookManager: {}", e)))?;

    // Listar webhooks
    let webhooks = manager.list_webhooks()
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Failed to list webhooks: {}", e)))?;

    log_info(&format!("📋 Listados {} webhooks", webhooks.len()));

    Ok(Json(json!({
        "count": webhooks.len(),
        "webhooks": webhooks
    })))
}

/// Handler para criar webhook (admin/setup)
///
/// POST /clickup/webhooks
/// Body: { "endpoint": "https://...", "events": ["taskCreated", ...] }
pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    axum::Json(body): axum::Json<Value>,
) -> Result<Json<Value>, AppError> {
    use clickup::webhooks::{WebhookManager, WebhookConfig, WebhookEvent};

    log_request_received("/clickup/webhooks", "POST");

    // Parsear body
    let endpoint = body.get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Missing 'endpoint' field".to_string()))?
        .to_string();

    let events: Vec<WebhookEvent> = body.get("events")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or_else(|| AppError::ValidationError("Invalid 'events' field".to_string()))?;

    // Criar WebhookManager
    let api_token = state.settings.clickup.token.clone();
    let workspace_id = state.settings.clickup.workspace_id
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("CLICKUP_WORKSPACE_ID não configurado".to_string()))?
        .clone();

    let manager = WebhookManager::from_token(api_token, workspace_id)
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create WebhookManager: {}", e)))?;

    // Criar webhook
    let config = WebhookConfig {
        endpoint,
        events,
        status: Some("active".to_string()),
    };

    let webhook = manager.create_webhook(&config)
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create webhook: {}", e)))?;

    log_info(&format!("✅ Webhook criado: {}", webhook.id));

    Ok(Json(json!({
        "status": "success",
        "webhook": webhook
    })))
}
