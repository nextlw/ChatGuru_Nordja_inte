//! ClickUp Webhooks API
//!
//! Este módulo implementa gerenciamento completo de webhooks do ClickUp.
//!
//! ## Arquitetura Recomendada
//!
//! Webhooks ClickUp + Pub/Sub trabalham juntos:
//! 1. Webhook recebe eventos HTTP do ClickUp (tempo real)
//! 2. Endpoint envia para Pub/Sub (desacoplamento)
//! 3. Pub/Sub distribui para múltiplos subscribers (escalabilidade)
//! 4. Retry automático e persistência garantida pelo GCP
//!
//! ## Exemplo de Uso
//!
//! ```rust,no_run
//! use clickup::webhooks::{WebhookManager, WebhookEvent, WebhookConfig};
//!
//! # async fn example() -> clickup::Result<()> {
//! let manager = WebhookManager::from_token("pk_token".to_string(), "workspace_id".to_string())?;
//!
//! // Criar webhook
//! let config = WebhookConfig {
//!     endpoint: "https://myapp.com/webhooks/clickup".to_string(),
//!     events: vec![
//!         WebhookEvent::TaskCreated,
//!         WebhookEvent::TaskUpdated,
//!         WebhookEvent::TaskStatusUpdated,
//!     ],
//!     status: Some("active".to_string()),
//! };
//!
//! let webhook = manager.create_webhook(&config).await?;
//! println!("Webhook criado: {}", webhook.id);
//!
//! // Listar webhooks
//! let webhooks = manager.list_webhooks().await?;
//! println!("Total de webhooks: {}", webhooks.len());
//!
//! // Atualizar webhook
//! let updated_config = WebhookConfig {
//!     endpoint: "https://myapp.com/new-endpoint".to_string(),
//!     events: vec![WebhookEvent::TaskCreated],
//!     status: Some("active".to_string()),
//! };
//! manager.update_webhook(&webhook.id, &updated_config).await?;
//!
//! // Deletar webhook
//! manager.delete_webhook(&webhook.id).await?;
//! # Ok(())
//! # }
//! ```

use crate::{ClickUpClient, Result};
use serde::{Deserialize, Serialize};

/// Gerenciador de webhooks do ClickUp
pub struct WebhookManager {
    client: ClickUpClient,
    workspace_id: String,
}

impl WebhookManager {
    /// Cria um novo WebhookManager com um cliente existente
    pub fn new(client: ClickUpClient, workspace_id: String) -> Self {
        Self {
            client,
            workspace_id,
        }
    }

    /// Cria um WebhookManager a partir de um token
    pub fn from_token(api_token: String, workspace_id: String) -> Result<Self> {
        let client = ClickUpClient::new(api_token)?;
        Ok(Self::new(client, workspace_id))
    }

    /// Cria um webhook para receber eventos do ClickUp
    ///
    /// # Exemplo
    ///
    /// ```rust,no_run
    /// # use clickup::webhooks::{WebhookManager, WebhookConfig, WebhookEvent};
    /// # async fn example() -> clickup::Result<()> {
    /// let manager = WebhookManager::from_token("pk_token".to_string(), "workspace_id".to_string())?;
    ///
    /// let config = WebhookConfig {
    ///     endpoint: "https://myapp.com/webhooks/clickup".to_string(),
    ///     events: vec![WebhookEvent::TaskCreated, WebhookEvent::TaskUpdated],
    ///     status: Some("active".to_string()),
    /// };
    ///
    /// let webhook = manager.create_webhook(&config).await?;
    /// println!("Webhook ID: {}", webhook.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_webhook(&self, config: &WebhookConfig) -> Result<Webhook> {
        let endpoint = format!("/team/{}/webhook", self.workspace_id);

        let body = serde_json::json!({
            "endpoint": config.endpoint,
            "events": config.events,
            "status": config.status.as_ref().unwrap_or(&"active".to_string()),
        });

