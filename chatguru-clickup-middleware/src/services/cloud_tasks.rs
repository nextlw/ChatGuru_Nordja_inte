use crate::{config::settings::CloudTasksConfig, models::webhook_payload::WebhookPayload};
use tracing::{error, info};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use reqwest;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose};

#[derive(Clone)]
pub struct CloudTasksService {
    client: reqwest::Client,
    config: CloudTasksConfig,
    access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload<T> {
    pub data: T,
    pub created_at: String,
    pub retry_count: u32,
}

#[derive(Serialize)]
struct CloudTasksTask {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    http_request: HttpTaskRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule_time: Option<String>,
}

#[derive(Serialize)]
struct HttpTaskRequest {
    http_method: String,
    url: String,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Serialize)]
struct CreateTaskRequest {
    task: CloudTasksTask,
}

impl CloudTasksService {
    pub async fn new(config: CloudTasksConfig) -> Result<Self> {
        let client = reqwest::Client::new();
        
        info!("Cloud Tasks service initialized with REST API");
        Ok(Self { 
            client, 
            config,
            access_token: None,
        })
    }

    async fn get_access_token(&mut self) -> Result<&str> {
        if self.access_token.is_none() {
            // Get access token using Application Default Credentials
            let output = std::process::Command::new("gcloud")
                .args(&["auth", "print-access-token"])
                .output()
                .map_err(|e| anyhow!("Failed to execute gcloud command: {}", e))?;

            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to get access token: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let token = String::from_utf8(output.stdout)
                .map_err(|e| anyhow!("Invalid UTF-8 in token: {}", e))?
                .trim()
                .to_string();

            self.access_token = Some(token);
        }

        Ok(self.access_token.as_ref().unwrap())
    }

    pub async fn enqueue_webhook_task(&mut self, payload: &WebhookPayload) -> Result<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let task_name = format!("webhook-task-{}", task_id);
        
        info!("Enqueuing task {} for webhook processing", task_name);

        // Create the task payload wrapper
        let task_payload = TaskPayload {
            data: payload.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
        };

        // Serialize payload to JSON
        let payload_json = serde_json::to_string(&task_payload)
            .map_err(|e| anyhow!("Failed to serialize webhook payload: {}", e))?;

        // Build the worker endpoint URL
        let worker_url = format!("https://{}/worker/process-task", 
            std::env::var("CLOUD_RUN_SERVICE_URL")
                .unwrap_or_else(|_| "localhost:8080".to_string())
        );

        // Create headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Cloud-Tasks-Queue".to_string(), self.config.queue_name.clone());
        headers.insert("X-Cloud-Tasks-Task-Name".to_string(), task_name.clone());

        // Create the task
        let task = CloudTasksTask {
            name: None, // Let Cloud Tasks generate the name
            http_request: HttpTaskRequest {
                http_method: "POST".to_string(),
                url: worker_url,
                headers,
                body: general_purpose::STANDARD.encode(payload_json.as_bytes()),
            },
            schedule_time: None, // Execute immediately
        };

        let create_request = CreateTaskRequest { task };

        // Build Cloud Tasks API URL
        let api_url = format!(
            "https://cloudtasks.googleapis.com/v2/projects/{}/locations/{}/queues/{}/tasks",
            self.config.project_id,
            self.config.location,
            self.config.queue_name
        );

        // Get access token and clone client to avoid borrowing issues
        let token = self.get_access_token().await?.to_string();
        let client = self.client.clone();

        // Make HTTP request to Cloud Tasks API
        let response = client
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&create_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Cloud Tasks: {}", e))?;

        if response.status().is_success() {
            let response_data: serde_json::Value = response.json().await
                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
            
            let task_name = response_data.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(&task_name)
                .to_string();
            
            info!("Successfully enqueued task: {}", task_name);
            Ok(task_name)
        } else {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Failed to enqueue task {}: {}", task_name, error_text);
            Err(anyhow!("Failed to enqueue Cloud Task: {}", error_text))
        }
    }
}