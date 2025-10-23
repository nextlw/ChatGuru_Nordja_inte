// ============================================================================
// ClickUp Service - Camada de integração com a API do ClickUp
// ============================================================================
//
// Este serviço encapsula todas as operações de comunicação com a API do ClickUp,
// incluindo:
//
// 1. **Gerenciamento de Tarefas:**
//    - Criação de tarefas (create_task_from_json)
//    - Atualização de tarefas existentes (update_task)
//    - Busca de duplicatas (find_existing_task_in_list)
//    - Adição de comentários (add_comment_to_task)
//
// 2. **Resolução Dinâmica de Estrutura:**
//    - Integração com EstruturaService para mapear Cliente + Atendente → Folder/List
//    - Sistema de 3 camadas: cache in-memory → DB → API do ClickUp
//    - Suporte a estrutura mensal automática (OUTUBRO 2025, NOVEMBRO 2025, etc)
//
// 3. **Administração e Debug:**
//    - Teste de conectividade (test_connection)
//    - Obtenção de informações de lista (get_list_info)
//
// # Arquitetura de Processamento
//
// ```
// WebhookPayload → process_payload_with_ai() → Resolve Folder/List → Create/Update Task
//                       ↓
//                  AI Classification (OpenAI)
//                       ↓
//                  Custom Fields Mapping
// ```
//
// # Autenticação
//
// Suporta dois modos:
// - **Personal Token**: Token fixo com todas as permissões
// - **OAuth2 Access Token**: Token dinâmico com permissões limitadas
//
// # Feature Flags
//
// - `DYNAMIC_STRUCTURE_ENABLED`: habilita/desabilita resolução dinâmica de estrutura
// - `FALLBACK_LIST_ID`: ID da lista para usar quando resolução falhar

use crate::config::Settings;
use crate::models::WebhookPayload;
// REMOVIDO: EstruturaService (substituído por SmartFolderFinder)
// use crate::services::estrutura::EstruturaService;
use crate::services::secrets::SecretManagerService;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
// use chrono::Duration;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::info;

/// Serviço de integração com a API do ClickUp
///
/// # Campos
///
/// - `client`: Cliente HTTP Reqwest configurado com timeouts (30s total, 5s connect)
/// - `token`: Token de autenticação (Personal Token ou OAuth2 Access Token)
/// - `list_id`: ID da lista padrão/fallback para criação de tarefas
/// - `base_url`: URL base da API do ClickUp (https://api.clickup.com/api/v2)
///
/// # Timeouts
///
/// - **Total timeout**: 30s (tempo máximo para completar requisição)
/// - **Connect timeout**: 5s (tempo máximo para estabelecer conexão)
///
/// # Thread-Safety
///
/// Este struct implementa `Clone` e pode ser compartilhado entre threads via `Arc<>`.
#[derive(Clone)]
pub struct ClickUpService {
    client: Client,
    token: String,
    list_id: String,
    base_url: String,
    // REMOVIDO: estrutura_service (substituído por SmartFolderFinder no worker)
}

impl ClickUpService {
    /// Cria uma nova instância do ClickUpService a partir de Settings
    ///
    /// # Argumentos
    ///
    /// - `settings`: Configurações carregadas de arquivo TOML ou variáveis de ambiente
    /// - `_estrutura_service`: Parâmetro deprecated (mantido por compatibilidade)
    ///
    /// # Configuração do Cliente HTTP
    ///
    /// O cliente Reqwest é configurado com:
    /// - **Timeout total**: 30s (previne requisições travadas indefinidamente)
    /// - **Connect timeout**: 5s (previne delays em conexões lentas)
    /// - **Fallback**: Se construção falhar, usa `Client::new()` com defaults
    ///
    /// # Uso
    ///
    /// ```rust,ignore
    /// let clickup = ClickUpService::new(settings, None);
    /// ```
    pub fn new(settings: Settings, _estrutura_service: Option<()>) -> Self {
        // OTIMIZAÇÃO FASE 1: Cliente HTTP com timeout padrão de 30s
        // Previne requisições travadas e melhora resiliência do sistema
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        log_info("⚡ ClickUp client configured with 30s timeout");

        Self {
            client,
            token: settings.clickup.token.clone(),
            list_id: settings.clickup.list_id.clone(),
            base_url: settings.clickup.base_url.clone(),
        }
    }

