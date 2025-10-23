/// Smart Assignee Finder: Busca inteligente de assignee (responsável) por nome
///
/// Estratégia:
/// 1. Recebe `responsavel_nome` do payload ChatGuru
/// 2. Busca em tarefas existentes pelo campo "assignees"
/// 3. Usa fuzzy matching para encontrar o usuário correto
/// 4. Retorna user_id do ClickUp para atribuir à tarefa
///
/// Exemplos de mapeamento:
/// - "William" → user_id do William
/// - "anne" → user_id da Anne
/// - "Gabriel Moreno" → user_id do Gabriel

use std::collections::HashMap;
use serde::Deserialize;
use serde_json::Value;
use crate::utils::{AppResult, AppError};

const FUZZY_THRESHOLD: f64 = 0.85;

#[derive(Debug, Clone)]
pub struct AssigneeSearchResult {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub confidence: f64,
    pub search_method: SearchMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMethod {
    ExactMatch,      // Nome exato encontrado
    FuzzyMatch,      // Similaridade >= 0.85
    HistoricalMatch, // Encontrado em tarefas anteriores
    NotFound,        // Não encontrado
}

/// Deserializa ID que pode vir como string ou integer da API do ClickUp
fn deserialize_id_flexible<'de, D>(deserializer: D) -> Result<String, D::Error>
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

/// Estrutura de usuário do ClickUp (assignee)
/// NOTA: id pode vir como integer ou string da API
#[derive(Debug, Deserialize, Clone)]
struct ClickUpUser {
    #[serde(deserialize_with = "deserialize_id_flexible")]
    id: String,
    username: String,
    email: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default, rename = "profilePicture")]
    profile_picture: Option<String>,
}

/// Estrutura de resposta da API do ClickUp para team members
#[derive(Debug, Deserialize)]
struct ClickUpTeamResponse {
    team: ClickUpTeamData,
}

#[derive(Debug, Deserialize)]
struct ClickUpTeamData {
    members: Vec<ClickUpMember>,
}

#[derive(Debug, Deserialize)]
struct ClickUpMember {
    user: ClickUpUser,
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
    assignees: Vec<ClickUpUser>,
}

pub struct SmartAssigneeFinder {
    client: reqwest::Client,
    api_token: String,
    team_id: String,
    cache: HashMap<String, AssigneeSearchResult>,
}

