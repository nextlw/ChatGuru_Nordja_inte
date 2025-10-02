use serde::{Deserialize, Serialize};
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
    /// Carrega a configuração do prompt de um arquivo YAML
    pub fn from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read prompt file: {}", e)))?;
        
        let config: AiPromptConfig = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YAML: {}", e)))?;
        
        Ok(config)
    }
    
    /// Carrega a configuração padrão do diretório config
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