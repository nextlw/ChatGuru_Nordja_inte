use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruEvent {
    pub campanha_id: String,
    pub campanha_nome: String,
    pub origem: String,
    pub email: String,
    pub nome: String,
    pub tags: Vec<String>,
    pub texto_mensagem: String,
    pub campos_personalizados: HashMap<String, Value>,
    pub bot_context: BotContext,
    pub responsavel_nome: Option<String>,
    pub responsavel_email: Option<String>,
    pub link_chat: String,
    pub celular: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotContext {
    #[serde(rename = "ChatGuru")]
    pub chat_guru: bool,
}

impl ChatGuruEvent {
    pub fn to_clickup_task_data(&self) -> serde_json::Value {
        let mut description = format!(
            "**Dados do Contato**\n\n\
             - Nome: {}\n\
             - Email: {}\n\
             - Celular: {}\n\
             - Campanha: {}\n\
             - Origem: {}\n\n\
             **Mensagem**\n{}\n\n\
             **Link do Chat**\n{}\n\n",
            self.nome,
            self.email,
            self.celular,
            self.campanha_nome,
            self.origem,
            self.texto_mensagem,
            self.link_chat
        );

        if !self.campos_personalizados.is_empty() {
            description.push_str("**Campos Personalizados**\n");
            for (key, value) in &self.campos_personalizados {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
            description.push_str("\n");
        }

        if let Some(ref responsavel) = self.responsavel_nome {
            description.push_str(&format!("**Responsável**: {}", responsavel));
            if let Some(ref email) = self.responsavel_email {
                description.push_str(&format!(" ({})", email));
            }
            description.push_str("\n");
        }

        serde_json::json!({
            "name": format!("[{}] {}", self.campanha_nome, self.nome),
            "description": description,
            "tags": self.tags.clone(),
            "status": "pendente",
            "priority": 3
        })
    }

    fn build_custom_fields(&self) -> Vec<serde_json::Value> {
        let mut fields = Vec::new();
        
        for (key, value) in &self.campos_personalizados {
            if let Some(string_value) = value.as_str() {
                fields.push(serde_json::json!({
                    "name": key,
                    "value": string_value
                }));
            } else {
                fields.push(serde_json::json!({
                    "name": key,
                    "value": value.to_string()
                }));
            }
        }
        
        fields
    }
    
    pub fn get_data(&self) -> Value {
        serde_json::json!({
            "campanha_id": self.campanha_id,
            "campanha_nome": self.campanha_nome,
            "origem": self.origem,
            "email": self.email,
            "nome": self.nome,
            "celular": self.celular,
            "texto_mensagem": self.texto_mensagem,
            "tags": self.tags,
            "campos_personalizados": self.campos_personalizados,
            "link_chat": self.link_chat
        })
    }
}

