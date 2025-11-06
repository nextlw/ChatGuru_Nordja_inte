//! Tipos de erro para o crate clickup

use thiserror::Error;

/// Erros do cliente ClickUp
#[derive(Debug, Error)]
pub enum ClickUpError {
    /// Erro de requisição HTTP
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Erro da API do ClickUp (status code não-200)
    #[error("ClickUp API error (status {status}): {message}")]
    ApiError { status: u16, message: String },

    /// Erro de autenticação
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Erro de parsing JSON
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Recurso não encontrado (folder, list, task, etc)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Erro de configuração
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Timeout de operação
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Erro de validação
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Tipo Result padrão para o crate
pub type Result<T> = std::result::Result<T, ClickUpError>;