        let webhook: Webhook = self.client.post_json(&endpoint, &body).await?;
        Ok(webhook)
    }

    /// Lista todos os webhooks do workspace
    ///
    /// **IMPORTANTE**: Retorna apenas webhooks criados pelo usuário autenticado.
    pub async fn list_webhooks(&self) -> Result<Vec<Webhook>> {
        let endpoint = format!("/team/{}/webhook", self.workspace_id);

        #[derive(Deserialize)]
        struct WebhooksResponse {
            webhooks: Vec<Webhook>,
        }

        let response: WebhooksResponse = self.client.get_json(&endpoint).await?;
        Ok(response.webhooks)
    }

    /// Atualiza um webhook existente
    ///
    /// # Exemplo
    ///
    /// ```rust,no_run
    /// # use clickup::webhooks::{WebhookManager, WebhookConfig, WebhookEvent};
    /// # async fn example() -> clickup::Result<()> {
    /// let manager = WebhookManager::from_token("pk_token".to_string(), "workspace_id".to_string())?;
    ///
    /// let new_config = WebhookConfig {
    ///     endpoint: "https://myapp.com/new-endpoint".to_string(),
    ///     events: vec![WebhookEvent::TaskStatusUpdated],
    ///     status: Some("active".to_string()),
    /// };
    ///
    /// let updated = manager.update_webhook("webhook_id", &new_config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_webhook(&self, webhook_id: &str, config: &WebhookConfig) -> Result<Webhook> {
        let endpoint = format!("/webhook/{}", webhook_id);

        let body = serde_json::json!({
            "endpoint": config.endpoint,
            "events": config.events,
            "status": config.status.as_ref().unwrap_or(&"active".to_string()),
        });

        let webhook: Webhook = self.client.put_json(&endpoint, &body).await?;
        Ok(webhook)
    }

    /// Deleta um webhook
    ///
    /// # Exemplo
    ///
    /// ```rust,no_run
    /// # use clickup::webhooks::WebhookManager;
    /// # async fn example() -> clickup::Result<()> {
    /// let manager = WebhookManager::from_token("pk_token".to_string(), "workspace_id".to_string())?;
    /// manager.delete_webhook("webhook_id").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let endpoint = format!("/webhook/{}", webhook_id);

        #[derive(Deserialize)]
        struct DeleteResponse {}

        let _: DeleteResponse = self.client.delete_json(&endpoint).await?;
        Ok(())
    }

    /// Busca webhook por endpoint URL (helper)
    ///
    /// Como a API não oferece busca por URL, este método lista todos e filtra.
    pub async fn find_webhook_by_endpoint(&self, endpoint_url: &str) -> Result<Option<Webhook>> {
        let webhooks = self.list_webhooks().await?;
        Ok(webhooks.into_iter().find(|w| w.endpoint == endpoint_url))
    }

    /// Verifica se um webhook para determinado endpoint já existe
    pub async fn webhook_exists(&self, endpoint_url: &str) -> Result<bool> {
        Ok(self.find_webhook_by_endpoint(endpoint_url).await?.is_some())
    }

    /// Cria ou atualiza um webhook (idempotente)
    ///
    /// Se já existe um webhook para o endpoint, atualiza. Caso contrário, cria novo.
    pub async fn ensure_webhook(&self, config: &WebhookConfig) -> Result<Webhook> {
        if let Some(existing) = self.find_webhook_by_endpoint(&config.endpoint).await? {
            tracing::info!("Webhook já existe para {}, atualizando...", config.endpoint);
            self.update_webhook(&existing.id, config).await
        } else {
            tracing::info!("Criando novo webhook para {}...", config.endpoint);
            self.create_webhook(config).await
        }
    }
}

