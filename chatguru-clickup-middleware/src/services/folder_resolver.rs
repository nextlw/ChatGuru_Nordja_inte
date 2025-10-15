/// Folder Resolver: Resolve cliente → folder_id usando mapeamento direto
///
/// NOVA LÓGICA SIMPLIFICADA:
/// - Recebe Info_2 (nome do cliente)
/// - Busca no YAML cliente_solicitante_mappings
/// - Usa fuzzy matching para tolerar erros de digitação
/// - Fallback para lista padrão se não encontrar
///
/// Exemplo:
/// ```
/// Info_2 = "Raphaela Spilberg" (erro de digitação)
/// Fuzzy match → "Raphaela Spielberg"
/// Retorna folder_id: "abeb7e51-2ca7-4322-9a91-4a3b7f4ebd85"
/// ```

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::utils::{AppResult, AppError};

/// Estrutura do YAML dropdown_to_folder_mapping.yaml
/// Mapeia dropdown names → folder IDs reais do ClickUp
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClienteMappings {
    #[serde(alias = "cliente_solicitante_mappings", alias = "dropdown_to_folder_mapping")]
    pub dropdown_to_folder_mapping: HashMap<String, String>,
}

/// Resultado da resolução de folder
#[derive(Debug, Clone)]
pub struct FolderResolution {
    pub folder_id: String,
    pub client_name: String,
    pub match_type: MatchType,
    pub similarity_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,      // Match exato
    Fuzzy,      // Fuzzy match (similaridade >= threshold)
    Fallback,   // Não encontrado, usando fallback
}

/// Serviço de resolução de folders
pub struct FolderResolver {
    mappings: HashMap<String, String>,
    fuzzy_threshold: f64,
    fallback_folder_id: String,
}

impl FolderResolver {
    /// Criar novo resolver carregando do YAML
    pub fn from_yaml(yaml_path: &str) -> AppResult<Self> {
        tracing::info!("📄 Carregando mapeamento de clientes: {}", yaml_path);

        let yaml_content = std::fs::read_to_string(yaml_path)
            .map_err(|e| AppError::ConfigError(format!("Falha ao ler YAML: {}", e)))?;

        let config: ClienteMappings = serde_yaml::from_str(&yaml_content)
            .map_err(|e| AppError::ConfigError(format!("Falha ao parsear YAML: {}", e)))?;

        tracing::info!("✅ {} clientes carregados do YAML", config.dropdown_to_folder_mapping.len());

        Ok(Self {
            mappings: config.dropdown_to_folder_mapping,
            fuzzy_threshold: 0.85, // 85% de similaridade mínima
            fallback_folder_id: std::env::var("FALLBACK_FOLDER_ID")
                .unwrap_or_else(|_| "901320655648".to_string()), // Lista OUTUBRO da Renata (fallback)
        })
    }

    /// Carregar do arquivo padrão (novo mapeamento com folder IDs reais)
    pub fn load_default() -> AppResult<Self> {
        Self::from_yaml("config/dropdown_to_folder_mapping.yaml")
    }

    /// Resolver folder_id a partir do nome do cliente (Info_2)
    ///
    /// Ordem de resolução:
    /// 1. Match exato (case-insensitive)
    /// 2. Fuzzy match (Jaro-Winkler >= threshold)
    /// 3. Fallback (lista padrão)
    pub fn resolve(&self, client_name: &str) -> FolderResolution {
        let normalized = self.normalize_name(client_name);

        tracing::info!("🔍 Resolvendo folder para cliente: '{}' (normalizado: '{}')",
            client_name, normalized);

        // 1. Tentar match exato
        if let Some(exact_match) = self.find_exact_match(&normalized) {
            tracing::info!("✅ Match exato: {} → {}", client_name, exact_match.folder_id);
            return exact_match;
        }

        // 2. Tentar fuzzy match
        if let Some(fuzzy_match) = self.find_fuzzy_match(&normalized) {
            tracing::info!("✅ Fuzzy match: {} → {} (score: {:.2}, matched: {})",
                client_name,
                fuzzy_match.folder_id,
                fuzzy_match.similarity_score.unwrap_or(0.0),
                fuzzy_match.client_name
            );
            return fuzzy_match;
        }

        // 3. Fallback
        tracing::warn!("⚠️ Cliente '{}' não encontrado, usando fallback: {}",
            client_name, self.fallback_folder_id);

        FolderResolution {
            folder_id: self.fallback_folder_id.clone(),
            client_name: client_name.to_string(),
            match_type: MatchType::Fallback,
            similarity_score: None,
        }
    }

