//! Handler para mensagens do Pub/Sub
//!
//! Recebe logs do Cloud Logging via Pub/Sub quando uma tarefa é criada.
//! Extrai o task_id e aciona o fluxo de enriquecimento.

use axum::{
    extract::State,
    response::Json,
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, error, warn};

use crate::AppState;
use crate::services::{
    log_parser::extract_task_id_from_log,
    task_fetcher::fetch_task_and_check_fields,
    task_classifier::classify_task,
    field_validator::validate_and_get_field_ids,
    task_enricher::enrich_task,
};

/// Payload do Pub/Sub
#[derive(Debug, Deserialize)]
pub struct PubSubPayload {
    pub message: PubSubMessage,
}

#[derive(Debug, Deserialize)]
pub struct PubSubMessage {
    pub data: String,
    #[serde(default)]
    pub attributes: Option<Value>,
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
}

/// Resposta do handler
#[derive(Debug, Serialize)]
pub struct EnrichResponse {
    pub success: bool,
    pub task_id: Option<String>,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<ClassificationResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClassificationResult {
    pub categoria: String,
    pub subcategoria: String,
    pub stars: u8,
}

/// Handler principal que recebe mensagens do Pub/Sub
pub async fn handle_pubsub_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PubSubPayload>,
) -> Json<Value> {
    let message_id = payload.message.message_id.clone().unwrap_or_else(|| "unknown".to_string());
    info!("📥 Recebida mensagem do Pub/Sub: {}", message_id);

    // 1. Decodificar o log entry do base64
    let log_data = match general_purpose::STANDARD.decode(&payload.message.data) {
        Ok(bytes) => {
            match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => {
                    error!("❌ Erro ao converter bytes para string: {}", e);
                    return Json(json!({
                        "success": false,
                        "error": format!("Invalid UTF-8: {}", e)
                    }));
                }
            }
        }
        Err(e) => {
            error!("❌ Erro ao decodificar base64: {}", e);
            return Json(json!({
                "success": false,
                "error": format!("Base64 decode error: {}", e)
            }));
        }
    };

    info!("🔍 Log recebido: {}", &log_data[..std::cmp::min(200, log_data.len())]);

    // 2. Extrair task_id do log
    let task_id = match extract_task_id_from_log(&log_data) {
        Some(id) => {
            info!("✅ Task ID extraído: {}", id);
            id
        }
        None => {
            warn!("⚠️ Não foi possível extrair task_id do log");
            return Json(json!({
                "success": false,
                "error": "Could not extract task_id from log"
            }));
        }
    };

    // 3. Buscar tarefa no ClickUp e verificar campos
    let (task, fields_empty) = match fetch_task_and_check_fields(&state.clickup_client, &task_id, &state.prompt_config).await {
        Ok(result) => result,
        Err(e) => {
            error!("❌ Erro ao buscar tarefa: {}", e);
            return Json(json!({
                "success": false,
                "task_id": task_id,
                "error": format!("Failed to fetch task: {}", e)
            }));
        }
    };

    // 4. Se campos já estão preenchidos, retornar
    if !fields_empty {
        info!("✅ Tarefa {} já possui campos preenchidos", task_id);
        return Json(json!({
            "success": true,
            "task_id": task_id,
            "action": "skipped",
            "reason": "Campos já preenchidos"
        }));
    }

    // 5. Usar IA Service para classificar a tarefa
    let ia_service = match &state.ia_service {
        Some(service) => service,
        None => {
            error!("❌ IA Service não disponível");
            return Json(json!({
                "success": false,
                "task_id": task_id,
                "error": "IA Service not available"
            }));
        }
    };

    let classification = match classify_task(ia_service, &state.prompt_config, &task).await {
        Ok(c) => {
            info!("✅ Classificação: {:?}", c);
            c
        }
        Err(e) => {
            error!("❌ Erro ao classificar tarefa: {}", e);
            return Json(json!({
                "success": false,
                "task_id": task_id,
                "error": format!("Classification failed: {}", e)
            }));
        }
    };

    // 6. Validar se valores existem nas opções do YAML
    let field_values = match validate_and_get_field_ids(
        &state.prompt_config,
        &classification.categoria,
        &classification.subcategoria,
    ) {
        Ok(values) => {
            info!("✅ Valores validados: categoria_id={}, subcategoria_id={}, stars={}",
                values.categoria_id, values.subcategoria_id, values.stars);
            values
        }
        Err(e) => {
            error!("❌ Validação falhou: {}", e);
            return Json(json!({
                "success": false,
                "task_id": task_id,
                "error": format!("Validation failed: {}", e)
            }));
        }
    };

    // 7. Atualizar tarefa com campos validados
    match enrich_task(&state.clickup_client, &task_id, &state.prompt_config, &field_values).await {
        Ok(_) => {
            info!("🎉 Tarefa {} enriquecida com sucesso!", task_id);
            Json(json!({
                "success": true,
                "task_id": task_id,
                "action": "enriched",
                "classification": {
                    "categoria": classification.categoria,
                    "subcategoria": classification.subcategoria,
                    "stars": field_values.stars
                }
            }))
        }
        Err(e) => {
            error!("❌ Erro ao enriquecer tarefa: {}", e);
            Json(json!({
                "success": false,
                "task_id": task_id,
                "error": format!("Enrichment failed: {}", e)
            }))
        }
    }
}