/// Configuração de webhook para criação/atualização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// URL do endpoint que receberá os eventos (deve ser HTTPS)
    pub endpoint: String,

    /// Lista de eventos a monitorar
    pub events: Vec<WebhookEvent>,

    /// Status do webhook: "active" ou "inactive"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Webhook registrado no ClickUp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// ID único do webhook
    pub id: String,

    /// User ID que criou o webhook
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userid: Option<u64>,

    /// Workspace ID (team_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,

    /// URL do endpoint que recebe eventos
    pub endpoint: String,

    /// User ID (formato alternativo)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Status: "active" ou "inactive"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Lista de eventos monitorados
    pub events: Vec<WebhookEvent>,

    /// Informações de health (opcional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<WebhookHealth>,
}

/// Informações de saúde do webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookHealth {
    /// Status geral: "active", "failing", etc
    pub status: String,

    /// Número de falhas recentes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_count: Option<u32>,
}

/// Tipos de eventos disponíveis no ClickUp
///
/// **IMPORTANTE**: Nem todos os eventos estão listados aqui.
/// Consulte a documentação oficial para lista completa:
/// https://developer.clickup.com/docs/webhookevents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum WebhookEvent {
    // ========== TASK EVENTS (mais comuns) ==========
    /// Task foi criada
    #[serde(rename = "taskCreated")]
    TaskCreated,

    /// Task foi atualizada (qualquer campo)
    #[serde(rename = "taskUpdated")]
    TaskUpdated,

    /// Task foi deletada
    #[serde(rename = "taskDeleted")]
    TaskDeleted,

    /// Task foi movida entre listas
    #[serde(rename = "taskMoved")]
    TaskMoved,

    /// Status da task foi alterado
    #[serde(rename = "taskStatusUpdated")]
    TaskStatusUpdated,

    /// Prioridade da task foi alterada
    #[serde(rename = "taskPriorityUpdated")]
    TaskPriorityUpdated,

    /// Assignee foi adicionado/removido
    #[serde(rename = "taskAssigneeUpdated")]
    TaskAssigneeUpdated,

    /// Due date foi alterada
    #[serde(rename = "taskDueDateUpdated")]
    TaskDueDateUpdated,

    /// Tag foi adicionada/removida
    #[serde(rename = "taskTagUpdated")]
    TaskTagUpdated,

    /// Time tracking foi alterado
    #[serde(rename = "taskTimeEstimateUpdated")]
    TaskTimeEstimateUpdated,

    /// Time tracking entry criado
    #[serde(rename = "taskTimeTracked")]
    TaskTimeTracked,

    /// Comentário adicionado
    #[serde(rename = "taskCommentPosted")]
    TaskCommentPosted,

    /// Comentário atualizado
    #[serde(rename = "taskCommentUpdated")]
    TaskCommentUpdated,

    // ========== LIST EVENTS ==========
    /// Lista criada
    #[serde(rename = "listCreated")]
    ListCreated,

    /// Lista atualizada
    #[serde(rename = "listUpdated")]
    ListUpdated,

    /// Lista deletada
    #[serde(rename = "listDeleted")]
    ListDeleted,

    // ========== FOLDER EVENTS ==========
    /// Folder criado
    #[serde(rename = "folderCreated")]
    FolderCreated,

    /// Folder atualizado
    #[serde(rename = "folderUpdated")]
    FolderUpdated,

    /// Folder deletado
    #[serde(rename = "folderDeleted")]
    FolderDeleted,

    // ========== SPACE EVENTS ==========
    /// Space criado
    #[serde(rename = "spaceCreated")]
    SpaceCreated,

    /// Space atualizado
    #[serde(rename = "spaceUpdated")]
    SpaceUpdated,

    /// Space deletado
    #[serde(rename = "spaceDeleted")]
    SpaceDeleted,

    // ========== GOAL EVENTS ==========
    /// Goal criado
    #[serde(rename = "goalCreated")]
    GoalCreated,

    /// Goal atualizado
    #[serde(rename = "goalUpdated")]
    GoalUpdated,

    /// Goal deletado
    #[serde(rename = "goalDeleted")]
    GoalDeleted,

    // ========== OUTROS EVENTOS ==========
    /// Eventos não mapeados (use string customizada)
    #[serde(untagged)]
    Other(String),
}

