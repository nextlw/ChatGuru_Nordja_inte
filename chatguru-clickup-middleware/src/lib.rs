//! Job de Enriquecimento de Tarefas via Cloud Logging
//!
//! Este crate contém a lógica para enriquecer tarefas do ClickUp
//! com categoria_nova, subcategoria_nova e estrelas.

use std::sync::Arc;

pub mod handlers;
pub mod services;
pub mod config;

/// Estado da aplicação compartilhado entre handlers
pub struct AppState {
    pub clickup_client: Arc<clickup_v2::client::ClickUpClient>,
    pub ia_service: Option<Arc<ia_service::IaService>>,
    pub prompt_config: Arc<services::prompts::AiPromptConfig>,
}

