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
    pub subcategory_examples: HashMap<String, Vec<String>>,
    pub rules: Vec<String>,
    pub response_format: String,
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
        
        // Subcategory examples
        prompt.push_str("EXEMPLOS DE SUB-CATEGORIAS POR CATEGORIA:\n");
        for (category, examples) in &self.subcategory_examples {
            let examples_str = examples.iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ");
            prompt.push_str(&format!("- {} → {}\n", category, examples_str));
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