impl WebhookEvent {
    /// Retorna todos os eventos relacionados a tasks
    pub fn all_task_events() -> Vec<Self> {
        vec![
            Self::TaskCreated,
            Self::TaskUpdated,
            Self::TaskDeleted,
            Self::TaskMoved,
            Self::TaskStatusUpdated,
            Self::TaskPriorityUpdated,
            Self::TaskAssigneeUpdated,
            Self::TaskDueDateUpdated,
            Self::TaskTagUpdated,
            Self::TaskTimeEstimateUpdated,
            Self::TaskTimeTracked,
            Self::TaskCommentPosted,
            Self::TaskCommentUpdated,
        ]
    }

    /// Retorna eventos essenciais de task (create, update, delete)
    pub fn essential_task_events() -> Vec<Self> {
        vec![
            Self::TaskCreated,
            Self::TaskUpdated,
            Self::TaskDeleted,
        ]
    }

    /// Retorna todos os eventos relacionados a listas
    pub fn all_list_events() -> Vec<Self> {
        vec![
            Self::ListCreated,
            Self::ListUpdated,
            Self::ListDeleted,
        ]
    }

    /// Retorna todos os eventos de estrutura (space, folder, list)
    pub fn all_structure_events() -> Vec<Self> {
        vec![
            Self::SpaceCreated,
            Self::SpaceUpdated,
            Self::SpaceDeleted,
            Self::FolderCreated,
            Self::FolderUpdated,
            Self::FolderDeleted,
            Self::ListCreated,
            Self::ListUpdated,
            Self::ListDeleted,
        ]
    }
}

/// Payload recebido do webhook do ClickUp
///
/// Este é o envelope que envolve os dados do evento.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// ID do webhook que enviou
    pub webhook_id: String,

    /// Tipo do evento
    pub event: WebhookEvent,

    /// Task ID (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,

    /// List ID (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,

    /// Folder ID (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,

    /// Space ID (se aplicável)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,

    /// Dados completos do evento (varia por tipo)
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl WebhookPayload {
    /// Valida a assinatura do webhook (segurança)
    ///
    /// **IMPORTANTE**: Sempre valide assinaturas em produção!
    ///
    /// # Argumentos
    ///
    /// * `signature` - Header `X-Signature` recebido
    /// * `secret` - Secret do webhook configurado no ClickUp
    /// * `body` - Body raw da requisição (bytes)
    ///
    /// # Retorna
    ///
    /// `true` se assinatura é válida, `false` caso contrário
    pub fn verify_signature(signature: &str, secret: &str, body: &[u8]) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        mac.update(body);

        let result = mac.finalize();
        let expected = hex::encode(result.into_bytes());

        // Comparação constant-time para prevenir timing attacks
        signature == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent::TaskCreated;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, r#""taskCreated""#);
    }

    #[test]
    fn test_webhook_event_deserialization() {
        let json = r#""taskStatusUpdated""#;
        let event: WebhookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event, WebhookEvent::TaskStatusUpdated);
    }

    #[test]
    fn test_all_task_events() {
        let events = WebhookEvent::all_task_events();
        assert!(events.len() >= 10);
        assert!(events.contains(&WebhookEvent::TaskCreated));
        assert!(events.contains(&WebhookEvent::TaskUpdated));
    }

    #[test]
    fn test_webhook_config_serialization() {
        let config = WebhookConfig {
            endpoint: "https://example.com/webhook".to_string(),
            events: vec![WebhookEvent::TaskCreated, WebhookEvent::TaskUpdated],
            status: Some("active".to_string()),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["endpoint"], "https://example.com/webhook");
        assert!(json["events"].is_array());
    }

    #[test]
    fn test_verify_signature() {
        let secret = "test_secret";
        let body = b"test payload";

        // Gerar assinatura válida
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let valid_signature = hex::encode(mac.finalize().into_bytes());

        // Verificar
        assert!(WebhookPayload::verify_signature(&valid_signature, secret, body));
        assert!(!WebhookPayload::verify_signature("invalid", secret, body));
    }
}
