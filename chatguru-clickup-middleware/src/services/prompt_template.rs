use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::utils::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptTemplate {
    pub system_role: String,
    pub task_description: String,
    pub categories_template: String,
    pub activity_types_template: String,
    pub status_template: String,
    pub subcategories_template: String,
    pub rules: Vec<String>,
    pub mapping_examples: Vec<MappingExample>,
    pub response_format: String,
    pub fallback_values: FallbackValues,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MappingExample {
    pub categoria: String,
    pub exemplos_subcategorias: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FallbackValues {
    pub categories: Vec<String>,
    pub activity_types: Vec<ActivityType>,
    pub status_options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityType {
    pub name: String,
    pub description: String,
}

impl PromptTemplate {
    /// Carrega o template do arquivo YAML
    pub fn from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read template file: {}", e)))?;
        
        let template: PromptTemplate = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YAML template: {}", e)))?;
        
        Ok(template)
    }
    
    /// Carrega o template padrão
    pub fn load_default() -> AppResult<Self> {
        Self::from_file("config/ai_prompt.yaml")
    }
    
    /// Renderiza o prompt completo com valores dinâmicos
    pub fn render(
        &self,
        context: &str,
        dynamic_values: Option<DynamicValues>,
    ) -> String {
        let values = dynamic_values.unwrap_or_else(|| self.get_fallback_dynamic_values());
        
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
        
        // Categories (substituir template)
        let categories_section = self.categories_template
            .replace("{{categories_list}}", &values.categories_list);
        prompt.push_str(&categories_section);
        prompt.push_str("\n");
        
        // Activity types (substituir template)
        let activity_types_section = self.activity_types_template
            .replace("{{activity_types_list}}", &values.activity_types_list);
        prompt.push_str(&activity_types_section);
        prompt.push_str("\n");
        
        // Status (substituir template)
        let status_section = self.status_template
            .replace("{{status_list}}", &values.status_list);
        prompt.push_str(&status_section);
        prompt.push_str("\n");
        
        // Subcategories se houver
        if !values.subcategories_list.is_empty() {
            let subcategories_section = self.subcategories_template
                .replace("{{subcategories_list}}", &values.subcategories_list);
            prompt.push_str(&subcategories_section);
            prompt.push_str("\n");
        }
        
        // Rules
        prompt.push_str("REGRAS IMPORTANTES:\n");
        for rule in &self.rules {
            prompt.push_str(&format!("- {}\n", rule));
        }
        prompt.push_str("\n");
        
        // Response format (substituir placeholders com orientações)
        let response_section = self.response_format
            .replace("{{activity_types_exact}}", "um dos valores da lista TIPO DE ATIVIDADE")
            .replace("{{categories_exact}}", "um dos valores da lista CATEGORIAS")
            .replace("{{subcategories_exact}}", "um dos valores da lista SUBCATEGORIAS")
            .replace("{{status_exact}}", "um dos valores da lista STATUS BACK OFFICE");
        
        prompt.push_str(&response_section);
        
        prompt
    }
    
    /// Converte valores de fallback para DynamicValues
    fn get_fallback_dynamic_values(&self) -> DynamicValues {
        DynamicValues {
            categories_list: self.fallback_values.categories
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            activity_types_list: self.fallback_values.activity_types
                .iter()
                .map(|t| format!("- {} ({})", t.name, t.description))
                .collect::<Vec<_>>()
                .join("\n"),
            status_list: self.fallback_values.status_options
                .iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n"),
            subcategories_list: String::new(), // Vazio nos fallbacks
        }
    }
}

/// Valores dinâmicos para substituir no template
#[derive(Debug, Clone)]
pub struct DynamicValues {
    pub categories_list: String,
    pub activity_types_list: String,
    pub status_list: String,
    pub subcategories_list: String,
}

impl DynamicValues {
    /// Cria valores dinâmicos a partir de listas
    pub fn from_lists(
        categories: Vec<String>,
        activity_types: Vec<(String, String)>, // (name, description)
        status_options: Vec<String>,
        subcategories: Vec<String>,
    ) -> Self {
        Self {
            categories_list: categories
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            activity_types_list: activity_types
                .iter()
                .map(|(name, desc)| format!("- {} ({})", name, desc))
                .collect::<Vec<_>>()
                .join("\n"),
            status_list: status_options
                .iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n"),
            subcategories_list: if subcategories.is_empty() {
                String::new()
            } else {
                // Limitar subcategorias para não ficar muito grande
                let limited: Vec<String> = subcategories
                    .iter()
                    .take(20)
                    .map(|s| format!("- {}", s))
                    .collect();
                
                if subcategories.len() > 20 {
                    format!("{}\n... e mais {} opções", 
                        limited.join("\n"), 
                        subcategories.len() - 20
                    )
                } else {
                    limited.join("\n")
                }
            },
        }
    }
}