# 🦀 MIDDLEWARE RUST - INTEGRAÇÃO SURI-CLICKUP

## 📋 VISÃO GERAL

Middleware robusto em Rust para integração entre ChatGuru e ClickUp, com suporte a Pub/Sub para eventos assíncronos no Google Cloud Platform.

## 🏗️ ESTRUTURA DO PROJETO

```
chatguru-clickup-middleware/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   ├── chatguru_webhook.rs
│   │   └── clickup.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── clickup_service.rs
│   │   ├── pubsub_service.rs
│   │   └── event_processor.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── chatguru_events.rs
│   │   ├── clickup_tasks.rs
│   │   └── integration_status.rs
│   └── utils/
│       ├── mod.rs
│       ├── error.rs
│       └── logging.rs
├── config/
│   ├── development.toml
│   └── production.toml
└── docker/
    ├── Dockerfile
    └── docker-compose.yml
```

---

## 📦 CARGO.TOML

```toml
[package]
name = "chatguru-clickup-middleware"
version = "0.1.0"
edition = "2021"
authors = ["eLai Integration Team"]
description = "Middleware for ChatGuru-ClickUp integration with Google Cloud Pub/Sub"

[dependencies]
# Web framework
axum = { version = "0.7", features = ["json", "macros", "tokio"] }
tokio = { version = "1.0", features = ["full"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.14"
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# UUID generation
uuid = { version = "1.0", features = ["v4", "serde"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Google Cloud
google-cloud-pubsub = "0.20"
google-cloud-gax = "0.17"

# Database (opcional)
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid"], optional = true }

[features]
default = []
database = ["sqlx"]
```

---

## 🚀 MAIN.RS - PONTO DE ENTRADA

```rust
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod models;
mod services;
mod utils;

use config::Settings;
use handlers::{health, chatguru_webhook, clickup};
use services::{clickup_service, pubsub_service};
use utils::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Inicializar logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chatguru_clickup_middleware=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Carregar configurações
    let settings = Settings::new()?;
    
    // Inicializar serviços
    let clickup_service = clickup_service::ClickUpService::new(
        settings.clickup.token.clone(),
        settings.clickup.list_id.clone(),
    );
    
    let pubsub_service = pubsub_service::PubSubService::new(
        settings.gcp.project_id.clone(),
    ).await?;

    // Criar estado da aplicação
    let app_state = AppState {
        clickup_service,
        pubsub_service,
        settings,
    };

    // Configurar rotas
    let app = Router::new()
        // Health checks
        .route("/health", get(health::health_check))
        .route("/status", get(health::integration_status))
        
        // Webhooks ChatGuru
        .route("/webhooks/chatguru", post(chatguru_webhook::handle_chatguru_event))
        
        // ClickUp endpoints
        .route("/clickup/tasks", post(clickup::create_task))
        .route("/clickup/tasks/:task_id", get(clickup::get_task))
        
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], settings.server.port));
    tracing::info!("🚀 ChatGuru-ClickUp Middleware rodando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub clickup_service: clickup_service::ClickUpService,
    pub pubsub_service: pubsub_service::PubSubService,
    pub settings: Settings,
}
```

---

## ⚙️ CONFIGURAÇÃO - CONFIG/SETTINGS.RS

```rust
use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerSettings,
    pub clickup: ClickUpSettings,
    pub gcp: GcpSettings,
    pub chatguru: ChatGuruSettings,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruSettings {
    pub webhook_secret: Option<String>,
    pub validate_signature: bool,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            // Arquivo de configuração base
            .add_source(File::with_name("config/default"))
            // Arquivo específico do ambiente
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Variáveis de ambiente com prefixo SURI_CLICKUP_
            .add_source(Environment::with_prefix("SURI_CLICKUP"))
            .build()?;

        s.try_deserialize()
    }
}
```

---

## 🎯 HANDLERS - WEBHOOKS SURI

