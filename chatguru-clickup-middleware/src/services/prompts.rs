use serde::{Deserialize, Serialize};
// use sqlx::PgPool; // DESABILITADO - sem PostgreSQL
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::Datelike;
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
    #[serde(default)]
    pub cliente_solicitante_mappings: HashMap<String, String>,  // Nome → ID da opção
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

// Estruturas auxiliares para parsing do YAML com nested options
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

impl AiPromptConfig {
    /// Carrega a configuração do prompt de um arquivo YAML
    pub fn from_file<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        use crate::utils::logging::log_info;

        log_info(&format!("📄 Loading AI prompt config from YAML file: {:?}", path.as_ref()));

        let contents = fs::read_to_string(path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read prompt file: {}", e)))?;

        // Parse usando a estrutura YAML auxiliar
        let yaml_config: YamlAiPromptConfig = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YAML: {}", e)))?;

        // Converter category_mappings de Vec<YamlCategoryField> para HashMap<String, CategoryMapping>
        let mut category_mappings = HashMap::new();
        for field in yaml_config.category_mappings {
            // Extrair todas as options e criar mapeamento nome -> id
            for option in field.type_config.options {
                category_mappings.insert(
                    option.name.clone(),
                    CategoryMapping {
                        id: option.id.clone(),
                    },
                );
            }
        }

        // Extrair lista de categorias dos options se não estiver especificada
        let categories = if yaml_config.categories.is_empty() {
            category_mappings.keys().cloned().collect()
        } else {
            yaml_config.categories
        };

        log_info(&format!(
            "✅ AI prompt config loaded from YAML successfully: {} categories, {} activity_types, {} rules",
            categories.len(),
            yaml_config.activity_types.len(),
            yaml_config.rules.len()
        ));

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

    /// Carrega a configuração de GCS (Google Cloud Storage)
    pub async fn from_gcs(bucket: &str, object: &str) -> AppResult<Self> {
        use crate::utils::logging::{log_info, log_error};
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::get::GetObjectRequest;
        use google_cloud_storage::http::objects::download::Range;

        log_info(&format!("☁️  Loading AI prompt config from GCS: gs://{}/{}", bucket, object));

        // Create GCS client
        let config = ClientConfig::default().with_auth().await
            .map_err(|e| AppError::ConfigError(format!("Failed to create GCS client: {}", e)))?;

        let client = Client::new(config);

        // Download file
        let data = client
            .download_object(
                &GetObjectRequest {
                    bucket: bucket.to_string(),
                    object: object.to_string(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| {
                log_error(&format!("❌ Failed to download from GCS: {}", e));
                AppError::ConfigError(format!("Failed to download from GCS: {}", e))
            })?;

        let contents = String::from_utf8(data)
            .map_err(|e| AppError::ConfigError(format!("Invalid UTF-8 in GCS file: {}", e)))?;

        // Parse usando a estrutura YAML auxiliar
        let yaml_config: YamlAiPromptConfig = serde_yaml::from_str(&contents)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse YAML from GCS: {}", e)))?;

        // Converter category_mappings de Vec<YamlCategoryField> para HashMap<String, CategoryMapping>
        let mut category_mappings = HashMap::new();
        for field in yaml_config.category_mappings {
            for option in field.type_config.options {
                category_mappings.insert(
                    option.name.clone(),
                    CategoryMapping {
                        id: option.id.clone(),
                    },
                );
            }
        }

        let categories = if yaml_config.categories.is_empty() {
            category_mappings.keys().cloned().collect()
        } else {
            yaml_config.categories
        };

        log_info(&format!(
            "✅ AI prompt config loaded from GCS successfully: {} categories, {} activity_types, {} rules",
            categories.len(),
            yaml_config.activity_types.len(),
            yaml_config.rules.len()
        ));

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

    /// Carrega a configuração padrão (GCS ou arquivo local)
    pub async fn load_default() -> AppResult<Self> {
        use crate::utils::logging::{log_info, log_warning};

        // Verificar variável de ambiente para escolher fonte
        let source = std::env::var("AI_PROMPT_SOURCE").unwrap_or_else(|_| "gcs".to_string());

        match source.as_str() {
            "gcs" => {
                let bucket = std::env::var("AI_PROMPT_BUCKET")
                    .unwrap_or_else(|_| "chatguru-clickup-configs".to_string());
                let object = std::env::var("AI_PROMPT_OBJECT")
                    .unwrap_or_else(|_| "ai_prompt.yaml".to_string());

                log_info(&format!("🌐 Loading AI prompt from GCS: gs://{}/{}", bucket, object));

                match Self::from_gcs(&bucket, &object).await {
                    Ok(config) => Ok(config),
                    Err(e) => {
                        log_warning(&format!("⚠️  Failed to load from GCS: {}. Falling back to local file.", e));
                        Self::from_file("config/ai_prompt.yaml")
                    }
                }
            }
            "file" | _ => {
                log_warning("⚠️  USING LOCAL FILE: Loading AI prompt config from config/ai_prompt.yaml");
                Self::from_file("config/ai_prompt.yaml")
            }
        }
    }

    /// Load configuration with GCS fallback
    /// First tries to load from local YAML file, then from GCS bucket
    pub async fn load_with_gcs_fallback() -> AppResult<Self> {
        use crate::utils::logging::{log_info, log_warning, log_error};
        
        // 1. First attempt: try to load from local YAML file
        match Self::from_file("config/ai_prompt.yaml") {
            Ok(config) => {
                log_info("✅ Prompt config loaded from local YAML file");
                Ok(config)
            },
            Err(local_err) => {
                log_warning(&format!("⚠️ Local YAML file not found, trying GCS fallback: {}", local_err));
                
                // 2. Fallback: try to load from GCS bucket
                match Self::load_from_gcs().await {
                    Ok(config) => {
                        log_info("✅ Prompt config loaded from GCS bucket");
                        Ok(config)
                    },
                    Err(gcs_err) => {
                        log_error(&format!("❌ Both local and GCS loading failed. Local: {} | GCS: {}", local_err, gcs_err));
                        Err(AppError::InternalError(format!(
                            "Failed to load prompt config from both local file and GCS. Local: {} | GCS: {}",
                            local_err, gcs_err
                        )))
                    }
                }
            }
        }
    }

    /// Load configuration from GCS bucket (private method)
    async fn load_from_gcs() -> AppResult<Self> {
        use crate::utils::logging::{log_info, log_error};
        use google_cloud_storage::client::{Client, ClientConfig};
        use google_cloud_storage::http::objects::download::Range;
        use google_cloud_storage::http::objects::get::GetObjectRequest;
        
        let bucket_name = "chatguru-clickup-configs";
        let object_name = "ai_prompt.yaml";
        
        log_info(&format!("🌐 Attempting to load prompt config from GCS: gs://{}/{}", bucket_name, object_name));
        
        // Initialize GCS client
        let config = ClientConfig::default().with_auth().await.map_err(|e| {
            AppError::InternalError(format!("Failed to initialize GCS client: {}", e))
        })?;
        let client = Client::new(config);
        
        // Download the file content
        let request = GetObjectRequest {
            bucket: bucket_name.to_string(),
            object: object_name.to_string(),
            ..Default::default()
        };
        
        let response = client.download_object(&request, &Range::default()).await.map_err(|e| {
            log_error(&format!("❌ Failed to download from GCS: {}", e));
            AppError::InternalError(format!("Failed to download prompt config from GCS: {}", e))
        })?;
        
        // Parse YAML content
        let yaml_content = String::from_utf8(response).map_err(|e| {
            AppError::InternalError(format!("Failed to parse GCS content as UTF-8: {}", e))
        })?;
        
        // Parse usando a estrutura YAML auxiliar
        let yaml_config: YamlAiPromptConfig = serde_yaml::from_str(&yaml_content)
            .map_err(|e| AppError::InternalError(format!("Failed to parse YAML from GCS: {}", e)))?;
        
        // Converter category_mappings de Vec<YamlCategoryField> para HashMap<String, CategoryMapping>
        let mut category_mappings = HashMap::new();
        for field in yaml_config.category_mappings {
            for option in field.type_config.options {
                category_mappings.insert(
                    option.name.clone(),
                    CategoryMapping {
                        id: option.id.clone(),
                    },
                );
            }
        }
        
        let categories = if yaml_config.categories.is_empty() {
            category_mappings.keys().cloned().collect()
        } else {
            yaml_config.categories
        };
        
        log_info(&format!(
            "✅ Successfully loaded prompt config from GCS bucket: {} categories, {} activity_types, {} rules",
            categories.len(),
            yaml_config.activity_types.len(),
            yaml_config.rules.len()
        ));
        
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
    
    /// Gera o prompt formatado para o Vertex AI
    pub fn generate_prompt(&self, context: &str) -> String {
        // Adicionar data atual ao contexto
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

        let context_with_date = format!("{}\n\n{}", current_date, context);

        self.generate_prompt_with_dynamic_fields(&context_with_date, None, None, None)
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

    /// Obtém o ID do cliente solicitante pelo nome (com normalização + fuzzy matching)
    ///
    /// Estratégia de matching em 3 níveis:
    /// 1. Match exato (normalizado)
    /// 2. Match por normalização de chaves
    /// 3. Fuzzy matching com Jaro-Winkler (threshold 85%)
    /// Busca o ID do cliente solicitante usando múltiplas fontes (Info_2 e nome do contato)
    pub fn get_cliente_solicitante_id_multi(
        &self,
        info_2: Option<&str>,
        nome_contato: Option<&str>,
    ) -> Option<String> {
        use strsim::jaro_winkler;

        // 1. Tentar pelo Info_2 (campo principal)
        let normalized_info_2 = info_2.map(Self::normalize_client_name);
        let normalized_nome = nome_contato.map(Self::normalize_client_name);

        // 1a. Match exato ou normalizado via Info_2
        if let Some(ref info_2_val) = normalized_info_2 {
            if let Some(id) = self.cliente_solicitante_mappings.get(info_2_val) {
                tracing::info!("✅ Match exato encontrado via Info_2: '{}'", info_2_val);
                return Some(id.clone());
            }
        }

        // 1b. Match exato ou normalizado via nome do contato
        if let Some(ref nome_val) = normalized_nome {
            if let Some(id) = self.cliente_solicitante_mappings.get(nome_val) {
                tracing::info!("✅ Match exato encontrado via nome do contato: '{}'", nome_val);
                return Some(id.clone());
            }
        }

        // 3. Se não encontrou, tenta fuzzy matching cruzado entre ambos se ambos disponíveis
        if let (Some(info_2_val), Some(nome_val)) = (info_2, nome_contato) {
            let normalized_info_2 = Self::normalize_client_name(info_2_val);
            let normalized_nome = Self::normalize_client_name(nome_val);

            const FUZZY_THRESHOLD: f64 = 0.70;

            for (key, id) in &self.cliente_solicitante_mappings {
                let normalized_key = Self::normalize_client_name(key);
                let sim_info_2 = jaro_winkler(&normalized_info_2, &normalized_key);
                let sim_nome = jaro_winkler(&normalized_nome, &normalized_key);

                if sim_info_2 >= FUZZY_THRESHOLD || sim_nome >= FUZZY_THRESHOLD {
                    tracing::info!(
                        "✨ Fuzzy match cruzado: Info_2='{}', Nome='{}', Chave='{}', Score_info_2={:.1}%, Score_nome={:.1}%",
                        info_2_val, nome_val, key, sim_info_2 * 100.0, sim_nome * 100.0
                    );
                    return Some(id.clone());
                }
            }
        }

        // Não encontrado - log para debug
        tracing::warn!(
            "❌ Cliente não encontrado via Info_2='{:?}' ou nome_contato='{:?}'.",
            info_2, nome_contato
        );
        None
    }

    /// Normalizar nome de cliente usando deunicode + limpeza
    ///
    /// Processo:
    /// 1. Remove acentos (deunicode): "José" → "Jose", "Dadá" → "Dada"
    /// 2. Lowercase: "Jose" → "jose"
    /// 3. Remove parênteses e números: "Agência (2)" → "agencia"
    /// 4. Trim whitespace e normaliza espaços
    fn normalize_client_name(name: &str) -> String {
        use deunicode::deunicode;

        deunicode(name)  // Remove acentos primeiro
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_numeric() && *c != '(' && *c != ')')
            .collect::<String>()
            .split_whitespace()  // Remove espaços extras
            .collect::<Vec<&str>>()
            .join(" ")
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
    
    #[tokio::test]
    async fn test_load_prompt_config() {
        // Este teste só funcionará se o arquivo YAML existir
        if Path::new("config/ai_prompt.yaml").exists() {
            let config = AiPromptConfig::load_default().await.unwrap();
            assert!(!config.categories.is_empty());
            assert!(!config.activity_types.is_empty());
        }
    }
}