use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub clickup: ClickUpSettings,
    pub gcp: GcpSettings,
    pub chatguru: ChatGuruSettings,
    pub ai: Option<AISettings>,
    pub cloud_tasks: CloudTasksConfig,
    pub rate_limiting: Option<RateLimitingConfig>,
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    pub batch_processing: Option<BatchProcessingConfig>,
    pub monitoring: Option<MonitoringConfig>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClickUpSettings {
    pub token: String,
    pub list_id: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GcpSettings {
    pub project_id: String,
    pub topic_name: String,
    pub subscription_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruSettings {
    pub webhook_secret: Option<String>,
    pub validate_signature: bool,
    #[serde(default)]
    pub legacy_response_mode: Option<bool>,
    pub api_token: Option<String>,  // Token para enviar anotações de volta
    pub api_endpoint: Option<String>,  // Endpoint da API do ChatGuru
    pub account_id: Option<String>,  // ID da conta no ChatGuru
    pub phone_ids: Option<Vec<String>>,  // IDs dos telefones configurados
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AISettings {
    pub enabled: bool,
    // Usa sempre Vertex AI no Google Cloud (mais eficiente e integrado)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudTasksConfig {
    pub project_id: String,
    pub location: String,
    pub queue_name: String,
    pub service_account_email: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let mut builder = Config::builder()
            // Arquivo de configuração base
            .add_source(File::with_name("config/default").required(false))
            // Arquivo específico do ambiente
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false));
        
        // Adicionar variáveis de ambiente específicas
        if let Ok(token) = std::env::var("CLICKUP_API_TOKEN") {
            builder = builder.set_override("clickup.token", token)?;
        }
        if let Ok(list_id) = std::env::var("CLICKUP_LIST_ID") {
            builder = builder.set_override("clickup.list_id", list_id)?;
        }
        
        // Cloud Tasks environment variables
        if let Ok(project_id) = std::env::var("CLOUD_TASKS_PROJECT") {
            builder = builder.set_override("cloud_tasks.project_id", project_id)?;
        }
        if let Ok(location) = std::env::var("CLOUD_TASKS_LOCATION") {
            builder = builder.set_override("cloud_tasks.location", location)?;
        }
        if let Ok(queue) = std::env::var("CLOUD_TASKS_QUEUE") {
            builder = builder.set_override("cloud_tasks.queue_name", queue)?;
        }
        if let Ok(service_account) = std::env::var("CLOUD_TASKS_SERVICE_ACCOUNT") {
            builder = builder.set_override("cloud_tasks.service_account_email", service_account)?;
        }
        
        // Também suportar o prefixo antigo
        builder = builder.add_source(Environment::with_prefix("CHATGURU_CLICKUP"));
        
        let s = builder.build()?;

        s.try_deserialize()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub max_dispatches_per_second: f64,
    pub max_concurrent_dispatches: u32,
    pub batch_size: usize,
    pub burst_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout_duration: u64,
    pub half_open_max_calls: u32,
    pub success_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingConfig {
    pub enabled: bool,
    pub batch_size: usize,
    pub batch_timeout: u64,
    pub max_batch_wait: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics_port: u16,
    pub custom_metrics: Vec<String>,
}