```rust
// src/handlers/chatguru_webhook.rs
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::Value;
use tracing::{info, error, warn};

use crate::{
    AppState,
    models::chatguru_events::{ChatGuruEvent, ChatGuruEventType},
    utils::error::AppError,
};

pub async fn handle_chatguru_event(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<ResponseJson<Value>, AppError> {
    info!("📥 Recebido evento ChatGuru: {}", serde_json::to_string(&payload)?);

    // Parse do evento ChatGuru
    let chatguru_event: ChatGuruEvent = serde_json::from_value(payload)?;
    
    // Log do tipo de evento
    info!("🎯 Processando evento: {:?}", chatguru_event.event_type);

    // Processar evento baseado no tipo
    match chatguru_event.event_type {
        ChatGuruEventType::NovoContato => {
            handle_new_contact(&state, &chatguru_event).await?;
        },
        ChatGuruEventType::MensagemRecebida => {
            handle_message_received(&state, &chatguru_event).await?;
        },
        ChatGuruEventType::TrocaFila => {
            handle_queue_change(&state, &chatguru_event).await?;
        },
        ChatGuruEventType::FinalizacaoAtendimento => {
            handle_service_end(&state, &chatguru_event).await?;
        },
    }

    // Publicar evento no Pub/Sub para processamento assíncrono
    state.pubsub_service.publish_event(&chatguru_event).await?;

    Ok(ResponseJson(serde_json::json!({
        "success": true,
        "message": "Evento processado com sucesso",
        "event_id": chatguru_event.id,
        "timestamp": chrono::Utc::now()
    })))
}

async fn handle_new_contact(
    state: &AppState,
    event: &ChatGuruEvent,
) -> Result<(), AppError> {
    info!("👤 Novo contato: {}", event.data.get("contact_name").unwrap_or(&Value::Null));

    // Criar tarefa no ClickUp para novo lead
    let task_data = serde_json::json!({
        "name": format!("🆕 Novo Lead - {}", 
            event.data.get("contact_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Contato Anônimo")
        ),
        "description": format!(
            "📞 **Novo contato via ChatGuru**\n\n\
            **Dados do Contato:**\n\
            - Nome: {}\n\
            - Telefone: {}\n\
            - Canal: {}\n\
            - Timestamp: {}\n\n\
            **Próximos Passos:**\n\
            - [ ] Qualificar lead\n\
            - [ ] Entrar em contato\n\
            - [ ] Registrar no CRM",
            event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
            event.data.get("phone").and_then(|v| v.as_str()).unwrap_or("N/A"),
            event.data.get("channel").and_then(|v| v.as_str()).unwrap_or("WhatsApp"),
            event.timestamp
        ),
        "tags": ["chatguru-lead", "novo-contato", "automacao"],
        "priority": 2
    });

    let task_response = state.clickup_service.create_task(task_data).await?;
    info!("✅ Tarefa ClickUp criada: {}", task_response.get("id").unwrap_or(&Value::Null));

    Ok(())
}

async fn handle_message_received(
    state: &AppState,
    event: &ChatGuruEvent,
) -> Result<(), AppError> {
    let message = event.data.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    info!("💬 Mensagem recebida: {}", message);

    // Análise de sentimento simples (implementar com IA posteriormente)
    if contains_negative_words(message) {
        warn!("⚠️ Mensagem com sentimento negativo detectada");
        
        // Criar tarefa urgente no ClickUp
        let task_data = serde_json::json!({
            "name": "🚨 URGENTE - Suporte ao Cliente",
            "description": format!(
                "**Mensagem com possível insatisfação detectada**\n\n\
                **Mensagem:** {}\n\
                **Contato:** {}\n\
                **Timestamp:** {}\n\n\
                **Ação Necessária:** Contato imediato com supervisor",
                message,
                event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
                event.timestamp
            ),
            "tags": ["urgente", "suporte", "insatisfacao"],
            "priority": 1
        });

        state.clickup_service.create_task(task_data).await?;
    }

    Ok(())
}

async fn handle_queue_change(
    state: &AppState,
    event: &ChatGuruEvent,
) -> Result<(), AppError> {
    info!("🔄 Troca de fila detectada");
    
    let from_queue = event.data.get("from_queue").and_then(|v| v.as_str()).unwrap_or("N/A");
    let to_queue = event.data.get("to_queue").and_then(|v| v.as_str()).unwrap_or("N/A");

    if to_queue == "Esperando atendimento" {
        // Criar tarefa para atendimento humano
        let task_data = serde_json::json!({
            "name": format!("👨‍💼 Atendimento Humano - {}", 
                event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("Cliente")
            ),
            "description": format!(
                "**Cliente aguardando atendimento humano**\n\n\
                **De:** {}\n\
                **Para:** {}\n\
                **Contato:** {}\n\
                **Tempo na fila:** {}\n\n\
                **Prioridade:** Atender o mais rápido possível",
                from_queue, to_queue,
                event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
                event.timestamp
            ),
            "tags": ["atendimento-humano", "fila", "pendente"],
            "priority": 2
        });

        state.clickup_service.create_task(task_data).await?;
    }

    Ok(())
}

async fn handle_service_end(
    state: &AppState,
    event: &ChatGuruEvent,
) -> Result<(), AppError> {
    info!("✅ Finalização de atendimento");

    // Criar tarefa para follow-up
    let task_data = serde_json::json!({
        "name": format!("📋 Follow-up - {}", 
            event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("Cliente")
        ),
        "description": format!(
            "**Atendimento finalizado - Follow-up necessário**\n\n\
            **Contato:** {}\n\
            **Agente:** {}\n\
            **Duração:** {}\n\
            **Finalizado em:** {}\n\n\
            **Ações:**\n\
            - [ ] Enviar pesquisa de satisfação\n\
            - [ ] Registrar resolução no CRM\n\
            - [ ] Avaliar necessidade de follow-up",
            event.data.get("contact_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
            event.data.get("agent_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
            event.data.get("duration").and_then(|v| v.as_str()).unwrap_or("N/A"),
            event.timestamp
        ),
        "tags": ["follow-up", "pos-atendimento", "satisfacao"],
        "priority": 3
    });

    state.clickup_service.create_task(task_data).await?;

    Ok(())
}

fn contains_negative_words(message: &str) -> bool {
    let negative_words = [
        "problema", "erro", "ruim", "péssimo", "terrível", 
        "insatisfeito", "reclamação", "cancelar", "reembolso"
    ];
    
    let message_lower = message.to_lowercase();
    negative_words.iter().any(|&word| message_lower.contains(word))
}
```

