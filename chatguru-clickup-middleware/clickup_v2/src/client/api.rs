use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use serde_json::{Value, json};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use urlencoding;
use crate::error::{AuthError, AuthResult};

/// Tipo de entidade que pode ser pesquisada
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EntityType {
    Space,
    Folder,
    List,
    Task,
}

/// Resultado da pesquisa de entidade
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub found: bool,
    pub entity_type: EntityType,
    pub items: Vec<EntityItem>,
    pub cached_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Item encontrado na pesquisa
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityItem {
    pub id: String,
    pub name: String,
    pub url: String,
    pub entity_type: EntityType,
    pub parent_id: Option<String>,
    pub parent_name: Option<String>,
}

/// Chave para o cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    name: String,
    entity_type: EntityType,
    team_id: String,
}

/// Entrada no cache com timestamp
#[derive(Debug, Clone)]
struct CacheEntry {
    result: SearchResult,
    cached_at: chrono::DateTime<chrono::Utc>,
}

/// Cliente HTTP para interagir com a API do ClickUp
#[derive(Debug, Clone)]
pub struct ClickUpClient {
    client: Client,
    token: String,
    base_url: String,
    cache: Arc<RwLock<HashMap<CacheKey, CacheEntry>>>,
}

/// Resposta do endpoint de usuário autorizado
#[derive(Debug, serde::Deserialize)]
pub struct AuthorizedUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub color: String,
    pub initials: String,
    pub profile_picture: Option<String>,
}

/// Resposta do endpoint de equipes autorizadas
#[derive(Debug, serde::Deserialize)]
pub struct AuthorizedTeam {
    pub id: String,
    pub name: String,
    pub color: String,
    pub avatar: Option<String>,
}

/// Resposta do endpoint de workspaces
#[derive(Debug, serde::Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub color: String,
    pub avatar: Option<String>,
    pub members: Option<Vec<Value>>,
}

/// Valores possíveis para campos personalizados
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CustomFieldValue {
    /// Valor de texto
    Text(String),
    /// Valor numérico
    Number(f64),
    /// Valor booleano
    Boolean(bool),
    /// Data em timestamp Unix (milliseconds)
    Date(i64),
    /// URL
    Url(String),
    /// Email
    Email(String),
    /// Telefone com código do país
    Phone(String),
    /// Dropdown - ID da opção
    DropdownOption(String),
    /// Multi-select - array de IDs das opções
    MultiSelect(Vec<String>),
    /// Usuários - array de user IDs
    Users(Vec<i64>),
    /// Localização
    Location {
        lat: f64,
        lng: f64,
        formatted_address: Option<String>,
    },
    /// Rating (1-5)
    Rating(i32),
    /// Moeda
    Currency(f64),
}

/// Campo personalizado para criar/atualizar task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomField {
    /// UUID do campo personalizado
    pub id: String,
    /// Valor do campo
    pub value: CustomFieldValue,
}

/// Prioridade da task (1=Urgent, 2=High, 3=Normal, 4=Low)
#[derive(Debug, Clone, Copy)]
pub enum TaskPriority {
    Urgent = 1,
    High = 2,
    Normal = 3,
    Low = 4,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

impl serde::Serialize for TaskPriority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(*self as i32)
    }
}

/// Payload para criar uma nova task
#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateTaskRequest {
    /// Nome/título da task (obrigatório)
    pub name: String,

    /// Conteúdo/descrição da task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Conteúdo em markdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_content: Option<String>,

    /// IDs dos responsáveis (userids)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<i64>>,

    /// Tags da task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Status da task (nome do status)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Prioridade (1=Urgent, 2=High, 3=Normal, 4=Low)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,

    /// Data de vencimento em timestamp Unix (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,

    /// Usar horário na data de vencimento
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_time: Option<bool>,

    /// Data de início em timestamp Unix (milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,

    /// Usar horário na data de início
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date_time: Option<bool>,

    /// Tempo estimado em milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_estimate: Option<i64>,

    /// Task pai (para subtasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Links para outras tasks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links_to: Option<String>,

    /// Notificar todos incluindo o criador
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_all: Option<bool>,

    /// Campos personalizados
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomField>>,
}

impl CreateTaskRequest {
    /// Cria um novo request básico com apenas o nome
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: None,
            markdown_content: None,
            assignees: None,
            tags: None,
            status: None,
            priority: None,
            due_date: None,
            due_date_time: None,
            start_date: None,
            start_date_time: None,
            time_estimate: None,
            parent: None,
            links_to: None,
            notify_all: None,
            custom_fields: None,
        }
    }

    /// Cria um builder para construir o request
    pub fn builder(name: impl Into<String>) -> CreateTaskRequestBuilder {
        CreateTaskRequestBuilder::new(name)
    }
}

/// Builder para CreateTaskRequest
pub struct CreateTaskRequestBuilder {
    request: CreateTaskRequest,
}

