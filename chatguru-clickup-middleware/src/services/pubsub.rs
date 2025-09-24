use crate::{config::settings::GcpSettings, models::webhook_payload::WebhookPayload};
use tracing::{error, info, warn};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use google_cloud_pubsub::{client::{Client, ClientConfig}, publisher::Publisher};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone)]
pub struct PubSubEventService {
    publisher: Publisher,
    topic_name: String,
    project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    TaskCreated,
    TaskUpdated,
    TaskDuplicate,
    ErrorCritical,
    AnnotationProcessed,
    WebhookReceived,
    CloudTaskEnqueued,
    ProcessingCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubEvent {
    pub event_type: EventType,
    pub event_id: String,
    pub timestamp: String,
    pub source: String,
    pub data: Value,
    pub metadata: Option<Value>,
}

impl PubSubEventService {
    pub async fn new(gcp_settings: &GcpSettings) -> Result<Self> {
        info!("Initializing PubSub Event Service for events and notifications");
        
        // Use default authentication (Service Account in Cloud Run)
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| anyhow!("Failed to create PubSub client config: {}", e))?;
            
        let client = Client::new(config)
            .await
            .map_err(|e| anyhow!("Failed to create PubSub client: {}", e))?;

        let topic = client.topic(&gcp_settings.topic_name);
        
        // Create topic if it doesn't exist
        if !topic.exists(None).await.unwrap_or(false) {
            info!("Creating PubSub topic: {}", gcp_settings.topic_name);
            topic.create(None, None).await
                .map_err(|e| anyhow!("Failed to create PubSub topic: {}", e))?;
        }

        let publisher = topic.new_publisher(None);

        info!("PubSub Event Service initialized successfully");
        Ok(Self {
            publisher,
            topic_name: gcp_settings.topic_name.clone(),
            project_id: gcp_settings.project_id.clone(),
        })
    }

