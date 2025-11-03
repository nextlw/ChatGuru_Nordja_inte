// Biblioteca do middleware ChatGuru-ClickUp
// Expõe módulos para uso em testes e binários

pub mod config;
pub mod models;
pub mod services;
pub mod utils;
pub mod auth; // Novo módulo OAuth2 isolado
pub mod middleware; // ✅ Middleware para autenticação e validação

// AppState é definido aqui para ser compartilhado
// Versão event-driven: SEM scheduler, SEM database
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub settings: config::Settings,
    pub clickup_client: reqwest::Client,
    pub clickup: clickup::tasks::TaskManager,  // ✅ Migrado para crate clickup
    pub ia_service: Option<Arc<services::IaService>>,  // Serviço de IA usando OpenAI (async-openai)
    pub message_queue: Arc<services::MessageQueueService>,  // Fila de mensagens por chat
    pub chatguru_client: chatguru::ChatGuruClient,  // ✅ Cliente ChatGuru configurado uma vez
}

impl AppState {
    /// Retorna o cliente ChatGuru já configurado
    /// Evita duplicação de configuração em cada uso
    pub fn chatguru(&self) -> &chatguru::ChatGuruClient {
        &self.chatguru_client
    }
}
