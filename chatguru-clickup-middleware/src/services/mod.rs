// Serviços necessários para arquitetura event-driven (SEM BANCO DE DADOS)
// NOTA: ClickUp agora é um crate separado em crates/clickup/
// NOTA: ChatGuru agora é um crate separado em crates/chatguru/
pub mod secrets;
pub mod prompts;  // Configuração de prompts AI (YAML-only, sem PostgreSQL)
pub mod workspace_hierarchy;  // Validação simplificada da hierarquia do workspace

// Re-exports (explícitos para evitar ambiguidade)
pub use chatguru::ChatGuruClient;
pub use prompts::AiPromptConfig;
pub use secrets::SecretManagerService;  // ✅ Exporta SecretManagerService
pub use workspace_hierarchy::{WorkspaceHierarchyService, WorkspaceValidation};

// Re-exporta do crate mensageria
pub use mensageria::{MessageQueueService, QueuedMessage};

// Re-exporta do crate ia-service
pub use ia_service::{IaService, IaServiceConfig, ActivityClassification, IaResult, IaServiceError};

// Compatibilidade com código legado: alias para ActivityClassification
pub type OpenAIClassification = ActivityClassification;

// OAuth2 module agora está em src/auth/ (módulo separado e isolado)