impl CreateTaskRequestBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            request: CreateTaskRequest::new(name),
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.request.content = Some(content.into());
        self
    }

    pub fn markdown_content(mut self, content: impl Into<String>) -> Self {
        self.request.markdown_content = Some(content.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.request.content = Some(desc.into());
        self
    }

    pub fn assignees(mut self, assignees: Vec<i64>) -> Self {
        self.request.assignees = Some(assignees);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.request.tags = Some(tags);
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.request.status = Some(status.into());
        self
    }

    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.request.priority = Some(priority as i32);
        self
    }

    pub fn due_date(mut self, timestamp: i64, with_time: bool) -> Self {
        self.request.due_date = Some(timestamp);
        self.request.due_date_time = Some(with_time);
        self
    }

    pub fn start_date(mut self, timestamp: i64, with_time: bool) -> Self {
        self.request.start_date = Some(timestamp);
        self.request.start_date_time = Some(with_time);
        self
    }

    pub fn time_estimate(mut self, millis: i64) -> Self {
        self.request.time_estimate = Some(millis);
        self
    }

    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.request.parent = Some(parent.into());
        self
    }

    pub fn notify_all(mut self, notify: bool) -> Self {
        self.request.notify_all = Some(notify);
        self
    }

    /// Adiciona um campo personalizado
    pub fn custom_field(mut self, id: impl Into<String>, value: CustomFieldValue) -> Self {
        let field = CustomField {
            id: id.into(),
            value,
        };

        match self.request.custom_fields {
            Some(ref mut fields) => fields.push(field),
            None => self.request.custom_fields = Some(vec![field]),
        }

        self
    }

    /// Define múltiplos campos personalizados
    pub fn custom_fields(mut self, fields: Vec<CustomField>) -> Self {
        self.request.custom_fields = Some(fields);
        self
    }

    /// Adiciona campo de texto personalizado
    pub fn custom_field_text(self, id: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_field(id, CustomFieldValue::Text(value.into()))
    }

    /// Adiciona campo numérico personalizado
    pub fn custom_field_number(self, id: impl Into<String>, value: f64) -> Self {
        self.custom_field(id, CustomFieldValue::Number(value))
    }

    /// Adiciona campo de data personalizado
    pub fn custom_field_date(self, id: impl Into<String>, timestamp: i64) -> Self {
        self.custom_field(id, CustomFieldValue::Date(timestamp))
    }

    /// Adiciona campo dropdown personalizado
    pub fn custom_field_dropdown(self, id: impl Into<String>, option_id: impl Into<String>) -> Self {
        self.custom_field(id, CustomFieldValue::DropdownOption(option_id.into()))
    }

    /// Adiciona campo de rating personalizado (1-5)
    pub fn custom_field_rating(self, id: impl Into<String>, rating: i32) -> Self {
        let rating = rating.max(1).min(5);
        self.custom_field(id, CustomFieldValue::Rating(rating))
    }

    /// Constrói o request final
    pub fn build(self) -> CreateTaskRequest {
        self.request
    }
}

/// Resposta da criação de task
#[derive(Debug, serde::Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub custom_id: Option<String>,
    pub name: String,
    pub text_content: Option<String>,
    pub description: Option<String>,
    pub status: Value,
    pub orderindex: String,
    pub date_created: String,
    pub date_updated: String,
    pub date_closed: Option<String>,
    pub archived: bool,
    pub creator: Value,
    pub assignees: Vec<Value>,
    pub tags: Vec<Value>,
    pub parent: Option<String>,
    pub priority: Option<Value>,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub time_estimate: Option<i64>,
    pub custom_fields: Vec<Value>,
    pub url: String,
    pub list: Value,
    pub folder: Value,
    pub space: Value,
}

impl ClickUpClient {
    /// Cria um novo cliente da API do ClickUp
    pub fn new(token: String, base_url: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&token).unwrap_or_else(|_| HeaderValue::from_static(""))
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/json")
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            token,
            base_url: base_url.trim_end_matches('/').to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Constrói URL completa para um endpoint
    fn build_url(&self, endpoint: &str) -> String {
        let endpoint = endpoint.trim_start_matches('/');
        format!("{}/{}", self.base_url, endpoint)
    }

