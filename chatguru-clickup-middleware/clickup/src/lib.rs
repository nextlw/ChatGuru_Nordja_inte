//! Cliente completo da API ClickUp
//!
//! Este crate fornece uma interface tipo-segura e ergonômica para interagir com a API do ClickUp,
//! incluindo funcionalidades avançadas como:
//!
//! - Smart folder finder (busca fuzzy de folders)
//! - Smart assignee finder (busca fuzzy de assignees)
//! - Custom fields manager (gerenciamento de campos personalizados)
//! - Fuzzy matching utilities (normalização e comparação de strings)
//!
//! # API ClickUp v2
//!
//! Este crate utiliza exclusivamente a API v2 do ClickUp que é estável e suporta todos os recursos necessários:
//! - **Spaces**: `/team/{team_id}/space`
//! - **Folders**: `/space/{space_id}/folder`
//! - **Lists**: `/folder/{folder_id}/list`
//! - **Tasks**: `/list/{list_id}/task`, `/team/{team_id}/task`
//! - **Custom Fields**: `/list/{list_id}/field`
//! - **Webhooks**: `/team/{team_id}/webhook`
//! - **Attachments**: Endpoints de upload/download
//!
//! ## Nomenclatura
//! O crate usa a nomenclatura workspace_id para consistência conceitual:
//! - ✅ `workspace_id` (internamente, mas mapeado para `team_id` na API v2)
//! - ✅ Variáveis de ambiente: `CLICKUP_WORKSPACE_ID` (com fallback para `CLICKUP_TEAM_ID`)
//!
//! # Exemplo Básico
//!
//! ```rust,ignore
//! use clickup::{ClickUpClient, folders::SmartFolderFinder};
//!
//! #[tokio::main]
//! async fn main() -> clickup::Result<()> {
//!     // IMPORTANTE: Ler de variáveis de ambiente (NUNCA hardcode!)
//!     let api_token = std::env::var("CLICKUP_API_TOKEN")
//!         .expect("CLICKUP_API_TOKEN não configurado");
//!     let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
//!         .expect("CLICKUP_WORKSPACE_ID não configurado");
//!
//!     let client = ClickUpClient::new(api_token)?;
//!
//!     // Usa internamente "workspace_id" mas chama API v2
//!     let mut finder = SmartFolderFinder::new(client, workspace_id);
//!     let result = finder.find_folder_for_client("Nexcode").await?;
//!
//!     Ok(())
//! }
//! ```

// Módulos públicos
pub mod assignees;
pub mod client;
pub mod error;
pub mod fields;
pub mod folders;
pub mod matching;
pub mod tasks;
pub mod types; // ✅ Type-safe structures (Task, Priority, Status, CustomField, etc.)
pub mod webhooks; // ✅ Webhook management (create, list, update, delete)

// Re-exports principais
pub use client::{ClickUpClient, CreateListRequest, ListInfo};
pub use error::{ClickUpError, Result};

// Re-exports de types para conveniência
pub use types::{
    Assignee, CustomField, CustomFieldValue, Priority, Status, Task, TaskBuilder, User,
};

// Módulos a serem implementados
// pub mod lists;
