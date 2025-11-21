/// Estrutura enriquecida para tarefas com contexto completo
/// Usada para passar mais informações à IA na detecção de duplicatas
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTask {
    pub title: String,
    pub created_at: i64,  // timestamp em milissegundos
    pub description_preview: String, // primeiros 200 chars
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub status: String,
    pub custom_fields: HashMap<String, String>,
}

impl EnrichedTask {
    /// Cria EnrichedTask a partir de uma task do ClickUp (formato JSON)
    pub fn from_clickup_task_json(
        task: &Value,
        category_field_id: &str,
        subcategory_field_id: &str,
    ) -> Self {
        let title = task.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

        // Extrair timestamp de criação
        let created_at = task.get("date_created")
            .and_then(|d| d.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        // Extrair descrição (preview)
        let description_preview = task.get("description")
            .and_then(|d| d.as_str())
            .or_else(|| task.get("text_content").and_then(|t| t.as_str()))
            .map(|d| d.chars().take(200).collect())
            .unwrap_or_default();

        // Extrair status
        let status = task.get("status")
            .and_then(|s| s.get("status"))
            .and_then(|st| st.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Extrair categoria e subcategoria dos custom_fields
        let (category, subcategory) = Self::extract_category_info(
            task,
            category_field_id,
            subcategory_field_id,
        );

        // Extrair outros custom_fields relevantes
        let mut custom_fields = HashMap::new();
        if let Some(fields) = task.get("custom_fields").and_then(|f| f.as_array()) {
            for field in fields {
                if let (Some(_id), Some(name)) = (
                    field.get("id").and_then(|i| i.as_str()),
                    field.get("name").and_then(|n| n.as_str()),
                ) {
                    // Extrair valor do campo
                    let value = if let Some(value) = field.get("value") {
                        if let Some(s) = value.as_str() {
                            s.to_string()
                        } else if let Some(n) = value.as_number() {
                            n.to_string()
                        } else if let Some(arr) = value.as_array() {
                            arr.iter()
                                .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                                .collect::<Vec<_>>()
                                .join(", ")
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };

                    custom_fields.insert(name.to_string(), value);
                }
            }
        }

        Self {
            title,
            created_at,
            description_preview,
            category,
            subcategory,
            status,
            custom_fields,
        }
    }

    /// Extrai categoria e subcategoria dos custom_fields
    fn extract_category_info(
        task: &Value,
        category_field_id: &str,
        subcategory_field_id: &str,
    ) -> (Option<String>, Option<String>) {
        let mut category = None;
        let mut subcategory = None;

        if let Some(fields) = task.get("custom_fields").and_then(|f| f.as_array()) {
            for field in fields {
                if let Some(id) = field.get("id").and_then(|i| i.as_str()) {
                    if id == category_field_id {
                        // Extrair valor da categoria
                        if let Some(value) = field.get("value") {
                            if let Some(name) = value.as_str() {
                                category = Some(name.to_string());
                            } else if let Some(arr) = value.as_array() {
                                if let Some(first) = arr.first() {
                                    if let Some(name) = first.get("name").and_then(|n| n.as_str()) {
                                        category = Some(name.to_string());
                                    }
                                }
                            }
                        }
                    } else if id == subcategory_field_id {
                        // Extrair valor da subcategoria
                        if let Some(value) = field.get("value") {
                            if let Some(name) = value.as_str() {
                                subcategory = Some(name.to_string());
                            } else if let Some(arr) = value.as_array() {
                                if let Some(first) = arr.first() {
                                    if let Some(name) = first.get("name").and_then(|n| n.as_str()) {
                                        subcategory = Some(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        (category, subcategory)
    }
}

