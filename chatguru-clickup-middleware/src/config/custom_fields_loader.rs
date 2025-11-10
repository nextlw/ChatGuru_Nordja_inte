///! Carregador de custom fields mappings
///!
///! Responsável por carregar configurações de custom fields do ClickUp
///! - Em desenvolvimento: arquivo local via CUSTOM_FIELDS_CONFIG_PATH
///! - Em produção: GCS via GCS_BUCKET e GCS_CONFIG_PATH

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Informações de uma subcategoria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcategoryInfo {
    pub id: String,
    pub name: String,
    pub stars: u8,
}

/// Mapeamento completo de custom fields
#[derive(Debug, Clone)]
pub struct CustomFieldsMappings {
    /// Categoria → ID no ClickUp
    pub categories: HashMap<String, String>,
    /// Subcategoria → {id, stars}
    pub subcategories: HashMap<String, SubcategoryInfo>,
    /// ID do campo Categoria_nova
    pub category_field_id: String,
    /// ID do campo SubCategoria_nova
    pub subcategory_field_id: String,
    /// ID do campo Estrelas (rating)
    pub stars_field_id: Option<String>,
}

impl CustomFieldsMappings {
    /// Carrega mapeamentos baseado nas variáveis de ambiente
    ///
    /// Variáveis:
    /// - CUSTOM_FIELDS_CONFIG_PATH: caminho do arquivo (dev: config/ai_prompt.yaml)
    /// - GCS_BUCKET: bucket do GCS (prod)
    /// - GCS_CONFIG_PATH: caminho no bucket (prod: ai_prompt.yaml)
    pub async fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Verificar se deve usar GCS
        if let (Ok(bucket), Ok(path)) = (
            std::env::var("GCS_BUCKET"),
            std::env::var("GCS_CONFIG_PATH")
        ) {
            return Self::load_from_gcs(&bucket, &path).await;
        }

        // Fallback: arquivo local
        let local_path = std::env::var("CUSTOM_FIELDS_CONFIG_PATH")
            .unwrap_or_else(|_| "config/ai_prompt.yaml".to_string());

        Self::load_from_file(&local_path).await
    }

    /// Carrega do arquivo local
    pub async fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("📂 Carregando custom fields de: {}", path);

        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| format!("Erro ao ler arquivo {}: {}", path, e))?;

        Self::parse_yaml(&content)
    }

    /// Carrega do Google Cloud Storage
    async fn load_from_gcs(
        bucket: &str,
        path: &str
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("☁️ Carregando custom fields do GCS: gs://{}/{}", bucket, path);

        // TODO: Implementar download do GCS usando google-cloud-storage crate
        // Por enquanto retorna erro
        Err(format!("GCS não implementado. Configure CUSTOM_FIELDS_CONFIG_PATH para usar arquivo local").into())
    }

    /// Parse do conteúdo YAML
    fn parse_yaml(content: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;

        // Extrair ID do campo Categoria_nova
        let category_field_id = yaml["category_mappings"][0]["id"]
            .as_str()
            .ok_or("Campo category_mappings[0].id não encontrado")?
            .to_string();

        // ID do campo SubCategoria_nova
        let subcategory_field_id = yaml["field_ids"]["subcategory_field_id"]
            .as_str()
            .unwrap_or("5333c095-eb40-4a5a-b0c2-76bfba4b1094")
            .to_string();

        // ID do campo Estrelas (rating)
        let stars_field_id = yaml["field_ids"]["stars_field_id"]
            .as_str()
            .map(|s| s.to_string());

        // Mapear categorias
        let mut categories = HashMap::new();
        if let Some(category_array) = yaml["category_mappings"].as_sequence() {
            if let Some(first_category) = category_array.first() {
                if let Some(options) = first_category["type_config"]["options"].as_sequence() {
                    for opt in options {
                        if let (Some(id), Some(name)) = (opt["id"].as_str(), opt["name"].as_str()) {
                            categories.insert(name.to_string(), id.to_string());
                        }
                    }
                }
            }
        }

        // Mapear subcategorias
        let mut subcategories = HashMap::new();
        if let Some(subcategory_map) = yaml["subcategory_mappings"].as_mapping() {
            for (_category, subs) in subcategory_map {
                if let Some(sub_array) = subs.as_sequence() {
                    for sub in sub_array {
                        if let (Some(name), Some(id), Some(stars)) = (
                            sub["name"].as_str(),
                            sub["id"].as_str(),
                            sub["stars"].as_u64()
                        ) {
                            subcategories.insert(
                                name.to_string(),
                                SubcategoryInfo {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    stars: stars as u8,
                                }
                            );
                        }
                    }
                }
            }
        }

        tracing::info!(
            "✅ Mapeamentos carregados: {} categorias, {} subcategorias",
            categories.len(),
            subcategories.len()
        );

        if let Some(ref stars_id) = stars_field_id {
            tracing::info!("✅ Campo Estrelas configurado: {}", stars_id);
        } else {
            tracing::warn!("⚠️ Campo Estrelas não encontrado no YAML");
        }

        Ok(Self {
            categories,
            subcategories,
            category_field_id,
            subcategory_field_id,
            stars_field_id,
        })
    }

    pub fn get_category_id(&self, name: &str) -> Option<&String> {
        self.categories.get(name)
    }

    pub fn get_subcategory_info(&self, name: &str) -> Option<&SubcategoryInfo> {
        self.subcategories.get(name)
    }

    pub fn get_stars(&self, subcategory_name: &str) -> Option<u8> {
        self.get_subcategory_info(subcategory_name).map(|info| info.stars)
    }
}
