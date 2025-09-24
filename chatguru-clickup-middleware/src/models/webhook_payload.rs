use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Estrutura flexível que aceita múltiplos formatos de webhook
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum WebhookPayload {
    /// Formato ChatGuru (campanha_id, nome, etc)
    ChatGuru(ChatGuruPayload),
    /// Formato com event_type (antigo)
    EventType(EventTypePayload),
    /// Formato genérico/mínimo
    Generic(GenericPayload),
}

/// Payload do ChatGuru atual
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruPayload {
    pub campanha_id: String,
    pub campanha_nome: String,
    pub origem: String,
    #[serde(default)]
    pub email: String,
    pub nome: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub texto_mensagem: String,
    #[serde(default)]
    pub campos_personalizados: HashMap<String, Value>,
    #[serde(default)]
    pub bot_context: Option<BotContext>,
    #[serde(default)]
    pub responsavel_nome: Option<String>,
    #[serde(default)]
    pub responsavel_email: Option<String>,
    #[serde(default)]
    pub link_chat: String,
    pub celular: String,
    #[serde(default)]
    pub phone_id: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
    #[serde(default)]
    pub chat_created: Option<String>,
}

/// Payload com event_type (formato antigo)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventTypePayload {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub data: EventData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventData {
    pub lead_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub project_name: Option<String>,
    pub task_title: Option<String>,
    pub annotation: Option<String>,
    pub amount: Option<f64>,
    pub status: Option<String>,
    #[serde(default)]
    pub custom_data: HashMap<String, Value>,
    
    // Campos adicionais que podem vir
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Payload genérico/mínimo
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericPayload {
    pub nome: Option<String>,
    pub celular: Option<String>,
    pub email: Option<String>,
    pub mensagem: Option<String>,
    
    // Captura campos extras
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotContext {
    #[serde(rename = "ChatGuru")]
    pub chat_guru: Option<bool>,
}

impl WebhookPayload {
    /// Converte qualquer formato para dados do ClickUp
    pub fn to_clickup_task_data(&self) -> serde_json::Value {
        match self {
            WebhookPayload::ChatGuru(payload) => {
                self.chatguru_to_clickup(payload)
            },
            WebhookPayload::EventType(payload) => {
                self.eventtype_to_clickup(payload)
            },
            WebhookPayload::Generic(payload) => {
                self.generic_to_clickup(payload)
            }
        }
    }
    
    /// Converte payload ChatGuru para ClickUp
    fn chatguru_to_clickup(&self, payload: &ChatGuruPayload) -> serde_json::Value {
        let mut description = format!(
            "**Dados do Contato**\n\n\
             - Nome: {}\n\
             - Email: {}\n\
             - Celular: {}\n\
             - Campanha: {}\n\
             - Origem: {}\n\n",
            payload.nome,
            payload.email,
            payload.celular,
            payload.campanha_nome,
            payload.origem
        );
        
        if !payload.texto_mensagem.is_empty() {
            description.push_str(&format!("**Mensagem**\n{}\n\n", payload.texto_mensagem));
        }
        
        if !payload.link_chat.is_empty() {
            description.push_str(&format!("**Link do Chat**\n{}\n\n", payload.link_chat));
        }
        
        if !payload.campos_personalizados.is_empty() {
            description.push_str("**Campos Personalizados**\n");
            for (key, value) in &payload.campos_personalizados {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
            description.push_str("\n");
        }
        
        if let Some(ref responsavel) = payload.responsavel_nome {
            description.push_str(&format!("**Responsável**: {}", responsavel));
            if let Some(ref email) = payload.responsavel_email {
                description.push_str(&format!(" ({})", email));
            }
            description.push_str("\n");
        }
        
        serde_json::json!({
            "name": format!("[{}] {}", payload.campanha_nome, payload.nome),
            "description": description,
            "tags": payload.tags.clone(),
            "status": "pendente",
            "priority": 3
        })
    }
    
    /// Converte payload EventType para ClickUp
    fn eventtype_to_clickup(&self, payload: &EventTypePayload) -> serde_json::Value {
        let data = &payload.data;
        
        // Determina o título da tarefa
        let title = if let Some(ref task_title) = data.task_title {
            task_title.clone()
        } else if let Some(ref annotation) = data.annotation {
            annotation.clone()
        } else if let Some(ref lead_name) = data.lead_name {
            format!("[{}] {}", payload.event_type, lead_name)
        } else {
            format!("Evento: {}", payload.event_type)
        };
        
        // Constrói descrição
        let mut description = format!(
            "**Tipo de Evento**: {}\n\
             **ID**: {}\n\
             **Timestamp**: {}\n\n",
            payload.event_type, payload.id, payload.timestamp
        );
        
        if let Some(ref lead_name) = data.lead_name {
            description.push_str(&format!("**Nome**: {}\n", lead_name));
        }
        if let Some(ref phone) = data.phone {
            description.push_str(&format!("**Telefone**: {}\n", phone));
        }
        if let Some(ref email) = data.email {
            description.push_str(&format!("**Email**: {}\n", email));
        }
        if let Some(ref project) = data.project_name {
            description.push_str(&format!("**Projeto**: {}\n", project));
        }
        if let Some(ref amount) = data.amount {
            description.push_str(&format!("**Valor**: R$ {:.2}\n", amount));
        }
        if let Some(ref status) = data.status {
            description.push_str(&format!("**Status**: {}\n", status));
        }
        
        if !data.custom_data.is_empty() {
            description.push_str("\n**Dados Customizados**\n");
            for (key, value) in &data.custom_data {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
        }
        
        // Define tags baseadas no tipo de evento
        let tags = match payload.event_type.as_str() {
            "new_lead" => vec!["lead".to_string(), "novo".to_string()],
            "payment_created" => vec!["pagamento".to_string(), "criado".to_string()],
            "payment_completed" => vec!["pagamento".to_string(), "concluído".to_string()],
            _ => vec![payload.event_type.clone()],
        };
        
        serde_json::json!({
            "name": title,
            "description": description,
            "tags": tags,
            "status": "pendente",
            "priority": 3
        })
    }
    
    /// Converte payload genérico para ClickUp
    fn generic_to_clickup(&self, payload: &GenericPayload) -> serde_json::Value {
        let nome = payload.nome.as_ref().map(|s| s.as_str()).unwrap_or("Contato");
        let celular = payload.celular.as_ref().map(|s| s.as_str()).unwrap_or("Não informado");
        let email = payload.email.as_ref().map(|s| s.as_str()).unwrap_or("Não informado");
        
        let mut description = format!(
            "**Dados do Contato**\n\n\
             - Nome: {}\n\
             - Celular: {}\n\
             - Email: {}\n",
            nome, celular, email
        );
        
        if let Some(ref mensagem) = payload.mensagem {
            description.push_str(&format!("\n**Mensagem**\n{}\n", mensagem));
        }
        
        if !payload.extra.is_empty() {
            description.push_str("\n**Dados Adicionais**\n");
            for (key, value) in &payload.extra {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
        }
        
        serde_json::json!({
            "name": format!("[Webhook] {}", nome),
            "description": description,
            "tags": vec!["webhook", "genérico"],
            "status": "pendente",
            "priority": 3
        })
    }
    
    /// Extrai um identificador único para busca de duplicatas
    pub fn get_task_title(&self) -> String {
        match self {
            WebhookPayload::ChatGuru(p) => format!("[{}] {}", p.campanha_nome, p.nome),
            WebhookPayload::EventType(p) => {
                if let Some(ref title) = p.data.task_title {
                    title.clone()
                } else if let Some(ref name) = p.data.lead_name {
                    format!("[{}] {}", p.event_type, name)
                } else {
                    format!("Evento: {}", p.event_type)
                }
            },
            WebhookPayload::Generic(p) => {
                format!("[Webhook] {}", p.nome.as_ref().unwrap_or(&"Contato".to_string()))
            }
        }
    }
}