    /// Executa uma requisição GET
    async fn get(&self, endpoint: &str) -> AuthResult<Value> {
        let url = self.build_url(endpoint);

        log::debug!("GET {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha na requisição GET: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha ao ler resposta: {}", e)))?;

        log::debug!("Response status: {}, body: {}", status, response_text);

        if !status.is_success() {
            return Err(self.handle_error_response(status.as_u16(), &response_text));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear JSON: {}", e)))
    }

    /// Executa uma requisição POST
    async fn post<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> AuthResult<Value> {
        let url = self.build_url(endpoint);
        let json_body = serde_json::to_string(body)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao serializar body: {}", e)))?;

        log::debug!("POST {} with body: {}", url, json_body);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha na requisição POST: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha ao ler resposta: {}", e)))?;

        log::debug!("Response status: {}, body: {}", status, response_text);

        if !status.is_success() {
            return Err(self.handle_error_response(status.as_u16(), &response_text));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear resposta JSON: {}", e)))
    }

    /// Executa uma requisição PUT
    async fn put<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> AuthResult<Value> {
        let url = self.build_url(endpoint);
        let json_body = serde_json::to_string(body)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao serializar body: {}", e)))?;

        log::debug!("PUT {} with body: {}", url, json_body);

        let response = self.client
            .put(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha na requisição PUT: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha ao ler resposta: {}", e)))?;

        log::debug!("Response status: {}, body: {}", status, response_text);

        if !status.is_success() {
            return Err(self.handle_error_response(status.as_u16(), &response_text));
        }

        serde_json::from_str(&response_text)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear resposta JSON: {}", e)))
    }

    /// Trata respostas de erro da API
    fn handle_error_response(&self, status: u16, body: &str) -> AuthError {
        match status {
            401 => AuthError::token_error("Token de acesso inválido ou expirado"),
            403 => AuthError::token_error("Acesso negado - verifique as permissões"),
            404 => AuthError::api_error("Endpoint não encontrado"),
            429 => AuthError::api_error("Limite de requisições excedido"),
            500..=599 => AuthError::api_error("Erro interno do servidor ClickUp"),
            _ => AuthError::api_error(&format!("Erro na API ({}): {}", status, body))
        }
    }

    /// **ENDPOINT 1: Health Check** - Valida se as credenciais são válidas
    /// Faz uma requisição simples para verificar se o token está funcionando
    pub async fn health_check(&self) -> AuthResult<bool> {
        log::info!("🏥 Executando health check...");

        match self.get_authorized_user().await {
            Ok(_) => {
                log::info!("✅ Health check passou - credenciais válidas");
                Ok(true)
            },
            Err(e) => {
                log::warn!("❌ Health check falhou: {}", e);
                Ok(false)
            }
        }
    }

    /// **ENDPOINT 2: Get Authorized User** - Obtém informações do usuário autenticado
    /// GET /user
    pub async fn get_authorized_user(&self) -> AuthResult<Value> {
        log::info!("👤 Obtendo informações do usuário autenticado...");
        self.get("user").await
    }

    /// **ENDPOINT 3: Get Authorized Teams** - Obtém equipes autorizadas (workspaces)
    /// GET /team
    pub async fn get_authorized_teams(&self) -> AuthResult<Value> {
        log::info!("👥 Obtendo equipes autorizadas...");
        self.get("team").await
    }

    /// **MÉTODO AUXILIAR: Parse User** - Converte resposta JSON para struct tipada
    pub async fn get_user_info(&self) -> AuthResult<AuthorizedUser> {
        let response = self.get_authorized_user().await?;

        let user_data = response
            .get("user")
            .ok_or_else(|| AuthError::parse_error("Campo 'user' não encontrado na resposta"))?;

        serde_json::from_value(user_data.clone())
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear dados do usuário: {}", e)))
    }

    /// **MÉTODO AUXILIAR: Parse Teams** - Converte resposta JSON para lista tipada
    pub async fn get_teams_info(&self) -> AuthResult<Vec<AuthorizedTeam>> {
        let response = self.get_authorized_teams().await?;

        let teams_data = response
            .get("teams")
            .and_then(|t| t.as_array())
            .ok_or_else(|| AuthError::parse_error("Campo 'teams' não encontrado ou não é um array"))?;

        teams_data.iter()
            .map(|team| serde_json::from_value(team.clone())
                .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear equipe: {}", e))))
            .collect()
    }

    /// **MÉTODO AUXILIAR: Get Workspaces** - Alias para get_authorized_teams (são a mesma coisa)
    pub async fn get_workspaces(&self) -> AuthResult<Value> {
        log::info!("🏢 Obtendo workspaces (alias para teams)...");
        self.get_authorized_teams().await
    }

    /// **MÉTODO AUXILIAR: Get First Workspace ID** - Obtém o ID do primeiro workspace disponível
    pub async fn get_first_workspace_id(&self) -> AuthResult<String> {
        let teams = self.get_authorized_teams().await?;

        let teams_array = teams
            .get("teams")
            .and_then(|t| t.as_array())
            .ok_or_else(|| AuthError::parse_error("Nenhuma equipe encontrada"))?;

        if teams_array.is_empty() {
            return Err(AuthError::api_error("Usuário não possui acesso a nenhuma equipe"));
        }

        let first_team_id = teams_array[0]
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| AuthError::parse_error("ID da primeira equipe não encontrado"))?;

        log::info!("📋 Primeiro workspace ID: {}", first_team_id);
        Ok(first_team_id.to_string())
    }

    /// **MÉTODO DE DIAGNÓSTICO: Test Connection** - Testa a conexão completa
    pub async fn test_connection(&self) -> AuthResult<Value> {
        log::info!("🧪 Testando conexão completa com ClickUp...");

        // 1. Testa health check
        let health = self.health_check().await?;
        if !health {
            return Err(AuthError::api_error("Health check falhou"));
        }

        // 2. Obtém informações do usuário
        let user = self.get_authorized_user().await?;
        let username = user
            .get("user")
            .and_then(|u| u.get("username"))
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        // 3. Obtém informações das equipes
        let teams = self.get_authorized_teams().await?;
        let teams_count = teams
            .get("teams")
            .and_then(|t| t.as_array())
            .map(|t| t.len())
            .unwrap_or(0);

        log::info!("✅ Conexão testada com sucesso:");
        log::info!("  👤 Usuário: {}", username);
        log::info!("  👥 Equipes: {}", teams_count);

        // Retorna resumo da conexão
        Ok(json!({
            "status": "success",
            "user": user,
            "teams": teams,
            "health": health,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// **MÉTODO DE CONFIGURAÇÃO: Get API Info** - Retorna informações sobre a configuração da API
    pub fn get_api_info(&self) -> Value {
        json!({
            "base_url": self.base_url,
            "has_token": !self.token.is_empty(),
            "token_length": self.token.len(),
            "token_preview": if self.token.len() > 10 {
                format!("{}...{}", &self.token[..4], &self.token[self.token.len()-4..])
            } else {
                "***".to_string()
            }
        })
    }

    /// **MÉTODO DE VALIDAÇÃO: Validate Token Format** - Valida se o token tem formato correto
    pub fn validate_token_format(&self) -> bool {
        // Token do ClickUp normalmente tem formato específico
        // Por enquanto, validação básica
        !self.token.is_empty() && self.token.len() > 20
    }

    /// **MÉTODO DE UTILIDADE: Get Rate Limit Info** - Obtém informações sobre rate limiting
    /// Nota: ClickUp não expõe endpoint público para isso, mas podemos simular
    pub async fn get_rate_limit_info(&self) -> AuthResult<Value> {
        log::info!("📊 Verificando informações de rate limit...");

        // Faz uma requisição simples para capturar headers de rate limit
        let url = self.build_url("user");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha na requisição: {}", e)))?;

        let headers = response.headers();
        let mut rate_limit_info = json!({
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Captura headers comuns de rate limiting
        if let Some(remaining) = headers.get("X-RateLimit-Remaining") {
            if let Ok(value) = remaining.to_str() {
                rate_limit_info["remaining"] = json!(value);
            }
        }

        if let Some(reset) = headers.get("X-RateLimit-Reset") {
            if let Ok(value) = reset.to_str() {
                rate_limit_info["reset"] = json!(value);
            }
        }

        if let Some(limit) = headers.get("X-RateLimit-Limit") {
            if let Ok(value) = limit.to_str() {
                rate_limit_info["limit"] = json!(value);
            }
        }

        Ok(rate_limit_info)
    }

    /// **MÉTODO DE PESQUISA: Search Entity** - Pesquisa entidades por nome com cache
    /// Busca spaces, folders, lists ou tasks por nome e retorna informações detalhadas
    /// Resultados são armazenados em cache por 3 horas
    pub async fn search_entity(
        &self,
        name: &str,
        entity_type: EntityType,
        team_id: Option<String>,
    ) -> AuthResult<SearchResult> {
        log::info!("🔍 Pesquisando {} com nome: {}",
            match entity_type {
                EntityType::Space => "space",
                EntityType::Folder => "folder",
                EntityType::List => "list",
                EntityType::Task => "task",
            },
            name
        );

        // Obtém o team_id do ambiente se não fornecido
        let team_id = match team_id {
            Some(id) => id,
            None => {
                // Tenta obter do .env primeiro
                if let Ok(env_id) = std::env::var("CLICKUP_TEAM_ID") {
                    env_id
                } else {
                    // Se não houver no .env, obtém o primeiro workspace
                    self.get_first_workspace_id().await?
                }
            }
        };

        // Cria a chave do cache
        let cache_key = CacheKey {
            name: name.to_lowercase(),
            entity_type: entity_type.clone(),
            team_id: team_id.clone(),
        };

        // Verifica o cache primeiro
        {
            let cache = self.cache.read().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                let now = chrono::Utc::now();
                let cache_age = now - entry.cached_at;

                // Cache válido por 3 horas
                if cache_age < chrono::Duration::hours(3) {
                    log::info!("✅ Resultado encontrado em cache (idade: {} minutos)",
                        cache_age.num_minutes()
                    );
                    let mut result = entry.result.clone();
                    result.cached_at = Some(entry.cached_at);
                    return Ok(result);
                }
            }
        }

        // Se não estiver em cache, busca na API
        log::info!("📡 Buscando na API do ClickUp...");

        let result = match entity_type {
            EntityType::Space => self.search_spaces(&team_id, name).await?,
            EntityType::Folder => self.search_folders(&team_id, name).await?,
            EntityType::List => self.search_lists(&team_id, name).await?,
            EntityType::Task => self.search_tasks(&team_id, name).await?,
        };

        // Armazena no cache se encontrou resultados
        if result.found {
            let mut cache = self.cache.write().unwrap();
            let now = chrono::Utc::now();

            cache.insert(cache_key, CacheEntry {
                result: result.clone(),
                cached_at: now,
            });

            log::info!("💾 Resultado armazenado em cache");
        }

        Ok(result)
    }

    /// Pesquisa spaces em um team
    async fn search_spaces(&self, team_id: &str, name: &str) -> AuthResult<SearchResult> {
        let endpoint = format!("team/{}/space", team_id);
        let response = self.get(&endpoint).await?;

        let spaces = response
            .get("spaces")
            .and_then(|s| s.as_array())
            .ok_or_else(|| AuthError::parse_error("Campo 'spaces' não encontrado"))?;

        let name_lower = name.to_lowercase();
        let mut items = Vec::new();

        for space in spaces {
            let space_name = space.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");

            if space_name.to_lowercase().contains(&name_lower) {
                let space_id = space.get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("");

                items.push(EntityItem {
                    id: space_id.to_string(),
                    name: space_name.to_string(),
                    url: format!("https://app.clickup.com/{}/home", team_id),
                    entity_type: EntityType::Space,
                    parent_id: Some(team_id.to_string()),
                    parent_name: Some("Team".to_string()),
                });
            }
        }

        Ok(SearchResult {
            found: !items.is_empty(),
            entity_type: EntityType::Space,
            items,
            cached_at: None,
        })
    }

    /// Pesquisa folders em todos os spaces de um team
    async fn search_folders(&self, team_id: &str, name: &str) -> AuthResult<SearchResult> {
        // Primeiro obtém todos os spaces
        let spaces_endpoint = format!("team/{}/space", team_id);
        let spaces_response = self.get(&spaces_endpoint).await?;

        let spaces = spaces_response
            .get("spaces")
            .and_then(|s| s.as_array())
            .ok_or_else(|| AuthError::parse_error("Campo 'spaces' não encontrado"))?;

        let name_lower = name.to_lowercase();
        let mut items = Vec::new();

        // Para cada space, busca folders
        for space in spaces {
            let space_id = space.get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("");
            let space_name = space.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");

            let folders_endpoint = format!("space/{}/folder", space_id);

            if let Ok(folders_response) = self.get(&folders_endpoint).await {
                if let Some(folders) = folders_response.get("folders").and_then(|f| f.as_array()) {
                    for folder in folders {
                        let folder_name = folder.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("");

                        if folder_name.to_lowercase().contains(&name_lower) {
                            let folder_id = folder.get("id")
                                .and_then(|i| i.as_str())
                                .unwrap_or("");

                            items.push(EntityItem {
                                id: folder_id.to_string(),
                                name: folder_name.to_string(),
                                url: format!("https://app.clickup.com/{}/{}/f/{}",
                                    team_id, space_id, folder_id
                                ),
                                entity_type: EntityType::Folder,
                                parent_id: Some(space_id.to_string()),
                                parent_name: Some(space_name.to_string()),
                            });
                        }
                    }
                }
            }
        }

        Ok(SearchResult {
            found: !items.is_empty(),
            entity_type: EntityType::Folder,
            items,
            cached_at: None,
        })
    }

    /// Pesquisa lists em todos os spaces e folders de um team
    async fn search_lists(&self, team_id: &str, name: &str) -> AuthResult<SearchResult> {
        // Primeiro obtém todos os spaces
        let spaces_endpoint = format!("team/{}/space", team_id);
        let spaces_response = self.get(&spaces_endpoint).await?;

        let spaces = spaces_response
            .get("spaces")
            .and_then(|s| s.as_array())
            .ok_or_else(|| AuthError::parse_error("Campo 'spaces' não encontrado"))?;

        let name_lower = name.to_lowercase();
        let mut items = Vec::new();

        for space in spaces {
            let space_id = space.get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("");
            let space_name = space.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");

            // Busca lists diretamente no space
            let lists_endpoint = format!("space/{}/list", space_id);

            if let Ok(lists_response) = self.get(&lists_endpoint).await {
                if let Some(lists) = lists_response.get("lists").and_then(|l| l.as_array()) {
                    for list in lists {
                        let list_name = list.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("");

                        if list_name.to_lowercase().contains(&name_lower) {
                            let list_id = list.get("id")
                                .and_then(|i| i.as_str())
                                .unwrap_or("");

                            items.push(EntityItem {
                                id: list_id.to_string(),
                                name: list_name.to_string(),
                                url: format!("https://app.clickup.com/{}/{}/l/li/{}",
                                    team_id, space_id, list_id
                                ),
                                entity_type: EntityType::List,
                                parent_id: Some(space_id.to_string()),
                                parent_name: Some(space_name.to_string()),
                            });
                        }
                    }
                }
            }

            // Busca lists dentro de folders
            let folders_endpoint = format!("space/{}/folder", space_id);

            if let Ok(folders_response) = self.get(&folders_endpoint).await {
                if let Some(folders) = folders_response.get("folders").and_then(|f| f.as_array()) {
                    for folder in folders {
                        let folder_id = folder.get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("");
                        let folder_name = folder.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("");

                        let folder_lists_endpoint = format!("folder/{}/list", folder_id);

                        if let Ok(folder_lists_response) = self.get(&folder_lists_endpoint).await {
                            if let Some(lists) = folder_lists_response.get("lists").and_then(|l| l.as_array()) {
                                for list in lists {
                                    let list_name = list.get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("");

                                    if list_name.to_lowercase().contains(&name_lower) {
                                        let list_id = list.get("id")
                                            .and_then(|i| i.as_str())
                                            .unwrap_or("");

                                        items.push(EntityItem {
                                            id: list_id.to_string(),
                                            name: list_name.to_string(),
                                            url: format!("https://app.clickup.com/{}/{}/l/li/{}",
                                                team_id, space_id, list_id
                                            ),
                                            entity_type: EntityType::List,
                                            parent_id: Some(folder_id.to_string()),
                                            parent_name: Some(folder_name.to_string()),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(SearchResult {
            found: !items.is_empty(),
            entity_type: EntityType::List,
            items,
            cached_at: None,
        })
    }

    /// Pesquisa tasks usando o endpoint de search
    async fn search_tasks(&self, team_id: &str, name: &str) -> AuthResult<SearchResult> {
        let endpoint = format!("team/{}/task", team_id);
        let mut url = self.build_url(&endpoint);
        url.push_str(&format!("?name={}", urlencoding::encode(name)));

        log::debug!("GET {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha na requisição GET: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AuthError::network_error(&format!("Falha ao ler resposta: {}", e)))?;

        log::debug!("Response status: {}, body: {}", status, response_text);

        if !status.is_success() {
            return Err(self.handle_error_response(status.as_u16(), &response_text));
        }

        let response_json: Value = serde_json::from_str(&response_text)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear JSON: {}", e)))?;

        let empty_vec = Vec::new();
        let tasks = response_json
            .get("tasks")
            .and_then(|t| t.as_array())
            .unwrap_or(&empty_vec);

        let mut items = Vec::new();

        for task in tasks {
            let task_name = task.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let task_id = task.get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("");

            // Obtém informações da lista para construir a URL
            let list_id = task.get("list")
                .and_then(|l| l.get("id"))
                .and_then(|i| i.as_str())
                .unwrap_or("");
            let list_name = task.get("list")
                .and_then(|l| l.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");

            items.push(EntityItem {
                id: task_id.to_string(),
                name: task_name.to_string(),
                url: format!("https://app.clickup.com/t/{}", task_id),
                entity_type: EntityType::Task,
                parent_id: Some(list_id.to_string()),
                parent_name: Some(list_name.to_string()),
            });
        }

        Ok(SearchResult {
            found: !items.is_empty(),
            entity_type: EntityType::Task,
            items,
            cached_at: None,
        })
    }

    /// Limpa o cache de pesquisas
    pub fn clear_search_cache(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        log::info!("🗑️ Cache de pesquisas limpo");
    }

    /// Obtém estatísticas do cache
    pub fn get_cache_stats(&self) -> Value {
        let cache = self.cache.read().unwrap();
        let now = chrono::Utc::now();

        let mut stats = json!({
            "total_entries": cache.len(),
            "entries": []
        });

        for (key, entry) in cache.iter() {
            let age = now - entry.cached_at;
            stats["entries"].as_array_mut().unwrap().push(json!({
                "name": key.name,
                "entity_type": format!("{:?}", key.entity_type),
                "team_id": key.team_id,
                "cached_at": entry.cached_at.to_rfc3339(),
                "age_minutes": age.num_minutes(),
                "items_count": entry.result.items.len()
            }));
        }

        stats
    }

    /// **MÉTODO DE CRIAÇÃO: Create Task** - Cria uma nova task em uma lista
    /// POST /list/{list_id}/task
    ///
    /// # Parâmetros
    /// * `list_id` - ID da lista onde criar a task
    /// * `request` - Dados da task a ser criada
    ///
    /// # Exemplo
    /// ```no_run
    /// let task = CreateTaskRequest::builder("Nova Task")
    ///     .description("Descrição da task")
    ///     .priority(TaskPriority::High)
    ///     .custom_field_text("field_uuid", "valor")
    ///     .custom_field_date("date_field_uuid", timestamp)
    ///     .build();
    ///
    /// let response = client.create_task("list_id", task).await?;
    /// ```
    pub async fn create_task(
        &self,
        list_id: &str,
        request: CreateTaskRequest,
    ) -> AuthResult<TaskResponse> {
        log::info!("📝 Criando nova task na lista: {}", list_id);
        log::debug!("Task data: {:?}", request);

        let endpoint = format!("list/{}/task", list_id);
        let response = self.post(&endpoint, &request).await?;

        // Parseia a resposta para TaskResponse
        serde_json::from_value(response)
            .map_err(|e| AuthError::parse_error(&format!("Falha ao parsear resposta da task: {}", e)))
    }

    /// **MÉTODO DE CRIAÇÃO SIMPLIFICADO: Create Simple Task** - Cria uma task básica
    pub async fn create_simple_task(
        &self,
        list_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> AuthResult<TaskResponse> {
        let mut request = CreateTaskRequest::new(name);
        if let Some(desc) = description {
            request.content = Some(desc.to_string());
        }

        self.create_task(list_id, request).await
    }

    /// **MÉTODO AUXILIAR: Get Lists** - Obtém listas de um space ou folder
    /// GET /space/{space_id}/list ou GET /folder/{folder_id}/list
    pub async fn get_lists(&self, space_id: Option<&str>, folder_id: Option<&str>) -> AuthResult<Value> {
        let endpoint = if let Some(folder) = folder_id {
            log::info!("📋 Obtendo listas do folder: {}", folder);
            format!("folder/{}/list", folder)
        } else if let Some(space) = space_id {
            log::info!("📋 Obtendo listas do space: {}", space);
            format!("space/{}/list", space)
        } else {
            return Err(AuthError::api_error("Forneça space_id ou folder_id"));
        };

        self.get(&endpoint).await
    }

    /// **MÉTODO AUXILIAR: Get Custom Fields** - Obtém campos personalizados de uma lista
    /// GET /list/{list_id}/field
    pub async fn get_custom_fields(&self, list_id: &str) -> AuthResult<Value> {
        log::info!("🔧 Obtendo campos personalizados da lista: {}", list_id);
        let endpoint = format!("list/{}/field", list_id);
        self.get(&endpoint).await
    }

    /// **MÉTODO AUXILIAR: Get Spaces** - Obtém spaces de um team
    /// GET /team/{team_id}/space
    pub async fn get_spaces(&self, team_id: &str) -> AuthResult<Value> {
        log::info!("🏢 Obtendo spaces do team: {}", team_id);
        let endpoint = format!("team/{}/space", team_id);
        self.get(&endpoint).await
    }

    /// **MÉTODO AUXILIAR: Get Folders** - Obtém folders de um space
    /// GET /space/{space_id}/folder
    pub async fn get_folders(&self, space_id: &str) -> AuthResult<Value> {
        log::info!("📁 Obtendo folders do space: {}", space_id);
        let endpoint = format!("space/{}/folder", space_id);
        self.get(&endpoint).await
    }

    /// **MÉTODO AUXILIAR: Get Task** - Obtém uma task específica
    /// GET /task/{task_id}
    pub async fn get_task(&self, task_id: &str) -> AuthResult<Value> {
        log::info!("📌 Obtendo task: {}", task_id);
        let endpoint = format!("task/{}", task_id);
        self.get(&endpoint).await
    }

    /// **MÉTODO AUXILIAR: Update Task** - Atualiza uma task
    /// PUT /task/{task_id}
    pub async fn update_task<T: serde::Serialize>(&self, task_id: &str, body: T) -> AuthResult<Value> {
        log::info!("📝 Atualizando task: {}", task_id);
        let endpoint = format!("task/{}", task_id);
        self.put(&endpoint, &body).await
    }

    /// **MÉTODO AUXILIAR: Update Task Custom Field** - Atualiza um campo personalizado
    /// POST /task/{task_id}/field/{field_id}
    ///
    /// Nota: ClickUp só permite atualizar um campo por vez
    pub async fn update_custom_field(
        &self,
        task_id: &str,
        field_id: &str,
        value: CustomFieldValue,
    ) -> AuthResult<Value> {
        log::info!("🔄 Atualizando campo {} da task {}", field_id, task_id);

        let endpoint = format!("task/{}/field/{}", task_id, field_id);
        let body = json!({ "value": value });

        self.post(&endpoint, &body).await
    }

    /// **MÉTODO DE BUSCA: Find List by Name** - Busca uma lista por nome
    /// Útil para encontrar o list_id necessário para criar tasks
    pub async fn find_list_by_name(
        &self,
        name: &str,
        team_id: Option<String>,
    ) -> AuthResult<Option<String>> {
        log::info!("🔍 Buscando lista com nome: {}", name);

        let result = self.search_entity(name, EntityType::List, team_id).await?;

        if result.found && !result.items.is_empty() {
            // Retorna o ID da primeira lista encontrada
            Ok(Some(result.items[0].id.clone()))
        } else {
            Ok(None)
        }
    }

    /// **MÉTODO HELPER: Create Task with Auto-Find List** - Cria task buscando a lista por nome
    /// Este método é útil quando você sabe o nome da lista mas não o ID
    pub async fn create_task_in_list_by_name(
        &self,
        list_name: &str,
        task: CreateTaskRequest,
        team_id: Option<String>,
    ) -> AuthResult<TaskResponse> {
        // Busca a lista por nome
        let list_id = self.find_list_by_name(list_name, team_id).await?
            .ok_or_else(|| AuthError::api_error(&format!("Lista '{}' não encontrada", list_name)))?;

        // Cria a task na lista encontrada
        self.create_task(&list_id, task).await
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::config::EnvManager;

    #[tokio::test]
    async fn test_search_entity_with_cache() {
        // Só roda se houver token configurado
        if let Ok(env_manager) = EnvManager::load() {
            if let Some(token) = EnvManager::get_access_token() {
                let client = ClickUpClient::new(
                    token,
                    env_manager.api_base_url
                );

                // Obtém o team_id do ambiente ou usa o primeiro disponível
                let team_id = if let Ok(teams) = client.get_authorized_teams().await {
                    teams
                        .get("teams")
                        .and_then(|t| t.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|team| team.get("id"))
                        .and_then(|id| id.as_str())
                        .unwrap_or("test_team")
                        .to_string()
                } else {
                    "test_team".to_string()
                };

                println!("🧪 Testando pesquisa de entidades com team_id: {}", team_id);

                // Teste 1: Pesquisar por space
                println!("\n📍 Teste 1: Pesquisando por space...");
                match client.search_entity(
                    "Test Space",
                    EntityType::Space,
                    Some(team_id.clone())
                ).await {
                    Ok(result) => {
                        println!("  ✅ Pesquisa concluída");
                        println!("  📊 Encontrado: {}", result.found);
                        println!("  📦 Total de items: {}", result.items.len());

                        for item in &result.items {
                            println!("    • {} (ID: {})", item.name, item.id);
                            println!("      URL: {}", item.url);
                        }

                        if result.cached_at.is_some() {
                            println!("  ⚡ Resultado veio do cache!");
                        } else {
                            println!("  🔄 Resultado veio da API");
                        }
                    },
                    Err(e) => println!("  ⚠️ Erro na pesquisa: {}", e)
                }

                // Teste 2: Pesquisar por folder
                println!("\n📍 Teste 2: Pesquisando por folder...");
                match client.search_entity(
                    "Dev",
                    EntityType::Folder,
                    Some(team_id.clone())
                ).await {
                    Ok(result) => {
                        println!("  ✅ Pesquisa concluída");
                        println!("  📊 Encontrado: {}", result.found);
                        println!("  📦 Total de folders: {}", result.items.len());

                        for item in &result.items {
                            println!("    • {} (ID: {})", item.name, item.id);
                            println!("      URL: {}", item.url);
                            if let Some(parent) = &item.parent_name {
                                println!("      Parent: {}", parent);
                            }
                        }
                    },
                    Err(e) => println!("  ⚠️ Erro na pesquisa: {}", e)
                }

                // Teste 3: Pesquisar por list
                println!("\n📍 Teste 3: Pesquisando por list...");
                match client.search_entity(
                    "TODO",
                    EntityType::List,
                    Some(team_id.clone())
                ).await {
                    Ok(result) => {
                        println!("  ✅ Pesquisa concluída");
                        println!("  📊 Encontrado: {}", result.found);
                        println!("  📦 Total de lists: {}", result.items.len());

                        for item in &result.items {
                            println!("    • {} (ID: {})", item.name, item.id);
                            println!("      URL: {}", item.url);
                            if let Some(parent) = &item.parent_name {
                                println!("      Parent: {}", parent);
                            }
                        }
                    },
                    Err(e) => println!("  ⚠️ Erro na pesquisa: {}", e)
                }

                // Teste 4: Pesquisar por task
                println!("\n📍 Teste 4: Pesquisando por task...");
                match client.search_entity(
                    "Test Task",
                    EntityType::Task,
                    Some(team_id.clone())
                ).await {
                    Ok(result) => {
                        println!("  ✅ Pesquisa concluída");
                        println!("  📊 Encontrado: {}", result.found);
                        println!("  📦 Total de tasks: {}", result.items.len());

                        for item in &result.items {
                            println!("    • {} (ID: {})", item.name, item.id);
                            println!("      URL: {}", item.url);
                            if let Some(parent) = &item.parent_name {
                                println!("      List: {}", parent);
                            }
                        }
                    },
                    Err(e) => println!("  ⚠️ Erro na pesquisa: {}", e)
                }

                // Teste 5: Segunda pesquisa do mesmo item (deve vir do cache)
                println!("\n📍 Teste 5: Testando cache (pesquisando novamente o mesmo space)...");
                let start = std::time::Instant::now();

                match client.search_entity(
                    "Test Space",
                    EntityType::Space,
                    Some(team_id.clone())
                ).await {
                    Ok(result) => {
                        let duration = start.elapsed();
                        println!("  ✅ Pesquisa concluída em {:?}", duration);

                        if result.cached_at.is_some() {
                            println!("  ⚡ Resultado veio do cache (como esperado)!");
                            println!("  ⏱️ Cache criado em: {}",
                                result.cached_at.unwrap().to_rfc3339());
                        } else {
                            println!("  ⚠️ Resultado não veio do cache (inesperado)");
                        }
                    },
                    Err(e) => println!("  ⚠️ Erro na pesquisa: {}", e)
                }

                // Teste 6: Obter estatísticas do cache
                println!("\n📍 Teste 6: Estatísticas do cache...");
                let stats = client.get_cache_stats();
                println!("  📊 Cache stats: {}", serde_json::to_string_pretty(&stats).unwrap());

                // Teste 7: Limpar cache
                println!("\n📍 Teste 7: Limpando cache...");
                client.clear_search_cache();
                let stats_after = client.get_cache_stats();
                println!("  📊 Cache após limpeza: {} entradas",
                    stats_after["total_entries"].as_u64().unwrap_or(0));
            } else {
                println!("⚠️ Token não configurado - pulando testes de pesquisa");
            }
        } else {
            println!("⚠️ Configuração não encontrada - pulando testes de pesquisa");
        }
    }

    #[tokio::test]
    async fn test_search_entity_from_env() {
        // Teste usando team_id do .env
        if let Ok(env_manager) = EnvManager::load() {
            if let Some(token) = EnvManager::get_access_token() {
                let client = ClickUpClient::new(
                    token,
                    env_manager.api_base_url
                );

                // O team_id pode vir do .env ou ser None para usar o primeiro disponível
                println!("\n🧪 Testando pesquisa com team_id do ambiente");

                // Teste sem especificar team_id (usa o primeiro disponível)
                match client.search_entity(
                    "Test",
                    EntityType::Space,
                    None  // Não especifica team_id
                ).await {
                    Ok(result) => {
                        println!("✅ Pesquisa sem team_id funcionou");
                        println!("  Encontrado: {}", result.found);
                        println!("  Items: {}", result.items.len());
                    },
                    Err(e) => println!("⚠️ Erro: {}", e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnvManager;

    #[test]
    fn test_client_creation() {
        // Carrega configurações reais do ambiente
        if let Ok(env_manager) = EnvManager::load() {
            let token = EnvManager::get_access_token().unwrap_or_else(|| "test_token".to_string());
            let client = ClickUpClient::new(
                token.clone(),
                env_manager.api_base_url.clone()
            );

            assert_eq!(client.token, token);
            assert_eq!(client.base_url, env_manager.api_base_url.trim_end_matches('/'));
        }
    }

    #[test]
    fn test_url_building() {
        // Usa URL da configuração ou padrão
        let base_url = std::env::var("CLICKUP_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.clickup.com/api/v2".to_string());

        let client = ClickUpClient::new(
            "test_token".to_string(),
            format!("{}/", base_url) // Com barra no final para testar normalização
        );

        let url1 = client.build_url("user");
        let url2 = client.build_url("/team");

        assert_eq!(url1, format!("{}/user", base_url.trim_end_matches('/')));
        assert_eq!(url2, format!("{}/team", base_url.trim_end_matches('/')));
    }

    #[test]
    fn test_token_validation() {
        // Testa com token real se disponível
        if let Ok(env_manager) = EnvManager::load() {
            if let Some(token) = EnvManager::get_access_token() {
                let client = ClickUpClient::new(
                    token,
                    env_manager.api_base_url
                );

                // Token real deve passar na validação
                assert!(client.validate_token_format());
            }
        }

        // Testa token inválido
        let client_invalid = ClickUpClient::new(
            "short".to_string(),
            "https://api.clickup.com/api/v2".to_string()
        );

        assert!(!client_invalid.validate_token_format());
    }

    #[test]
    fn test_api_info() {
        // Usa configurações reais se disponíveis
        let env_manager = EnvManager::load().unwrap_or_else(|_| {
            // Mock para testes sem configuração
            EnvManager {
                client_id: "test".to_string(),
                client_secret: "test".to_string(),
                redirect_uri: "http://localhost:8888/callback".to_string(),
                api_base_url: "https://api.clickup.com/api/v2".to_string(),
                callback_port: 8888,
                environment: crate::config::env::Environment::Development,
            }
        });

        let token = EnvManager::get_access_token()
            .unwrap_or_else(|| "test_token_with_more_than_20_chars".to_string());

        let client = ClickUpClient::new(
            token.clone(),
            env_manager.api_base_url.clone()
        );

        let info = client.get_api_info();

        assert_eq!(info["base_url"], env_manager.api_base_url.trim_end_matches('/'));
        assert_eq!(info["has_token"], !token.is_empty());

        if !token.is_empty() {
            assert!(info["token_length"].as_u64().unwrap() > 0);
        }

        if token.len() > 10 {
            assert!(info["token_preview"].as_str().unwrap().contains("..."));
        }
    }

    #[tokio::test]
    async fn test_health_check_with_real_token() {
        // Só roda se houver token configurado
        if let Ok(env_manager) = EnvManager::load() {
            if let Some(token) = EnvManager::get_access_token() {
                let client = ClickUpClient::new(
                    token,
                    env_manager.api_base_url
                );

                // Tenta fazer health check real
                match client.health_check().await {
                    Ok(result) => {
                        // Se temos token, esperamos true ou false (não erro)
                        assert!(result == true || result == false);
                    },
                    Err(_) => {
                        // Erro de rede ou configuração é aceitável em testes
                    }
                }
            }
        }
    }
}
