use crate::client::ClickUpClient;
use crate::error::Result;
/// Custom Field Manager: Gerencia campos personalizados do ClickUp
///
/// Responsabilidades:
/// 1. Garantir que "Cliente Solicitante" sempre corresponda ao nome do folder
/// 2. Criar opções no dropdown automaticamente se não existirem
/// 3. Sincronizar folder_name → Cliente Solicitante field value
use serde::Deserialize;
use serde_json::Value;

const CLIENT_SOLICITANTE_FIELD_ID: &str = "0ed63eec-1c50-4190-91c1-59b4b17557f6";

#[derive(Debug, Deserialize)]
struct ClickUpCustomField {
    id: String,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    field_type: String,
    #[serde(default)]
    type_config: Option<CustomFieldTypeConfig>,
}

#[derive(Debug, Deserialize)]
struct CustomFieldTypeConfig {
    #[serde(default)]
    options: Vec<CustomFieldOption>,
}

#[derive(Debug, Deserialize, Clone)]
struct CustomFieldOption {
    #[allow(dead_code)]
    id: Option<String>,
    name: String,
    #[allow(dead_code)]
    #[serde(default)]
    color: Option<String>,
    #[allow(dead_code)]
    #[serde(default)]
    orderindex: Option<i32>,
}

pub struct CustomFieldManager {
    client: ClickUpClient,
}

impl CustomFieldManager {
    /// Criar novo manager
    pub fn new(client: ClickUpClient) -> Self {
        Self { client }
    }

    /// Criar novo manager a partir de API token (conveniência)
    pub fn from_token(api_token: String) -> Result<Self> {
        let client = ClickUpClient::new(api_token)?;
        Ok(Self::new(client))
    }

    /// Garante que o campo "Cliente Solicitante" tem o valor correto
    ///
    /// Fluxo:
    /// 1. Recebe folder_name encontrado pelo SmartFolderFinder
    /// 2. Normaliza o nome (remove números, parênteses)
    /// 3. Verifica se existe opção no dropdown
    /// 4. Se não existir, cria a opção
    /// 5. Retorna o custom_field formatado para task_data
    pub async fn ensure_client_solicitante_option(
        &self,
        list_id: &str,
        folder_name: &str,
    ) -> Result<Value> {
        tracing::info!(
            "🔧 Garantindo opção 'Cliente Solicitante' para folder: '{}'",
            folder_name
        );

        // Normalizar nome do folder (remover números e parênteses)
        let client_name = self.normalize_folder_name(folder_name);

        tracing::info!("📝 Nome normalizado do cliente: '{}'", client_name);

        // Buscar campo personalizado na lista
        let custom_fields = self.get_list_custom_fields(list_id).await?;

        // Encontrar campo "Cliente Solicitante"
        let client_field = custom_fields
            .iter()
            .find(|f| f.id == CLIENT_SOLICITANTE_FIELD_ID)
            .ok_or_else(|| {
                crate::error::ClickUpError::ConfigError(format!(
                    "Campo 'Cliente Solicitante' (ID: {}) não encontrado na lista {}",
                    CLIENT_SOLICITANTE_FIELD_ID, list_id
                ))
            })?;

        // Verificar se a opção já existe
        let empty_vec = vec![];
        let existing_options = client_field
            .type_config
            .as_ref()
            .map(|tc| &tc.options)
            .unwrap_or(&empty_vec);

        // Buscar opção exata ou similar
        let option_match = self.find_matching_option(existing_options, &client_name);

        let option_value = match option_match {
            Some(option) => {
                tracing::info!("✅ Opção já existe: '{}' (usando existente)", option.name);
                option.name.clone()
            }
            None => {
                tracing::warn!(
                    "⚠️ Opção '{}' não existe no dropdown, criando...",
                    client_name
                );

                // Criar nova opção no dropdown
                self.create_dropdown_option(list_id, CLIENT_SOLICITANTE_FIELD_ID, &client_name)
                    .await?;

                client_name.clone()
            }
        };

        // Retornar custom_field formatado para task_data
        Ok(serde_json::json!({
            "id": CLIENT_SOLICITANTE_FIELD_ID,
            "value": option_value
        }))
    }

    /// Normaliza nome do folder (remove números e parênteses)
    fn normalize_folder_name(&self, folder_name: &str) -> String {
        // Remover números entre parênteses: "Raphaela Spielberg (10)" → "Raphaela Spielberg"
        let without_numbers = folder_name
            .chars()
            .filter(|c| !c.is_numeric() && *c != '(' && *c != ')')
            .collect::<String>()
            .trim()
            .to_string();

        // Remover espaços extras
        without_numbers
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Busca opção correspondente no dropdown (exata ou fuzzy)
    fn find_matching_option<'a>(
        &self,
        options: &'a [CustomFieldOption],
        target_name: &str,
    ) -> Option<&'a CustomFieldOption> {
        // 1. Buscar match exato (case-insensitive)
        let target_lower = target_name.to_lowercase();

