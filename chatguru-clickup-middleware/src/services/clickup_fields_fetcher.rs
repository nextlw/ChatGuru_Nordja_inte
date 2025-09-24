use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_config: Option<TypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<DropdownOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldsCache {
    pub fields: Vec<CustomField>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Clone)]
pub struct ClickUpFieldsFetcher {
    client: Client,
    token: String,
    list_id: String,
    cache: Arc<RwLock<Option<FieldsCache>>>,
    cache_duration_minutes: i64,
}

impl ClickUpFieldsFetcher {
    pub fn new(token: String, list_id: String) -> Self {
        Self {
            client: Client::new(),
            token,
            list_id,
            cache: Arc::new(RwLock::new(None)),
            cache_duration_minutes: 15, // Cache por 15 minutos
        }
    }
    
    /// Busca os campos customizados do ClickUp (com cache)
    pub async fn get_custom_fields(&self) -> AppResult<Vec<CustomField>> {
        // Verificar cache primeiro
        let cache_guard = self.cache.read().await;
        if let Some(ref cached) = *cache_guard {
            let age = Utc::now() - cached.last_updated;
            if age < Duration::minutes(self.cache_duration_minutes) {
                log_info(&format!("Using cached ClickUp fields (age: {} minutes)", age.num_minutes()));
                return Ok(cached.fields.clone());
            }
        }
        drop(cache_guard);
        
        // Cache expirado ou não existe, buscar novos dados
        log_info("Fetching fresh ClickUp custom fields from API");
        let fields = self.fetch_fields_from_api().await?;
        
        // Atualizar cache
        let mut cache_guard = self.cache.write().await;
        *cache_guard = Some(FieldsCache {
            fields: fields.clone(),
            last_updated: Utc::now(),
        });
        
        Ok(fields)
    }
    
