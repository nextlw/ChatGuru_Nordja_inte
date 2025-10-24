/// Main Application: Event-Driven Architecture com Pub/Sub
///
/// Arquitetura:
/// - Webhook recebe payload e envia RAW para Pub/Sub (ACK <100ms)
/// - Worker processa mensagens do Pub/Sub de forma assíncrona
/// - OpenAI classifica atividades
/// - ClickUp recebe tarefas criadas
///
/// SEM scheduler, SEM agrupamento de mensagens em memória
/// Processamento 100% event-driven via Pub/Sub

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;

// Importar módulos da biblioteca
use chatguru_clickup_middleware::{AppState, config, services, utils, auth};

mod handlers;

use config::Settings;
use handlers::{
    health_check, ready_check, status_check,
    handle_webhook,
    handle_worker,
    list_clickup_tasks, get_clickup_list_info, test_clickup_connection,
};
use services::ClickUpService;
use utils::{AppError, logging::*};

// Importar novo módulo OAuth2
use auth::{OAuth2Config, TokenManager, OAuth2State, start_oauth_flow, handle_oauth_callback};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing
    tracing_subscriber::fmt::init();

    // Carregar configurações
    let settings = Settings::new()
        .map_err(|e| AppError::ConfigError(format!("Failed to load settings: {}", e)))?;

    log_config_loaded(&std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()));

    // Sistema usa apenas YAML e API do ClickUp (sem banco de dados)
    log_info("📄 Modo YAML-only: usando apenas configuração YAML e API do ClickUp");

    // Inicializar serviços
    // NOTA: EstruturaService (DB) e FolderResolver (YAML) foram substituídos por:
    // - SmartFolderFinder (busca via API do ClickUp)
    // - SmartAssigneeFinder (busca assignees via API)
    // - CustomFieldManager (sincroniza "Cliente Solicitante")
    log_info("ℹ️ Usando SmartFolderFinder/SmartAssigneeFinder (busca via API)");

    let clickup_service = ClickUpService::new(settings.clone(), None);

    // Inicializar Vertex AI service se habilitado
    let vertex_service = if let Some(ref vertex_config) = settings.vertex {
        if vertex_config.enabled {
            let topic_name = settings.gcp.media_processing_topic
                .clone()
                .unwrap_or_else(|| "media-processing-requests".to_string());

            match services::VertexAIService::new(
                vertex_config.project_id.clone(),
                vertex_config.location.clone(),
                Some(topic_name)
            ).await {
                Ok(service) => {
                    log_info("✅ Vertex AI service initialized with authentication");
                    Some(service)
                }
                Err(e) => {
                    log_warning(&format!("⚠️ Failed to initialize Vertex AI service: {}. Service disabled.", e));
                    None
                }
            }
        } else {
            log_info("Vertex AI service disabled in config");
            None
        }
    } else {
        log_info("Vertex AI service not configured");
        None
    };

    // Inicializar MediaSync service se Vertex AI estiver habilitado
    let media_sync_service = if vertex_service.is_some() {
        let timeout = settings.vertex.as_ref()
            .map(|v| v.timeout_seconds)
            .unwrap_or(30);

        let service = services::MediaSyncService::new(timeout);
        log_info("Media Sync service initialized");
        Some(service)
    } else {
        None
    };

    // ConfigService desabilitado (sem banco de dados)
    log_info("ℹ️  ConfigService disabled (no database - using YAML only)");

    // Inicializar OAuth2 State (novo módulo isolado)
    let oauth_config = OAuth2Config::from_env()
        .map_err(|e| {
            log_warning(&format!("⚠️  OAuth2 config not loaded: {}. OAuth endpoints will not work.", e));
            e
        })
        .ok();

    let oauth_state = if let Some(config) = oauth_config {
        match TokenManager::new(config.clone()).await {
            Ok(token_manager) => {
                log_info("✅ OAuth2 TokenManager initialized");
                Some(Arc::new(OAuth2State {
                    config,
                    token_manager: Arc::new(token_manager),
                }))
            }
            Err(e) => {
                log_warning(&format!("⚠️  Failed to initialize TokenManager: {}. OAuth endpoints disabled.", e));
                None
            }
        }
    } else {
        None
    };

    // Inicializar fila de mensagens por chat com callback para Pub/Sub
    let message_queue = Arc::new(
        services::MessageQueueService::new()
            .with_batch_callback({
                let settings = settings.clone();
                move |chat_id: String, messages: Vec<services::QueuedMessage>| {
                    let settings = settings.clone();
                    tokio::spawn(async move {
                        match publish_batch_to_pubsub(&settings, chat_id.clone(), messages).await {
                            Ok(_) => {
                                tracing::info!(
                                    "✅ Batch do chat '{}' publicado no Pub/Sub com sucesso",
                                    chat_id
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ Erro ao publicar batch do chat '{}' no Pub/Sub: {}",
                                    chat_id, e
                                );
                            }
                        }
                    });
                }
            })
    );
    
    // Iniciar scheduler da fila (verifica a cada 10s)
    message_queue.clone().start_scheduler();
    log_info("✅ Message Queue Scheduler iniciado - COM CALLBACK para Pub/Sub (5 msgs ou 100s por chat)");

    // Inicializar HybridAI service se habilitado
    let hybrid_ai_service = if settings.ai.as_ref().and_then(|ai| ai.use_hybrid).unwrap_or(false) {
        match services::HybridAIService::new(&settings).await {
            Ok(service) => {
                log_info("✅ HybridAI Service inicializado (Vertex AI + OpenAI fallback)");
                Some(Arc::new(service))
            }
            Err(e) => {
                log_warning(&format!("⚠️ Falha ao inicializar HybridAI: {}. Worker usará OpenAI direto.", e));
                None
            }
        }
    } else {
        log_info("HybridAI Service desabilitado na configuração");
        None
    };

    // Inicializar estado da aplicação
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        settings: settings.clone(),
        vertex: vertex_service,
        media_sync: media_sync_service,
        hybrid_ai: hybrid_ai_service,
        message_queue,
    });

    log_info("Event-driven architecture com Message Queue");

    // Configurar rotas base
    let mut app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/status", get(status_check))

        // Webhook ChatGuru: Envia RAW para Pub/Sub
        .route("/webhooks/chatguru", post(handle_webhook))

        // Worker: Processa mensagens do Pub/Sub
        .route("/worker/process", post(handle_worker))

        // ClickUp endpoints (debug/admin)
        .route("/clickup/tasks", get(list_clickup_tasks))
        .route("/clickup/list", get(get_clickup_list_info))
        .route("/clickup/test", get(test_clickup_connection))

        .with_state(app_state);

    // Adicionar rotas OAuth2 se configurado
    if let Some(oauth_st) = oauth_state {
        log_info("✅ OAuth2 endpoints enabled: /auth/clickup, /auth/clickup/callback");

        let oauth_router = Router::new()
            .route("/auth/clickup", get(start_oauth_flow))
            .route("/auth/clickup/callback", get(handle_oauth_callback))
            .with_state(oauth_st);

        app = app.merge(oauth_router);
    } else {
        log_warning("⚠️  OAuth2 endpoints disabled (missing CLICKUP_CLIENT_ID or CLICKUP_CLIENT_SECRET)");
    }

    // Iniciar servidor
    // No Cloud Run, usar a variável de ambiente PORT
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(settings.server.port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    log_server_startup(port);
    log_server_ready(port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Função auxiliar para publicar batch no Pub/Sub (reutiliza lógica do webhook handler)
async fn publish_batch_to_pubsub(
    settings: &Settings,
    chat_id: String,
    mut messages: Vec<services::QueuedMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serde_json::json;

    // 0. PROCESSAR MÍDIAS PRIMEIRO (se houver)
    tracing::info!("🔍 Verificando se há mídias no batch do chat '{}'...", chat_id);
    messages = process_media_in_batch(settings, messages).await?;

    // 1. Agregar mensagens em um único payload (agora com mídias processadas!)
    let aggregated_payload = services::MessageQueueService::process_batch_sync(chat_id.clone(), messages)?;

    tracing::info!(
        "📦 Payload agregado para chat '{}' criado com sucesso",
        chat_id
    );

    // 2. Configurar cliente Pub/Sub
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    // 3. Obter nome do tópico
    let default_topic = "chatguru-webhook-raw".to_string();
    let topic_name = settings.gcp.pubsub_topic
        .as_ref()
        .unwrap_or(&default_topic);

    let topic = client.topic(topic_name);

    // 4. Verificar se tópico existe
    if !topic.exists(None).await? {
        return Err(format!("Topic '{}' does not exist", topic_name).into());
    }

    // 5. Criar publisher
    let publisher = topic.new_publisher(None);

    // 6. Preparar envelope com payload agregado
    let envelope = json!({
        "raw_payload": serde_json::to_string(&aggregated_payload)?,
        "received_at": chrono::Utc::now().to_rfc3339(),
        "source": "chatguru-webhook-queue",
        "chat_id": chat_id,
        "is_batch": true
    });

    let msg_bytes = serde_json::to_vec(&envelope)?;

    // 7. Criar mensagem Pub/Sub
    let msg = PubsubMessage {
        data: msg_bytes.into(),
        ..Default::default()
    };

    // 8. Publicar mensagem com retry (máximo 3 tentativas)
    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 100;

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        match publisher.publish(msg.clone()).await.get().await {
            Ok(_) => {
                if attempt > 1 {
                    tracing::info!(
                        "✅ Payload publicado no Pub/Sub após {} tentativa(s) - tópico: '{}', chat: {}",
                        attempt, topic_name, chat_id
                    );
                } else {
                    tracing::info!(
                        "📤 Payload agregado publicado no tópico '{}' (chat: {})",
                        topic_name, chat_id
                    );
                }
                return Ok(());
            }
            Err(e) => {
                last_error = Some(e);

                if attempt < MAX_RETRIES {
                    let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                    tracing::warn!(
                        "⚠️ Tentativa {}/{} falhou ao publicar no Pub/Sub (chat: {}). Tentando novamente em {}ms...",
                        attempt, MAX_RETRIES, chat_id, backoff_ms
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                } else {
                    tracing::error!(
                        "❌ Todas as {} tentativas falharam ao publicar no Pub/Sub para chat '{}'",
                        MAX_RETRIES, chat_id
                    );
                }
            }
        }
    }

    // Se chegou aqui, todas as tentativas falharam
    Err(last_error.unwrap().into())
}

/// Processa mídias no batch ANTES de publicar no Pub/Sub
/// Detecta mídias, publica em media-processing-requests, aguarda resultado e substitui por texto
async fn process_media_in_batch(
    settings: &Settings,
    messages: Vec<services::QueuedMessage>,
) -> Result<Vec<services::QueuedMessage>, Box<dyn std::error::Error + Send + Sync>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serde_json::{json, Value};
    use uuid::Uuid;

    let mut processed_messages = Vec::new();
    let mut has_media = false;

    // Configurar cliente Pub/Sub para mídias (se necessário)
    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    let media_topic_name = settings.gcp.media_processing_topic
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("media-processing-requests");

    for message in messages {
        let mut payload = message.payload.clone();

        // Verificar se payload tem mídia
        let has_media_url = payload.get("media_url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .is_some();

        let media_type = payload.get("media_type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if has_media_url && media_type.is_some() {
            has_media = true;
            let media_url = payload["media_url"].as_str().unwrap();
            let media_type_str = media_type.as_ref().unwrap();

            tracing::info!(
                "📎 Mídia detectada no batch: {} (tipo: {})",
                media_url, media_type_str
            );

            // Gerar correlation ID para rastrear
            let correlation_id = Uuid::new_v4().to_string();

            // Publicar requisição de processamento de mídia
            let media_request = json!({
                "correlation_id": correlation_id,
                "media_url": media_url,
                "media_type": media_type_str,
                "chat_id": payload.get("id_chatguru").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            let topic = client.topic(media_topic_name);
            let publisher = topic.new_publisher(None);

            let msg = PubsubMessage {
                data: serde_json::to_vec(&media_request)?.into(),
                ..Default::default()
            };

            tracing::info!("📤 Publicando mídia para processamento: {}", correlation_id);

            match publisher.publish(msg).await.get().await {
                Ok(_) => {
                    tracing::info!("✅ Mídia publicada com sucesso: {}", correlation_id);

                    // Aguardar resultado da Cloud Function (timeout: 60s)
                    match wait_for_media_result(&client, settings, &correlation_id, 60).await {
                        Ok(processed_text) => {
                            tracing::info!("✅ Mídia processada pela Cloud Function: {} chars", processed_text.len());

                            // ENVIAR ANOTAÇÃO NO CHATGURU com a transcrição/descrição
                            let chat_id = payload.get("id_chatguru")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");

                            let annotation = if media_type_str.contains("audio") {
                                format!("🎤 Transcrição do áudio:\n\n{}", processed_text)
                            } else if media_type_str.contains("image") {
                                format!("🖼️ Descrição da imagem:\n\n{}", processed_text)
                            } else {
                                format!("📎 Mídia processada:\n\n{}", processed_text)
                            };

                            if let Err(e) = send_annotation_to_chatguru(settings, chat_id, &annotation).await {
                                tracing::error!("❌ Erro ao enviar anotação ChatGuru: {}", e);
                            } else {
                                tracing::info!("✅ Anotação enviada ao ChatGuru para chat: {}", chat_id);
                            }

                            // Marcar mídia como processada (não precisa substituir texto!)
                            payload["media_processing_status"] = json!("completed_with_annotation");
                        }
                        Err(e) => {
                            tracing::error!("❌ Timeout aguardando Cloud Function (60s): {}", e);
                            payload["media_processing_status"] = json!("timeout");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Erro ao publicar mídia: {}. Continuando sem processamento.", e);
                    payload["media_processing_status"] = json!("failed");
                }
            }
        }

        processed_messages.push(services::QueuedMessage {
            payload,
            received_at: message.received_at,
        });
    }

    if has_media {
        tracing::info!("📊 Batch processado: {} mensagens (com mídia)", processed_messages.len());
    } else {
        tracing::info!("📊 Batch processado: {} mensagens (sem mídia)", processed_messages.len());
    }

    Ok(processed_messages)
}

/// Aguarda resultado da Cloud Function via subscription media-processing-results
async fn wait_for_media_result(
    client: &google_cloud_pubsub::client::Client,
    settings: &Settings,
    correlation_id: &str,
    timeout_secs: u64,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::time::{timeout, Duration};
    use futures_util::StreamExt;

    let subscription_name = settings.gcp.media_results_subscription
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("media-results-sub");

    let subscription = client.subscription(subscription_name);

    tracing::info!("⏳ Aguardando resultado da mídia (correlation_id: {}, timeout: {}s)", correlation_id, timeout_secs);

    let result = timeout(Duration::from_secs(timeout_secs), async {
        loop {
            // Pull mensagens (max 1)
            let mut stream = subscription.subscribe(None).await?;

            while let Some(message) = stream.next().await {
                let data = String::from_utf8_lossy(&message.message.data);

                if let Ok(result_json) = serde_json::from_str::<serde_json::Value>(&data) {
                    let msg_correlation_id = result_json.get("correlation_id")
                        .and_then(|v| v.as_str());

                    if msg_correlation_id == Some(correlation_id) {
                        // Ack mensagem
                        let _ = message.ack().await;

                        // Extrair resultado
                        if let Some(result_text) = result_json.get("result")
                            .and_then(|v| v.as_str()) {
                            return Ok(result_text.to_string());
                        } else if let Some(error) = result_json.get("error")
                            .and_then(|v| v.as_str()) {
                            return Err(format!("Cloud Function error: {}", error).into());
                        }
                    } else {
                        // Não é nossa mensagem, ack e continua
                        let _ = message.ack().await;
                    }
                }
            }

            // Pequeno delay antes de tentar novamente
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await;

    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!("Timeout após {}s aguardando resultado da mídia", timeout_secs).into()),
    }
}

/// Envia anotação para o ChatGuru
async fn send_annotation_to_chatguru(
    settings: &Settings,
    chat_id: &str,
    annotation: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_token = std::env::var("CHATGURU_API_TOKEN")
        .or_else(|_| std::env::var("chatguru_api_token"))
        .unwrap_or_else(|_| settings.chatguru.api_token.clone().unwrap_or_default());

    let api_endpoint = settings.chatguru.api_endpoint
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("https://s15.chatguru.app/api/v1");

    let account_id = settings.chatguru.account_id
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_default();

    let client = reqwest::Client::new();
    let url = format!("{}/annotations", api_endpoint);

    let body = serde_json::json!({
        "id_account": account_id,
        "chat_id": chat_id,
        "note": annotation
    });

    let response = client
        .post(&url)
        .header("apikey", api_token)
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("ChatGuru API error: {} - {}", status, text).into())
    }
}
