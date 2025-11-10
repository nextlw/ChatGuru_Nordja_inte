//! # ClickUp v2 Rust Crate
//!
//! Uma biblioteca Rust para integração com a API v2 do ClickUp.
//!
//! ## Features
//!
//! - Autenticação OAuth2
//! - Cliente HTTP assíncrono
//! - Gerenciamento de configurações
//! - Tratamento de erros robusto
//!
//! ## Exemplo
//!
//! ```no_run
//! use clickup_v2::auth::oauth::OAuthFlow;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let oauth = OAuthFlow::new()?;
//!     let token = oauth.authenticate().await?;
//!     println!("Token obtido: {}", token);
//!     Ok(())
//! }
//! ```

/// Módulo de autenticação OAuth2
pub mod auth;

/// Módulo de cliente API
pub mod client;

/// Módulo de configuração
pub mod config;

/// Módulo de tratamento de erros
pub mod error;

// Re-exportações para conveniência
pub use auth::oauth::OAuthFlow;
pub use client::api::ClickUpClient;
pub use config::EnvManager;
pub use error::{AuthError, AuthResult};