use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::utils::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiPromptConfig {
    pub system_role: String,
    pub task_description: String,
    pub categories: Vec<String>,
    pub activity_types: Vec<ActivityType>,
    pub status_options: Vec<StatusOption>,
    pub category_mappings: HashMap<String, CategoryMapping>,
    #[serde(default)]
    pub subcategory_mappings: HashMap<String, Vec<SubcategoryMapping>>,
    #[serde(default)]
    pub subcategory_examples: HashMap<String, Vec<String>>,
    pub rules: Vec<String>,
    pub response_format: String,
    #[serde(default)]
    pub field_ids: Option<FieldIds>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityType {
    pub name: String,
    pub description: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusOption {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryMapping {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubcategoryMapping {
    pub name: String,
    pub id: String,
    pub stars: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldIds {
    pub category_field_id: String,
    pub subcategory_field_id: String,
    pub activity_type_field_id: String,
    pub status_field_id: String,
}

impl AiPromptConfig {
    /// NOVO: Carrega configuração do banco de dados PostgreSQL
    pub async fn from_database(db: &PgPool) -> AppResult<Self> {
        // Se DB não está disponível, usa configuração padrão
        if db.is_closed() {
            return Self::load_default();
        }

        // Carregar configurações básicas
        let system_role = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'system_role' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Você é um assistente IA especializado em categorização de tarefas.".to_string());

        let task_description = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'task_description' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Analise a mensagem e extraia informações para criação de tarefa.".to_string());

        let response_format = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'response_format' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "JSON válido conforme especificação".to_string());

        // Carregar categorias
        let categories: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT name FROM categories WHERE is_active = true ORDER BY display_order"
        )
        .fetch_all(db)
        .await
        .unwrap_or_else(|_| vec!["Geral".to_string()]);

        // Carregar tipos de atividade usando runtime query
        let activity_types_rows = sqlx::query_as::<_, (String, Option<String>, String)>(
            "SELECT name, description, clickup_field_id FROM activity_types WHERE is_active = true"
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        let activity_types: Vec<ActivityType> = activity_types_rows
            .into_iter()
            .map(|(name, description, clickup_field_id)| ActivityType {
                name,
                description: description.unwrap_or_default(),
                id: clickup_field_id,
            })
            .collect();

        // Carregar status usando runtime query
        let status_rows = sqlx::query_as::<_, (String, String)>(
            "SELECT name, clickup_field_id FROM status_options WHERE is_active = true ORDER BY display_order"
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        let status_options: Vec<StatusOption> = status_rows
            .into_iter()
            .map(|(name, clickup_field_id)| StatusOption {
                name,
                id: clickup_field_id,
            })
            .collect();

        // Carregar mapeamentos de categorias usando runtime query
        let category_rows = sqlx::query_as::<_, (String, String)>(
            "SELECT name, clickup_field_id FROM categories WHERE is_active = true"
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        let mut category_mappings = HashMap::new();
        for (name, clickup_field_id) in category_rows {
            category_mappings.insert(
                name.clone(),
                CategoryMapping {
                    id: clickup_field_id,
                },
            );
        }

        // Carregar subcategorias usando runtime query
        let subcategory_rows = sqlx::query_as::<_, (String, String, String, Option<i32>)>(
            r#"
            SELECT c.name as category_name, s.name as sub_name, s.clickup_field_id, s.stars
            FROM subcategories s
            JOIN categories c ON s.category_id = c.id
            WHERE s.is_active = true
            ORDER BY c.display_order, s.display_order
            "#
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        let mut subcategory_mappings: HashMap<String, Vec<SubcategoryMapping>> = HashMap::new();
        for (category_name, sub_name, clickup_field_id, stars) in subcategory_rows {
            let mapping = SubcategoryMapping {
                name: sub_name,
                id: clickup_field_id,
                stars: stars.unwrap_or(1) as u8,
            };

            subcategory_mappings
                .entry(category_name)
                .or_insert_with(Vec::new)
                .push(mapping);
        }

        // Carregar regras usando runtime query
        let rules: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT rule_text FROM prompt_rules WHERE is_active = true ORDER BY display_order"
        )
        .fetch_all(db)
        .await
        .unwrap_or_default();

        // Carregar field IDs usando runtime queries com fallbacks
        let category_field_id = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'category_field_id' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "default_category_field".to_string());

        let subcategory_field_id = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'subcategory_field_id' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "default_subcategory_field".to_string());

        let activity_type_field_id = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'activity_type_field_id' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "default_activity_field".to_string());

        let status_field_id = sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = 'status_field_id' AND is_active = true"
        )
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "default_status_field".to_string());

        Ok(AiPromptConfig {
            system_role,
            task_description,
            categories,
            activity_types,
            status_options,
            category_mappings,
            subcategory_mappings,
            subcategory_examples: HashMap::new(), // Pode ser populado depois se necessário
            rules,
            response_format,
            field_ids: Some(FieldIds {
                category_field_id,
                subcategory_field_id,
                activity_type_field_id,
                status_field_id,
            }),
        })
    }

    /// Carrega a configuração do prompt de um arquivo YAML (fallback/legacy)
    pub fn from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read prompt file: {}", e)))?;

        let config: AiPromptConfig = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YAML: {}", e)))?;

        Ok(config)
    }

    /// Carrega a configuração padrão do diretório config (fallback/legacy)
    pub fn load_default() -> AppResult<Self> {
        Self::from_file("config/ai_prompt.yaml")
    }
    
    /// Gera o prompt formatado para o Vertex AI
    pub fn generate_prompt(&self, context: &str) -> String {
        self.generate_prompt_with_dynamic_fields(context, None, None, None)
    }
    
    /// Gera o prompt com campos dinâmicos do ClickUp
    pub fn generate_prompt_with_dynamic_fields(
        &self, 
        context: &str,
        dynamic_categories: Option<Vec<String>>,
        dynamic_activity_types: Option<Vec<(String, String)>>, // (name, description)
        dynamic_status_options: Option<Vec<String>>
    ) -> String {
        let mut prompt = String::new();
        
        // System role
        prompt.push_str(&self.system_role);
        prompt.push_str("\n\n");
        
        // Context
        prompt.push_str("CONTEXTO DA MENSAGEM:\n");
        prompt.push_str(context);
        prompt.push_str("\n\n");
        
        // Task description
        prompt.push_str(&self.task_description);
        prompt.push_str("\n\n");
        
        // Categories - usar dinâmicas se disponíveis
        prompt.push_str("CATEGORIAS DISPONÍVEIS NO CLICKUP:\n");
        if let Some(categories) = dynamic_categories {
            for category in categories {
                prompt.push_str(&format!("- {}\n", category));
            }
        } else {
            // Fallback para categorias estáticas do YAML
            for category in &self.categories {
                prompt.push_str(&format!("- {}\n", category));
            }
        }
        prompt.push_str("\n");
        
        // Activity types - usar dinâmicos se disponíveis
        prompt.push_str("TIPO DE ATIVIDADE:\n");
        if let Some(types) = dynamic_activity_types {
            for (name, description) in types {
                prompt.push_str(&format!("- {} ({})\n", name, description));
            }
        } else {
            // Fallback para tipos estáticos do YAML
            for activity_type in &self.activity_types {
                prompt.push_str(&format!("- {} ({})\n", 
                    activity_type.name, 
                    activity_type.description
                ));
            }
        }
        prompt.push_str("\n");
        
        // Status options - usar dinâmicos se disponíveis
        prompt.push_str("STATUS BACK OFFICE:\n");
        if let Some(statuses) = dynamic_status_options {
            for status in statuses {
                prompt.push_str(&format!("- {}\n", status));
            }
        } else {
            // Fallback para status estáticos do YAML
            for status in &self.status_options {
                prompt.push_str(&format!("- {}\n", status.name));
            }
        }
        prompt.push_str("\n");
        
        // Subcategories - usar mapeamentos completos com IDs e estrelas
        prompt.push_str("SUBCATEGORIAS DISPONÍVEIS (por categoria):\n");
        if !self.subcategory_mappings.is_empty() {
            for (category, subcats) in &self.subcategory_mappings {
                prompt.push_str(&format!("\n{}:\n", category));
                for subcat in subcats {
                    prompt.push_str(&format!("  - {} ({} estrela{})\n",
                        subcat.name,
                        subcat.stars,
                        if subcat.stars > 1 { "s" } else { "" }
                    ));
                }
            }
        } else if !self.subcategory_examples.is_empty() {
            // Fallback para exemplos antigos se subcategory_mappings não estiver disponível
            for (category, examples) in &self.subcategory_examples {
                let examples_str = examples.iter()
                    .map(|e| format!("\"{}\"", e))
                    .collect::<Vec<_>>()
                    .join(", ");
                prompt.push_str(&format!("- {} → {}\n", category, examples_str));
            }
        }
        prompt.push_str("\n");
        
        // Rules
        prompt.push_str("REGRAS IMPORTANTES:\n");
        for rule in &self.rules {
            prompt.push_str(&format!("- {}\n", rule));
        }
        prompt.push_str("\n");
        
        // Response format
        prompt.push_str(&self.response_format);
        
        prompt
    }
    
    /// Obtém o ID da categoria pelo nome
    pub fn get_category_id(&self, name: &str) -> Option<String> {
        self.category_mappings.get(name)
            .map(|cm| cm.id.clone())
    }
    
    /// Obtém o ID do status pelo nome
    pub fn get_status_id(&self, name: &str) -> Option<String> {
        self.status_options.iter()
            .find(|so| so.name == name)
            .map(|so| so.id.clone())
    }

    /// Obtém o ID da subcategoria pelo nome e categoria (busca case-insensitive)
    pub fn get_subcategory_id(&self, category: &str, subcategory: &str) -> Option<String> {
        let category_normalized = category.trim();
        let subcategory_normalized = subcategory.trim().to_lowercase();

        self.subcategory_mappings.get(category_normalized)
            .and_then(|subcats| {
                subcats.iter()
                    .find(|sc| sc.name.to_lowercase() == subcategory_normalized)
                    .map(|sc| sc.id.clone())
            })
    }

    /// Obtém o número de estrelas de uma subcategoria (busca case-insensitive)
    pub fn get_subcategory_stars(&self, category: &str, subcategory: &str) -> Option<u8> {
        let category_normalized = category.trim();
        let subcategory_normalized = subcategory.trim().to_lowercase();

        self.subcategory_mappings.get(category_normalized)
            .and_then(|subcats| {
                subcats.iter()
                    .find(|sc| sc.name.to_lowercase() == subcategory_normalized)
                    .map(|sc| sc.stars)
            })
    }

    /// Obtém todas as subcategorias de uma categoria
    pub fn get_subcategories_for_category(&self, category: &str) -> Vec<String> {
        let category_normalized = category.trim();

        self.subcategory_mappings.get(category_normalized)
            .map(|subcats| {
                subcats.iter()
                    .map(|sc| sc.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Obtém os IDs dos campos customizados
    pub fn get_field_ids(&self) -> Option<&FieldIds> {
        self.field_ids.as_ref()
    }

    /// Valida se uma subcategoria pertence à categoria especificada
    pub fn validate_subcategory(&self, category: &str, subcategory: &str) -> bool {
        self.get_subcategory_id(category, subcategory).is_some()
    }

    /// Obtém informações completas de uma subcategoria
    pub fn get_subcategory_info(&self, category: &str, subcategory: &str) -> Option<SubcategoryMapping> {
        let category_normalized = category.trim();
        let subcategory_normalized = subcategory.trim().to_lowercase();

        self.subcategory_mappings.get(category_normalized)
            .and_then(|subcats| {
                subcats.iter()
                    .find(|sc| sc.name.to_lowercase() == subcategory_normalized)
                    .cloned()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_prompt_config() {
        // Este teste só funcionará se o arquivo YAML existir
        if Path::new("config/ai_prompt.yaml").exists() {
            let config = AiPromptConfig::load_default().unwrap();
            assert!(!config.categories.is_empty());
            assert!(!config.activity_types.is_empty());
        }
    }
}