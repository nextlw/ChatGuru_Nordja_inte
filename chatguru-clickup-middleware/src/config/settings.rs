use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub clickup: ClickUpSettings,
    pub gcp: GcpSettings,
    pub chatguru: ChatGuruSettings,
    pub ai: Option<AISettings>,
    pub vertex: Option<VertexSettings>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
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
    pub pubsub_topic: Option<String>,  // Tópico para envio de webhooks RAW
    pub media_processing_topic: Option<String>,  // Tópico para requisições de processamento de mídia
    pub media_results_topic: Option<String>,  // Tópico para resultados de processamento
    pub media_results_subscription: Option<String>,  // Subscription para ler resultados
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
pub struct VertexSettings {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub project_id: String,
    pub location: String,
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
        
        // Também suportar o prefixo antigo
        builder = builder.add_source(Environment::with_prefix("CHATGURU_CLICKUP"));
        
        let s = builder.build()?;

        s.try_deserialize()
    }
}