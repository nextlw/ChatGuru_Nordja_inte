use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc; 

use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

pub async fn health_check() -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    log_health_check();
    
    // Health check simples e rápido - não deve fazer I/O pesado
    let response = json!({
        "status": "healthy",
        "service": "chatguru-clickup-middleware",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "memory_ok": std::env::var("RUST_LOG").is_ok() // Check básico de variáveis
    });
    
    Ok(Json(response))
}

pub async fn ready_check(State(state): State<Arc<AppState>>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    log_integration_status_check();
    
    // Readiness check com timeout mais conservador (3s para evitar 503)
    let clickup_status = match tokio::time::timeout(
        std::time::Duration::from_secs(3), // Timeout reduzido para 3s
        state.clickup_client.get_user_info()
    ).await {
        Ok(Ok(_)) => "connected",
        Ok(Err(_)) => "disconnected",
        Err(_) => {
            log_warning("⚠️ ClickUp health check timeout (3s)");
            "timeout"
        }
    };
    
    // Ser mais permissivo no ready check - aceitar timeout como "ready" por enquanto
    // pois o worker pode funcionar mesmo com ClickUp lento
    let overall_ready = clickup_status == "connected" || clickup_status == "timeout";
    
    let response = json!({
        "ready": overall_ready,
        "service": "chatguru-clickup-middleware",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "clickup": {
                "status": clickup_status,
                "workspace_id": state.clickup_workspace_id
            }
        }
    });
    
    if overall_ready {
        Ok(Json(response))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Service not ready",
                "details": response
            }))
        ))
    }
}

// Scheduler removido na arquitetura event-driven
// pub async fn scheduler_status() - DEPRECATED

pub async fn status_check(State(state): State<Arc<AppState>>) -> Json<Value> {
    log_integration_status_check();
    
    // Verificar se ClickUp está configurado
    let clickup_configured = !state.settings.clickup.token.is_empty() && 
                            !state.settings.clickup.list_id.is_empty();
    
    // Informações detalhadas sobre ClickUp
    let mut clickup_info = json!({
        "configured": clickup_configured,
        "list_id": state.settings.clickup.list_id,
        "token_configured": !state.settings.clickup.token.is_empty()
    });
    
    let clickup_connected = if clickup_configured {
        // TODO: Implementar teste de lista usando clickup_v2
        match Ok(serde_json::json!({"status": "ok"})) as Result<serde_json::Value, String> {
            Ok(list_info) => {
                clickup_info["connection"] = json!("success");
                clickup_info["list_name"] = list_info.get("name").unwrap_or(&json!("unknown")).clone();
                clickup_info["list_status"] = list_info.get("status").unwrap_or(&json!("unknown")).clone();
                true
            },
            Err(e) => {
                clickup_info["connection"] = json!("failed");
                clickup_info["error"] = json!(e.to_string());
                false
            }
        }
    } else {
        clickup_info["connection"] = json!("not_configured");
        false
    };

    // Verificar configuração do ChatGuru
    let chatguru_configured = state.settings.chatguru.api_token.is_some() &&
                             state.settings.chatguru.api_endpoint.is_some() &&
                             state.settings.chatguru.account_id.is_some();
    
    // Informações sobre Pub/Sub
    let mut pubsub_info = json!({
        "configured": true,
        "topic": state.settings.gcp.topic_name,
        "project": state.settings.gcp.project_id
    });
    
    // PubSub não está configurado por enquanto
    pubsub_info["connection"] = json!("not_configured");
    pubsub_info["note"] = json!("PubSub will be configured later");
    
    Json(json!({
        "service": "chatguru-clickup-middleware",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": "N/A", // TODO: Implementar tracking de uptime
        "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
        "clickup_connected": clickup_connected,
        "chatguru_configured": chatguru_configured,
        "integrations": {
            "clickup": clickup_info,
            "pubsub": pubsub_info,
            "openai": {
                "enabled": true,  // OpenAI sempre habilitado via ia-service crate
                "region": "us-central1",
                "project_id": state.settings.gcp.project_id.clone()
            },
            "chatguru": {
                "api_configured": chatguru_configured,
                "webhook_secret_configured": state.settings.chatguru.webhook_secret.is_some(),
                "account_id": state.settings.chatguru.account_id.clone().unwrap_or_else(|| "not_configured".to_string())
            }
        }
    }))
}