use crate::config::Settings;
use crate::models::ChatGuruEvent;
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

    pub async fn create_task_from_event(&self, event: &ChatGuruEvent) -> AppResult<Value> {
        let task_data = self.build_task_data(event);
        let url = format!("https://api.clickup.com/api/v2/list/{}/task", self.list_id);

        log_chatguru_event(&event.event_type, &serde_json::to_value(event)?);

        let response = self.client
            .post(&url)
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&task_data)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            let task_response: Value = response.json().await?;
            
            if let (Some(id), Some(name)) = (
                task_response.get("id").and_then(|v| v.as_str()),
                task_response.get("name").and_then(|v| v.as_str())
            ) {
                log_clickup_task_created(id, name);
            }
            
            Ok(task_response)
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

    fn build_task_data(&self, event: &ChatGuruEvent) -> Value {
        let title = self.generate_task_title(event);
        let description = self.generate_task_description(event);
        
        json!({
            "name": title,
            "description": description,
            "status": "pendente",
            "priority": self.get_priority_for_event(&event.event_type),
            "tags": self.generate_tags(event)
            // Temporariamente removido: custom_fields requerem UUIDs válidos configurados no ClickUp
            // "custom_fields": self.generate_custom_fields(event)
        })
    }

    /// Gera o título da tarefa baseado no evento do ChatGuru.
    /// 
    /// # Prioridade:
    /// 1. Usa o campo "task_title" ou "annotation" se presente (vindo das anotações do ChatGuru)
    /// 2. Usa o campo "title" se presente
    /// 3. Gera título baseado no tipo de evento e dados específicos
    /// 
    /// # Parâmetros
    /// - `event`: Referência ao evento ChatGuru com todos os dados
    /// 
    /// # Retorno
    /// String com o título formatado para a tarefa do ClickUp
    fn generate_task_title(&self, event: &ChatGuruEvent) -> String {
        // Prioridade 1: Buscar título vindo das anotações do ChatGuru
        if let Some(task_title) = event.data.get("task_title")
            .or_else(|| event.data.get("annotation"))
            .or_else(|| event.data.get("anotacao"))
            .and_then(|v| v.as_str()) {
            return task_title.to_string();
        }

        // Prioridade 2: Buscar campo genérico de título
        if let Some(title) = event.data.get("title")
            .or_else(|| event.data.get("titulo"))
            .and_then(|v| v.as_str()) {
            return title.to_string();
        }

        // Prioridade 3: Gerar título baseado no tipo de evento
        match event.event_type.as_str() {
            // Eventos de Lead
            "new_lead" => {
                let lead_name = event.data.get("lead_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Novo Lead");
                let project = event.data.get("project_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                if !project.is_empty() {
                    format!("🎯 {} - {}", lead_name, project)
                } else {
                    format!("🎯 Novo Lead: {}", lead_name)
                }
            },
            
            // Eventos de Pagamento
            "payment_created" => {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_f64()) {
                    format!("💰 Novo Pagamento - R$ {:.2}", amount)
                } else {
                    "💰 Novo Pagamento Criado".to_string()
                }
            },
            "payment_completed" => {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_f64()) {
                    format!("✅ Pagamento Concluído - R$ {:.2}", amount)
                } else {
                    "✅ Pagamento Concluído".to_string()
                }
            },
            "payment_failed" => {
                format!("❌ Falha no Pagamento - {}", 
                    event.data.get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Motivo não especificado"))
            },
            
            // Eventos de Cliente
            "customer_created" => {
                format!("👤 Novo Cliente - {}", 
                    event.data.get("name")
                        .or_else(|| event.data.get("customer_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Nome não informado"))
            },
            
            // Eventos Fiscais
            "invoice_generated" => {
                if let Some(number) = event.data.get("invoice_number").and_then(|v| v.as_str()) {
                    format!("📄 Nota Fiscal Gerada - {}", number)
                } else {
                    "📄 Nova Nota Fiscal Gerada".to_string()
                }
            },
            
            // Eventos de PIX
            "pix_received" => {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_f64()) {
                    format!("⚡ PIX Recebido - R$ {:.2}", amount)
                } else {
                    "⚡ PIX Recebido".to_string()
                }
            },
            
            // Eventos de Agendamento
            "appointment_scheduled" => {
                let lead_name = event.data.get("lead_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Cliente");
                let appointment_type = event.data.get("appointment_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Agendamento");
                
                format!("📅 {} - {}", appointment_type, lead_name)
            },
            
            // Eventos de Status
            "status_change" => {
                let lead_name = event.data.get("lead_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let new_status = event.data.get("new_status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("status alterado");
                
                if !lead_name.is_empty() {
                    format!("🔄 {} - {}", lead_name, new_status)
                } else {
                    format!("🔄 Status alterado para: {}", new_status)
                }
            },
            
            // Eventos de Mensagem
            "mensagem_recebida" | "message_received" => {
                let sender = event.data.get("sender_name")
                    .or_else(|| event.data.get("chat_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Cliente");
                format!("💬 Mensagem de {}", sender)
            },
            
            // Evento padrão
            _ => {
                // Tenta usar qualquer campo de nome disponível
                if let Some(name) = event.data.get("name")
                    .or_else(|| event.data.get("lead_name"))
                    .or_else(|| event.data.get("customer_name"))
                    .and_then(|v| v.as_str()) {
                    format!("🔔 {} - {}", event.event_type, name)
                } else {
                    format!("🔔 Evento ChatGuru - {}", event.event_type)
                }
            }
        }
    }

    fn generate_task_description(&self, event: &ChatGuruEvent) -> String {
        let mut description = format!("**Evento:** {}\n\n", event.event_type);
        description.push_str(&format!("**Timestamp:** {}\n\n", event.timestamp));
        
        if let Some(ref source) = event.source {
            description.push_str(&format!("**Origem:** {}\n\n", source));
        }

        description.push_str("**Dados do Evento:**\n");
        description.push_str(&format!("```json\n{}\n```\n\n", 
            serde_json::to_string_pretty(&event.data).unwrap_or_default()));

        if let Some(ref metadata) = event.metadata {
            description.push_str("**Metadados:**\n");
            description.push_str(&format!("```json\n{}\n```", 
                serde_json::to_string_pretty(metadata).unwrap_or_default()));
        }

        description
    }

    fn get_priority_for_event(&self, event_type: &str) -> u8 {
        match event_type {
            "payment_failed" => 1, // Urgent
            "pix_received" | "payment_completed" => 2, // High
            "payment_created" | "invoice_generated" => 3, // Normal
            _ => 4 // Low
        }
    }

    fn generate_tags(&self, event: &ChatGuruEvent) -> Vec<String> {
        let mut tags = vec!["chatguru".to_string(), event.event_type.clone()];
        
        // Adicionar tags baseadas no tipo de evento
        match event.event_type.as_str() {
            "payment_created" | "payment_completed" | "payment_failed" => {
                tags.push("pagamento".to_string());
            },
            "pix_received" => {
                tags.push("pix".to_string());
                tags.push("pagamento".to_string());
            },
            "customer_created" => {
                tags.push("cliente".to_string());
            },
            "invoice_generated" => {
                tags.push("fiscal".to_string());
                tags.push("nota".to_string());
            },
            _ => {
                tags.push("outros".to_string());
            }
        }

        // Adicionar tags baseadas no valor do pagamento
        if let Some(amount) = event.data.get("amount").and_then(|v| v.as_f64()) {
            if amount >= 1000.0 {
                tags.push("alto-valor".to_string());
            } else if amount >= 100.0 {
                tags.push("medio-valor".to_string());
            } else {
                tags.push("baixo-valor".to_string());
            }
        }

        // Adicionar tag de urgência para falhas
        if event.event_type == "payment_failed" {
            tags.push("urgente".to_string());
        }

        tags
    }

    fn _generate_custom_fields(&self, event: &ChatGuruEvent) -> Vec<Value> {
        let mut custom_fields = Vec::new();

        // Campo customizado para ID do evento (gera um se não existir)
        let event_id = event.id.clone().unwrap_or_else(|| {
            format!("generated_{}", chrono::Utc::now().timestamp_millis())
        });
        custom_fields.push(json!({
            "id": "event_id",
            "value": event_id
        }));

        // Campo customizado para timestamp
        custom_fields.push(json!({
            "id": "event_timestamp", 
            "value": event.timestamp
        }));

        // Campo customizado para origem
        if let Some(ref source) = event.source {
            custom_fields.push(json!({
                "id": "event_source",
                "value": source
            }));
        }

        // Campos específicos baseados no tipo de evento
        match event.event_type.as_str() {
            "payment_created" | "payment_completed" | "payment_failed" | "pix_received" => {
                // Valor do pagamento
                if let Some(amount) = event.data.get("amount") {
                    custom_fields.push(json!({
                        "id": "payment_amount",
                        "value": amount
                    }));
                }
                
                // Método de pagamento
                if let Some(method) = event.data.get("payment_method") {
                    custom_fields.push(json!({
                        "id": "payment_method",
                        "value": method
                    }));
                }
                
                // ID da transação
                if let Some(transaction_id) = event.data.get("transaction_id") {
                    custom_fields.push(json!({
                        "id": "transaction_id",
                        "value": transaction_id
                    }));
                }
            },
            "customer_created" => {
                // Email do cliente
                if let Some(email) = event.data.get("email") {
                    custom_fields.push(json!({
                        "id": "customer_email",
                        "value": email
                    }));
                }
                
                // Telefone do cliente
                if let Some(phone) = event.data.get("phone") {
                    custom_fields.push(json!({
                        "id": "customer_phone",
                        "value": phone
                    }));
                }
            },
            "invoice_generated" => {
                // Número da nota fiscal
                if let Some(invoice_number) = event.data.get("invoice_number") {
                    custom_fields.push(json!({
                        "id": "invoice_number",
                        "value": invoice_number
                    }));
                }
                
                // URL da nota fiscal
                if let Some(invoice_url) = event.data.get("invoice_url") {
                    custom_fields.push(json!({
                        "id": "invoice_url",
                        "value": invoice_url
                    }));
                }
            },
            _ => {}
        }

        custom_fields
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

    /// Função principal que processa a tarefa conforme regra de negócio:
    /// 1. Consulta lista correta (baseado em anotação do ChatGuru)
    /// 2. Verifica se já existe tarefa com mesmo título
    /// 3. Se existir, adiciona comentário com histórico e atualiza tarefa
    /// 4. Se não existir, cria tarefa nova
    pub async fn process_clickup_task(&self, event: &ChatGuruEvent) -> AppResult<Value> {
        // Log do evento recebido
        let event_id = event.id.clone().unwrap_or_else(|| {
            format!("generated_{}", chrono::Utc::now().timestamp_millis())
        });
        log_info(&format!("Processing ChatGuru event - Type: {}, ID: {}",
            event.event_type, event_id));
        
        // 1. Gera dados da tarefa (título, descrição, custom_fields, etc)
        let task_data = self.build_task_data(event);

        let title = task_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        
        log_info(&format!("Task title generated: {}", title));

        // 2. Busca tarefa existente na lista com mesmo título
        log_info(&format!("Searching for existing task with title: {}", title));
        
        if let Some(existing_task) = self.find_existing_task_in_list(title).await? {
            let task_id = existing_task.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            
            log_info(&format!("Found existing task with ID: {} - Will update", task_id));

            // 3. Preserva histórico adicionando comentário com dados anteriores
            let prev_title = existing_task.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let prev_description = existing_task.get("description").and_then(|v| v.as_str()).unwrap_or("");
            
            // Obter data de atualização anterior
            let prev_updated = if let Some(date_str) = existing_task.get("date_updated").and_then(|v| v.as_str()) {
                date_str.to_string()
            } else if let Some(date_num) = existing_task.get("date_updated").and_then(|v| v.as_u64()) {
                date_num.to_string()
            } else {
                "Unknown".to_string()
            };

            let event_id = event.id.clone().unwrap_or_else(|| {
                format!("generated_{}", chrono::Utc::now().timestamp_millis())
            });
            let history_comment = format!(
                "📝 **Atualização Automática via ChatGuru**\n\n\
                **Evento:** {}\n\
                **ID do Evento:** {}\n\
                **Timestamp:** {}\n\n\
                ---\n\n\
                **Histórico da Versão Anterior:**\n\
                - **Título:** {}\n\
                - **Última Atualização:** {}\n\n\
                **Descrição Anterior:**\n```\n{}\n```\n\n\
                **Novos Dados do Evento:**\n```json\n{}\n```",
                event.event_type,
                event_id,
                event.timestamp,
                prev_title,
                prev_updated,  // Agora é uma String própria, não uma referência
                prev_description,
                serde_json::to_string_pretty(&event.data).unwrap_or_default()
            );

            log_info(&format!("Adding history comment to task {}", task_id));
            self.add_comment_to_task(task_id, &history_comment).await?;

            // Atualiza a tarefa com os novos dados
            log_info(&format!("Updating task {} with new data", task_id));
            let updated_task = self.update_task(task_id, &task_data).await?;
            
            log_clickup_task_updated(task_id, title);

            Ok(updated_task)
        } else {
            log_info(&format!("No existing task found - Creating new task"));
            
            // 4. Cria a tarefa nova
            let url = format!("https://api.clickup.com/api/v2/list/{}/task", self.list_id);

            let resp = self.client.post(&url)
                .header("Authorization", &self.token)
                .header("Content-Type", "application/json")
                .json(&task_data)
                .send()
                .await?;

            let status = resp.status();
            if !status.is_success() {
                let err_text = resp.text().await.unwrap_or_default();
                log_clickup_api_error(&url, Some(status.as_u16()), &err_text);
                return Err(AppError::ClickUpApi(format!("Failed to create task: {}", err_text)));
            }

            let created_task: Value = resp.json().await?;
            
            if let (Some(id), Some(name)) = (
                created_task.get("id").and_then(|v| v.as_str()),
                created_task.get("name").and_then(|v| v.as_str())
            ) {
                log_clickup_task_created(id, name);
            }
            
            Ok(created_task)
        }
    }
}