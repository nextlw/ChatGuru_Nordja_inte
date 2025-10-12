// Biblioteca do middleware ChatGuru-ClickUp
// Expõe módulos para uso em testes e binários

pub mod config;
pub mod models;
pub mod services;
pub mod utils;
pub mod auth; // Novo módulo OAuth2 isolado

// AppState é definido aqui para ser compartilhado
// Versão event-driven: SEM scheduler
#[derive(Clone)]
pub struct AppState {
    pub settings: config::Settings,
    pub clickup_client: reqwest::Client,
    pub clickup: services::ClickUpService,
    pub config_db: services::ConfigService,
    pub vertex: Option<services::VertexAIService>,
    pub media_sync: Option<services::MediaSyncService>,
}
