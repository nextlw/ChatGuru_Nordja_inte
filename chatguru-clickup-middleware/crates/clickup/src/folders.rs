/// Smart Folder Finder: Busca inteligente de folder_id usando API do ClickUp
///
/// Estratégia em 2 fases:
/// 1. **API Search**: GET folders do ClickUp + fuzzy matching semântico
/// 2. **Historical Fallback**: Busca tarefas anteriores pelo campo "Cliente Solicitante"
///
/// Retorna:
/// - folder_id: ID da pasta encontrada
/// - list_id: ID da lista do mês atual (ou cria se não existir)
/// - confidence: Nível de confiança da busca (1.0 = exact, 0.85+ = fuzzy, 0.5 = historical)

use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;
use chrono::{Utc, Datelike};
use crate::error::{Result, ClickUpError};
use crate::client::ClickUpClient;

const CLIENT_SOLICITANTE_FIELD_ID: &str = "0ed63eec-1c50-4190-91c1-59b4b17557f6";
const FUZZY_THRESHOLD: f64 = 0.85;

/// Deserializa ID que pode vir como string ou integer da API do ClickUp
fn deserialize_id_flexible<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(de::Error::custom("id must be string or number")),
    }
}
const MIN_HISTORICAL_CONFIDENCE: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct FolderSearchResult {
    pub folder_id: String,
    pub folder_name: String,
    pub list_id: Option<String>,
    pub list_name: Option<String>,
    pub confidence: f64,
    pub search_method: SearchMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMethod {
    ExactMatch,      // Nome exato encontrado
    FuzzyMatch,      // Similaridade >= 0.85
    SemanticMatch,   // Busca semântica (embeddings)
    HistoricalMatch, // Encontrado em tarefas anteriores
    NotFound,        // Não encontrado (usar fallback)
}

/// Estrutura de resposta da API do ClickUp para folders
#[derive(Debug, Deserialize)]
struct ClickUpFoldersResponse {
    folders: Vec<ClickUpFolder>,
}

#[derive(Debug, Deserialize, Clone)]
struct ClickUpFolder {
    #[serde(deserialize_with = "deserialize_id_flexible")]
    id: String,
    name: String,
    #[allow(dead_code)]
    lists: Option<Vec<ClickUpList>>,
}

#[derive(Debug, Deserialize, Clone)]
struct ClickUpList {
    #[serde(deserialize_with = "deserialize_id_flexible")]
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
}

/// Estrutura de resposta da API do ClickUp para tasks
#[derive(Debug, Deserialize)]
struct ClickUpTasksResponse {
    tasks: Vec<ClickUpTask>,
}

#[derive(Debug, Deserialize)]
struct ClickUpTask {
    #[serde(deserialize_with = "deserialize_id_flexible")]
    id: String,
    folder: Option<ClickUpTaskFolder>,
    #[allow(dead_code)]
    list: Option<ClickUpTaskList>,
    custom_fields: Option<Vec<ClickUpCustomField>>,
}

#[derive(Debug, Deserialize)]
struct ClickUpTaskFolder {
    #[serde(deserialize_with = "deserialize_id_flexible")]
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpTaskList {
    #[allow(dead_code)]
    #[serde(deserialize_with = "deserialize_id_flexible")]
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ClickUpCustomField {
    id: String,
    value: Option<serde_json::Value>,
}

pub struct SmartFolderFinder {
    client: ClickUpClient,
    workspace_id: String,
    cache: HashMap<String, FolderSearchResult>,
}

impl SmartFolderFinder {
    /// Criar novo finder
    pub fn new(client: ClickUpClient, workspace_id: String) -> Self {
        Self {
            client,
            workspace_id,
            cache: HashMap::new(),
        }
    }

    /// Criar novo finder a partir de API token (conveniência)
    pub fn from_token(api_token: String, workspace_id: String) -> Result<Self> {
        let client = ClickUpClient::new(api_token)?;
        Ok(Self::new(client, workspace_id))
    }

    /// Busca inteligente de folder por nome do cliente (Info_2)
    ///
    /// Fases:
    /// 1. Cache lookup (se já buscou antes)
    /// 2. API search com fuzzy matching
    /// 3. Historical search em tarefas anteriores
    /// 4. Fallback (retorna None)
    pub async fn find_folder_for_client(&mut self, client_name: &str) -> Result<Option<FolderSearchResult>> {
        let normalized_name = Self::normalize_name(client_name);

        tracing::info!("🔍 SmartFolderFinder: Buscando folder para '{}'", client_name);

        // 1. Cache lookup
        if let Some(cached) = self.cache.get(&normalized_name) {
            tracing::info!("✅ Encontrado em cache: folder_id={}", cached.folder_id);
            return Ok(Some(cached.clone()));
        }

        // 2. API Search (folders)
        match self.search_folders_via_api(&normalized_name).await {
            Ok(Some(result)) => {
                self.cache.insert(normalized_name.clone(), result.clone());
                return Ok(Some(result));
            }
            Ok(None) => {
                tracing::info!("⚠️ Não encontrado via API, tentando busca histórica...");
            }
            Err(e) => {
                tracing::warn!("⚠️ Erro na busca via API: {}, tentando busca histórica...", e);
            }
        }

        // 3. Historical Search (tarefas anteriores)
        match self.search_historical_tasks(&normalized_name).await {
            Ok(Some(result)) => {
                self.cache.insert(normalized_name.clone(), result.clone());
                return Ok(Some(result));
            }
            Ok(None) => {
                tracing::warn!("⚠️ Cliente '{}' não encontrado (nem API, nem histórico)", client_name);
            }
            Err(e) => {
                tracing::error!("❌ Erro na busca histórica: {}", e);
            }
        }

        // 4. Fallback
        Ok(None)
    }

    /// Fase 1: Buscar folders via API do ClickUp
    async fn search_folders_via_api(&self, normalized_client: &str) -> Result<Option<FolderSearchResult>> {
        tracing::info!("📡 Buscando folders via API do ClickUp...");

        // GET /team/{team_id}/space (API v2)
        // Nota: Na v2, usa-se "team" mas internamente chamamos de "workspace" para clareza
        // Como não sabemos o space_id, vamos buscar em todos os spaces

        let endpoint = format!("/team/{}/space", self.workspace_id);
        let spaces: serde_json::Value = self.client.get_json(&endpoint).await?;

        let spaces_array = spaces["spaces"].as_array()
            .ok_or_else(|| ClickUpError::ValidationError("Campo 'spaces' não é array".to_string()))?;

        // Para cada space, buscar folders
        let mut all_folders = Vec::new();
        for space in spaces_array {
            let space_id = space["id"].as_str()
                .ok_or_else(|| ClickUpError::ValidationError("Space sem ID".to_string()))?;

            match self.fetch_folders_from_space(space_id).await {
                Ok(folders) => all_folders.extend(folders),
                Err(e) => {
                    tracing::warn!("⚠️ Erro ao buscar folders do space {}: {}", space_id, e);
                }
            }
        }

        tracing::info!("📁 Total de folders encontrados: {}", all_folders.len());

        // Buscar melhor match usando fuzzy matching
        self.find_best_folder_match(normalized_client, &all_folders).await
    }

    /// Buscar folders de um space específico
    async fn fetch_folders_from_space(&self, space_id: &str) -> Result<Vec<ClickUpFolder>> {
        let endpoint = format!("/space/{}/folder", space_id);
        let folders_response: ClickUpFoldersResponse = self.client.get_json(&endpoint).await?;
        Ok(folders_response.folders)
    }

    /// Encontrar melhor match usando fuzzy matching
    async fn find_best_folder_match(
        &self,
        normalized_client: &str,
        folders: &[ClickUpFolder],
    ) -> Result<Option<FolderSearchResult>> {
        let mut best_match: Option<(ClickUpFolder, f64, SearchMethod)> = None;

        for folder in folders {
            let normalized_folder = Self::normalize_name(&folder.name);

            // 1. Exact match
            if normalized_folder == normalized_client {
                tracing::info!("✅ Match exato: '{}'", folder.name);
                best_match = Some((folder.clone(), 1.0, SearchMethod::ExactMatch));
                break;
            }

            // 2. Fuzzy match (Jaro-Winkler)
            let similarity = strsim::jaro_winkler(normalized_client, &normalized_folder);

            tracing::debug!("  Comparando: '{}' vs '{}' → score: {:.3}",
                normalized_client, normalized_folder, similarity);

            if similarity >= FUZZY_THRESHOLD {
                if let Some((_, best_score, _)) = &best_match {
                    if similarity > *best_score {
                        best_match = Some((folder.clone(), similarity, SearchMethod::FuzzyMatch));
                    }
                } else {
                    best_match = Some((folder.clone(), similarity, SearchMethod::FuzzyMatch));
                }
            }

            // 3. Token-based matching (para casos como "Breno / Leticia" → "Leticia e Breno")
            // Verifica se os principais tokens estão presentes, independente da ordem
            if best_match.is_none() || best_match.as_ref().map(|(_, score, _)| *score).unwrap_or(0.0) < 0.90 {
                let client_tokens = Self::extract_name_tokens(normalized_client);
                let folder_tokens = Self::extract_name_tokens(&normalized_folder);

                if !client_tokens.is_empty() && !folder_tokens.is_empty() {
                    let matching_tokens = client_tokens.iter()
                        .filter(|ct| folder_tokens.iter().any(|ft| strsim::jaro_winkler(ct, ft) >= 0.90))
                        .count();

                    let token_score = matching_tokens as f64 / client_tokens.len().max(folder_tokens.len()) as f64;

                    if token_score >= 0.60 {  // Pelo menos 60% dos tokens devem dar match
                        tracing::debug!("  Token match: {}/{} tokens → score: {:.3}",
                            matching_tokens, client_tokens.len().max(folder_tokens.len()), token_score);

                        if let Some((_, best_score, _)) = &best_match {
                            if token_score > *best_score {
                                best_match = Some((folder.clone(), token_score, SearchMethod::FuzzyMatch));
                            }
                        } else {
                            best_match = Some((folder.clone(), token_score, SearchMethod::FuzzyMatch));
                        }
                    }
                }
            }
        }

        if let Some((folder, score, method)) = best_match {
            // Buscar lista do mês atual
            let (list_id, list_name) = self.find_or_create_current_month_list(&folder.id).await?;

            Ok(Some(FolderSearchResult {
                folder_id: folder.id,
                folder_name: folder.name,
                list_id: Some(list_id),
                list_name: Some(list_name),
                confidence: score,
                search_method: method,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fase 2: Buscar em tarefas anteriores pelo campo "Cliente Solicitante"
    async fn search_historical_tasks(&self, normalized_client: &str) -> Result<Option<FolderSearchResult>> {
        tracing::info!("🕐 Buscando tarefas históricas com 'Cliente Solicitante' = '{}'", normalized_client);

        // GET /team/{team_id}/task with query params (API v2)
        // Nota: Na v2, usa-se "team" mas internamente chamamos de "workspace"
        let endpoint = format!("/team/{}/task?archived=false&subtasks=false&include_closed=true", self.workspace_id);
        let tasks_response: ClickUpTasksResponse = self.client.get_json(&endpoint).await?;

        tracing::info!("📋 Total de tarefas encontradas: {}", tasks_response.tasks.len());

        // Filtrar tarefas que contêm o cliente no campo "Cliente Solicitante"
        for task in tasks_response.tasks {
            if let Some(custom_fields) = task.custom_fields {
                for field in custom_fields {
                    if field.id == CLIENT_SOLICITANTE_FIELD_ID {
                        if let Some(value) = field.value {
                            if let Some(client_value) = value.as_str() {
                                let normalized_value = Self::normalize_name(client_value);

                                // Fuzzy match com threshold menor (histórico é menos confiável)
                                let similarity = strsim::jaro_winkler(normalized_client, &normalized_value);

                                if similarity >= MIN_HISTORICAL_CONFIDENCE {
                                    tracing::info!(
                                        "✅ Match histórico encontrado: tarefa {} → folder {:?} (score: {:.2})",
                                        task.id,
                                        task.folder.as_ref().map(|f| f.name.as_str()),
                                        similarity
                                    );

                                    if let Some(folder) = task.folder {
                                        // Buscar lista do mês atual nessa folder
                                        let (list_id, list_name) = self
                                            .find_or_create_current_month_list(&folder.id)
                                            .await?;

                                        return Ok(Some(FolderSearchResult {
                                            folder_id: folder.id,
                                            folder_name: folder.name,
                                            list_id: Some(list_id),
                                            list_name: Some(list_name),
                                            confidence: similarity,
                                            search_method: SearchMethod::HistoricalMatch,
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::warn!("⚠️ Nenhuma tarefa histórica encontrada para '{}'", normalized_client);
        Ok(None)
    }

    /// Gera nome do mês em português e caixa alta (ex: "OUTUBRO 2025")
    fn get_month_name_pt(&self, date: chrono::DateTime<Utc>) -> String {
        let month = date.month();
        let year = date.year();

        let month_pt = match month {
            1 => "JANEIRO",
            2 => "FEVEREIRO",
            3 => "MARÇO",
            4 => "ABRIL",
            5 => "MAIO",
            6 => "JUNHO",
            7 => "JULHO",
            8 => "AGOSTO",
            9 => "SETEMBRO",
            10 => "OUTUBRO",
            11 => "NOVEMBRO",
            12 => "DEZEMBRO",
            _ => "DESCONHECIDO",
        };

        format!("{} {}", month_pt, year)
    }

    /// Buscar ou criar lista do mês atual na folder
    async fn find_or_create_current_month_list(&self, folder_id: &str) -> Result<(String, String)> {
        let now = Utc::now();
        let month_name_pt = self.get_month_name_pt(now); // Ex: "OUTUBRO 2025"
        let month_number = now.month();

        tracing::info!("📅 Buscando lista do mês atual: '{}'", month_name_pt);

        // GET /folder/{folder_id}
        let endpoint = format!("/folder/{}", folder_id);
        let folder: serde_json::Value = self.client.get_json(&endpoint).await?;

        // Meses em português para busca (aceita variações)
        let months_pt = ["janeiro", "fevereiro", "março", "abril", "maio", "junho",
                         "julho", "agosto", "setembro", "outubro", "novembro", "dezembro"];
        let current_month_pt = months_pt[(month_number - 1) as usize];
        let year_str = now.year().to_string();

        // Buscar lista com nome do mês (aceita em português ou inglês, case-insensitive)
        if let Some(lists) = folder["lists"].as_array() {
            for list in lists {
                if let Some(name) = list["name"].as_str() {
                    let name_lower = name.to_lowercase();

                    // Aceita: "OUTUBRO 2025", "outubro 2025", "October 2025", etc.
                    if (name_lower.contains(current_month_pt) || name_lower.contains(&now.format("%B").to_string().to_lowercase()))
                        && name_lower.contains(&year_str)
                    {
                        let list_id = list["id"].as_str()
                            .ok_or_else(|| ClickUpError::ValidationError("Lista sem ID".to_string()))?;

                        tracing::info!("✅ Lista do mês encontrada: {} (id: {})", name, list_id);
                        return Ok((list_id.to_string(), name.to_string()));
                    }
                }
            }
        }

        // Lista não encontrada, criar nova em português e caixa alta
        tracing::info!("📝 Criando lista do mês: '{}'", month_name_pt);
        self.create_list(folder_id, &month_name_pt).await
    }

    /// Criar lista na folder
    async fn create_list(&self, folder_id: &str, list_name: &str) -> Result<(String, String)> {
        let endpoint = format!("/folder/{}/list", folder_id);
        let payload = serde_json::json!({
            "name": list_name,
            "content": format!("Lista criada automaticamente para {}", list_name),
        });

        let list: serde_json::Value = self.client.post_json(&endpoint, &payload).await?;

        let list_id = list["id"].as_str()
            .ok_or_else(|| ClickUpError::ValidationError("Lista criada sem ID".to_string()))?;

        tracing::info!("✅ Lista criada com sucesso: {} (id: {})", list_name, list_id);

        // Aguardar 2 segundos para ClickUp configurar custom fields da lista
        tracing::debug!("⏳ Aguardando 2s para custom fields serem configurados...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok((list_id.to_string(), list_name.to_string()))
    }

    /// Normalizar nome: lowercase, remover acentos, números e pontuação
    pub fn normalize_name(name: &str) -> String {
        use deunicode::deunicode;

        // Substituir "/" e outros separadores por espaço antes de processar
        let normalized = name
            .replace('/', " ")
            .replace('\\', " ")
            .replace('|', " ")
            .replace('-', " ");

        // Remover acentos, converter para lowercase, remover caracteres especiais
        deunicode(&normalized)
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Extrai tokens individuais de um nome para matching mais flexível
    /// Exemplo: "Breno / Leticia" → ["breno", "leticia"]
    fn extract_name_tokens(name: &str) -> Vec<String> {
        Self::normalize_name(name)
            .split_whitespace()
            .filter(|token| token.len() > 2) // Ignorar tokens muito curtos como "e", "de"
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(SmartFolderFinder::normalize_name("Raphaela Spielberg"), "raphaela spielberg");
        assert_eq!(SmartFolderFinder::normalize_name("José Muritiba (123)"), "jose muritiba 123");
        assert_eq!(SmartFolderFinder::normalize_name("Gabriel Benarros!!!"), "gabriel benarros");

        // Novos testes para "/" e separadores
        assert_eq!(SmartFolderFinder::normalize_name("Breno / Leticia"), "breno leticia");
        assert_eq!(SmartFolderFinder::normalize_name("Leticia e Breno"), "leticia e breno");
        assert_eq!(SmartFolderFinder::normalize_name("Carlos | Pedro"), "carlos pedro");
        assert_eq!(SmartFolderFinder::normalize_name("Ana-Paula"), "ana paula");
    }

    #[test]
    fn test_extract_name_tokens() {
        let tokens = SmartFolderFinder::extract_name_tokens("Breno / Leticia");
        assert_eq!(tokens, vec!["breno", "leticia"]);

        let tokens2 = SmartFolderFinder::extract_name_tokens("Leticia e Breno");
        assert_eq!(tokens2, vec!["leticia", "breno"]);

        let tokens3 = SmartFolderFinder::extract_name_tokens("José de Oliveira");
        assert_eq!(tokens3, vec!["jose", "oliveira"]); // "de" é filtrado por ser muito curto
    }
}