    /// Cria uma nova instância usando Google Secret Manager
    ///
    /// # Descrição
    ///
    /// Método alternativo de construção que busca credenciais do Google Secret Manager
    /// ao invés de usar arquivo de configuração. Útil para ambientes Cloud Run/GCP.
    ///
    /// # Secrets Necessários
    ///
    /// - `clickup-api-token` ou `CLICKUP_API_TOKEN` (env var fallback)
    /// - `clickup-list-id` ou `CLICKUP_LIST_ID` (env var fallback)
    ///
    /// # Processo
    ///
    /// 1. Inicializa SecretManagerService (com autenticação GCP)
    /// 2. Busca `clickup-api-token` do Secret Manager
    /// 3. Busca `clickup-list-id` do Secret Manager
    /// 4. Remove whitespace do token (newlines são comuns em secrets)
    /// 5. Constrói cliente HTTP com timeouts configurados
    ///
    /// # Retorno
    ///
    /// - `Ok(ClickUpService)`: Instância configurada com sucesso
    /// - `Err(AppError::ConfigError)`: Falha ao buscar secrets
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// let clickup = ClickUpService::new_with_secrets().await?;
    /// ```
    pub async fn new_with_secrets() -> AppResult<Self> {
        // Inicializar Secret Manager (autenticação automática via GCP credentials)
        let secret_service = SecretManagerService::new().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao inicializar Secret Manager: {}", e)))?;

        // Buscar credenciais do Secret Manager com fallback para env vars
        let api_token = secret_service.get_clickup_api_token().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp API Token: {}", e)))?;
        let list_id = secret_service.get_clickup_list_id().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp List ID: {}", e)))?;

        // Limpar token de espaços em branco (newlines, spaces) que podem vir do Secret Manager
        // IMPORTANTE: Secret Manager frequentemente adiciona \n ao final de secrets
        let api_token = api_token.trim().to_string();

        info!("ClickUp Service configurado - List ID: {}", list_id);

        // OTIMIZAÇÃO FASE 1: Cliente HTTP com timeout padrão de 30s
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Ok(Self {
            client,
            token: api_token,
            list_id: list_id.clone(),
            base_url: "https://api.clickup.com/api/v2".to_string(),
        })
    }

    /// Define o EstruturaService após a construção (pattern Builder)
    ///
    /// # Descrição
    ///
    /// Permite injetar o EstruturaService após a construção inicial, útil quando:
    /// - ClickUpService e EstruturaService têm dependências circulares
    /// - EstruturaService precisa ser inicializado com pool de DB primeiro
    ///
    /// # Argumentos
    ///
    /// - `service`: Arc do EstruturaService (permite compartilhamento thread-safe)
    ///
    /// # Retorno
    ///
    /// Retorna `self` para permitir chaining (pattern fluent)
    ///
    // MÉTODO REMOVIDO: with_estrutura_service
    // Substituído por SmartFolderFinder usado diretamente no worker
    //
    // pub fn with_estrutura_service(mut self, service: std::sync::Arc<EstruturaService>) -> Self {
    //     self.estrutura_service = Some(service);
    //     self
    // }

    /// Cria uma tarefa no ClickUp a partir de dados JSON
    ///
    /// # Descrição
    ///
    /// Método de baixo nível que cria uma tarefa na lista configurada (`self.list_id`).
    /// Usado internamente por `process_payload()` e `create_task_dynamic()`.
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/list/{list_id}/task`
    ///
    /// # Argumentos
    ///
    /// - `task_data`: JSON com estrutura da tarefa do ClickUp:
    ///   ```json
    ///   {
    ///     "name": "Título da tarefa",
    ///     "description": "Descrição markdown",
    ///     "priority": 3,
    ///     "tags": ["tag1", "tag2"],
    ///     "custom_fields": [
    ///       {"id": "field-id", "value": "valor"}
    ///     ]
    ///   }
    ///   ```
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Tarefa criada com sucesso, retorna JSON com `id`, `url`, etc
    /// - `Err(AppError::ClickUpApi)`: Falha na criação (status HTTP != 2xx)
    ///
    /// # Erros Comuns
    ///
    /// - **401 Unauthorized**: Token inválido ou expirado
    /// - **403 Forbidden**: Sem permissão para criar tarefas nesta lista
    /// - **404 Not Found**: list_id não existe
    /// - **400 Bad Request**: Estrutura JSON inválida ou custom_field_id incorreto
    pub async fn create_task_from_json(&self, task_data: &Value) -> AppResult<Value> {
        let url = format!("{}/list/{}/task", self.base_url, self.list_id);

        // Enviar requisição POST com JSON no body
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(task_data)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            // Sucesso: retornar JSON da tarefa criada (contém id, url, etc)
            Ok(response.json().await?)
        } else {
            // Erro: logar detalhes e retornar erro estruturado
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }

    /// Testa conectividade com a API do ClickUp
    ///
    /// # Descrição
    ///
    /// Faz uma chamada simples à API para validar:
    /// - Token de autenticação está válido
    /// - API está acessível
    /// - Permissões básicas estão corretas
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/user`
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Conexão bem-sucedida, retorna informações do usuário:
    ///   ```json
    ///   {
    ///     "user": {
    ///       "id": 123456,
    ///       "username": "user@example.com",
    ///       "email": "user@example.com",
    ///       "color": "#FF0000"
    ///     }
    ///   }
    ///   ```
    /// - `Err(AppError::ClickUpApi)`: Falha na conexão ou autenticação
    ///
    /// # Uso
    ///
    /// - Health checks
    /// - Validação de configuração no startup
    /// - Debug de problemas de autenticação
    pub async fn test_connection(&self) -> AppResult<Value> {
        let url = format!("{}/user", self.base_url);

        // Fazer requisição GET simples para endpoint /user
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            // Sucesso: retornar informações do usuário
            Ok(response.json().await?)
        } else {
            // Erro: token inválido, sem permissão, ou API inacessível
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }

    /// Obtém informações detalhadas sobre a lista configurada
    ///
    /// # Descrição
    ///
    /// Busca metadados completos da lista, incluindo:
    /// - Nome e descrição da lista
    /// - Status disponíveis (To Do, In Progress, Done, etc)
    /// - Custom fields e seus IDs
    /// - Configurações de permissões
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/list/{list_id}`
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Informações da lista em formato JSON
    ///   ```json
    ///   {
    ///     "id": "901321080769",
    ///     "name": "OUTUBRO 2025",
    ///     "status": [...],
    ///     "custom_fields": [
    ///       {
    ///         "id": "field-uuid",
    ///         "name": "Categoria",
    ///         "type": "drop_down",
    ///         "type_config": { "options": [...] }
    ///       }
    ///     ]
    ///   }
    ///   ```
    /// - `Err(AppError::ClickUpApi)`: Falha ao buscar informações
    ///
    /// # Uso
    ///
    /// - Debug: verificar custom fields disponíveis
    /// - Mapeamento: obter IDs de campos para usar em payloads
    /// - Validação: conferir se lista está corretamente configurada
    pub async fn get_list_info(&self) -> AppResult<Value> {
        let url = format!("{}/list/{}", self.base_url, self.list_id);

        // Buscar informações da lista via GET
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            // Sucesso: retornar JSON completo da lista
            Ok(response.json().await?)
        } else {
            // Erro: list_id não existe ou sem permissão
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }



