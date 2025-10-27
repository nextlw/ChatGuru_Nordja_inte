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
//! # Versões da API (Híbrido v2 + v3)
//!
//! Este crate implementa uma **abordagem híbrida** que utiliza o melhor de ambas as versões:
//!
//! ## API v2 (Padrão)
//! Utilizada para a maioria dos endpoints que ainda não foram migrados:
//! - **Spaces**: `/team/{team_id}/space`
//! - **Folders**: `/space/{space_id}/folder`
//! - **Lists**: `/folder/{folder_id}/list`
//! - **Tasks**: `/list/{list_id}/task`, `/team/{team_id}/task`
//! - **Custom Fields**: `/list/{list_id}/field`
//! - **Attachments**: Endpoints de upload/download
//!
//! ## API v3 (Workspace-Centric)
//! Utilizada apenas onde há suporte nativo:
//! - **Workspaces**: `/workspaces` (listar workspaces)
//! - **Groups**: `/workspaces/{workspace_id}/groups`
//! - **Docs**: `/workspaces/{workspace_id}/docs`
//! - **Webhooks**: Endpoints workspace-scoped
//!
//! ## Nomenclatura Interna
//! Apesar de usar a API v2 para a maioria dos endpoints, este crate adota a nomenclatura da v3:
//! - ✅ `workspace_id` (internamente, mas mapeado para `team_id` na API v2)
//! - ✅ Variáveis de ambiente: `CLICKUP_WORKSPACE_ID` (com fallback para `CLICKUP_TEAM_ID`)
//!
//! Isso garante **compatibilidade** com a API v2 (estável) enquanto **prepara** o código
//! para migração futura quando todos os endpoints estiverem disponíveis em v3.
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
//!     // Usa internamente "workspace_id" mas chama API v2 com "team_id"
//!     let mut finder = SmartFolderFinder::new(client, workspace_id);
//!     let result = finder.find_folder_for_client("Nexcode").await?;
//!
//!     Ok(())
//! }
//! ```

// Módulos públicos
pub mod client;
pub mod error;
pub mod matching;
pub mod folders;
pub mod assignees;
pub mod fields;
pub mod tasks;

// Re-exports principais
pub use client::ClickUpClient;
pub use error::{ClickUpError, Result};

// Módulos a serem implementados
// pub mod types;
// pub mod lists;
