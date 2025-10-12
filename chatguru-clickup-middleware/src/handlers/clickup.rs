use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use chatguru_clickup_middleware::utils::AppError;
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

pub async fn list_clickup_tasks(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/tasks", "GET");
    
    let url = format!("https://api.clickup.com/api/v2/list/{}/task", state.settings.clickup.list_id);
    
    let response = state.clickup_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", &state.settings.clickup.token))
        .send()
        .await?;

    let status = response.status();
    
    if status.is_success() {
        let tasks: Value = response.json().await?;
        Ok(Json(json!({
            "success": true,
            "tasks": tasks.get("tasks").unwrap_or(&json!([])),
            "list_id": state.settings.clickup.list_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
        Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
    }
}

pub async fn get_clickup_list_info(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/list", "GET");
    
    match state.clickup.get_list_info().await {
        Ok(list_info) => {
            Ok(Json(json!({
                "success": true,
                "list": list_info,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            log_clickup_api_error("get_list_info", None, &e.to_string());
            Err(e)
        }
    }
}

pub async fn test_clickup_connection(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/test", "GET");
    
    match state.clickup.test_connection().await {
        Ok(user_info) => {
            Ok(Json(json!({
                "success": true,
                "message": "ClickUp connection successful",
                "user": user_info,
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            log_clickup_api_error("test_connection", None, &e.to_string());
            Ok(Json(json!({
                "success": false,
                "message": "ClickUp connection failed",
                "error": e.to_string(),
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}