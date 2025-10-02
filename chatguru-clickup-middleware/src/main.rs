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
    health_check, ready_check, status_check, scheduler_status,
    handle_webhook_flexible,
    list_clickup_tasks, get_clickup_list_info, test_clickup_connection,
};
use services::{ClickUpService, MessageScheduler};
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
    
    // Inicializar scheduler com intervalos otimizados para performance
    // Redução de 100s para 10s em produção = melhoria de 90% no tempo de resposta
    let is_development = cfg!(debug_assertions) || 
        std::env::var("RUST_ENV").unwrap_or_default() == "development";
    
    let interval = if is_development { 
        5  // 5s em desenvolvimento para testes rápidos
    } else { 
        10 // 10s em produção (era 100s - redução de 90%)
    };
    
    log_info(&format!("Scheduler interval configured: {}s ({})", 
        interval, 
        if is_development { "development" } else { "production" }
    ));
    
    let mut scheduler = MessageScheduler::new(interval);
    scheduler.configure(settings.clone(), clickup_service.clone());
    
    // PubSub é opcional - se falhar, apenas log warning
    // if let Err(e) = PubSubService::new(&settings).await {
    //     tracing::warn!("PubSub service not available: {}. Running without PubSub.", e);
    // }

    // Inicializar estado da aplicação
    let app_state = Arc::new(AppState {
        clickup_client: reqwest::Client::new(),
        clickup: clickup_service,
        scheduler: scheduler.clone(),
        settings: settings.clone(),
    });
    
    // Iniciar o scheduler (como no legado)
    scheduler.start().await;
    log_info("Scheduler started - verificar_e_enviar_mensagens job enabled");

    // Configurar rotas
    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .route("/status", get(status_check))
        .route("/scheduler/status", get(scheduler_status))
        
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