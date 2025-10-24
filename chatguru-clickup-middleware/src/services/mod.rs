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
pub mod message_queue;  // Fila de mensagens por chat (5 msgs ou 100s)

// Re-exports (explícitos para evitar ambiguidade)
pub use clickup::ClickUpService;
pub use chatguru::ChatGuruApiService;
pub use openai::{OpenAIService, OpenAIClassification};
pub use prompts::AiPromptConfig;
pub use vertex::VertexAIService;
pub use hybrid_ai::{HybridAIService, HybridAIResult};
pub use media_sync::MediaSyncService;
pub use folder_resolver::FolderResolver;
pub use smart_folder_finder::{SmartFolderFinder, FolderSearchResult};
pub use smart_assignee_finder::{SmartAssigneeFinder, AssigneeSearchResult};
pub use custom_field_manager::CustomFieldManager;
pub use message_queue::{MessageQueueService, QueuedMessage};

// OAuth2 module agora está em src/auth/ (módulo separado e isolado)
