//! Job de Enriquecimento de Tarefas via Cloud Logging
//!
//! Este serviço é acionado automaticamente via Pub/Sub quando os logs do Cloud Logging
//! detectam criação de tarefa no ClickUp. Não depende de webhooks do ClickUp.
//!
//! Fluxo:
//! 1. Cloud Logging Sink detecta log "Task criada"
//! 2. Pub/Sub envia para este serviço
//! 3. Extrai task_id do log
//! 4. Busca tarefa no ClickUp
//! 5. Verifica se campos categoria_nova/subcategoria_nova/estrelas estão vazios
//! 6. Se vazios, usa IA Service para classificar
//! 7. Valida se valores existem nas opções do YAML
//! 8. Atualiza tarefa com campos validados

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, error};

use chatguru_clickup_middleware::{AppState, services};
use chatguru_clickup_middleware::handlers::pubsub_handler::handle_pubsub_message;

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "task-enrichment-job",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Ready check endpoint
async fn ready_check(State(state): State<Arc<AppState>>) -> Json<Value> {
    let ia_available = state.ia_service.is_some();

    Json(json!({
        "status": "ready",
        "clickup": "connected",
        "ia_service": if ia_available { "available" } else { "unavailable" }
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Iniciando Job de Enriquecimento de Tarefas");

    // Carregar variáveis de ambiente
    if let Err(_) = dotenvy::dotenv() {
        info!("Arquivo .env não encontrado - usando variáveis de ambiente do sistema");
    }

    // Carregar configuração do prompt (ai_prompt.yaml)
    let prompt_config = services::prompts::AiPromptConfig::load_from_yaml("config/ai_prompt.yaml")
        .map_err(|e| format!("Falha ao carregar ai_prompt.yaml: {}", e))?;
    info!("✅ Configuração do prompt carregada");

    // Obter token do ClickUp
    let clickup_token = std::env::var("CLICKUP_API_TOKEN")
        .or_else(|_| std::env::var("clickup_api_token"))
        .map_err(|_| "CLICKUP_API_TOKEN não configurado")?;

    // Criar cliente ClickUp
    let clickup_client = Arc::new(
        clickup_v2::client::ClickUpClient::new(
            clickup_token,
            "https://api.clickup.com/api/v2".to_string()
        )
    );
    info!("✅ Cliente ClickUp configurado");

    // Inicializar IA Service
    let ia_service = match std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("openai_api_key")) {
        Ok(api_key) => {
            let config = ia_service::IaServiceConfig::new(api_key)
                .with_chat_model("gpt-4o-mini")
                .with_temperature(0.1)
                .with_max_tokens(500);

            match ia_service::IaService::new(config) {
                Ok(service) => {
                    info!("✅ IA Service inicializado");
                    Some(Arc::new(service))
                }
                Err(e) => {
                    error!("⚠️ Falha ao inicializar IA Service: {}", e);
                    None
                }
            }
        }
        Err(_) => {
            error!("⚠️ OPENAI_API_KEY não configurada. IA Service desabilitado.");
            None
        }
    };

    // Criar estado da aplicação
    let app_state = Arc::new(AppState {
        clickup_client,
        ia_service,
        prompt_config: Arc::new(prompt_config),
    });

    // Configurar rotas
    let app = Router::new()
        // Health checks
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))

        // Endpoint principal - recebe mensagens do Pub/Sub
        .route("/enrich", post(handle_pubsub_message))

        .with_state(app_state);

    // Iniciar servidor
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("✅ Servidor pronto em http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

