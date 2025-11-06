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
    middleware,
};
use std::sync::Arc;
use tokio::net::TcpListener;

// Importar módulos da biblioteca
use chatguru_clickup_middleware::{AppState, config, services, utils, auth, middleware as app_middleware};

mod handlers;

use config::Settings;
use handlers::{
    health_check, ready_check, status_check,
    handle_webhook,
    handle_worker,
};
use utils::{AppError, logging::*};

// Importar novo módulo OAuth2
use auth::{OAuth2Config, TokenManager, OAuth2State, start_oauth_flow, handle_oauth_callback};

// ============================================================================
// ClickUp Handlers usando diretamente o crate clickup
// ============================================================================

/// Handler para listar tarefas de uma lista específica do ClickUp
async fn list_clickup_tasks(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>
) -> Result<axum::response::Json<serde_json::Value>, AppError> {
    use serde_json::json;
    
    log_request_received("/admin/clickup/tasks", "GET");

    match state.clickup.get_tasks_in_list(None).await {
        Ok(tasks) => {
            log_info(&format!("✅ Listadas {} tasks", tasks.len()));
            Ok(axum::response::Json(json!({
                "success": true,
                "tasks": tasks,
                "count": tasks.len(),
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            log_clickup_api_error("get_tasks_in_list", None, &e.to_string());
            Err(AppError::ClickUpApi(e.to_string()))
        }
    }
}

/// Handler para obter informações detalhadas sobre uma lista do ClickUp
async fn get_clickup_list_info(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>
) -> Result<axum::response::Json<serde_json::Value>, AppError> {
    use serde_json::json;
    
    log_request_received("/admin/clickup/list", "GET");

    match state.clickup.get_list_info(None).await {
        Ok(list_info) => {
            Ok(axum::response::Json(json!({
                "success": true,
                "list": list_info,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            log_clickup_api_error("get_list_info", None, &e.to_string());
            Err(AppError::ClickUpApi(e.to_string()))
        }
    }
}

/// Handler para receber eventos de webhooks do ClickUp
async fn handle_clickup_webhook(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    request: axum::extract::Request<axum::body::Body>,
) -> Result<axum::response::Json<serde_json::Value>, AppError> {
    use tokio::time::Instant;
    use serde_json::json;
    
    let start_time = Instant::now();
    log_request_received("/webhooks/clickup", "POST");

    // Extrair body como bytes
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    // Validar assinatura
    let signature = headers
        .get("X-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ValidationError("Missing X-Signature header".to_string()))?;

    let webhook_secret = std::env::var("CLICKUP_WEBHOOK_SECRET")
        .map_err(|_| AppError::ConfigError("CLICKUP_WEBHOOK_SECRET não configurado".to_string()))?;

    if !clickup::webhooks::WebhookPayload::verify_signature(signature, &webhook_secret, &body_bytes) {
        log_warning("❌ Assinatura inválida do webhook ClickUp!");
        return Err(AppError::ValidationError("Invalid webhook signature".to_string()));
    }

    log_info("✅ Assinatura do webhook validada");

    // Parsear payload
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8: {}", e)))?;

    let payload: clickup::webhooks::WebhookPayload = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON: {}", e)))?;

    log_info(&format!(
        "📥 Evento ClickUp recebido: {:?} (webhook_id: {})",
        payload.event, payload.webhook_id
    ));

    // Publicar no Pub/Sub
    match publish_clickup_webhook_to_pubsub(&state, &payload).await {
        Ok(_) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            log_request_processed("/webhooks/clickup", 200, processing_time);

            Ok(axum::response::Json(json!({
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

/// Handler para listar webhooks registrados
async fn list_registered_webhooks(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<axum::response::Json<serde_json::Value>, AppError> {
    use serde_json::json;
    
    log_request_received("/admin/clickup/webhooks", "GET");

    // Obter token e workspace_id
    let api_token = state.settings.clickup.token.clone();
    let workspace_id = state.settings.clickup.workspace_id
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("CLICKUP_WORKSPACE_ID não configurado".to_string()))?
        .clone();

    // Criar WebhookManager
    let manager = clickup::webhooks::WebhookManager::from_token(api_token, workspace_id)
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create WebhookManager: {}", e)))?;

    // Listar webhooks
    let webhooks = manager.list_webhooks()
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Failed to list webhooks: {}", e)))?;

    log_info(&format!("📋 Listados {} webhooks", webhooks.len()));

    Ok(axum::response::Json(json!({
        "count": webhooks.len(),
        "webhooks": webhooks
    })))
}

/// Handler para criar webhook
async fn create_webhook(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> Result<axum::response::Json<serde_json::Value>, AppError> {
    use serde_json::json;
    
    log_request_received("/admin/clickup/webhooks", "POST");

    // Parsear body
    let endpoint = body.get("endpoint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Missing 'endpoint' field".to_string()))?
        .to_string();

    let events: Vec<clickup::webhooks::WebhookEvent> = body.get("events")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or_else(|| AppError::ValidationError("Invalid 'events' field".to_string()))?;

    // Criar WebhookManager
    let api_token = state.settings.clickup.token.clone();
    let workspace_id = state.settings.clickup.workspace_id
        .as_ref()
        .ok_or_else(|| AppError::ConfigError("CLICKUP_WORKSPACE_ID não configurado".to_string()))?
        .clone();

    let manager = clickup::webhooks::WebhookManager::from_token(api_token, workspace_id)
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create WebhookManager: {}", e)))?;

    // Criar webhook
    let config = clickup::webhooks::WebhookConfig {
        endpoint,
        events,
        status: Some("active".to_string()),
    };

    let webhook = manager.create_webhook(&config)
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create webhook: {}", e)))?;

    log_info(&format!("✅ Webhook criado: {}", webhook.id));

    Ok(axum::response::Json(json!({
        "status": "success",
        "webhook": webhook
    })))
}

/// Função auxiliar para publicar evento do ClickUp no Pub/Sub
async fn publish_clickup_webhook_to_pubsub(
    state: &AppState,
    payload: &clickup::webhooks::WebhookPayload,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serde_json::json;

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    let topic_name = state.settings.gcp.clickup_webhook_topic
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("clickup-webhook-events");

    let topic = client.topic(topic_name);

    if !topic.exists(None).await? {
        return Err(format!("Topic '{}' does not exist", topic_name).into());
    }

    let publisher = topic.new_publisher(None);

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
    let msg = PubsubMessage {
        data: msg_bytes.into(),
        ..Default::default()
    };

    // Publicar com retry
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 🔧 Carregar variáveis de ambiente do arquivo .env (se existir)
    if let Err(_) = dotenvy::dotenv() {
        // Em produção (Cloud Run), não existe .env - variáveis vêm do ambiente
        tracing::debug!("Arquivo .env não encontrado - usando variáveis de ambiente do sistema");
    } else {
        tracing::info!("✅ Arquivo .env carregado com sucesso");
    }

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

    // ✅ Inicializar SecretManagerService primeiro
    let secret_manager = services::SecretManagerService::new()
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to initialize SecretManagerService: {}", e)))?;
    
    // ✅ Obter token do ClickUp via SecretManagerService (OAuth2 > Personal > Config)
    let clickup_token = secret_manager.get_clickup_api_token()
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to get ClickUp token: {}", e)))?;
    
    log_info(&format!("🔑 ClickUp token loaded from SecretManagerService"));
    
    // ✅ Criar TaskManager do crate clickup com token do Secret Manager
    let clickup_client = clickup::ClickUpClient::new(clickup_token)
        .map_err(|e| AppError::ConfigError(format!("Failed to create ClickUp client: {}", e)))?;
    let clickup_service = clickup::tasks::TaskManager::new(
        clickup_client,
        Some(settings.clickup.list_id.clone())
    );
    log_info("⚡ ClickUp TaskManager configured from crate with SecretManager token");

    // Inicializar IA Service (OpenAI)
    let ia_service = match std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("openai_api_key")) {
        Ok(api_key) => {
            let config = services::IaServiceConfig::new(api_key)
                .with_chat_model("gpt-4o-mini")
                .with_temperature(0.1)
                .with_max_tokens(500);

            match services::IaService::new(config) {
                Ok(service) => {
                    log_info("✅ IaService inicializado com OpenAI (gpt-4o-mini)");
                    Some(Arc::new(service))
                }
                Err(e) => {
                    log_warning(&format!("⚠️ Falha ao inicializar IaService: {}. Serviço desabilitado.", e));
                    None
                }
            }
        }
        Err(_) => {
            log_warning("⚠️ OPENAI_API_KEY não configurada. IaService desabilitado.");
            None
        }
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

    // ✅ Inicializar cliente ChatGuru uma única vez no AppState
    let chatguru_client = {
        // Tentar obter credenciais do config, senão buscar nos secrets
        let api_token = match settings.chatguru.api_token.clone() {
            Some(token) if !token.is_empty() => token,
            _ => {
                log_info("🔐 Buscando CHATGURU_API_TOKEN no Secret Manager...");
                secret_manager.get_secret_value("chatguru-api-token").await
                    .map_err(|e| format!("Falha ao buscar chatguru-api-token no Secret Manager: {}", e))?
            }
        };
        
        let api_endpoint = settings.chatguru.api_endpoint.clone()
            .ok_or("CHATGURU_API_ENDPOINT não configurado no default.toml")?;
            
        let account_id = match settings.chatguru.account_id.clone() {
            Some(id) if !id.is_empty() => id,
            _ => {
                log_info("🔐 Buscando CHATGURU_ACCOUNT_ID no Secret Manager...");
                secret_manager.get_secret_value("chatguru-account-id").await
                    .map_err(|e| format!("Falha ao buscar chatguru-account-id no Secret Manager: {}", e))?
            }
        };
        
        chatguru::ChatGuruClient::new(api_token, api_endpoint, account_id)
    };
    log_info("✅ ChatGuru client inicializado no AppState (configuração centralizada)");

    // Inicializar estado da aplicação
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        settings: settings.clone(),
        ia_service,
        message_queue,
        chatguru_client,  // ✅ Cliente configurado uma única vez
    });

    log_info("Event-driven architecture com Message Queue");

    // Configurar rotas base
    let mut app = Router::new()
        // Health checks (públicos)
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/status", get(status_check))

        // Webhooks (públicos - validação própria)
        .route("/webhooks/chatguru", post(handle_webhook))
        .route("/webhooks/clickup", post(handle_clickup_webhook))

        // Worker: Processa mensagens do Pub/Sub (público - autenticado pelo GCP)
        .route("/worker/process", post(handle_worker))

        .with_state(app_state.clone());

    // ✅ Rotas administrativas protegidas com API key
    let admin_routes = Router::new()
        .route("/admin/clickup/tasks", get(list_clickup_tasks))
        .route("/admin/clickup/list", get(get_clickup_list_info))
        .route("/admin/clickup/webhooks", get(list_registered_webhooks))
        .route("/admin/clickup/webhooks", post(create_webhook))
        .layer(middleware::from_fn(app_middleware::require_admin_key))
        .with_state(app_state);

    app = app.merge(admin_routes);

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

    // Graceful shutdown com signal handling
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    log_info("🛑 Server shut down gracefully");
    Ok(())
}

/// Signal handler para graceful shutdown
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log_info("🛑 Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            log_info("🛑 Received SIGTERM, shutting down gracefully...");
        }
    }
}

/// Função auxiliar para publicar batch no Pub/Sub (reutiliza lógica do webhook handler)
async fn publish_batch_to_pubsub(
    settings: &Settings,
    chat_id: String,
    messages: Vec<services::QueuedMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serde_json::json;

    // Nota: Processamento de mídia é feito individualmente no worker.rs usando ia-service
    tracing::info!("🔍 Batch do chat '{}' será enviado para processamento no worker", chat_id);

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
