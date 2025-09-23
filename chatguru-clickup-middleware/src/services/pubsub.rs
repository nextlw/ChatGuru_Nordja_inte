#![allow(dead_code)]

use crate::config::Settings;
use crate::models::ChatGuruEvent;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_pubsub::publisher::Publisher;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use serde_json::{json, Value};
use std::collections::HashMap;



#[derive(Clone)]
pub struct PubSubService {
    publisher: Publisher,
    topic_name: String,
}

impl PubSubService {
    pub async fn new(settings: &Settings) -> AppResult<Self> {
        // Use the recommended authentication method from the documentation
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| AppError::PubSubError(format!("Failed to create client config: {}", e)))?;
            
        let client = Client::new(config)
            .await
            .map_err(|e| AppError::PubSubError(format!("Failed to create Pub/Sub client: {}", e)))?;

        let topic = client.topic(&settings.gcp.topic_name);
        
        // Create topic if it doesn't exist
        if !topic.exists(None).await.unwrap_or(false) {
            topic.create(None, None).await
                .map_err(|e| AppError::PubSubError(format!("Failed to create topic: {}", e)))?;
        }

        let publisher = topic.new_publisher(None);

        Ok(Self {
            publisher,
            topic_name: settings.gcp.topic_name.clone(),
        })
    }

    pub async fn publish_event(&self, event: &ChatGuruEvent, clickup_task: Option<&Value>) -> AppResult<String> {
        let message_data = self.build_message_data(event, clickup_task)?;
        let message_json = serde_json::to_string(&message_data)?;
        
        let mut attributes = HashMap::new();
        attributes.insert("campanha_id".to_string(), event.campanha_id.clone());
        attributes.insert("campanha_nome".to_string(), event.campanha_nome.clone());
        attributes.insert("origem".to_string(), event.origem.clone());
        attributes.insert("nome_contato".to_string(), event.nome.clone());
        attributes.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());

        if clickup_task.is_some() {
            attributes.insert("has_clickup_task".to_string(), "true".to_string());
        }

        // Criar e publicar mensagem PubSub
        let msg = PubsubMessage {
            data: message_json.into_bytes(),
            attributes,
            ..Default::default()
        };
        
        let awaiter = self.publisher
            .publish(msg)
            .await;

        match awaiter.get().await {
            Ok(message_id) => {
                let id = message_id.to_string();
                log_pubsub_published(&self.topic_name, &id);
                Ok(id)
            },
            Err(e) => {
                let error_msg = format!("Failed to publish message: {}", e);
                log_pubsub_error(&self.topic_name, &error_msg);
                Err(AppError::PubSubError(error_msg))
            }
        }
    }

    pub async fn publish_status_update(&self, status: &str, details: &Value) -> AppResult<String> {
        let message_data = json!({
            "type": "status_update",
            "status": status,
            "details": details,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        let message_json = serde_json::to_string(&message_data)?;
        
        // Criar e publicar mensagem PubSub
        let msg = PubsubMessage {
            data: message_json.into_bytes(),
            ..Default::default()
        };
        
        let awaiter = self.publisher
            .publish(msg)
            .await;

        match awaiter.get().await {
            Ok(message_id) => {
                let id = message_id.to_string();
                log_pubsub_published(&self.topic_name, &id);
                Ok(id)
            },
            Err(e) => {
                let error_msg = format!("Failed to publish status update: {}", e);
                log_pubsub_error(&self.topic_name, &error_msg);
                Err(AppError::PubSubError(error_msg))
            }
        }
    }

    pub async fn publish_error(&self, error_context: &str, error_message: &str, event: Option<&ChatGuruEvent>) -> AppResult<String> {
        let mut message_data = json!({
            "type": "error",
            "context": error_context,
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Some(event) = event {
            message_data["original_event"] = serde_json::to_value(event)?;
        }

        let message_json = serde_json::to_string(&message_data)?;
        // Criar e publicar mensagem PubSub
        let msg = PubsubMessage {
            data: message_json.into_bytes(),
            ..Default::default()
        };
        
        let awaiter = self.publisher
            .publish(msg)
            .await;

        match awaiter.get().await {
            Ok(message_id) => {
                let id = message_id.to_string();
                log_pubsub_published(&self.topic_name, &id);
                Ok(id)
            },
            Err(e) => {
                let error_msg = format!("Failed to publish error message: {}", e);
                log_pubsub_error(&self.topic_name, &error_msg);
                Err(AppError::PubSubError(error_msg))
            }
        }
    }

    fn build_message_data(&self, event: &ChatGuruEvent, clickup_task: Option<&Value>) -> AppResult<Value> {
        let mut message = json!({
            "type": "chatguru_event_processed",
            "chatguru_event": event,
            "processing_timestamp": chrono::Utc::now().to_rfc3339(),
            "middleware_version": env!("CARGO_PKG_VERSION")
        });

        if let Some(task) = clickup_task {
            message["clickup_task"] = task.clone();
            message["task_created"] = json!(true);
        } else {
            message["task_created"] = json!(false);
        }

        // Adicionar informações de processamento
        message["processing_info"] = json!({
            "event_size_bytes": serde_json::to_string(event)?.len(),
            "has_custom_fields": !event.campos_personalizados.is_empty(),
            "custom_fields_count": event.campos_personalizados.len(),
            "tags_count": event.tags.len()
        });

        Ok(message)
    }

    pub async fn test_connection(&self) -> AppResult<()> {
        // Testa a conexão publicando uma mensagem de teste
        let test_message = json!({
            "type": "connection_test",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "message": "Connection test from ChatGuru-ClickUp middleware"
        });

        let message_json = serde_json::to_string(&test_message)?;
        
        // Criar e publicar mensagem PubSub
        let msg = PubsubMessage {
            data: message_json.into_bytes(),
            ..Default::default()
        };
        
        let awaiter = self.publisher
            .publish(msg)
            .await;

        match awaiter.get().await {
            Ok(message_id) => {
                let id = message_id.to_string();
                log_pubsub_published(&self.topic_name, &id);
                Ok(())
            },
            Err(e) => {
                let error_msg = format!("Connection test failed: {}", e);
                log_pubsub_error(&self.topic_name, &error_msg);
                Err(AppError::PubSubError(error_msg))
            }
        }
    }

    pub async fn shutdown(&mut self) {
        self.publisher.shutdown().await;
    }
}