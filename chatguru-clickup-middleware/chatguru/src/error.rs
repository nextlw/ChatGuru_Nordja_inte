use thiserror::Error;

/// Erros específicos do cliente ChatGuru
#[derive(Error, Debug)]
pub enum ChatGuruError {
    /// Erro de rede ou HTTP
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Erro retornado pela API do ChatGuru
    #[error("ChatGuru API error: {0}")]
    ApiError(String),

    /// Erro de serialização/deserialização
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Dados inválidos
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Erro interno do cliente
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type para operações do ChatGuru
pub type Result<T> = std::result::Result<T, ChatGuruError>;

// Implementar conversão de reqwest::Error
impl From<reqwest::Error> for ChatGuruError {
    fn from(err: reqwest::Error) -> Self {
        ChatGuruError::NetworkError(err.to_string())
    }
}

// Implementar conversão de serde_json::Error
impl From<serde_json::Error> for ChatGuruError {
    fn from(err: serde_json::Error) -> Self {
        ChatGuruError::SerializationError(err.to_string())
    }
}