    /// Busca uma tarefa existente na lista pelo título (detecção de duplicatas)
    ///
    /// # Descrição
    ///
    /// Lista todas as tarefas não-arquivadas da lista e busca por título exato.
    /// Usado para detectar duplicatas antes de criar nova tarefa.
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/list/{list_id}/task?archived=false`
    ///
    /// # Argumentos
    ///
    /// - `title`: Título exato da tarefa para buscar (case-sensitive)
    ///
    /// # Retorno
    ///
    /// - `Ok(Some(Value))`: Tarefa encontrada, retorna JSON completo da tarefa
    /// - `Ok(None)`: Nenhuma tarefa com este título encontrada
    /// - `Err(AppError)`: Erro na comunicação com API
    ///
    /// # Tratamento Especial de Permissões
    ///
    /// Se o token OAuth2 não tiver permissão para listar tarefas (erro OAUTH_027),
    /// retorna `Ok(None)` ao invés de falhar. Isso permite que o fluxo continue
    /// criando a tarefa mesmo sem poder verificar duplicatas.
    ///
    /// **Códigos de erro tolerados:**
    /// - `OAUTH_027`: Team not authorized
    /// - Mensagens contendo "Team not authorized"
    ///
    /// # Performance
    ///
    /// ATENÇÃO: Este método pode ser lento se a lista tiver muitas tarefas (>1000),
    /// pois a API do ClickUp não suporta busca por título. Considera-se implementar
    /// cache ou índice em banco de dados para listas grandes.
    pub async fn find_existing_task_in_list(
        &self,
        title: &str,
    ) -> AppResult<Option<Value>> {
        let url = format!("{}/list/{}/task?archived=false", self.base_url, self.list_id);

        // Listar todas as tarefas não-arquivadas da lista
        let resp = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();

            // TRATAMENTO ESPECIAL: Se token OAuth2 não tem permissão para listar,
            // retornar None (assume que não há duplicatas) em vez de falhar completamente
            if err_text.contains("OAUTH_027") || err_text.contains("Team not authorized") {
                log_warning(&format!("⚠️  Token sem permissão para listar tasks ({}). Assumindo que não há duplicatas.", err_text));
                return Ok(None);
            }

            // Outros erros: logar e propagar
            log_clickup_api_error(&url, Some(status.as_u16()), &err_text);
            return Err(AppError::ClickUpApi(format!("Failed to get tasks: {}", err_text)));
        }

        // Buscar tarefa com título exato (case-sensitive)
        let json_resp: Value = resp.json().await?;
        if let Some(tasks) = json_resp.get("tasks").and_then(|v| v.as_array()) {
            for task in tasks {
                if let Some(task_name) = task.get("name").and_then(|v| v.as_str()) {
                    if task_name == title {
                        // Tarefa encontrada: retornar JSON completo
                        return Ok(Some(task.clone()));
                    }
                }
            }
        }

        // Nenhuma tarefa com este título encontrada
        Ok(None)
    }

    /// Adiciona comentário a uma tarefa existente (preservar histórico)
    ///
    /// # Descrição
    ///
    /// Cria um novo comentário na tarefa especificada. Usado para:
    /// - Registrar histórico de atualizações
    /// - Preservar dados da versão anterior ao atualizar tarefa
    /// - Adicionar contexto sobre mudanças automáticas via webhook
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/task/{task_id}/comment`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa no ClickUp (string numérica ou UUID)
    /// - `comment`: Texto do comentário (suporta Markdown)
    ///
    /// # Retorno
    ///
    /// - `Ok(())`: Comentário adicionado com sucesso
    /// - `Err(AppError::ClickUpApi)`: Falha ao adicionar comentário
    ///
    /// # Formato do Comentário de Histórico
    ///
    /// Comentários criados automaticamente seguem este formato:
    /// ```markdown
    /// 📝 **Atualização Automática via Webhook**
    ///
    /// **Timestamp:** 2025-10-14T16:20:00Z
    /// **Tipo de Payload:** ChatGuru
    ///
    /// ---
    ///
    /// **Histórico da Versão Anterior:**
    /// - **Título:** [título antigo]
    /// - **Última Atualização:** [timestamp]
    ///
    /// **Descrição Anterior:**
    /// ```
    /// [descrição antiga]
    /// ```
    /// ```
    ///
    /// # Uso
    ///
    /// ```rust,ignore
    /// self.add_comment_to_task(
    ///     "task-123",
    ///     "📝 Tarefa atualizada automaticamente via webhook"
    /// ).await?;
    /// ```
    pub async fn add_comment_to_task(&self, task_id: &str, comment: &str) -> AppResult<()> {
        let url = format!("{}/task/{}/comment", self.base_url, task_id);

        // Construir payload JSON para comentário
        let body = json!({
            "comment_text": comment
        });

        // Enviar POST com comentário
        let resp = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &err_text);
            return Err(AppError::ClickUpApi(format!("Failed to add comment: {}", err_text)));
        }

        Ok(())
    }

    /// Atualiza uma tarefa existente no ClickUp
    ///
    /// # Descrição
    ///
    /// Atualiza campos de uma tarefa existente via API PUT. Apenas os campos
    /// presentes em `task_data` serão atualizados (patch parcial).
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa a atualizar
    /// - `task_data`: JSON com campos a atualizar:
    ///   ```json
    ///   {
    ///     "name": "Novo título",
    ///     "description": "Nova descrição",
    ///     "priority": 1,
    ///     "custom_fields": [...]
    ///   }
    ///   ```
    ///
    /// # Comportamento de Atualização
    ///
    /// - **Campos omitidos**: mantêm valor atual (não são limpos)
    /// - **Campos presentes**: substituem valor atual
    /// - **custom_fields**: SUBSTITUIÇÃO COMPLETA (não merge)
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Tarefa atualizada, retorna JSON completo da nova versão
    /// - `Err(AppError::ClickUpApi)`: Falha na atualização
    ///
    /// # IMPORTANTE: Custom Fields
    ///
    /// Ao atualizar custom_fields, o array SUBSTITUI completamente os valores
    /// anteriores. Para preservar campos não modificados, inclua-os no payload.
    ///
    /// # Uso
    ///
    /// ```rust,ignore
    /// let updated = self.update_task(
    ///     "task-123",
    ///     &json!({"name": "Novo título", "priority": 1})
    /// ).await?;
    /// ```
    pub async fn update_task(&self, task_id: &str, task_data: &Value) -> AppResult<Value> {
        let url = format!("{}/task/{}", self.base_url, task_id);

        // Enviar PUT com dados da atualização
        let resp = self.client.put(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(task_data)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &err_text);
            return Err(AppError::ClickUpApi(format!("Failed to update task: {}", err_text)));
        }

        // Retornar JSON da tarefa atualizada
        let updated_task = resp.json().await?;
        Ok(updated_task)
    }

    /// Processa webhook payload de qualquer formato (ChatGuru, EventType, Generic)
    /// Regras de negócio:
    /// 1. Verifica se já existe tarefa com mesmo título
    /// 2. Se existir, adiciona comentário com histórico e atualiza tarefa
    /// 3. Se não existir, cria tarefa nova
    pub async fn process_payload(&self, payload: &WebhookPayload) -> AppResult<Value> {
        self.process_payload_with_ai(payload, None).await
    }
    
    /// Processa webhook payload com classificação AI opcional
    pub async fn process_payload_with_ai(
        &self, 
        payload: &WebhookPayload,
        ai_classification: Option<&crate::services::openai::OpenAIClassification>
    ) -> AppResult<Value> {
        // Extrair título e dados da tarefa
        let task_title = payload.get_task_title();
        let task_data = if ai_classification.is_some() {
            payload.to_clickup_task_data_with_ai(ai_classification)
        } else {
            payload.to_clickup_task_data()
        };
        
        log_info(&format!("Processing webhook payload - Task title: {}", task_title));
        
        // Buscar tarefa existente
        if let Some(existing_task) = self.find_existing_task_in_list(&task_title).await? {
            // Tarefa existe - atualizar
            let task_id = existing_task.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            log_info(&format!("Found existing task with ID: {} - Will update", task_id));
            
            // Adicionar comentário com histórico
            let history_comment = self.build_history_comment(&existing_task, payload);
            self.add_comment_to_task(task_id, &history_comment).await?;
            
            // Atualizar tarefa
            let updated_task = self.update_task(task_id, &task_data).await?;
            log_clickup_task_updated(task_id, &task_title);
            
            Ok(updated_task)
        } else {
            // Tarefa não existe - criar nova
            log_info("No existing task found - Creating new task");
            let new_task = self.create_task_from_json(&task_data).await?;
            
            if let (Some(id), Some(name)) = (
                new_task.get("id").and_then(|v| v.as_str()),
                new_task.get("name").and_then(|v| v.as_str())
            ) {
                log_clickup_task_created(id, name);
            }
            
            Ok(new_task)
        }
    }
    
    /// Constrói comentário com histórico para atualização de tarefa
    fn build_history_comment(&self, existing_task: &Value, payload: &WebhookPayload) -> String {
        let prev_title = existing_task.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let prev_description = existing_task.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let prev_updated = if let Some(date_str) = existing_task.get("date_updated").and_then(|v| v.as_str()) {
            date_str.to_string()
        } else if let Some(date_num) = existing_task.get("date_updated").and_then(|v| v.as_u64()) {
            date_num.to_string()
        } else {
            "Unknown".to_string()
        };
        
        let payload_type = match payload {
            WebhookPayload::ChatGuru(_) => "ChatGuru",
            WebhookPayload::EventType(_) => "EventType", 
            WebhookPayload::Generic(_) => "Generic",
        };
        
        format!(
            "📝 **Atualização Automática via Webhook**\n\n\
            **Timestamp:** {}\n\
            **Tipo de Payload:** {}\n\n\
            ---\n\n\
            **Histórico da Versão Anterior:**\n\
            - **Título:** {}\n\
            - **Última Atualização:** {}\n\n\
            **Descrição Anterior:**\n```\n{}\n```",
            chrono::Utc::now().to_rfc3339(),
            payload_type,
            prev_title,
            prev_updated,
            prev_description
        )
    }

    // ==================================================================================
    // DEPRECATED: create_task_by_client() - Usa FolderResolver (YAML) e EstruturaService (DB)
    // ==================================================================================
    // SUBSTITUÍDO POR: SmartFolderFinder (busca via API do ClickUp)
    //
    // Este método foi deprecado porque:
    // 1. Dependia de YAML estático (client_to_folder_mapping.yaml)
    // 2. Dependia de EstruturaService (PostgreSQL)
    // 3. Não conseguia encontrar clientes novos sem atualizar YAML manualmente
    //
    // A nova arquitetura usa:
    // - SmartFolderFinder: Busca folders via API do ClickUp com fuzzy matching
    // - Fallback para histórico de tarefas (campo Cliente Solicitante)
    // - Auto-criação de listas mensais quando necessário
    //
    // Veja: src/handlers/worker.rs (linhas 509-735) para implementação atual
    // ==================================================================================
    /*
    pub async fn create_task_by_client(
        &self,
        task_data: &Value,
        client_name: &str,
    ) -> AppResult<Value> {
        use crate::services::folder_resolver::FolderResolver;

        tracing::info!("📝 Criando tarefa para cliente: '{}'", client_name);

        // 1. Carregar folder resolver
        let folder_resolver = FolderResolver::load_default()
            .map_err(|e| {
                tracing::error!("❌ Falha ao carregar folder resolver: {}", e);
                AppError::ConfigError(format!("Falha ao carregar mapeamento de clientes: {}", e))
            })?;

        // 2. Resolver folder_id a partir do nome do cliente
        let resolution = folder_resolver.resolve(client_name);

        tracing::info!("📊 Resolução: match_type={:?}, folder_id={}, similarity={:?}",
            resolution.match_type,
            resolution.folder_id,
            resolution.similarity_score
        );

        // 3. Obter lista mensal
        // IMPORTANTE: Não passamos client_name aqui porque cada pasta (folder_id)
        // deve ter apenas uma lista "OUTUBRO 2025", não "Cliente - OUTUBRO 2025"
        let list_id = if let Some(ref estrutura_service) = self.estrutura_service {
            match estrutura_service.resolve_monthly_list(&resolution.folder_id, None).await {
                Ok(monthly_list_id) => {
                    tracing::info!("✅ Lista mensal resolvida: {}", monthly_list_id);
                    monthly_list_id
                }
                Err(e) => {
                    tracing::warn!("⚠️ Falha ao resolver lista mensal: {} - Usando folder direto", e);
                    resolution.folder_id.clone()
                }
            }
        } else {
            tracing::warn!("⚠️ EstruturaService não disponível - Usando folder_id como list_id");
            resolution.folder_id.clone()
        };

        // 4. Criar task na lista
        let url = format!("{}/list/{}/task", self.base_url, list_id);

        tracing::info!("🚀 POST {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(task_data)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let task_response = response.json().await?;
            tracing::info!("✅ Task criada com sucesso: list_id={}, client={}",
                list_id, client_name);
            Ok(task_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!("❌ Falha ao criar task: status={}, error={}",
                status, error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }
    */

    // ==================================================================================
    // DEPRECATED: create_task_dynamic() - Usa EstruturaService (DB) para resolução de estrutura
    // ==================================================================================
    // SUBSTITUÍDO POR: SmartFolderFinder + SmartAssigneeFinder + CustomFieldManager
    //
    // Este método foi deprecado porque:
    // 1. Dependia de EstruturaService (PostgreSQL) para mapear Client+Attendant → Folder/List
    // 2. Usava feature flag DYNAMIC_STRUCTURE_ENABLED (complexidade desnecessária)
    // 3. Não conseguia encontrar estruturas novas sem popular DB manualmente
    //
    // A nova arquitetura usa:
    // - SmartFolderFinder: Busca folders via API usando Info_2 (nome do cliente)
    // - SmartAssigneeFinder: Busca assignees via API usando responsavel_nome
    // - CustomFieldManager: Sincroniza "Cliente Solicitante" com folder name
    // - Todas as buscas têm fallback para histórico de tarefas
    //
    // Veja: src/handlers/worker.rs (linhas 509-735) para implementação atual
    // ==================================================================================
    /*
    pub async fn create_task_dynamic(
        &self,
        task_data: &Value,
        attendant_name: &str,  // responsavel_nome determina Space
        client_name: &str,     // usado para resolução de estrutura
    ) -> AppResult<Value> {
        use std::env;

        // Verificar feature flag DYNAMIC_STRUCTURE_ENABLED
        let dynamic_enabled = env::var("DYNAMIC_STRUCTURE_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let list_id = if dynamic_enabled {
            // Tentar resolução dinâmica se EstruturaService estiver disponível
            if let Some(ref estrutura_service) = self.estrutura_service {
                // LÓGICA CORRIGIDA: attendant_name (responsavel_nome) determina Space, client_name usado para resolução
                match estrutura_service.resolve_folder(client_name, attendant_name).await {
                    Ok(folder_info) => {
                        info!("✅ Resolved folder: {} (id: {})", folder_info.folder_path, folder_info.folder_id);

                        // Resolver lista mensal dentro da pasta (passando folder_path para incluir nome do cliente se necessário)
                        match estrutura_service.resolve_monthly_list(&folder_info.folder_id, Some(&folder_info.folder_path)).await {
                            Ok(monthly_list_id) => {
                                info!("✅ Resolved monthly list: {}", monthly_list_id);
                                monthly_list_id
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ Failed to resolve monthly list: {} - Using fallback", e);
                                self.get_fallback_list_id(Some(client_name), Some(attendant_name)).await
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ Failed to resolve folder: {} - Using fallback", e);
                        self.get_fallback_list_id(Some(client_name), Some(attendant_name)).await
                    }
                }
            } else {
                // Sem EstruturaService - usar fallback
                tracing::warn!("⚠️ EstruturaService not available - Using fallback");
                self.get_fallback_list_id(Some(client_name), Some(attendant_name)).await
            }
        } else {
            // Feature desabilitada - usar sempre fallback
            info!("ℹ️ Dynamic structure disabled by DYNAMIC_STRUCTURE_ENABLED=false - Using fallback");
            self.get_fallback_list_id(Some(client_name), Some(attendant_name)).await
        };

        // Criar task usando a lista resolvida
        let url = format!("{}/list/{}/task", self.base_url, list_id);

        info!("📝 Creating task in list: {} (dynamic_enabled: {})", list_id, dynamic_enabled);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(task_data)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let task_response = response.json().await?;
            info!("✅ Dynamic task created successfully in list: {}", list_id);
            Ok(task_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }
    */

    /// Obtém ID da lista de fallback - agora apenas usa configuração
    /// Não tenta mais criar estrutura dinâmica
    async fn get_fallback_list_id(&self, _client_name: Option<&str>, _attendant_name: Option<&str>) -> String {
        // Usar ID da env var ou config (sem tentar criar dinamicamente)
        std::env::var("FALLBACK_LIST_ID")
            .unwrap_or_else(|_| {
                log_warning("⚠️ Usando fallback hardcoded da configuração");
                self.list_id.clone()
            })
    }
}
