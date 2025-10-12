//! # ClickUp OAuth2 Authentication Module
//!
//! Módulo completamente isolado para gerenciar autenticação OAuth2 com ClickUp API.
//!
//! ## Responsabilidades:
//! - Iniciar fluxo OAuth2 (authorization URL)
//! - Trocar authorization code por access token
//! - Gerenciar tokens (armazenamento, refresh, validação)
//! - Verificar workspaces autorizados
//! - Fornecer tokens válidos para API calls
//!
//! ## Estrutura:
//! - `config.rs`: Configurações OAuth2
//! - `client.rs`: Cliente HTTP OAuth2
//! - `token_manager.rs`: Gerenciamento de tokens
//! - `handlers.rs`: Handlers HTTP (start_oauth, callback)

pub mod config;
pub mod client;
pub mod token_manager;
pub mod handlers;

pub use config::OAuth2Config;
pub use client::OAuth2Client;
pub use token_manager::TokenManager;
pub use handlers::{start_oauth_flow, handle_oauth_callback, OAuth2State};
