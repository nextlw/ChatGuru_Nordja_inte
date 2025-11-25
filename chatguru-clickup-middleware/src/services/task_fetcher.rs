//! Serviço para buscar tarefas no ClickUp
//!
//! Busca uma tarefa pelo ID e verifica se os campos
//! categoria_nova, subcategoria_nova e estrelas estão preenchidos.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

use super::prompts::AiPromptConfig;

/// Representa uma tarefa do ClickUp com campos relevantes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub custom_fields: Vec<CustomFieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub id: String,
    pub name: Option<String>,
    pub value: Option<Value>,
}

/// Busca uma tarefa no ClickUp e verifica se os campos estão vazios
///
/// Retorna (TaskInfo, campos_vazios: bool)
pub async fn fetch_task_and_check_fields(
    client: &Arc<clickup_v2::client::ClickUpClient>,
    task_id: &str,
    prompt_config: &AiPromptConfig,
) -> Result<(TaskInfo, bool), String> {
    info!("📥 Buscando tarefa {} no ClickUp", task_id);

    // Buscar tarefa via API
    let task_json = client
        .get_task(task_id)
        .await
        .map_err(|e| format!("Erro ao buscar tarefa: {}", e))?;

    // Extrair informações da tarefa
    let task_info = parse_task_json(&task_json)?;

    // Obter IDs dos campos do YAML
    let field_ids = prompt_config.get_field_ids()
        .ok_or("field_ids não encontrados no YAML")?;

    // Verificar se campos estão vazios
    let categoria_empty = is_field_empty(&task_info.custom_fields, &field_ids.category_field_id);
    let subcategoria_empty = is_field_empty(&task_info.custom_fields, &field_ids.subcategory_field_id);
    let stars_empty = is_field_empty(&task_info.custom_fields, &field_ids.stars_field_id);

    info!(
        "📋 Campos da tarefa {}: categoria_nova={}, subcategoria_nova={}, estrelas={}",
        task_id,
        if categoria_empty { "VAZIO" } else { "preenchido" },
        if subcategoria_empty { "VAZIO" } else { "preenchido" },
        if stars_empty { "VAZIO" } else { "preenchido" }
    );

    // Considerar campos vazios se ALGUM deles estiver vazio
    let fields_empty = categoria_empty || subcategoria_empty || stars_empty;

    Ok((task_info, fields_empty))
}

/// Parseia JSON da tarefa para TaskInfo
fn parse_task_json(json: &Value) -> Result<TaskInfo, String> {
    let id = json.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Campo 'id' não encontrado na tarefa")?
        .to_string();

    let name = json.get("name")
        .and_then(|v| v.as_str())
        .ok_or("Campo 'name' não encontrado na tarefa")?
        .to_string();

    let description = json.get("description")
        .or_else(|| json.get("text_content"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let custom_fields = json.get("custom_fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|cf| {
                    let id = cf.get("id")?.as_str()?.to_string();
                    let name = cf.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                    let value = cf.get("value").cloned();
                    Some(CustomFieldValue { id, name, value })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(TaskInfo {
        id,
        name,
        description,
        custom_fields,
    })
}

/// Verifica se um campo está vazio (null, não existe, ou valor vazio)
fn is_field_empty(fields: &[CustomFieldValue], field_id: &str) -> bool {
    match fields.iter().find(|f| f.id == field_id) {
        None => {
            warn!("⚠️ Campo {} não encontrado na tarefa", field_id);
            true
        }
        Some(field) => {
            match &field.value {
                None => true,
                Some(Value::Null) => true,
                Some(Value::String(s)) if s.is_empty() => true,
                Some(Value::Number(n)) if n.as_i64() == Some(0) => true,
                Some(Value::Array(arr)) if arr.is_empty() => true,
                Some(_) => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_task_json() {
        let json = json!({
            "id": "abc123",
            "name": "Reembolso Médico",
            "description": "Descrição da tarefa",
            "custom_fields": [
                {
                    "id": "field1",
                    "name": "Categoria",
                    "value": null
                },
                {
                    "id": "field2",
                    "name": "Subcategoria",
                    "value": "valor"
                }
            ]
        });

        let task = parse_task_json(&json).unwrap();
        assert_eq!(task.id, "abc123");
        assert_eq!(task.name, "Reembolso Médico");
        assert_eq!(task.custom_fields.len(), 2);
    }

    #[test]
    fn test_is_field_empty() {
        let fields = vec![
            CustomFieldValue { id: "f1".to_string(), name: None, value: None },
            CustomFieldValue { id: "f2".to_string(), name: None, value: Some(Value::Null) },
            CustomFieldValue { id: "f3".to_string(), name: None, value: Some(json!("valor")) },
        ];

        assert!(is_field_empty(&fields, "f1"));
        assert!(is_field_empty(&fields, "f2"));
        assert!(!is_field_empty(&fields, "f3"));
        assert!(is_field_empty(&fields, "inexistente"));
    }
}

