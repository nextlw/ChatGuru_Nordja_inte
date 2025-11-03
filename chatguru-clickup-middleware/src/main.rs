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
    list_clickup_tasks, get_clickup_list_info,  // ❌ Removido: test_clickup_connection (redundante com /ready)
    handle_clickup_webhook, list_registered_webhooks, create_webhook,
};
use utils::{AppError, logging::*};

// Importar novo módulo OAuth2
use auth::{OAuth2Config, TokenManager, OAuth2State, start_oauth_flow, handle_oauth_callback};

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
        let api_token = settings.chatguru.api_token.clone()
            .unwrap_or_else(|| "default_token".to_string());
        let api_endpoint = settings.chatguru.api_endpoint.clone()
            .unwrap_or_else(|| "https://s15.chatguru.app/api/v1".to_string());
        let account_id = settings.chatguru.account_id.clone()
            .unwrap_or_else(|| "default_account".to_string());
        
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
