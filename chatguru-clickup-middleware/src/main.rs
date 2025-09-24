use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;

mod config;
mod handlers;
mod models;
mod services;
mod utils;

use config::Settings;
use handlers::{
    health_check, ready_check, status_check,
    handle_webhook_flexible,
    list_clickup_tasks, get_clickup_list_info, test_clickup_connection,
};
use services::ClickUpService;
use utils::{AppError, logging::*};

#[derive(Clone)]
pub struct AppState {
    pub settings: Settings,
    pub clickup_client: reqwest::Client,
    pub clickup: ClickUpService,
}

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
    
    // PubSub é opcional - se falhar, apenas log warning
    // if let Err(e) = PubSubService::new(&settings).await {
    //     tracing::warn!("PubSub service not available: {}. Running without PubSub.", e);
    // }

    // Inicializar estado da aplicação
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        settings: settings.clone(),
    });

    // Configurar rotas
    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/status", get(status_check))
        
        // Webhooks ChatGuru (aceita múltiplos formatos)
        .route("/webhooks/chatguru", post(handle_webhook_flexible))
        
        // ClickUp endpoints
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