//! Configuração do Prompt de IA
//!
//! Carrega e gerencia as configurações de mapeamento de categorias,
//! subcategorias e estrelas do arquivo ai_prompt.yaml.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::Datelike;
use tracing::info;

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
    #[serde(default)]
    pub cliente_solicitante_mappings: HashMap<String, String>,
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
    pub stars_field_id: String,
}

// Estruturas auxiliares para parsing do YAML
#[derive(Debug, Deserialize)]
struct YamlCategoryField {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
    #[serde(rename = "type_config")]
    type_config: YamlTypeConfig,
}

#[derive(Debug, Deserialize)]
struct YamlTypeConfig {
    options: Vec<YamlOption>,
}

#[derive(Debug, Deserialize)]
struct YamlOption {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct YamlAiPromptConfig {
    system_role: String,
    task_description: String,
    #[serde(default)]
    categories: Vec<String>,
    activity_types: Vec<ActivityType>,
    status_options: Vec<StatusOption>,
    category_mappings: Vec<YamlCategoryField>,
    #[serde(default)]
    subcategory_mappings: HashMap<String, Vec<SubcategoryMapping>>,
    #[serde(default)]
    subcategory_examples: HashMap<String, Vec<String>>,
    rules: Vec<String>,
    response_format: String,
    #[serde(default)]
    field_ids: Option<FieldIds>,
    #[serde(default)]
    cliente_solicitante_mappings: HashMap<String, String>,
}

impl AiPromptConfig {
    /// Carrega a configuração do prompt de um arquivo YAML
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        info!("📄 Carregando configuração do prompt: {:?}", path.as_ref());

        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Falha ao ler arquivo: {}", e))?;

        // Parse usando a estrutura YAML auxiliar
        let yaml_config: YamlAiPromptConfig = serde_yaml::from_str(&contents)
            .map_err(|e| format!("Falha ao parsear YAML: {}", e))?;

        // Converter category_mappings de Vec<YamlCategoryField> para HashMap
        let mut category_mappings = HashMap::new();
        for field in yaml_config.category_mappings {
            for option in field.type_config.options {
                category_mappings.insert(
                    option.name.clone(),
                    CategoryMapping { id: option.id.clone() },
                );
            }
        }

        // Extrair lista de categorias dos options se não estiver especificada
        let categories = if yaml_config.categories.is_empty() {
            category_mappings.keys().cloned().collect()
        } else {
            yaml_config.categories
        };

        info!(
            "✅ Configuração carregada: {} categorias, {} subcategorias",
            categories.len(),
            yaml_config.subcategory_mappings.values().map(|v| v.len()).sum::<usize>()
        );

        Ok(AiPromptConfig {
            system_role: yaml_config.system_role,
            task_description: yaml_config.task_description,
            categories,
            activity_types: yaml_config.activity_types,
            status_options: yaml_config.status_options,
            category_mappings,
            subcategory_mappings: yaml_config.subcategory_mappings,
            subcategory_examples: yaml_config.subcategory_examples,
            rules: yaml_config.rules,
            response_format: yaml_config.response_format,
            field_ids: yaml_config.field_ids,
            cliente_solicitante_mappings: yaml_config.cliente_solicitante_mappings,
        })
    }

    /// Gera o prompt formatado para classificação
    pub fn generate_prompt(&self, context: &str) -> String {
        let now = chrono::Local::now();
        let current_month = match now.month() {
            1 => "JANEIRO", 2 => "FEVEREIRO", 3 => "MARÇO",
            4 => "ABRIL", 5 => "MAIO", 6 => "JUNHO",
            7 => "JULHO", 8 => "AGOSTO", 9 => "SETEMBRO",
            10 => "OUTUBRO", 11 => "NOVEMBRO", 12 => "DEZEMBRO",
            _ => "DESCONHECIDO"
        };

        let current_date = format!(
            "DATA ATUAL: {} (Mês: {}, Ano: {})",
            now.format("%Y-%m-%d"),
            current_month,
            now.format("%Y")
        );

        let mut prompt = String::new();

        // System role
        prompt.push_str(&self.system_role);
        prompt.push_str("\n\n");

        // Context com data
        prompt.push_str("CONTEXTO DA MENSAGEM:\n");
        prompt.push_str(&current_date);
        prompt.push_str("\n");
        prompt.push_str(context);
        prompt.push_str("\n\n");

        // Task description
        prompt.push_str(&self.task_description);
        prompt.push_str("\n\n");

        // Categories
        prompt.push_str("CATEGORIAS DISPONÍVEIS NO CLICKUP:\n");
        for category in &self.categories {
            prompt.push_str(&format!("- {}\n", category));
        }
        prompt.push_str("\n");

        // Activity types
        prompt.push_str("TIPO DE ATIVIDADE:\n");
        for activity_type in &self.activity_types {
            prompt.push_str(&format!("- {} ({})\n",
                activity_type.name,
                activity_type.description
            ));
        }
        prompt.push_str("\n");

        // Status options
        prompt.push_str("STATUS BACK OFFICE:\n");
        for status in &self.status_options {
            prompt.push_str(&format!("- {}\n", status.name));
        }
        prompt.push_str("\n");

        // Subcategories
        prompt.push_str("SUBCATEGORIAS DISPONÍVEIS (por categoria):\n");
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
        self.category_mappings.get(name).map(|cm| cm.id.clone())
    }

    /// Obtém lista de categorias disponíveis
    pub fn get_available_categories(&self) -> Vec<String> {
        self.category_mappings.keys().cloned().collect()
    }

    /// Obtém o ID da subcategoria pelo nome e categoria (case-insensitive)
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

    /// Obtém subcategorias disponíveis para uma categoria
    pub fn get_subcategories_for_category(&self, category: &str) -> Vec<String> {
        self.subcategory_mappings.get(category.trim())
            .map(|subcats| subcats.iter().map(|sc| sc.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Obtém o número de estrelas de uma subcategoria (case-insensitive)
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

    /// Obtém os IDs dos campos customizados
    pub fn get_field_ids(&self) -> Option<&FieldIds> {
        self.field_ids.as_ref()
    }

    /// Valida se uma subcategoria pertence à categoria especificada
    pub fn validate_subcategory(&self, category: &str, subcategory: &str) -> bool {
        self.get_subcategory_id(category, subcategory).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        if Path::new("config/ai_prompt.yaml").exists() {
            let config = AiPromptConfig::load_from_yaml("config/ai_prompt.yaml").unwrap();
            assert!(!config.categories.is_empty());
        }
    }
}

