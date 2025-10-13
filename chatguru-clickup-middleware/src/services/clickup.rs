use crate::config::Settings;
use crate::models::WebhookPayload;
use crate::services::estrutura::EstruturaService;
use crate::services::secrets::SecretManagerService;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
// use chrono::Duration;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::info;

#[derive(Clone)]
pub struct ClickUpService {
    client: Client,
    token: String,
    list_id: String,
    base_url: String,
    pub estrutura_service: Option<std::sync::Arc<EstruturaService>>,
}

impl ClickUpService {
    pub fn new(settings: Settings, estrutura_service: Option<std::sync::Arc<EstruturaService>>) -> Self {
        // OTIMIZAÇÃO FASE 1: Cliente HTTP com timeout padrão de 30s
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
            estrutura_service,
        }
    }
    
    /// Cria uma nova instância usando Secret Manager com fallback para env vars
    pub async fn new_with_secrets() -> AppResult<Self> {
        let secret_service = SecretManagerService::new().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao inicializar Secret Manager: {}", e)))?;

        // Obtém configurações com fallback automático
        let api_token = secret_service.get_clickup_api_token().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp API Token: {}", e)))?;
        let list_id = secret_service.get_clickup_list_id().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp List ID: {}", e)))?;

        // Limpar token de espaços em branco (newlines, spaces) que podem vir do Secret Manager
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
            estrutura_service: None,
        })
    }

    /// Define o EstruturaService após a construção
    pub fn with_estrutura_service(mut self, service: std::sync::Arc<EstruturaService>) -> Self {
        self.estrutura_service = Some(service);
        self
    }

    pub async fn create_task_from_json(&self, task_data: &Value) -> AppResult<Value> {
        let url = format!("{}/list/{}/task", self.base_url, self.list_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .header("Content-Type", "application/json")
            .json(task_data)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }

    pub async fn test_connection(&self) -> AppResult<Value> {
        let url = format!("{}/user", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }

    pub async fn get_list_info(&self) -> AppResult<Value> {
        let url = format!("{}/list/{}", self.base_url, self.list_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }



    /// Busca uma tarefa existente na lista que tenha título igual ao título da nova tarefa.
    ///
    /// NOTA: Se o token não tiver permissão para listar tasks (OAuth2 com permissões limitadas),
    /// retorna None (assume que não há duplicata) em vez de falhar.
    pub async fn find_existing_task_in_list(
        &self,
        title: &str,
    ) -> AppResult<Option<Value>> {
        let url = format!("{}/list/{}/task?archived=false", self.base_url, self.list_id);

        let resp = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", &self.token))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();

            // Se erro é OAUTH_027 (sem permissão para listar), retorna None em vez de falhar
            if err_text.contains("OAUTH_027") || err_text.contains("Team not authorized") {
                log_warning(&format!("⚠️  Token sem permissão para listar tasks ({}). Assumindo que não há duplicatas.", err_text));
                return Ok(None);
            }

            log_clickup_api_error(&url, Some(status.as_u16()), &err_text);
            return Err(AppError::ClickUpApi(format!("Failed to get tasks: {}", err_text)));
        }

        let json_resp: Value = resp.json().await?;
        if let Some(tasks) = json_resp.get("tasks").and_then(|v| v.as_array()) {
            for task in tasks {
                if let Some(task_name) = task.get("name").and_then(|v| v.as_str()) {
                    if task_name == title {
                        return Ok(Some(task.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Adiciona comentário à tarefa para preservar o histórico.
    pub async fn add_comment_to_task(&self, task_id: &str, comment: &str) -> AppResult<()> {
        let url = format!("{}/task/{}/comment", self.base_url, task_id);

        let body = json!({
            "comment_text": comment
        });

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

    /// Atualiza tarefa existente via API PUT.
    pub async fn update_task(&self, task_id: &str, task_data: &Value) -> AppResult<Value> {
        let url = format!("{}/task/{}", self.base_url, task_id);

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

    /// Cria tarefa dinamicamente na estrutura correta baseada em Info_1 (attendant) e Info_2 (client)
    ///
    /// LÓGICA CORRIGIDA:
    /// - attendant_name (responsavel_nome): Determina o SPACE no ClickUp
    /// - client_name (usado para resolução da estrutura, mas Info_1/Info_2 são apenas campos personalizados)
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