        for option in options {
            if option.name.to_lowercase() == target_lower {
                return Some(option);
            }
        }

        // 2. Buscar fuzzy match (Jaro-Winkler >= 0.90)
        let mut best_match: Option<(&CustomFieldOption, f64)> = None;

        for option in options {
            let similarity = strsim::jaro_winkler(&target_lower, &option.name.to_lowercase());

            if similarity >= 0.90 {
                if let Some((_, best_score)) = best_match {
                    if similarity > best_score {
                        best_match = Some((option, similarity));
                    }
                } else {
                    best_match = Some((option, similarity));
                }
            }
        }

        best_match.map(|(option, score)| {
            tracing::info!(
                "🔍 Fuzzy match encontrado: '{}' → '{}' (score: {:.2})",
                target_name,
                option.name,
                score
            );
            option
        })
    }

    /// Busca custom fields de uma lista (API v2)
    async fn get_list_custom_fields(&self, list_id: &str) -> Result<Vec<ClickUpCustomField>> {
        let endpoint = format!("/list/{}", list_id);
        let list_data: Value = self.client.get_json(&endpoint).await?;

        // Extrair custom_fields
        let custom_fields = list_data["custom_fields"].as_array().ok_or_else(|| {
            crate::error::ClickUpError::ValidationError("Lista sem custom_fields".to_string())
        })?;

        let fields: Vec<ClickUpCustomField> = custom_fields
            .iter()
            .filter_map(|f| serde_json::from_value(f.clone()).ok())
            .collect();

        tracing::info!("📋 Lista tem {} custom fields", fields.len());

        Ok(fields)
    }

    /// Cria nova opção no dropdown do campo personalizado (API v2)
    async fn create_dropdown_option(
        &self,
        list_id: &str,
        field_id: &str,
        option_name: &str,
    ) -> Result<()> {
        tracing::info!("➕ Criando opção '{}' no campo {}", option_name, field_id);

        // POST /list/{list_id}/field/{field_id}/option
        let endpoint = format!("/list/{}/field/{}/option", list_id, field_id);
        let payload = serde_json::json!({
            "name": option_name
        });

        let _response: Value = self.client.post_json(&endpoint, &payload).await?;

        tracing::info!("✅ Opção '{}' criada com sucesso", option_name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_folder_name() {
        let client = ClickUpClient::new("dummy").unwrap();
        let manager = CustomFieldManager::new(client);

        assert_eq!(
            manager.normalize_folder_name("Raphaela Spielberg (10)"),
            "Raphaela Spielberg"
        );
        assert_eq!(
            manager.normalize_folder_name("Bruno Assis (10)"),
            "Bruno Assis"
        );
        assert_eq!(
            manager.normalize_folder_name("Gabriel Benarros"),
            "Gabriel Benarros"
        );
        assert_eq!(
            manager.normalize_folder_name("Adriano Miranda (5)"),
            "Adriano Miranda"
        );
        assert_eq!(
            manager.normalize_folder_name("Alessandra Caiado (20)"),
            "Alessandra Caiado"
        );
    }

    #[test]
    fn test_find_matching_option() {
        let client = ClickUpClient::new("dummy").unwrap();
        let manager = CustomFieldManager::new(client);

        let options = vec![
            CustomFieldOption {
                id: Some("1".to_string()),
                name: "Raphaela Spielberg".to_string(),
                color: None,
                orderindex: None,
            },
            CustomFieldOption {
                id: Some("2".to_string()),
                name: "Bruno Assis".to_string(),
                color: None,
                orderindex: None,
            },
            CustomFieldOption {
                id: Some("3".to_string()),
                name: "Gabriel Benarros".to_string(),
                color: None,
                orderindex: None,
            },
        ];

        // Exact match
        let result = manager.find_matching_option(&options, "Raphaela Spielberg");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Raphaela Spielberg");

        // Case insensitive
        let result = manager.find_matching_option(&options, "raphaela spielberg");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Raphaela Spielberg");

        // Fuzzy match (typo)
        let result = manager.find_matching_option(&options, "Raphaela Spilberg");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Raphaela Spielberg");

        // No match
        let result = manager.find_matching_option(&options, "João Silva");
        assert!(result.is_none());
    }
}
