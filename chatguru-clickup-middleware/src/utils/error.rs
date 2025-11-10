use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ClickUpApi(String),
    PubSubError(String),
    ConfigError(String),
    DatabaseError(String),
    JsonError(serde_json::Error),
    HttpError(reqwest::Error),
    ValidationError(String),
    InternalError(String),
    Timeout(String),
    StructureNotFound(String),
    ServiceUnavailable(String),
}

#[derive(Debug)]
pub enum SecretsError {
    ClientNotAvailable,
    AccessError(String),
    DecodingError(String),
}

impl fmt::Display for SecretsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretsError::ClientNotAvailable => write!(f, "Secret Manager client not available"),
            SecretsError::AccessError(msg) => write!(f, "Secret access error: {}", msg),
            SecretsError::DecodingError(msg) => write!(f, "Secret decoding error: {}", msg),
        }
    }
}

impl std::error::Error for SecretsError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ClickUpApi(msg) => write!(f, "ClickUp API error: {}", msg),
            AppError::PubSubError(msg) => write!(f, "Pub/Sub error: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::JsonError(err) => write!(f, "JSON error: {}", err),
            AppError::HttpError(err) => write!(f, "HTTP error: {}", err),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
            AppError::StructureNotFound(msg) => write!(f, "Structure not found: {}", msg),
            AppError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonError(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::HttpError(err)
    }
}

impl From<chatguru::ChatGuruError> for AppError {
    fn from(err: chatguru::ChatGuruError) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<ia_service::IaServiceError> for AppError {
    fn from(err: ia_service::IaServiceError) -> Self {
        AppError::InternalError(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::InternalError(err)
    }
}

// Database error conversion desabilitado (sem banco)
// impl From<sqlx::Error> for AppError {
//     fn from(err: sqlx::Error) -> Self {
//         AppError::ConfigError(format!("Database error: {}", err))
//     }
// }

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ClickUpApi(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::PubSubError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ConfigError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::JsonError(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::HttpError(err) => (StatusCode::BAD_GATEWAY, err.to_string()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Timeout(msg) => (StatusCode::GATEWAY_TIMEOUT, msg),
            AppError::StructureNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        };

        let body = json!({
            "error": error_message,
            "status": status.as_u16()
        });

        (status, axum::Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;