    /// Normalizar nome: lowercase, trim, remover acentos, parênteses e números
    fn normalize_name(&self, name: &str) -> String {
        name.trim()
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_numeric() && *c != '(' && *c != ')')  // Remove números e parênteses
            .map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'è' | 'ê' => 'e',
                'í' | 'ì' | 'î' => 'i',
                'ó' | 'ò' | 'ô' | 'õ' => 'o',
                'ú' | 'ù' | 'û' => 'u',
                'ç' => 'c',
                _ => c,
            })
            .collect::<String>()
            .split_whitespace()  // Remove espaços extras criados pela remoção
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Buscar match exato (case-insensitive)
    fn find_exact_match(&self, normalized_name: &str) -> Option<FolderResolution> {
        for (client, folder_id) in &self.mappings {
            let normalized_client = self.normalize_name(client);
            if normalized_client == normalized_name {
                return Some(FolderResolution {
                    folder_id: folder_id.clone(),
                    client_name: client.clone(),
                    match_type: MatchType::Exact,
                    similarity_score: Some(1.0),
                });
            }
        }
        None
    }

    /// Buscar fuzzy match usando Jaro-Winkler
    fn find_fuzzy_match(&self, normalized_name: &str) -> Option<FolderResolution> {
        let mut best_match: Option<(String, String, f64)> = None;

        for (client, folder_id) in &self.mappings {
            let normalized_client = self.normalize_name(client);

            // Usar Jaro-Winkler (melhor para nomes de pessoas)
            let similarity = strsim::jaro_winkler(&normalized_name, &normalized_client);

            tracing::debug!("  Comparando: '{}' vs '{}' → score: {:.3}",
                normalized_name, normalized_client, similarity);

            if similarity >= self.fuzzy_threshold {
                if let Some((_, _, best_score)) = best_match {
                    if similarity > best_score {
                        best_match = Some((client.clone(), folder_id.clone(), similarity));
                    }
                } else {
                    best_match = Some((client.clone(), folder_id.clone(), similarity));
                }
            }
        }

        best_match.map(|(client, folder_id, score)| {
            FolderResolution {
                folder_id,
                client_name: client,
                match_type: MatchType::Fuzzy,
                similarity_score: Some(score),
            }
        })
    }

    /// Verificar se um cliente existe no mapeamento
    pub fn has_client(&self, client_name: &str) -> bool {
        let normalized = self.normalize_name(client_name);
        self.mappings.keys()
            .any(|k| self.normalize_name(k) == normalized)
    }

    /// Listar todos os clientes mapeados
    pub fn list_clients(&self) -> Vec<String> {
        self.mappings.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        let resolver = FolderResolver {
            mappings: HashMap::new(),
            fuzzy_threshold: 0.85,
            fallback_folder_id: "fallback".to_string(),
        };

        assert_eq!(resolver.normalize_name("Raphaela Spielberg"), "raphaela spielberg");
        assert_eq!(resolver.normalize_name("  Gabriel Benarros  "), "gabriel benarros");
        assert_eq!(resolver.normalize_name("José Muritiba"), "jose muritiba");

        // Testes com parênteses e números
        assert_eq!(resolver.normalize_name("William Duarte (123)"), "william duarte");
        assert_eq!(resolver.normalize_name("Fernanda Cruz (456)"), "fernanda cruz");
        assert_eq!(resolver.normalize_name("Alex 123 Test"), "alex test");
        assert_eq!(resolver.normalize_name("Carlos (Secron) 789"), "carlos secron");
    }

    #[test]
    fn test_exact_match() {
        let mut mappings = HashMap::new();
        mappings.insert("Raphaela Spielberg".to_string(), "folder-123".to_string());

        let resolver = FolderResolver {
            mappings,
            fuzzy_threshold: 0.85,
            fallback_folder_id: "fallback".to_string(),
        };

        let result = resolver.resolve("Raphaela Spielberg");
        assert_eq!(result.match_type, MatchType::Exact);
        assert_eq!(result.folder_id, "folder-123");
    }

    #[test]
    fn test_fuzzy_match() {
        let mut mappings = HashMap::new();
        mappings.insert("Raphaela Spielberg".to_string(), "folder-123".to_string());

        let resolver = FolderResolver {
            mappings,
            fuzzy_threshold: 0.85,
            fallback_folder_id: "fallback".to_string(),
        };

        // "Spilberg" vs "Spielberg" → alta similaridade
        let result = resolver.resolve("Raphaela Spilberg");
        assert_eq!(result.match_type, MatchType::Fuzzy);
        assert_eq!(result.folder_id, "folder-123");
    }
}
