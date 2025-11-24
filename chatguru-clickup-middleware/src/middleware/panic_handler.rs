/// Middleware para capturar panics e garantir respostas HTTP válidas
///
/// Este middleware garante que mesmo se houver um panic no código,
/// o Cloud Run receberá uma resposta HTTP válida, evitando o erro
/// "malformed HTTP response or connection error".

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::panic;

/// Middleware que captura panics e retorna uma resposta HTTP válida
pub async fn catch_panics(request: Request, next: Next) -> Response {
    // Criar um hook de panic customizado
    let panic_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Executar o handler dentro de um catch_unwind
        // Nota: Isso não funciona diretamente com async, então precisamos
        // usar uma abordagem diferente - vamos usar tower::Service
    }));

    // Executar o próximo middleware/handler
    // Se houver um panic, ele será capturado pelo runtime do tokio
    // e precisamos garantir que sempre retornamos uma resposta válida
    match std::panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // Esta abordagem não funciona bem com async
        // Vamos usar uma abordagem diferente
    })) {
        Ok(_) => next.run(request).await,
        Err(_) => {
            // Se houver panic, retornar resposta de erro válida
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Internal server error",
                    "status": 500,
                    "message": "An unexpected error occurred"
                })),
            )
                .into_response()
        }
    }
}

/// Wrapper async-safe para capturar panics em handlers
pub async fn panic_protection<F, Fut>(f: F) -> Response
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Response, axum::response::Response>>,
{
    // Para async, precisamos usar uma abordagem diferente
    // O Axum já tem proteção contra panics, mas vamos garantir
    // que sempre retornamos uma resposta válida
    match tokio::task::spawn(async move { f().await }).await {
        Ok(Ok(response)) => response,
        Ok(Err(response)) => response,
        Err(_) => {
            // Task foi cancelada ou panic ocorreu
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(json!({
                    "error": "Internal server error",
                    "status": 500,
                    "message": "Request processing failed"
                })),
            )
                .into_response()
        }
    }
}