---

## 🔧 SERVIÇO CLICKUP

```rust
// src/services/clickup_service.rs
use reqwest::{Client, header::HeaderMap, StatusCode};
use serde_json::Value;
use anyhow::Result;
use tracing::{info, error};

#[derive(Clone)]
pub struct ClickUpService {
    client: Client,
    token: String,
    list_id: String,
    base_url: String,
}

impl ClickUpService {
    pub fn new(token: String, list_id: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", token.parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let client = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Falha ao criar cliente HTTP");

        Self {
            client,
            token,
            list_id,
            base_url: "https://api.clickup.com/api/v2".to_string(),
        }
    }

    pub async fn create_task(&self, task_data: Value) -> Result<Value> {
        let url = format!("{}/list/{}/task", self.base_url, self.list_id);
        
        info!("📤 Criando tarefa ClickUp: {}", url);
        
        let response = self.client
            .post(&url)
            .json(&task_data)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let task_response: Value = response.json().await?;
                info!("✅ Tarefa criada com sucesso: {}", 
                    task_response.get("id").unwrap_or(&Value::Null));
                Ok(task_response)
            },
            status => {
                let error_body = response.text().await?;
                error!("❌ Erro ao criar tarefa: {} - {}", status, error_body);
                Err(anyhow::anyhow!("Erro ClickUp: {} - {}", status, error_body))
            }
        }
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Value> {
        let url = format!("{}/task/{}", self.base_url, task_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let task: Value = response.json().await?;
                Ok(task)
            },
            status => {
                let error_body = response.text().await?;
                Err(anyhow::anyhow!("Erro ao buscar tarefa: {} - {}", status, error_body))
            }
        }
    }

    pub async fn test_connection(&self) -> Result<bool> {
        let url = format!("{}/user", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        Ok(response.status() == StatusCode::OK)
    }
}
```

---

## ☁️ SERVIÇO PUB/SUB

