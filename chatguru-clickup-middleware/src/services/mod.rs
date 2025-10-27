// Serviços necessários para arquitetura event-driven (SEM BANCO DE DADOS)
pub mod clickup;
pub mod chatguru;
pub mod secrets;
pub mod prompts;  // Configuração de prompts AI (YAML-only, sem PostgreSQL)
pub mod smart_folder_finder;  // Busca inteligente de folders via API + histórico
pub mod smart_assignee_finder;  // Busca inteligente de assignees (responsáveis)
pub mod custom_field_manager;  // Gerenciamento de campos personalizados (Cliente Solicitante)

// Re-exports (explícitos para evitar ambiguidade)
pub use clickup::ClickUpService;
pub use chatguru::ChatGuruApiService;
pub use prompts::AiPromptConfig;
pub use smart_folder_finder::{SmartFolderFinder, FolderSearchResult};
pub use smart_assignee_finder::{SmartAssigneeFinder, AssigneeSearchResult};
pub use custom_field_manager::CustomFieldManager;

// Re-exporta do crate mensageria
pub use mensageria::{MessageQueueService, QueuedMessage};

// Re-exporta do crate ia-service
pub use ia_service::{IaService, IaServiceConfig, ActivityClassification, IaResult, IaServiceError};

// Compatibilidade com código legado: alias para ActivityClassification
pub type OpenAIClassification = ActivityClassification;

// OAuth2 module agora está em src/auth/ (módulo separado e isolado)
