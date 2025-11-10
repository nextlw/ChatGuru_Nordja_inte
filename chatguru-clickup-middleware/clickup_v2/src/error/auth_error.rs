use thiserror::Error;

/// Tipos de erro específicos para autenticação OAuth2
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Token de acesso expirado ou inválido")]
    TokenExpired,

    #[error("Código de autorização inválido: {0}")]
    InvalidCode(String),

    #[error("Erro de rede: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Erro de parsing de URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Erro de IO: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Erro de variável de ambiente: {0}")]
    EnvError(String),

    #[error("Erro de serialização: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Erro do servidor de callback: {0}")]
    CallbackServerError(String),

    #[error("Acesso negado pelo usuário")]
    AccessDenied,

    #[error("Estado OAuth2 inválido")]
    InvalidState,

    #[error("Configuração inválida: {0}")]
    ConfigError(String),

    #[error("Erro do navegador: {0}")]
    BrowserError(String),

    #[error("Timeout durante autenticação")]
    Timeout,
    
    #[error("Erro de token: {0}")]
    TokenError(String),
    
    #[error("Erro de API: {0}")]
    ApiError(String),
    
    #[error("Erro de parsing: {0}")]
    ParseError(String),

    #[error("Erro genérico: {0}")]
    Generic(String),
}

impl AuthError {
    pub fn env_error(msg: impl Into<String>) -> Self {
        Self::EnvError(msg.into())
    }

    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    pub fn callback_error(msg: impl Into<String>) -> Self {
        Self::CallbackServerError(msg.into())
    }

    pub fn browser_error(msg: impl Into<String>) -> Self {
        Self::BrowserError(msg.into())
    }
    
    pub fn token_error(msg: impl Into<String>) -> Self {
        Self::TokenError(msg.into())
    }
    
    pub fn api_error(msg: impl Into<String>) -> Self {
        Self::ApiError(msg.into())
    }
    
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }
    
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn oauth_error(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
}

/// Tipo de resultado padrão para operações de autenticação
pub type AuthResult<T> = Result<T, AuthError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let token_expired = AuthError::TokenExpired;
        assert_eq!(token_expired.to_string(), "Token de acesso expirado ou inválido");

        let invalid_code = AuthError::InvalidCode("ABC123".to_string());
        assert_eq!(invalid_code.to_string(), "Código de autorização inválido: ABC123");

        let access_denied = AuthError::AccessDenied;
        assert_eq!(access_denied.to_string(), "Acesso negado pelo usuário");

        let invalid_state = AuthError::InvalidState;
        assert_eq!(invalid_state.to_string(), "Estado OAuth2 inválido");

        let timeout = AuthError::Timeout;
        assert_eq!(timeout.to_string(), "Timeout durante autenticação");
    }

    #[test]
    fn test_env_error_constructor() {
        let error = AuthError::env_error("ENV_VAR not found");
        assert_eq!(error.to_string(), "Erro de variável de ambiente: ENV_VAR not found");

        // Teste com String
        let error = AuthError::env_error(String::from("Another env error"));
        assert_eq!(error.to_string(), "Erro de variável de ambiente: Another env error");
    }

    #[test]
    fn test_config_error_constructor() {
        let error = AuthError::config_error("Invalid configuration");
        assert_eq!(error.to_string(), "Configuração inválida: Invalid configuration");
    }

    #[test]
    fn test_callback_error_constructor() {
        let error = AuthError::callback_error("Server failed to start");
        assert_eq!(error.to_string(), "Erro do servidor de callback: Server failed to start");
    }

    #[test]
    fn test_browser_error_constructor() {
        let error = AuthError::browser_error("Failed to open browser");
        assert_eq!(error.to_string(), "Erro do navegador: Failed to open browser");
    }

    #[test]
    fn test_token_error_constructor() {
        let error = AuthError::token_error("Invalid token format");
        assert_eq!(error.to_string(), "Erro de token: Invalid token format");
    }

    #[test]
    fn test_api_error_constructor() {
        let error = AuthError::api_error("API rate limit exceeded");
        assert_eq!(error.to_string(), "Erro de API: API rate limit exceeded");
    }

    #[test]
    fn test_parse_error_constructor() {
        let error = AuthError::parse_error("Failed to parse response");
        assert_eq!(error.to_string(), "Erro de parsing: Failed to parse response");
    }

    #[test]
    fn test_network_error_constructor() {
        let error = AuthError::network_error("Connection timeout");
        assert_eq!(error.to_string(), "Erro genérico: Connection timeout");
    }

    #[test]
    fn test_generic_error_constructor() {
        let error = AuthError::generic("Something went wrong");
        assert_eq!(error.to_string(), "Erro genérico: Something went wrong");
    }

    #[test]
    fn test_url_parse_error_from() {
        let url_error = url::Url::parse("not-a-valid-url").unwrap_err();
        let auth_error = AuthError::from(url_error);
        assert!(auth_error.to_string().contains("Erro de parsing de URL"));
    }

    #[test]
    fn test_io_error_from() {
        use std::io::{Error, ErrorKind};
        let io_error = Error::new(ErrorKind::NotFound, "File not found");
        let auth_error = AuthError::from(io_error);
        assert!(auth_error.to_string().contains("Erro de IO"));
    }

    #[test]
    fn test_serialization_error_from() {
        let json_str = "{invalid json}";
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        if let Err(json_error) = parse_result {
            let auth_error = AuthError::from(json_error);
            assert!(auth_error.to_string().contains("Erro de serialização"));
        }
    }

    #[test]
    fn test_auth_result_type() {
        fn returns_auth_result() -> AuthResult<String> {
            Ok("Success".to_string())
        }

        fn returns_auth_error() -> AuthResult<String> {
            Err(AuthError::TokenExpired)
        }

        assert!(returns_auth_result().is_ok());
        assert_eq!(returns_auth_result().unwrap(), "Success");

        assert!(returns_auth_error().is_err());
        assert!(matches!(returns_auth_error().unwrap_err(), AuthError::TokenExpired));
    }

    #[test]
    fn test_error_debug_format() {
        let error = AuthError::InvalidCode("XYZ".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidCode"));
        assert!(debug_str.contains("XYZ"));
    }
}