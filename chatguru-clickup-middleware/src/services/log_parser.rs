//! Parser de logs do Cloud Logging
//!
//! Extrai task_id de logs de criação de tarefa.
//! Padrões suportados:
//! - "🎉 Task criada - ID: abc123"
//! - "✅ Task criada com sucesso: Nome (abc123)"
//! - "Tarefa criada com sucesso!" (busca ID no contexto)

use regex::Regex;
use serde_json::Value;
use tracing::{info, warn};

/// Extrai task_id de um log entry
///
/// Suporta múltiplos formatos de log:
/// - Texto puro com padrões conhecidos
/// - JSON com campo textPayload
pub fn extract_task_id_from_log(log_data: &str) -> Option<String> {
    // Tentar parsear como JSON primeiro
    if let Ok(json) = serde_json::from_str::<Value>(log_data) {
        // Tentar extrair de textPayload
        if let Some(text_payload) = json.get("textPayload").and_then(|v| v.as_str()) {
            if let Some(id) = extract_from_text(text_payload) {
                return Some(id);
            }
        }

        // Tentar extrair de jsonPayload.message
        if let Some(message) = json.get("jsonPayload")
            .and_then(|jp| jp.get("message"))
            .and_then(|m| m.as_str())
        {
            if let Some(id) = extract_from_text(message) {
                return Some(id);
            }
        }
    }

    // Se não é JSON, tentar como texto puro
    extract_from_text(log_data)
}

/// Extrai task_id de uma string de texto
fn extract_from_text(text: &str) -> Option<String> {
    // Padrão 1: "🎉 Task criada - ID: abc123"
    // Padrão 2: "Task criada - ID: abc123"
    let re_task_id = Regex::new(r"Task criada\s*-\s*ID:\s*([a-zA-Z0-9]+)").ok()?;
    if let Some(caps) = re_task_id.captures(text) {
        if let Some(id) = caps.get(1) {
            info!("📍 Task ID encontrado (padrão 1): {}", id.as_str());
            return Some(id.as_str().to_string());
        }
    }

    // Padrão 2: "✅ Task criada com sucesso: Nome (abc123)"
    let re_success = Regex::new(r"Task criada com sucesso:\s*[^(]+\(([a-zA-Z0-9]+)\)").ok()?;
    if let Some(caps) = re_success.captures(text) {
        if let Some(id) = caps.get(1) {
            info!("📍 Task ID encontrado (padrão 2): {}", id.as_str());
            return Some(id.as_str().to_string());
        }
    }

    // Padrão 3: "Tarefa 'Nome' e subtarefas criadas com sucesso" - buscar ID próximo
    // Este padrão não tem ID diretamente, mas podemos tentar extrair de contexto
    if text.contains("Tarefa") && text.contains("criada") {
        // Tentar encontrar qualquer ID de tarefa no formato do ClickUp (ex: 9 dígitos)
        let re_clickup_id = Regex::new(r"\b(\d{9,12})\b").ok()?;
        if let Some(caps) = re_clickup_id.captures(text) {
            if let Some(id) = caps.get(1) {
                info!("📍 Task ID encontrado (padrão numérico): {}", id.as_str());
                return Some(id.as_str().to_string());
            }
        }

        // Tentar ID alfanumérico curto (ex: abc123def)
        let re_alpha_id = Regex::new(r"ID[:\s]+([a-zA-Z0-9]{6,12})").ok()?;
        if let Some(caps) = re_alpha_id.captures(text) {
            if let Some(id) = caps.get(1) {
                info!("📍 Task ID encontrado (padrão alfanumérico): {}", id.as_str());
                return Some(id.as_str().to_string());
            }
        }
    }

    // Padrão 4: task_id em JSON-like structure
    let re_json_id = Regex::new(r#"["']?task_id["']?\s*[=:]\s*["']?([a-zA-Z0-9]+)["']?"#).ok()?;
    if let Some(caps) = re_json_id.captures(text) {
        if let Some(id) = caps.get(1) {
            info!("📍 Task ID encontrado (padrão JSON): {}", id.as_str());
            return Some(id.as_str().to_string());
        }
    }

    warn!("⚠️ Não foi possível extrair task_id do texto");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_task_id_pattern1() {
        let log = "🎉 Task criada - ID: abc123xyz";
        assert_eq!(extract_task_id_from_log(log), Some("abc123xyz".to_string()));
    }

    #[test]
    fn test_extract_task_id_pattern2() {
        let log = "✅ Task criada com sucesso: Reembolso Médico (def456ghi)";
        assert_eq!(extract_task_id_from_log(log), Some("def456ghi".to_string()));
    }

    #[test]
    fn test_extract_task_id_numeric() {
        let log = "Tarefa criada com ID 901322079100";
        assert_eq!(extract_task_id_from_log(log), Some("901322079100".to_string()));
    }

    #[test]
    fn test_extract_task_id_json() {
        let log = r#"{"textPayload": "🎉 Task criada - ID: xyz789abc"}"#;
        assert_eq!(extract_task_id_from_log(log), Some("xyz789abc".to_string()));
    }

    #[test]
    fn test_extract_task_id_none() {
        let log = "Mensagem sem ID de tarefa";
        assert_eq!(extract_task_id_from_log(log), None);
    }
}