impl SmartAssigneeFinder {
    /// Criar novo finder
    pub fn new(api_token: String, team_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_token,
            team_id,
            cache: HashMap::new(),
        }
    }

    /// Busca inteligente de assignee por nome do responsável
    ///
    /// Fases:
    /// 1. Cache lookup (se já buscou antes)
    /// 2. Team members API com fuzzy matching
    /// 3. Historical search em tarefas anteriores (assignees)
    /// 4. Fallback (retorna None)
    pub async fn find_assignee_by_name(&mut self, responsavel_nome: &str) -> AppResult<Option<AssigneeSearchResult>> {
        let normalized_name = Self::normalize_name(responsavel_nome);

        tracing::info!("🔍 SmartAssigneeFinder: Buscando assignee para '{}'", responsavel_nome);

        // 1. Cache lookup
        if let Some(cached) = self.cache.get(&normalized_name) {
            tracing::info!("✅ Encontrado em cache: user_id={}", cached.user_id);
            return Ok(Some(cached.clone()));
        }

        // 2. Team Members API Search
        match self.search_team_members(&normalized_name).await {
            Ok(Some(result)) => {
                self.cache.insert(normalized_name.clone(), result.clone());
                return Ok(Some(result));
            }
            Ok(None) => {
                tracing::info!("⚠️ Não encontrado via Team Members API, tentando busca histórica...");
            }
            Err(e) => {
                tracing::warn!("⚠️ Erro na busca via Team Members API: {}, tentando busca histórica...", e);
            }
        }

        // 3. Historical Search (tarefas anteriores com assignees)
        match self.search_historical_assignees(&normalized_name).await {
            Ok(Some(result)) => {
                self.cache.insert(normalized_name.clone(), result.clone());
                return Ok(Some(result));
            }
            Ok(None) => {
                tracing::warn!("⚠️ Responsável '{}' não encontrado (nem Team API, nem histórico)", responsavel_nome);
            }
            Err(e) => {
                tracing::error!("❌ Erro na busca histórica de assignees: {}", e);
            }
        }

        // 4. Fallback
        Ok(None)
    }

    /// Fase 1: Buscar membros do time via API do ClickUp
    async fn search_team_members(&self, normalized_name: &str) -> AppResult<Option<AssigneeSearchResult>> {
        tracing::info!("👥 Buscando team members via API do ClickUp...");

        // GET /team/{team_id}
        let url = format!("https://api.clickup.com/api/v2/team/{}", self.team_id);

        let response = self.client
            .get(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Falha ao buscar team members: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ClickUpApi(format!(
                "GET team members failed: {} - {}",
                status, body
            )));
        }

        let team_response: ClickUpTeamResponse = response.json().await
            .map_err(|e| AppError::InternalError(format!("Falha ao parsear team response: {}", e)))?;

        tracing::info!("👥 Total de membros encontrados: {}", team_response.team.members.len());

        // Buscar melhor match usando fuzzy matching
        self.find_best_assignee_match(normalized_name, &team_response.team.members).await
    }

    /// Encontrar melhor match de assignee usando fuzzy matching
    async fn find_best_assignee_match(
        &self,
        normalized_name: &str,
        members: &[ClickUpMember],
    ) -> AppResult<Option<AssigneeSearchResult>> {
        let mut best_match: Option<(ClickUpUser, f64, SearchMethod)> = None;

        for member in members {
            let user = &member.user;
            let normalized_username = Self::normalize_name(&user.username);

            // 1. Exact match no username
            if normalized_username == normalized_name {
                tracing::info!("✅ Match exato: '{}'", user.username);
                best_match = Some((user.clone(), 1.0, SearchMethod::ExactMatch));
                break;
            }

            // 2. Fuzzy match (Jaro-Winkler)
            let similarity = strsim::jaro_winkler(normalized_name, &normalized_username);

            tracing::debug!("  Comparando: '{}' vs '{}' → score: {:.3}",
                normalized_name, normalized_username, similarity);

            if similarity >= FUZZY_THRESHOLD {
                if let Some((_, best_score, _)) = &best_match {
                    if similarity > *best_score {
                        best_match = Some((user.clone(), similarity, SearchMethod::FuzzyMatch));
                    }
                } else {
                    best_match = Some((user.clone(), similarity, SearchMethod::FuzzyMatch));
                }
            }
        }

        if let Some((user, score, method)) = best_match {
            Ok(Some(AssigneeSearchResult {
                user_id: user.id,
                username: user.username,
                email: user.email,
                confidence: score,
                search_method: method,
            }))
        } else {
            Ok(None)
        }
    }

    /// Fase 2: Buscar em tarefas anteriores pelos assignees
    async fn search_historical_assignees(&self, normalized_name: &str) -> AppResult<Option<AssigneeSearchResult>> {
        tracing::info!("🕐 Buscando assignees em tarefas históricas...");

        // GET /team/{team_id}/task
        let url = format!(
            "https://api.clickup.com/api/v2/team/{}/task",
            self.team_id
        );

        let response = self.client
            .get(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .query(&[
                ("archived", "false"),
                ("subtasks", "false"),
                ("include_closed", "true"), // Incluir tarefas fechadas
            ])
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Falha ao buscar tasks: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::ClickUpApi(format!(
                "GET tasks failed: {} - {}",
                status, body
            )));
        }

        let tasks_response: ClickUpTasksResponse = response.json().await
            .map_err(|e| AppError::InternalError(format!("Falha ao parsear tasks: {}", e)))?;

        tracing::info!("📋 Total de tarefas encontradas: {}", tasks_response.tasks.len());

        // Buscar assignees que correspondem ao nome
        let mut all_assignees: Vec<ClickUpUser> = Vec::new();

        for task in tasks_response.tasks {
            for assignee in task.assignees {
                // Evitar duplicatas (mesmo user_id)
                if !all_assignees.iter().any(|a| a.id == assignee.id) {
                    all_assignees.push(assignee);
                }
            }
        }

        tracing::info!("👥 Total de assignees únicos encontrados: {}", all_assignees.len());

        // Buscar melhor match usando fuzzy matching
        let mut best_match: Option<(ClickUpUser, f64)> = None;

        for assignee in all_assignees {
            let normalized_username = Self::normalize_name(&assignee.username);

            let similarity = strsim::jaro_winkler(normalized_name, &normalized_username);

            if similarity >= FUZZY_THRESHOLD {
                if let Some((_, best_score)) = &best_match {
                    if similarity > *best_score {
                        best_match = Some((assignee, similarity));
                    }
                } else {
                    best_match = Some((assignee, similarity));
                }
            }
        }

        if let Some((user, score)) = best_match {
            tracing::info!(
                "✅ Match histórico encontrado: {} (user_id: {}, score: {:.2})",
                user.username,
                user.id,
                score
            );

            Ok(Some(AssigneeSearchResult {
                user_id: user.id,
                username: user.username,
                email: user.email,
                confidence: score,
                search_method: SearchMethod::HistoricalMatch,
            }))
        } else {
            tracing::warn!("⚠️ Nenhum assignee histórico encontrado para '{}'", normalized_name);
            Ok(None)
        }
    }

    /// Normalizar nome: lowercase, remover acentos e pontuação
    pub fn normalize_name(name: &str) -> String {
        use deunicode::deunicode;

        deunicode(name)
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(SmartAssigneeFinder::normalize_name("William"), "william");
        assert_eq!(SmartAssigneeFinder::normalize_name("Anne Souza"), "anne souza");
        assert_eq!(SmartAssigneeFinder::normalize_name("Gabriel Moreno"), "gabriel moreno");
        assert_eq!(SmartAssigneeFinder::normalize_name("WILLIAM DUARTE"), "william duarte");
        assert_eq!(SmartAssigneeFinder::normalize_name("  Anne  "), "anne");
    }

    #[test]
    fn test_fuzzy_matching_assignees() {
        let test_cases = vec![
            // (nome_original, nome_digitado, deve_dar_match)
            ("William", "william", true),
            ("William", "Wiliam", true),   // Typo
            ("Anne", "anne", true),
            ("Anne", "Ann", true),          // Abreviação
            ("Gabriel Moreno", "gabriel moreno", true),
            ("Gabriel Moreno", "gabriel", true), // Nome parcial
            ("William Duarte", "william duarte", true),
            ("Renata", "renata", true),
            ("Renata", "Renatta", true),    // Typo
            ("William", "João", false),     // Nome diferente
        ];

        for (original, digitado, should_match) in test_cases {
            let original_norm = SmartAssigneeFinder::normalize_name(original);
            let digitado_norm = SmartAssigneeFinder::normalize_name(digitado);

            let similarity = strsim::jaro_winkler(&original_norm, &digitado_norm);
            let matches = similarity >= 0.85;

            println!(
                "Comparando '{}' vs '{}' → score: {:.3} (match: {})",
                original, digitado, similarity, matches
            );

            assert_eq!(
                matches, should_match,
                "Falha ao comparar '{}' vs '{}': score {:.3}",
                original, digitado, similarity
            );
        }
    }
}
