// Biblioteca do middleware ChatGuru-ClickUp
// Expõe módulos para uso em testes e binários

pub mod config;
pub mod models;
pub mod services;
pub mod utils;

// AppState é definido aqui para ser compartilhado
#[derive(Clone)]
pub struct AppState {
    pub settings: config::Settings,
    pub clickup_client: reqwest::Client,
    pub clickup: services::ClickUpService,
    pub scheduler: services::MessageScheduler,
}
