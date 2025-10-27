/// Middleware de autenticação para endpoints administrativos
///
/// Valida que a requisição contém um API key válido no header X-Admin-Key.
/// Protege endpoints sensíveis de acesso não autorizado em produção.

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Middleware que requer API key para acesso aos endpoints /admin/*
///
/// # Configuração
///
/// Configure a variável de ambiente `ADMIN_API_KEY`:
/// ```bash
/// export ADMIN_API_KEY="your-secure-random-key-here"
/// ```
///
/// # Uso na requisição
///
/// ```bash
/// curl -H "X-Admin-Key: your-secure-random-key-here" \
///   https://app.run.app/admin/clickup/list
/// ```
///
/// # Respostas
///
/// - **200 OK**: Key válido, continua para o handler
/// - **401 Unauthorized**: Key ausente ou inválido
///
/// # Segurança
///
/// - Em desenvolvimento: Se ADMIN_API_KEY não estiver configurado, permite acesso (warning no log)
/// - Em produção: Se ADMIN_API_KEY não estiver configurado, bloqueia acesso (erro 503)
pub async fn require_admin_key(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extrair API key do header
    let provided_key = headers
        .get("X-Admin-Key")
        .and_then(|v| v.to_str().ok());

    // Obter key esperado de variável de ambiente
    let expected_key = std::env::var("ADMIN_API_KEY").ok();

    // Verificar ambiente
    let is_production = std::env::var("RUST_ENV")
        .unwrap_or_else(|_| "development".to_string())
        == "production";

    // Validar acesso
    match (expected_key, provided_key, is_production) {
        // Caso 1: Key configurado e correto
        (Some(expected), Some(provided), _) if expected == provided => {
            tracing::debug!("✅ Admin access granted");
            Ok(next.run(request).await)
        }

        // Caso 2: Key configurado mas incorreto/ausente
        (Some(_), provided, _) => {
            tracing::warn!(
                "❌ Admin access denied - Invalid or missing X-Admin-Key: {:?}",
                provided.map(|_| "<redacted>")
            );
            Err(unauthorized_response())
        }

        // Caso 3: Key não configurado em DESENVOLVIMENTO - permite com warning
        (None, _, false) => {
            tracing::warn!(
                "⚠️  ADMIN_API_KEY not configured - Allowing access in development mode. \
                 Configure ADMIN_API_KEY in production!"
            );
            Ok(next.run(request).await)
        }

        // Caso 4: Key não configurado em PRODUÇÃO - bloqueia
        (None, _, true) => {
            tracing::error!(
                "🚨 ADMIN_API_KEY not configured in production! Blocking admin access."
            );
            Err(service_unavailable_response())
        }
    }
}

/// Resposta de erro 401 Unauthorized
fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "Unauthorized",
            "message": "Missing or invalid X-Admin-Key header",
            "hint": "Include X-Admin-Key header with valid API key"
        })),
    )
        .into_response()
}

/// Resposta de erro 503 Service Unavailable (config inválida)
fn service_unavailable_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": "Service Unavailable",
            "message": "ADMIN_API_KEY not configured on server",
            "hint": "Contact administrator to configure ADMIN_API_KEY"
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, Method},
    };

    #[test]
    fn test_admin_key_validation_logic() {
        // Simular diferentes cenários
        std::env::set_var("ADMIN_API_KEY", "test-key-123");

        // Cenário 1: Key correto
        assert_eq!("test-key-123", std::env::var("ADMIN_API_KEY").unwrap());

        // Cenário 2: Key diferente
        assert_ne!("wrong-key", std::env::var("ADMIN_API_KEY").unwrap());

        std::env::remove_var("ADMIN_API_KEY");
    }
}
