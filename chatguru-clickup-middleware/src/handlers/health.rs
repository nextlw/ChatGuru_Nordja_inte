use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

pub async fn health_check() -> Json<Value> {
    log_health_check();
    
    Json(json!({
        "status": "healthy",
        "service": "chatguru-clickup-middleware",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn ready_check(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    log_integration_status_check();
    
    // Testa a conexão com ClickUp
    let clickup_status = match state.clickup.test_connection().await {
        Ok(_) => "connected",
        Err(_) => "disconnected"
    };
    
    // PubSub é opcional - marcar como não disponível por enquanto
    let pubsub_status = "not_configured";
    
    let overall_ready = clickup_status == "connected";
    
    let response = json!({
        "ready": overall_ready,
        "service": "chatguru-clickup-middleware",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "dependencies": {
            "clickup": {
                "status": clickup_status,
                "list_id": state.settings.clickup.list_id
            },
            "pubsub": {
                "status": pubsub_status,
                "topic": state.settings.gcp.topic_name,
                "project": state.settings.gcp.project_id
            }
        }
    });
    
    if overall_ready {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
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
        match state.clickup.get_list_info().await {
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
    
    // Verificar configuração de AI
    let ai_enabled = state.settings.ai.as_ref().map_or(false, |ai| ai.enabled);
    
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
        "ai_enabled": ai_enabled,
        "chatguru_configured": chatguru_configured,
        "integrations": {
            "clickup": clickup_info,
            "pubsub": pubsub_info,
            "vertex_ai": {
                "enabled": ai_enabled,
                "region": "us-central1",  // Gemini só está disponível em us-central1
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