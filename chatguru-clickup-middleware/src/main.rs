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
use sqlx::postgres::PgPoolOptions;

// Importar módulos da biblioteca
use chatguru_clickup_middleware::{AppState, config, services, utils, auth};

mod handlers;

use config::Settings;
use handlers::{
    health_check, ready_check, status_check,
    handle_webhook,
    handle_worker,
    list_clickup_tasks, get_clickup_list_info, test_clickup_connection,
    check_database,
    apply_migrations,
    sync_clickup_data,
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

    // Inicializar conexão com Cloud SQL PostgreSQL
    // Cloud Run usa Unix socket: /cloudsql/PROJECT:REGION:INSTANCE
    // Local development usa DATABASE_URL com host/port
    log_info("🗄️  Conectando ao Cloud SQL PostgreSQL...");

    let db_pool = if let Ok(database_url) = std::env::var("DATABASE_URL") {
        // Usar DATABASE_URL diretamente (desenvolvimento local ou override)
        log_info("📍 Usando DATABASE_URL fornecida");
        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(&database_url)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to connect with DATABASE_URL: {}", e)))?
    } else {
        // Construir conexão via Unix socket para Cloud Run
        let instance_connection_name = std::env::var("INSTANCE_CONNECTION_NAME")
            .unwrap_or_else(|_| "buzzlightear:southamerica-east1:chatguru-middleware-db".to_string());

        let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let db_pass = std::env::var("DB_PASS").unwrap_or_else(|_| {
            log_warning("⚠️ DB_PASS não definida!");
            "".to_string()
        });
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "chatguru_middleware".to_string());

        log_info(&format!("🔧 DB Config - User: {}, DB: {}, Instance: {}", db_user, db_name, instance_connection_name));

        // Usar sqlx connect_with para configurar Unix socket
        let socket_path = format!("/cloudsql/{}", instance_connection_name);
        log_info(&format!("🔌 Conectando via Unix socket: {}", socket_path));

        use sqlx::postgres::PgConnectOptions;
        

        let options = PgConnectOptions::new()
            .host(&socket_path)
            .username(&db_user)
            .password(&db_pass)
            .database(&db_name);

        PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect_with(options)
            .await
            .map_err(|e| AppError::ConfigError(format!("Failed to connect via Unix socket: {}", e)))?
    };

    log_info("✅ Cloud SQL PostgreSQL connected");

    // Inicializar EstruturaService (se dinâmico estiver habilitado)
    let estrutura_service = if std::env::var("DYNAMIC_STRUCTURE_ENABLED")
        .unwrap_or_else(|_| "true".to_string()) == "true" {

        let clickup_token = std::env::var("CLICKUP_API_TOKEN")
            .or_else(|_| std::env::var("clickup_api_token"))
            .or_else(|_| std::env::var("CLICKUP_TOKEN"))
            .map_err(|_| AppError::ConfigError("clickup_api_token not set".to_string()))?;

        let service = services::EstruturaService::new(db_pool.clone(), clickup_token);
        log_info("✅ EstruturaService initialized (dynamic structure enabled)");
        Some(Arc::new(service))
    } else {
        log_info("ℹ️ Dynamic structure disabled (DYNAMIC_STRUCTURE_ENABLED=false)");
        None
    };

    // Inicializar serviços
    let mut clickup_service = ClickUpService::new_with_secrets().await
        .map_err(|e| AppError::ConfigError(format!("Failed to initialize ClickUp service via Secret Manager: {}", e)))?;

    // Injetar EstruturaService no ClickUpService se disponível
    if let Some(ref estrutura) = estrutura_service {
        clickup_service = clickup_service.with_estrutura_service(estrutura.clone());
        log_info("✅ ClickUp service configured with EstruturaService");
    } else {
        log_info("ℹ️ ClickUp service initialized without EstruturaService (static mode)");
    }

    // Inicializar Vertex AI service se habilitado
    let vertex_service = if let Some(ref vertex_config) = settings.vertex {
        if vertex_config.enabled {
            let topic_name = settings.gcp.media_processing_topic
                .clone()
                .unwrap_or_else(|| "media-processing-requests".to_string());

            let service = services::VertexAIService::new(
                vertex_config.project_id.clone(),
                topic_name
            );
            log_info("Vertex AI service initialized");
            Some(service)
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

    // Inicializar ConfigService para ler configurações do banco
    let config_service = services::ConfigService::new(db_pool.clone());
    log_info("ConfigService initialized");

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

    // Inicializar estado da aplicação (SEM scheduler)
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        config_db: config_service,
        settings: settings.clone(),
        vertex: vertex_service,
        media_sync: media_sync_service,
    });

    log_info("Event-driven architecture - No scheduler needed");

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

        // Database check endpoint (admin)
        .route("/admin/db-check", get({
            let pool = db_pool.clone();
            move || check_database(axum::extract::State(pool))
        }))

        // Database migration endpoint (admin)
        .route("/admin/migrate", post({
            let pool = db_pool.clone();
            move || apply_migrations(axum::extract::State(pool))
        }))

        // ClickUp sync endpoint (admin) - sincroniza spaces, folders e lists
        .route("/admin/sync-clickup", post({
            let pool = db_pool.clone();
            move || sync_clickup_data(axum::extract::State(pool))
        }))

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
