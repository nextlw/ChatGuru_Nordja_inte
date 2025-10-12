/// Serviço de resolução de estrutura dinâmica do ClickUp
///
/// Este módulo é responsável por:
/// 1. Resolver a pasta (folder) do cliente com base no atendente e nome do cliente
/// 2. Buscar ou criar a lista mensal correspondente ao mês vigente
/// 3. Cachear resultados para evitar chamadas desnecessárias à API do ClickUp
///
/// Fluxo:
/// ```
/// Cliente (info_2) + Atendente (responsavel_nome)
///     ↓
/// Normalização via DB (attendant_mappings, client_mappings)
///     ↓
/// Folder Path ("Anne Souza / Carolina Tavares")
///     ↓
/// Lista Mensal ("OUTUBRO 2025") - cache ou criação
///     ↓
/// List ID para criação de tarefa
/// ```

use sqlx::PgPool;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::utils::{AppResult, AppError};
use crate::utils::logging::*;
use chrono::{Datelike, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct EstruturaService {
    db: PgPool,
    clickup_token: String,
    client: Client,
    cache: Arc<RwLock<EstruturaCache>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    pub folder_id: String,
    pub folder_path: String,
    pub space_id: Option<String>,
}

#[derive(Debug)]
struct EstruturaCache {
    folders: HashMap<String, (FolderInfo, Instant)>, // Key: "attendant|client"
    lists: HashMap<String, (String, Instant)>,       // Key: "folder_id|year-month", Value: list_id
    ttl: Duration,
}

impl EstruturaCache {
    fn new() -> Self {
        Self {
            folders: HashMap::new(),
            lists: HashMap::new(),
            ttl: Duration::from_secs(3600), // 1 hora
        }
    }

    fn is_expired(&self, timestamp: Instant) -> bool {
        timestamp.elapsed() > self.ttl
    }
}

impl EstruturaService {
    pub fn new(db: PgPool, clickup_token: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            db,
            clickup_token,
            client,
            cache: Arc::new(RwLock::new(EstruturaCache::new())),
        }
    }

    /// Resolver folder baseado em cliente e atendente
    /// LÓGICA SIMPLIFICADA: Apenas busca no banco de dados
    /// Se não encontrar, retorna erro para que o worker peça criação manual
    pub async fn resolve_folder(
        &self,
        client_name: &str,
        attendant_name: &str,
    ) -> AppResult<FolderInfo> {
        // Normalizar nomes
        let client_normalized = self.normalize_client_name(client_name).await?;
        let attendant_normalized = self.normalize_attendant_name(attendant_name).await?;

        log_info(&format!("🔍 Resolvendo estrutura: Cliente='{}', Atendente='{}'",
            client_normalized, attendant_normalized));

        // Verificar cache em memória
        let cache_key = format!("{}|{}", attendant_normalized, client_normalized);
        {
            let cache = self.cache.read().await;
            if let Some((folder, timestamp)) = cache.folders.get(&cache_key) {
                if !cache.is_expired(*timestamp) {
                    log_info(&format!("💾 Usando cache: {}", folder.folder_path));
                    return Ok(folder.clone());
                }
            }
        }

        // Buscar no banco de dados (única fonte da verdade)
        if let Some(folder) = self.find_folder_in_db(&client_normalized, &attendant_normalized).await? {
            log_info(&format!("✅ Pasta encontrada no DB: {}", folder.folder_path));

            // Atualizar cache
            let mut cache = self.cache.write().await;
            cache.folders.insert(cache_key, (folder.clone(), Instant::now()));

            return Ok(folder);
        }

        // Se não encontrou, retornar erro específico
        log_warning(&format!("❌ Estrutura não encontrada para Cliente='{}', Atendente='{}'",
            client_normalized, attendant_normalized));

        Err(AppError::StructureNotFound(format!(
            "Pasta não encontrada no banco de dados para Cliente='{}' e Atendente='{}'. \
            Por favor, crie a pasta no ClickUp e adicione o mapeamento no banco de dados.",
            client_normalized, attendant_normalized
        )))
    }

    /// Normalizar nome do cliente via DB
    async fn normalize_client_name(&self, client_name: &str) -> AppResult<String> {
        let normalized = client_name.trim().to_lowercase();

        // Se não há conexão com DB, usa normalização local
        if self.db.is_closed() {
            return Ok(self.normalize_name_local(client_name));
        }

        // Buscar no DB se há aliases usando query simples
        match sqlx::query_scalar::<_, String>("SELECT client_key FROM client_mappings WHERE client_key = $1 OR $1 = ANY(client_aliases) LIMIT 1")
            .bind(&normalized)
            .fetch_optional(&self.db)
            .await
        {
            Ok(Some(key)) => Ok(key),
            _ => Ok(self.normalize_name_local(client_name))
        }
    }

    /// Buscar atendente mapeado para um cliente no banco de dados
    /// Útil quando responsavel_nome vem vazio do ChatGuru
    pub async fn find_attendant_for_client(&self, client_name: &str) -> AppResult<Option<String>> {
        if self.db.is_closed() {
            return Ok(None);
        }

        let client_normalized = self.normalize_client_name(client_name).await?;

        // Buscar na tabela client_mappings qual atendente está mapeado para este cliente
        // A folder_path tem formato: "Atendente / Cliente", então extraímos o atendente
        let query = r#"
            SELECT am.attendant_key
            FROM client_mappings cm
            JOIN attendant_mappings am ON cm.folder_path LIKE CONCAT(am.attendant_full_name, ' /%')
            WHERE cm.client_key = $1
            AND cm.is_active = true
            LIMIT 1
        "#;

        match sqlx::query_scalar::<_, String>(query)
            .bind(&client_normalized)
            .fetch_optional(&self.db)
            .await
        {
            Ok(result) => {
                if let Some(ref attendant) = result {
                    log_info(&format!("✅ Atendente encontrado para cliente '{}': {}", client_name, attendant));
                }
                Ok(result)
            }
            Err(e) => {
                log_warning(&format!("⚠️ Erro ao buscar atendente para cliente '{}': {}", client_name, e));
                Ok(None)
            }
        }
    }

    /// Normalizar nome do atendente via DB
    async fn normalize_attendant_name(&self, attendant_name: &str) -> AppResult<String> {
        let normalized = attendant_name.trim().to_lowercase();

        // Se não há conexão com DB, usa normalização local
        if self.db.is_closed() {
            return Ok(self.normalize_name_local(attendant_name));
        }

        // Buscar no DB usando query simples
        match sqlx::query_scalar::<_, String>("SELECT attendant_key FROM attendant_mappings WHERE attendant_key = $1 OR $1 = ANY(attendant_aliases) LIMIT 1")
            .bind(&normalized)
            .fetch_optional(&self.db)
            .await
        {
            Ok(Some(key)) => Ok(key),
            _ => Ok(self.normalize_name_local(attendant_name))
        }
    }

    /// Buscar folder no banco de dados
    async fn find_folder_in_db(
        &self,
        client_key: &str,
        attendant_key: &str,
    ) -> AppResult<Option<FolderInfo>> {
        if self.db.is_closed() {
            return Ok(None);
        }

        let query = r#"
            SELECT cm.folder_id, cm.folder_path, cm.space_id
            FROM client_mappings cm
            JOIN attendant_mappings am ON cm.folder_path LIKE CONCAT(am.attendant_full_name, ' /%')
            WHERE cm.client_key = $1
            AND am.attendant_key = $2
            AND cm.is_active = true
            LIMIT 1
        "#;

        match sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>)>(query)
            .bind(client_key)
            .bind(attendant_key)
            .fetch_optional(&self.db)
            .await
        {
            Ok(Some((folder_id, folder_path, space_id))) => Ok(Some(FolderInfo {
                folder_id: folder_id.unwrap_or_default(),
                folder_path: folder_path.unwrap_or_default(),
                space_id,
            })),
            _ => Ok(None)
        }
    }

    /// Resolve a lista mensal apropriada para uma pasta
    /// Se folder_path contiver "Clientes Inativos / [Nome]", o nome será incluído na lista
    pub async fn resolve_monthly_list(&self, folder_id: &str, folder_path: Option<&str>) -> AppResult<String> {
        let year_month = Self::get_current_year_month();
        let mut month_name = Self::get_month_name(&year_month);

        // Se for cliente inativo, extrair nome do cliente e incluir no nome da lista
        if let Some(path) = folder_path {
            if path.starts_with("Clientes Inativos / ") {
                let client_name = path.strip_prefix("Clientes Inativos / ").unwrap_or("");
                if !client_name.is_empty() {
                    month_name = format!("{} - {}", client_name, month_name);
                    log_info(&format!("📝 Lista para cliente inativo: {}", month_name));
                }
            }
        }

        log_info(&format!("📅 Buscando lista para: {} (folder: {})", month_name, folder_id));

        // Verificar cache em memória
        let cache_key = format!("{}|{}", folder_id, year_month);
        {
            let cache = self.cache.read().await;
            if let Some((list_id, timestamp)) = cache.lists.get(&cache_key) {
                if !cache.is_expired(*timestamp) {
                    if self.validate_list_exists(list_id).await? {
                        log_info(&format!("💾 Lista em cache válida: {}", list_id));
                        return Ok(list_id.clone());
                    }
                }
            }
        }

        // Buscar no cache local (DB) - agora inclui o nome da lista para evitar duplicatas
        if let Some(list_id) = self.find_list_in_cache(folder_id, &year_month, &month_name).await? {
            if self.validate_list_exists(&list_id).await? {
                log_info(&format!("📋 Lista encontrada no DB cache: {}", list_id));

                let mut cache = self.cache.write().await;
                cache.lists.insert(cache_key.clone(), (list_id.clone(), Instant::now()));

                return Ok(list_id);
            } else {
                self.remove_list_from_cache(&list_id).await?;
            }
        }

        // Buscar no ClickUp
        if let Some(list_id) = self.find_list_in_clickup(folder_id, &month_name).await? {
            log_info(&format!("📋 Lista encontrada no ClickUp: {}", list_id));
            self.update_list_cache(folder_id, &list_id, &month_name, &year_month).await?;

            let mut cache = self.cache.write().await;
            cache.lists.insert(cache_key, (list_id.clone(), Instant::now()));

            return Ok(list_id);
        }

        // Criar nova lista
        log_info(&format!("🆕 Criando nova lista: {}", month_name));
        let list_id = self.create_monthly_list(folder_id, &month_name).await?;
        self.update_list_cache(folder_id, &list_id, &month_name, &year_month).await?;

        let mut cache = self.cache.write().await;
        cache.lists.insert(cache_key, (list_id.clone(), Instant::now()));

        Ok(list_id)
    }

    /// Normaliza nome local (sem DB)
    fn normalize_name_local(&self, name: &str) -> String {
        use regex::Regex;

        let normalized = name.trim().to_lowercase();

        // Remover acentos (normalização NFD + remoção de diacríticos)
        let normalized = normalized
            .chars()
            .filter_map(|c| match c {
                'á' | 'à' | 'â' | 'ã' => Some('a'),
                'é' | 'è' | 'ê' => Some('e'),
                'í' | 'ì' | 'î' => Some('i'),
                'ó' | 'ò' | 'ô' | 'õ' => Some('o'),
                'ú' | 'ù' | 'û' => Some('u'),
                'ç' => Some('c'),
                'ñ' => Some('n'),
                _ => Some(c),
            })
            .collect::<String>();

        // Remover caracteres especiais exceto espaços e hífens
        let re = Regex::new(r"[^a-z0-9\s\-]").unwrap();
        let normalized = re.replace_all(&normalized, "").to_string();

        // Normalizar múltiplos espaços para um único espaço
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(&normalized, " ").trim().to_string()
    }

    // ========== MÉTODOS DE CRIAÇÃO DE PASTA REMOVIDOS ==========
    // A lógica agora é simplificada: não criamos pastas automaticamente.
    // Se a estrutura não existir no banco, retornamos erro para que
    // o usuário crie manualmente no ClickUp e adicione o mapeamento no DB.

    /// Busca lista no cache local (DB)
    /// IMPORTANTE: Busca por folder_id + year_month + list_name para evitar duplicatas
    /// em pastas compartilhadas (ex: múltiplos clientes em "Clientes Inativos")
    async fn find_list_in_cache(&self, folder_id: &str, year_month: &str, list_name: &str) -> AppResult<Option<String>> {
        log_info(&format!("🔍 Buscando lista no cache local: folder_id='{}', year_month='{}', list_name='{}'",
            folder_id, year_month, list_name));
        if self.db.is_closed() {
            return Ok(None);
        }

        match sqlx::query_scalar::<_, String>(
            "SELECT list_id FROM list_cache
             WHERE folder_id = $1 AND year_month = $2 AND list_name = $3 AND is_active = true
             ORDER BY last_verified DESC
             LIMIT 1"
        )
        .bind(folder_id)
        .bind(year_month)
        .bind(list_name)
        .fetch_optional(&self.db)
        .await
        {
            Ok(result) => {
                log_info(&format!("🔍 Resultado da busca no cache local: {:?}", result));
                Ok(result)
            },
            _ => {
                log_warning("⚠️ Falha ao buscar lista no cache local");
                Ok(None)
            }
        }
    }

    /// Validar se lista existe no ClickUp
    async fn validate_list_exists(&self, list_id: &str) -> AppResult<bool> {
        let url = format!("https://api.clickup.com/api/v2/list/{}", list_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.clickup_token))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Buscar lista no ClickUp
    async fn find_list_in_clickup(&self, folder_id: &str, list_name: &str) -> AppResult<Option<String>> {
        log_info(&format!("🔍 Buscando lista '{}' no ClickUp na pasta '{}'", list_name, folder_id));
        let url = format!("https://api.clickup.com/api/v2/folder/{}/list", folder_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.clickup_token))
            .send()
            .await?;

        if !response.status().is_success() {
            log_warning(&format!("⚠️ Falha ao buscar listas no ClickUp: status {}", response.status()));
            return Ok(None);
        }

        let json: serde_json::Value = response.json().await?;

        if let Some(lists) = json["lists"].as_array() {
            for list in lists {
                if let Some(name) = list["name"].as_str() {
                    log_info(&format!("📋 Lista encontrada no ClickUp: {}", name));
                    if name == list_name {
                        log_info(&format!("✅ Lista corresponde ao nome buscado: {}", list_name));
                        return Ok(list["id"].as_str().map(|s| s.to_string()));
                    }
                }
            }
        }

        log_info("❌ Lista não encontrada no ClickUp");
        Ok(None)
    }

    /// Criar lista mensal no ClickUp
    async fn create_monthly_list(&self, folder_id: &str, month_name: &str) -> AppResult<String> {
        let url = format!("https://api.clickup.com/api/v2/folder/{}/list", folder_id);
 
        let payload = serde_json::json!({
            "name": month_name,
            "content": format!("Lista criada automaticamente em {}", Utc::now().format("%Y-%m-%d %H:%M:%S")),
        });
 
        log_info(&format!("📡 POST {} - Criando lista '{}'", url, month_name));
        log_info(&format!("📦 Payload: {}", serde_json::to_string_pretty(&payload).unwrap_or_default()));
 
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.clickup_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
 
        if !response.status().is_success() {
            let error_text = response.text().await?;
            log_error(&format!("❌ Falha ao criar lista: {}", error_text));
            return Err(AppError::ClickUpApi(format!("Falha ao criar lista: {}", error_text)));
        }
 
        let result: serde_json::Value = response.json().await?;
        let list_id = result["id"]
            .as_str()
            .ok_or_else(|| AppError::InternalError("list_id não encontrado na resposta".to_string()))?
            .to_string();
 
        log_info(&format!("✅ Lista criada: {} (ID: {})", month_name, list_id));
 
        Ok(list_id)
    }

    /// Atualizar cache de lista no DB
    async fn update_list_cache(
        &self,
        folder_id: &str,
        list_id: &str,
        list_name: &str,
        year_month: &str,
    ) -> AppResult<()> {
        if self.db.is_closed() {
            return Ok(());
        }

        let query = r#"
            INSERT INTO list_cache (folder_id, list_id, list_name, year_month, last_verified)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (list_id) DO UPDATE SET
              last_verified = NOW(),
              is_active = true
        "#;

        sqlx::query(query)
            .bind(folder_id)
            .bind(list_id)
            .bind(list_name)
            .bind(year_month)
            .execute(&self.db)
            .await
            .ok();

        Ok(())
    }

    /// Remover lista do cache (quando deletada no ClickUp)
    async fn remove_list_from_cache(&self, list_id: &str) -> AppResult<()> {
        if self.db.is_closed() {
            return Ok(());
        }

        sqlx::query("UPDATE list_cache SET is_active = false WHERE list_id = $1")
            .bind(list_id)
            .execute(&self.db)
            .await
            .ok();

        Ok(())
    }

    // ========== MÉTODOS DE BUSCA E CRIAÇÃO REMOVIDOS ==========
    // Removidos: search_client_folder_across_spaces, get_all_spaces,
    // search_folder_in_space, save_folder_mapping, create_folder_in_attendant_space,
    // find_attendant_space, create_folder_in_specific_space
    //
    // Agora apenas lemos do banco de dados. Se não existir, retorna erro.

    // ========== HELPER FUNCTIONS ==========

    /// Obter ano-mês atual no formato YYYY-MM
    fn get_current_year_month() -> String {
        let now = Utc::now();
        format!("{}-{:02}", now.year(), now.month())
    }

    /// Obter nome do mês em português baseado no ano-mês
    fn get_month_name(year_month: &str) -> String {
        let parts: Vec<&str> = year_month.split('-').collect();
        if parts.len() != 2 {
            return "DESCONHECIDO".to_string();
        }

        let year = parts[0];
        let month_num: u32 = parts[1].parse().unwrap_or(1);

        let month_names = [
            "JANEIRO", "FEVEREIRO", "MARÇO", "ABRIL", "MAIO", "JUNHO",
            "JULHO", "AGOSTO", "SETEMBRO", "OUTUBRO", "NOVEMBRO", "DEZEMBRO"
        ];

        let month_name = month_names.get((month_num as usize).saturating_sub(1))
            .unwrap_or(&"DESCONHECIDO");

        format!("{} {}", month_name, year)
    }
}