    /// Busca campos diretamente da API do ClickUp
    async fn fetch_fields_from_api(&self) -> AppResult<Vec<CustomField>> {
        let url = format!("https://api.clickup.com/api/v2/list/{}/field", self.list_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.token)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to fetch fields: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ClickUpApi(format!("Failed to get custom fields: {}", error_text)));
        }
        
        #[derive(Deserialize)]
        struct FieldsResponse {
            fields: Vec<CustomField>,
        }
        
        let fields_response: FieldsResponse = response
            .json()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to parse fields response: {}", e)))?;
        
        Ok(fields_response.fields)
    }
    
    
    /// Gera lista de categorias disponíveis
    pub async fn get_available_categories(&self) -> AppResult<Vec<String>> {
        let fields = self.get_custom_fields().await?;
        
        if let Some(categoria_field) = fields.iter().find(|f| f.name == "Categoria") {
            if let Some(ref type_config) = categoria_field.type_config {
                if let Some(ref options) = type_config.options {
                    return Ok(options.iter().map(|o| o.name.clone()).collect());
                }
            }
        }
        
        Ok(vec![])
    }
    
    /// Gera lista de subcategorias disponíveis
    pub async fn get_available_subcategories(&self) -> AppResult<Vec<String>> {
        let fields = self.get_custom_fields().await?;
        
        if let Some(subcategoria_field) = fields.iter().find(|f| f.name == "Sub Categoria") {
            if let Some(ref type_config) = subcategoria_field.type_config {
                if let Some(ref options) = type_config.options {
                    return Ok(options.iter().map(|o| o.name.clone()).collect());
                }
            }
        }
        
        Ok(vec![])
    }
    
    #[allow(dead_code)]
    /// Obtém campo específico por nome - pode ser útil no futuro
    pub async fn get_field_by_name(&self, name: &str) -> AppResult<Option<CustomField>> {
        let fields = self.get_custom_fields().await?;
        Ok(fields.into_iter().find(|f| f.name == name))
    }
    
    #[allow(dead_code)]
    /// Gera prompt atualizado com campos dinâmicos - mantido para possível uso futuro
    pub async fn generate_dynamic_prompt_section(&self) -> AppResult<String> {
        let fields = self.get_custom_fields().await?;
        let mut prompt = String::new();
        
        // Categorias
        if let Some(categoria_field) = fields.iter().find(|f| f.name == "Categoria") {
            prompt.push_str("CATEGORIAS DISPONÍVEIS NO CLICKUP:\n");
            if let Some(ref type_config) = categoria_field.type_config {
                if let Some(ref options) = type_config.options {
                    for option in options {
                        prompt.push_str(&format!("- {}\n", option.name));
                    }
                }
            }
            prompt.push_str("\n");
        }
        
        // Tipo de Atividade
        if let Some(tipo_field) = fields.iter().find(|f| f.name == "Tipo de Atividade") {
            prompt.push_str("TIPO DE ATIVIDADE:\n");
            if let Some(ref type_config) = tipo_field.type_config {
                if let Some(ref options) = type_config.options {
                    for option in options {
                        prompt.push_str(&format!("- {}\n", option.name));
                    }
                }
            }
            prompt.push_str("\n");
        }
        
        // Status Back Office
        if let Some(status_field) = fields.iter().find(|f| f.name == "Status Back Office") {
            prompt.push_str("STATUS BACK OFFICE:\n");
            if let Some(ref type_config) = status_field.type_config {
                if let Some(ref options) = type_config.options {
                    for option in options {
                        prompt.push_str(&format!("- {}\n", option.name));
                    }
                }
            }
            prompt.push_str("\n");
        }
        
        // Subcategorias
        if let Some(sub_field) = fields.iter().find(|f| f.name == "Sub Categoria") {
            prompt.push_str("SUBCATEGORIAS DISPONÍVEIS:\n");
            if let Some(ref type_config) = sub_field.type_config {
                if let Some(ref options) = type_config.options {
                    // Agrupar por categoria se possível
                    for option in options.iter().take(10) { // Limitar para não ficar muito grande
                        prompt.push_str(&format!("- {}\n", option.name));
                    }
                    if options.len() > 10 {
                        prompt.push_str(&format!("... e mais {} opções\n", options.len() - 10));
                    }
                }
            }
            prompt.push_str("\n");
        }
        
        Ok(prompt)
    }
    
    /// Retorna todos os mapeamentos de IDs necessários
    pub async fn get_all_field_mappings(&self) -> AppResult<FieldMappings> {
        let fields = self.get_custom_fields().await?;
        let mut mappings = FieldMappings::default();
        
        for field in fields {
            match field.name.as_str() {
                "Categoria" => {
                    mappings.category_field_id = field.id.clone();
                    if let Some(ref type_config) = field.type_config {
                        if let Some(ref options) = type_config.options {
                            for option in options {
                                mappings.categories.insert(option.name.clone(), option.id.clone());
                            }
                        }
                    }
                },
                "Tipo de Atividade" => {
                    mappings.activity_type_field_id = field.id.clone();
                    if let Some(ref type_config) = field.type_config {
                        if let Some(ref options) = type_config.options {
                            for option in options {
                                mappings.activity_types.insert(option.name.clone(), option.id.clone());
                            }
                        }
                    }
                },
                "Status Back Office" => {
                    mappings.status_field_id = field.id.clone();
                    if let Some(ref type_config) = field.type_config {
                        if let Some(ref options) = type_config.options {
                            for option in options {
                                mappings.status_options.insert(option.name.clone(), option.id.clone());
                            }
                        }
                    }
                },
                "Sub Categoria" => {
                    mappings.subcategory_field_id = field.id.clone();
                    if let Some(ref type_config) = field.type_config {
                        if let Some(ref options) = type_config.options {
                            for option in options {
                                mappings.subcategories.insert(option.name.clone(), option.id.clone());
                            }
                        }
                    }
                },
                "Solicitante (Info_1)" => {
                    mappings.info1_field_id = field.id.clone();
                },
                "Outro cliente" => {
                    mappings.info2_field_id = field.id.clone();
                },
                _ => {}
            }
        }
        
        Ok(mappings)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FieldMappings {
    pub categories: HashMap<String, String>,
    pub subcategories: HashMap<String, String>,
    pub activity_types: HashMap<String, String>,
    pub status_options: HashMap<String, String>,
    pub category_field_id: String,
    pub subcategory_field_id: String,
    pub activity_type_field_id: String,
    pub status_field_id: String,
    pub info1_field_id: String,
    pub info2_field_id: String,
}