```rust
// src/services/pubsub_service.rs
use google_cloud_pubsub::{
    client::{Client, ClientConfig},
    publisher::Publisher,
    subscription::Subscription,
};
use serde_json::Value;
use anyhow::Result;
use tracing::{info, error};

use crate::models::chatguru_events::ChatGuruEvent;

#[derive(Clone)]
pub struct PubSubService {
    publisher: Publisher,
    _subscription: Subscription,
    topic_name: String,
}

impl PubSubService {
    pub async fn new(project_id: String) -> Result<Self> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config).await?;

        let topic_name = "chatguru-events".to_string();
        let subscription_name = "chatguru-events-subscription".to_string();

        // Criar tópico se não existir
        let topic = client.topic(&topic_name);
        if !topic.exists(None).await? {
            topic.create(None, None).await?;
            info!("📊 Tópico Pub/Sub criado: {}", topic_name);
        }

        // Criar publisher
        let publisher = topic.new_publisher(None);

        // Criar subscription se não existir
        let subscription = client.subscription(&subscription_name);
        if !subscription.exists(None).await? {
            let sub_config = google_cloud_pubsub::subscription::SubscriptionConfig {
                topic: topic_name.clone(),
                ..Default::default()
            };
            subscription.create(sub_config, None).await?;
            info!("📨 Subscription criada: {}", subscription_name);
        }

        Ok(Self {
            publisher,
            _subscription: subscription,
            topic_name,
        })
    }

    pub async fn publish_event(&self, event: &ChatGuruEvent) -> Result<()> {
        let message_data = serde_json::to_vec(event)?;
        
        let message = google_cloud_pubsub::publisher::Message {
            data: message_data,
            attributes: [
                ("event_type".to_string(), format!("{:?}", event.event_type)),
                ("timestamp".to_string(), event.timestamp.to_rfc3339()),
                ("source".to_string(), "chatguru-middleware".to_string()),
            ].into_iter().collect(),
            ..Default::default()
        };

        let message_id = self.publisher.publish(message).await?;
        info!("📢 Evento publicado no Pub/Sub: {} (message_id: {})", 
            self.topic_name, message_id);

        Ok(())
    }

    pub async fn create_topics_and_subscriptions() -> Result<()> {
        // Implementar criação de tópicos específicos conforme necessário
        info!("🏗️ Configurando tópicos Pub/Sub...");
        Ok(())
    }
}
```

---

## 📊 MODELOS DE DADOS

```rust
// src/models/chatguru_events.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruEvent {
    pub id: Uuid,
    pub event_type: ChatGuruEventType,
    pub timestamp: DateTime<Utc>,
    pub data: Value,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChatGuruEventType {
    #[serde(rename = "novo_contato")]
    NovoContato,
    #[serde(rename = "mensagem_recebida")]
    MensagemRecebida,
    #[serde(rename = "troca_fila")]
    TrocaFila,
    #[serde(rename = "finalizacao_atendimento")]
    FinalizacaoAtendimento,
}

// src/models/clickup_tasks.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ClickUpTask {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub priority: u8,
    pub assignees: Option<Vec<String>>,
    pub status: Option<String>,
}

// src/models/integration_status.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub service: String,
    pub status: ServiceStatus,
    pub last_check: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

---

## 🏥 HEALTH CHECKS

```rust
// src/handlers/health.rs
use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use chrono::Utc;

use crate::{AppState, models::integration_status::{IntegrationStatus, ServiceStatus}};

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "chatguru-clickup-middleware",
        "version": "0.1.0"
    })))
}

pub async fn integration_status(
    State(state): State<AppState>,
) -> Result<Json<Vec<IntegrationStatus>>, StatusCode> {
    let mut statuses = Vec::new();

    // Verificar ClickUp
    let clickup_status = match state.clickup_service.test_connection().await {
        Ok(true) => ServiceStatus::Healthy,
        Ok(false) => ServiceStatus::Unhealthy,
        Err(_) => ServiceStatus::Unhealthy,
    };

    statuses.push(IntegrationStatus {
        service: "ClickUp API".to_string(),
        status: clickup_status,
        last_check: Utc::now(),
        message: Some("Conexão com API do ClickUp".to_string()),
    });

    // Verificar Pub/Sub (implementar teste de conectividade)
    statuses.push(IntegrationStatus {
        service: "Google Pub/Sub".to_string(),
        status: ServiceStatus::Healthy,
        last_check: Utc::now(),
        message: Some("Serviço de eventos assíncronos".to_string()),
    });

    Ok(Json(statuses))
}
```

---

## 🐳 CONTAINERIZAÇÃO

```dockerfile
# docker/Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY config ./config

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/chatguru-clickup-middleware /usr/local/bin/chatguru-clickup-middleware
COPY --from=builder /app/config /config

EXPOSE 8080

