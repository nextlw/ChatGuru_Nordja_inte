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
use chatguru_clickup_middleware::{AppState, config, services, utils};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing
    tracing_subscriber::fmt::init();

    // Carregar configurações
    let settings = Settings::new()
        .map_err(|e| AppError::ConfigError(format!("Failed to load settings: {}", e)))?;

    log_config_loaded(&std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()));

    // Inicializar serviços
    let clickup_service = ClickUpService::new(&settings);
    log_info("ClickUp service initialized");

    // Inicializar Vertex AI service se habilitado
    let vertex_service = if let Some(ref vertex_config) = settings.vertex {
        if vertex_config.enabled {
            let topic_name = settings.gcp.media_processing_topic
                .clone()
                .unwrap_or_else(|| "media-processing-requests".to_string());

            match services::VertexAIService::new(
                vertex_config.project_id.clone(),
                topic_name
            ).await {
                Ok(service) => {
                    log_info("Vertex AI service initialized");
                    Some(service)
                }
                Err(e) => {
                    log_error(&format!("Failed to initialize Vertex AI service: {}", e));
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

    // Inicializar estado da aplicação (SEM scheduler)
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        settings: settings.clone(),
        vertex: vertex_service,
        media_sync: media_sync_service,
    });

    log_info("Event-driven architecture - No scheduler needed");

    // Configurar rotas
    let app = Router::new()
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
