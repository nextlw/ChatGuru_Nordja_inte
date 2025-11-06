use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Payload do ChatGuru atual
///
/// Estrutura completa do payload recebido nos webhooks do ChatGuru,
/// incluindo campos personalizados, mídia anexada e contexto do bot.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruPayload {
    #[serde(default)]
    pub campanha_id: String,
    #[serde(default)]
    pub campanha_nome: String,
    #[serde(default)]
    pub origem: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub nome: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, alias = "mensagem", alias = "message", alias = "text")]
    pub texto_mensagem: String,

    // Campos de mídia - formato antigo (media_url, media_type)
    #[serde(default)]
    pub media_url: Option<String>,  // URL do áudio ou mídia anexada
    #[serde(default)]
    pub media_type: Option<String>, // Tipo da mídia (audio, image, video)

    // Campos de mídia - formato novo ChatGuru (tipo_mensagem, url_arquivo)
    #[serde(default)]
    pub tipo_mensagem: Option<String>, // "image", "ptt" (áudio), "video", etc
    #[serde(default, alias = "url_midia")]
    pub url_arquivo: Option<String>, // URL do arquivo de mídia

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
    #[serde(default)]
    pub celular: String,
    #[serde(default)]
    pub phone_id: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
    #[serde(default)]
    pub chat_created: Option<String>,
}

impl ChatGuruPayload {
    /// Normaliza os campos de mídia do ChatGuru
    ///
    /// Converte tipo_mensagem + url_arquivo → media_type + media_url
    /// para garantir compatibilidade com diferentes versões da API.
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// let mut payload = ChatGuruPayload { ... };
    /// payload.normalize_media_fields();
    /// assert!(payload.media_url.is_some());
    /// assert!(payload.media_type.is_some());
    /// ```
    pub fn normalize_media_fields(&mut self) {
        // Se já tem media_url e media_type, não faz nada
        if self.media_url.is_some() && self.media_type.is_some() {
            return;
        }

        // Mapear url_arquivo → media_url
        if self.url_arquivo.is_some() && self.media_url.is_none() {
            self.media_url = self.url_arquivo.clone();
        }

        // Mapear tipo_mensagem → media_type
        if let Some(ref tipo) = self.tipo_mensagem {
            if self.media_type.is_none() {
                self.media_type = Some(match tipo.as_str() {
                    "image" => "image/jpeg".to_string(),
                    "ptt" => "audio/ogg".to_string(), // ptt = push-to-talk (áudio)
                    "audio" => "audio/ogg".to_string(),
                    "video" => "video/mp4".to_string(),
                    "document" => "application/pdf".to_string(),
                    other => format!("application/{}", other),
                });
            }
        }
    }
}

/// Contexto do bot ChatGuru
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotContext {
    #[serde(rename = "ChatGuru")]
    pub chat_guru: Option<bool>,
}

/// Payload com event_type (formato antigo/legado)
///
/// Estrutura usada em versões antigas do webhook, mantida para compatibilidade.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventTypePayload {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub data: EventData,
}

/// Dados do evento (formato legado)
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
///
/// Usado como fallback quando o formato do webhook não é reconhecido.
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