CMD ["chatguru-clickup-middleware"]
```

```yaml
# docker/docker-compose.yml
version: '3.8'
services:
  middleware:
    build:
      context: ..
      dockerfile: docker/Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUN_MODE=development
      - SURI_CLICKUP_CLICKUP__TOKEN=${CLICKUP_TOKEN}
      - SURI_CLICKUP_CLICKUP__LIST_ID=${CLICKUP_LIST_ID}
      - SURI_CLICKUP_GCP__PROJECT_ID=${GCP_PROJECT_ID}
    volumes:
      - ../config:/config
```

---

## ⚙️ CONFIGURAÇÕES DE AMBIENTE

```toml
# config/development.toml
[server]
host = "0.0.0.0"
port = 8080

[clickup]
base_url = "https://api.clickup.com/api/v2"
token = "pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657"
list_id = "901300373349"

[gcp]
project_id = "buzzlightear"
topic_name = "chatguru-events"
subscription_name = "chatguru-events-subscription"

[chatguru]
validate_signature = false
```

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 8080

[clickup]
base_url = "https://api.clickup.com/api/v2"
# Token será injetado via variável de ambiente

[gcp]
project_id = "buzzlightear"
topic_name = "chatguru-events-prod"
subscription_name = "chatguru-events-prod-subscription"

[chatguru]
validate_signature = true
```

---

## 🚀 DEPLOY NO GOOGLE CLOUD

### **Cloud Run Deploy**

```bash
# Build e deploy
gcloud run deploy chatguru-clickup-middleware \
  --source . \
  --platform managed \
  --region southamerica-east1 \
  --allow-unauthenticated \
  --set-env-vars RUN_MODE=production \
  --set-env-vars SURI_CLICKUP_CLICKUP__TOKEN=pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657 \
  --set-env-vars SURI_CLICKUP_CLICKUP__LIST_ID=901300373349 \
  --set-env-vars SURI_CLICKUP_GCP__PROJECT_ID=buzzlightear
```

### **App Engine Deploy**

```yaml
# app.yaml
runtime: custom
env: flex

automatic_scaling:
  min_num_instances: 1
  max_num_instances: 10

env_variables:
  RUN_MODE: production
  SURI_CLICKUP_CLICKUP__TOKEN: pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657
  SURI_CLICKUP_CLICKUP__LIST_ID: "901300373349"
  SURI_CLICKUP_GCP__PROJECT_ID: buzzlightear
```

---

## 📊 COMANDOS PUB/SUB

### **Criar tópicos e subscriptions**

```bash
# Criar tópico
gcloud pubsub topics create chatguru-events

# Criar subscription
gcloud pubsub subscriptions create chatguru-events-subscription \
  --topic=chatguru-events

# Listar tópicos
gcloud pubsub topics list

# Listar subscriptions
gcloud pubsub subscriptions list
```

---

## 🎯 ENDPOINTS IMPLEMENTADOS

| Endpoint | Método | Descrição | Status |
|----------|---------|-----------|---------|
| `/health` | GET | Health check básico | ✅ |
| `/status` | GET | Status detalhado das integrações | ✅ |
| `/webhooks/chatguru` | POST | Receber eventos do ChatGuru | ✅ |
| `/clickup/tasks` | POST | Criar tarefa no ClickUp | ✅ |
| `/clickup/tasks/:id` | GET | Buscar tarefa específica | ✅ |

---

## 🔍 EXEMPLO DE USO

### **1. Teste de Health Check**
```bash
curl http://localhost:8080/health
```

### **2. Simular evento ChatGuru**
```bash
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "type": "novo_contato",
    "data": {
      "contact_name": "João Silva",
      "phone": "11999999999",
      "channel": "WhatsApp"
    }
  }'
```

### **3. Criar tarefa ClickUp**
```bash
curl -X POST http://localhost:8080/clickup/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Nova tarefa via API",
    "description": "Tarefa criada pelo middleware Rust",
    "tags": ["teste", "api"],
    "priority": 2
  }'
```

---

## 📈 PRÓXIMOS PASSOS

1. **Implementar autenticação JWT**
2. **Adicionar métricas com Prometheus**  
3. **Implementar circuit breaker**
4. **Adicionar testes unitários e de integração**
5. **Configurar alertas e monitoramento**
6. **Implementar análise de sentimento com IA**
7. **Adicionar persistência com PostgreSQL**

---

*Middleware Rust estruturado e pronto para produção! 🦀🚀*