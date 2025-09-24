use crate::config::Settings;
use crate::models::WebhookPayload;
use crate::services::secret_manager::SecretManagerService;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::info;

#[derive(Clone)]
pub struct ClickUpService {
    client: Client,
    token: String,
    list_id: String,
}

impl ClickUpService {
    pub fn new(settings: &Settings) -> Self {
        Self {
            client: Client::new(),
            token: settings.clickup.token.clone(),
            list_id: settings.clickup.list_id.clone(),
        }
    }
    
    /// Cria uma nova instância usando Secret Manager com fallback para env vars
    pub async fn new_with_secret_manager() -> AppResult<Self> {
        let secret_service = SecretManagerService::new().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao inicializar Secret Manager: {}", e)))?;
        
        // Obtém configurações com fallback automático
        let api_token = secret_service.get_clickup_api_token().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp API Token: {}", e)))?;
        let list_id = secret_service.get_clickup_list_id().await
            .map_err(|e| AppError::ConfigError(format!("Erro ao obter ClickUp List ID: {}", e)))?;
        
        info!("ClickUp Service configurado - List ID: {}", list_id);
        
        Ok(Self {
            client: Client::new(),
            token: api_token,
            list_id,
        })
    }

    pub async fn create_task_from_json(&self, task_data: &Value) -> AppResult<Value> {
        let url = format!("https://api.clickup.com/api/v2/list/{}/task", self.list_id);
        
        let response = self.client
            .post(&url)
            .header("Authorization", &self.token)
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
        let url = "https://api.clickup.com/api/v2/user";
        
        let response = self.client
            .get(url)
            .header("Authorization", &self.token)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_clickup_api_error(url, Some(status.as_u16()), &error_text);
            Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
        }
    }

    pub async fn get_list_info(&self) -> AppResult<Value> {
        let url = format!("https://api.clickup.com/api/v2/list/{}", self.list_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.token)
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
    pub async fn find_existing_task_in_list(
        &self,
        title: &str,
    ) -> AppResult<Option<Value>> {
        let url = format!("https://api.clickup.com/api/v2/list/{}/task?archived=false", self.list_id);

        let resp = self.client.get(&url)
            .header("Authorization", &self.token)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
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
        let url = format!("https://api.clickup.com/api/v2/task/{}/comment", task_id);

        let body = json!({
            "comment_text": comment
        });

        let resp = self.client.post(&url)
            .header("Authorization", &self.token)
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
        let url = format!("https://api.clickup.com/api/v2/task/{}", task_id);

        let resp = self.client.put(&url)
            .header("Authorization", &self.token)
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
    pub async fn process_webhook_payload(&self, payload: &WebhookPayload) -> AppResult<Value> {
        self.process_webhook_payload_with_ai(payload, None).await
    }
    
    /// Processa webhook payload com classificação AI opcional
    pub async fn process_webhook_payload_with_ai(
        &self, 
        payload: &WebhookPayload,
        ai_classification: Option<&crate::services::vertex_ai::ActivityClassification>
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
}
