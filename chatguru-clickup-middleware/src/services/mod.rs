// Serviços necessários para arquitetura event-driven (SEM BANCO DE DADOS)
pub mod clickup;
pub mod chatguru;
pub mod openai;
pub mod secrets;
pub mod prompts;  // Configuração de prompts AI (YAML-only, sem PostgreSQL)
pub mod vertex;
pub mod hybrid_ai;  // Serviço híbrido AI (Vertex + OpenAI com fallback)
pub mod media_sync;
pub mod folder_resolver;  // Resolução simplificada de folders
pub mod smart_folder_finder;  // Busca inteligente de folders via API + histórico
pub mod smart_assignee_finder;  // Busca inteligente de assignees (responsáveis)
pub mod custom_field_manager;  // Gerenciamento de campos personalizados (Cliente Solicitante)

// Re-exports
pub use clickup::*;
pub use chatguru::*;
pub use openai::*;
pub use prompts::*;
pub use vertex::*;
pub use hybrid_ai::*;
pub use media_sync::*;
pub use folder_resolver::*;
pub use smart_folder_finder::*;
pub use smart_assignee_finder::*;
pub use custom_field_manager::*;

// OAuth2 module agora está em src/auth/ (módulo separado e isolado)
