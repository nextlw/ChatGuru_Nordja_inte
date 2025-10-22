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

    // Inicializar estado da aplicação (SEM scheduler, SEM database)
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
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