    /// Publica evento de task criada no ClickUp
    pub async fn publish_task_created(&self, webhook_payload: &WebhookPayload, task_data: &Value) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::TaskCreated,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "webhook_payload": webhook_payload,
                "clickup_task": task_data,
                "task_title": webhook_payload.get_task_title(),
                "payload_type": match webhook_payload {
                    crate::models::webhook_payload::WebhookPayload::ChatGuru(_) => "ChatGuru",
                    crate::models::webhook_payload::WebhookPayload::EventType(_) => "EventType",
                    crate::models::webhook_payload::WebhookPayload::Generic(_) => "Generic"
                }
            }),
            metadata: Some(json!({
                "processing_type": "task_creation",
                "middleware_version": env!("CARGO_PKG_VERSION")
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de task atualizada (duplicada encontrada)
    pub async fn publish_task_updated(&self, webhook_payload: &WebhookPayload, existing_task: &Value, update_details: &Value) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::TaskUpdated,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "webhook_payload": webhook_payload,
                "existing_task": existing_task,
                "update_details": update_details,
                "task_title": webhook_payload.get_task_title()
            }),
            metadata: Some(json!({
                "processing_type": "task_update",
                "duplicate_prevention": true
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de erro crítico
    pub async fn publish_critical_error(&self, error_context: &str, error_message: &str, payload: Option<&WebhookPayload>) -> Result<String> {
        let mut data = json!({
            "error_context": error_context,
            "error_message": error_message,
            "severity": "critical",
            "requires_attention": true
        });

        if let Some(webhook_payload) = payload {
            data["webhook_payload"] = serde_json::to_value(webhook_payload)?;
        }

        let event = PubSubEvent {
            event_type: EventType::ErrorCritical,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data,
            metadata: Some(json!({
                "alert_level": "high",
                "notification_required": true
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de anotação processada
    pub async fn publish_annotation_processed(&self, task_id: &str, annotation_data: &Value) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::AnnotationProcessed,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "task_id": task_id,
                "annotation": annotation_data,
                "processing_completed": true
            }),
            metadata: Some(json!({
                "processing_type": "annotation",
                "chatguru_integration": true
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de webhook recebido (para auditoria)
    pub async fn publish_webhook_received(&self, webhook_payload: &WebhookPayload) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::WebhookReceived,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "webhook_payload": webhook_payload,
                "payload_size_bytes": serde_json::to_string(webhook_payload)?.len(),
                "task_title": webhook_payload.get_task_title()
            }),
            metadata: Some(json!({
                "processing_stage": "webhook_received",
                "audit_event": true
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de Cloud Task enfileirada
    pub async fn publish_cloud_task_enqueued(&self, task_name: &str, webhook_payload: &WebhookPayload) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::CloudTaskEnqueued,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "cloud_task_name": task_name,
                "webhook_payload": webhook_payload,
                "async_processing": true
            }),
            metadata: Some(json!({
                "processing_stage": "cloud_task_enqueued",
                "architecture": "cloud_tasks"
            })),
        };

        self.publish_event(&event).await
    }

    /// Publica evento de processamento completado
    pub async fn publish_processing_completed(&self, webhook_payload: &WebhookPayload, result: &Value) -> Result<String> {
        let event = PubSubEvent {
            event_type: EventType::ProcessingCompleted,
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "webhook_payload": webhook_payload,
                "processing_result": result,
                "success": true
            }),
            metadata: Some(json!({
                "processing_stage": "completed",
                "final_event": true
            })),
        };

        self.publish_event(&event).await
    }

    /// Método genérico para publicar eventos
    async fn publish_event(&self, event: &PubSubEvent) -> Result<String> {
        let event_json = serde_json::to_string(event)
            .map_err(|e| anyhow!("Failed to serialize event: {}", e))?;

        // Criar atributos para facilitar filtragem
        let mut attributes = HashMap::new();
        attributes.insert("event_type".to_string(), format!("{:?}", event.event_type));
        attributes.insert("source".to_string(), event.source.clone());
        attributes.insert("event_id".to_string(), event.event_id.clone());
        attributes.insert("timestamp".to_string(), event.timestamp.clone());
        
        // Adicionar atributos específicos baseados no tipo de evento
        match &event.event_type {
            EventType::TaskCreated | EventType::TaskUpdated => {
                if let Some(campanha) = event.data.get("campanha").and_then(|c| c.as_str()) {
                    attributes.insert("campanha".to_string(), campanha.to_string());
                }
            },
            EventType::ErrorCritical => {
                attributes.insert("severity".to_string(), "critical".to_string());
                attributes.insert("alert_required".to_string(), "true".to_string());
            },
            _ => {}
        }

        // Criar e publicar mensagem PubSub
        let msg = PubsubMessage {
            data: event_json.into_bytes(),
            attributes,
            ..Default::default()
        };
        
        let awaiter = self.publisher
            .publish(msg)
            .await;

        match awaiter.get().await {
            Ok(message_id) => {
                let id = message_id.to_string();
                info!(
                    event_type = ?event.event_type,
                    event_id = %event.event_id,
                    message_id = %id,
                    topic = %self.topic_name,
                    "Published PubSub event successfully"
                );
                Ok(id)
            },
            Err(e) => {
                let error_msg = format!("Failed to publish PubSub event: {}", e);
                error!(
                    event_type = ?event.event_type,
                    event_id = %event.event_id,
                    topic = %self.topic_name,
                    error = %error_msg,
                    "Failed to publish PubSub event"
                );
                Err(anyhow!(error_msg))
            }
        }
    }

    /// Testa a conexão publicando uma mensagem de teste
    pub async fn test_connection(&self) -> Result<String> {
        let test_event = PubSubEvent {
            event_type: EventType::WebhookReceived, // Usar um tipo existente
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source: "chatguru-clickup-middleware".to_string(),
            data: json!({
                "test": true,
                "message": "PubSub connection test",
                "middleware_version": env!("CARGO_PKG_VERSION")
            }),
            metadata: Some(json!({
                "test_event": true,
                "connection_test": true
            })),
        };

        match self.publish_event(&test_event).await {
            Ok(message_id) => {
                info!("PubSub connection test successful, message_id: {}", message_id);
                Ok(message_id)
            },
            Err(e) => {
                error!("PubSub connection test failed: {}", e);
                Err(e)
            }
        }
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) {
        info!("Shutting down PubSub Event Service");
        self.publisher.shutdown().await;
    }
}