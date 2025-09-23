use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruEvent {
    #[serde(rename = "event_id")]
    pub id: Option<String>,  // Tornando opcional pois ChatGuru pode não enviar
    pub event_type: String,
    pub timestamp: String, // Mudando para String para compatibilidade
    
    // Campo data agora é opcional
    #[serde(default = "default_value")]
    pub data: Value, // Usando Value vazio como padrão se não existir
    
    pub source: Option<String>,
    pub metadata: Option<Value>,
    
    // Campos específicos do ChatGuru
    pub account_id: Option<String>,
    pub phone_id: Option<String>,
    pub contact: Option<Value>,
    pub annotation: Option<Value>,
}

// Função auxiliar para fornecer valor padrão
fn default_value() -> Value {
    json!({})
}

impl ChatGuruEvent {
    pub fn event_id(&self) -> Option<&str> {
        self.id.as_deref()
    }
    
    /// Retorna os dados do evento - se o campo data estiver vazio,
    /// extrai dados de annotation.data
    pub fn get_data(&self) -> Value {
        if !self.data.is_null() && self.data.is_object() && !self.data.as_object().unwrap().is_empty() {
            self.data.clone()
        } else {
            // Se não há campo data direto, tentar extrair de annotation.data
            if let Some(annotation) = &self.annotation {
                if let Some(ann_data) = annotation.get("data") {
                    return ann_data.clone();
                }
            }
            
            // Se nenhum dos casos acima, construir objeto com dados disponíveis
            let mut data = json!({});
            
            if let Some(annotation) = &self.annotation {
                // Extrair campos relevantes da anotação
                if let Some(title) = annotation.get("title") {
                    data["task_title"] = title.clone();
                }
                if let Some(description) = annotation.get("description") {
                    data["description"] = description.clone();
                }
                if let Some(tags) = annotation.get("tags") {
                    data["tags"] = tags.clone();
                }
                // Adicionar dados extras da annotation se existirem
                if let Some(ann_data) = annotation.get("data") {
                    if let Some(obj) = ann_data.as_object() {
                        for (key, value) in obj {
                            data[key] = value.clone();
                        }
                    }
                }
            }
            
            if let Some(contact) = &self.contact {
                if let Some(phone) = contact.get("phone") {
                    data["phone"] = phone.clone();
                }
                if let Some(name) = contact.get("name") {
                    data["contact_name"] = name.clone();
                }
            }
            
            data
        }
    }
    
    pub fn get_timestamp_as_datetime(&self) -> Result<DateTime<Utc>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .map(|dt| dt.with_timezone(&Utc))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatGuruEventType {
    #[serde(rename = "payment_created")]
    PaymentCreated,
    #[serde(rename = "payment_completed")]
    PaymentCompleted,
    #[serde(rename = "payment_failed")]
    PaymentFailed,
    #[serde(rename = "customer_created")]
    CustomerCreated,
    #[serde(rename = "invoice_generated")]
    InvoiceGenerated,
    #[serde(rename = "pix_received")]
    PixReceived,
    #[serde(rename = "novo_contato")]
    NovoContato,
    #[serde(rename = "mensagem_recebida")]
    MensagemRecebida,
    #[serde(rename = "troca_fila")]
    TrocaFila,
    #[serde(rename = "finalizacao_atendimento")]
    FinalizacaoAtendimento,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for ChatGuruEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_val = match self {
            ChatGuruEventType::PaymentCreated => "payment_created",
            ChatGuruEventType::PaymentCompleted => "payment_completed", 
            ChatGuruEventType::PaymentFailed => "payment_failed",
            ChatGuruEventType::CustomerCreated => "customer_created",
            ChatGuruEventType::InvoiceGenerated => "invoice_generated",
            ChatGuruEventType::PixReceived => "pix_received",
            ChatGuruEventType::NovoContato => "novo_contato",
            ChatGuruEventType::MensagemRecebida => "mensagem_recebida",
            ChatGuruEventType::TrocaFila => "troca_fila",
            ChatGuruEventType::FinalizacaoAtendimento => "finalizacao_atendimento",
            ChatGuruEventType::Unknown => "unknown",
        };
        write!(f, "{}", str_val)
    }
}