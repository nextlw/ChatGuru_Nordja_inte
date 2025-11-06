use serde::{Deserialize, Serialize};
use super::payload::{ChatGuruPayload, EventTypePayload, GenericPayload};

/// Estrutura flexível que aceita múltiplos formatos de webhook
///
/// O ChatGuru pode enviar webhooks em diferentes formatos dependendo da
/// configuração e versão. Este enum permite aceitar todos os formatos conhecidos.
///
/// # Variantes
///
/// * `ChatGuru` - Formato atual do ChatGuru (campanha_id, campos_personalizados, etc)
/// * `EventType` - Formato legado com event_type
/// * `Generic` - Formato genérico/mínimo (fallback)
///
/// # Exemplo
///
/// ```rust,ignore
/// use chatguru::types::WebhookPayload;
///
/// let payload_json = r#"{"campanha_id": "123", "nome": "João", ...}"#;
/// let payload: WebhookPayload = serde_json::from_str(payload_json)?;
///
/// match payload {
///     WebhookPayload::ChatGuru(p) => println!("ChatGuru: {}", p.nome),
///     WebhookPayload::EventType(p) => println!("Event: {}", p.event_type),
///     WebhookPayload::Generic(p) => println!("Generic: {:?}", p.nome),
/// }
/// ```
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

impl WebhookPayload {
    /// Extrai o nome/título do contato do payload
    ///
    /// Útil para identificação rápida independente do formato do webhook.
    ///
    /// # Retorno
    ///
    /// String com o nome do contato ou "Contato" como fallback.
    pub fn get_contact_name(&self) -> String {
        match self {
            WebhookPayload::ChatGuru(p) => p.nome.clone(),
            WebhookPayload::EventType(p) => {
                p.data.lead_name.clone().unwrap_or_else(|| "Contato".to_string())
            },
            WebhookPayload::Generic(p) => {
                p.nome.clone().unwrap_or_else(|| "Contato".to_string())
            }
        }
    }

    /// Extrai o número de telefone do payload
    ///
    /// # Retorno
    ///
    /// `Some(String)` com o número de telefone, ou `None` se não disponível.
    pub fn get_phone_number(&self) -> Option<String> {
        match self {
            WebhookPayload::ChatGuru(p) => {
                if !p.celular.is_empty() {
                    Some(p.celular.clone())
                } else {
                    None
                }
            },
            WebhookPayload::EventType(p) => p.data.phone.clone(),
            WebhookPayload::Generic(p) => p.celular.clone(),
        }
    }

    /// Extrai o texto da mensagem do payload
    ///
    /// # Retorno
    ///
    /// `Some(String)` com o texto da mensagem, ou `None` se não disponível.
    pub fn get_message_text(&self) -> Option<String> {
        match self {
            WebhookPayload::ChatGuru(p) => {
                if !p.texto_mensagem.is_empty() {
                    Some(p.texto_mensagem.clone())
                } else {
                    None
                }
            },
            WebhookPayload::EventType(p) => p.data.annotation.clone(),
            WebhookPayload::Generic(p) => p.mensagem.clone(),
        }
    }

    /// Extrai o chat_id do payload (se disponível)
    ///
    /// # Retorno
    ///
    /// `Some(String)` com o chat_id, ou `None` se não disponível.
    pub fn get_chat_id(&self) -> Option<String> {
        match self {
            WebhookPayload::ChatGuru(p) => p.chat_id.clone(),
            WebhookPayload::EventType(p) => Some(p.id.clone()),
            WebhookPayload::Generic(_) => None,
        }
    }

    /// Verifica se o payload contém mídia anexada
    ///
    /// # Retorno
    ///
    /// `true` se há mídia (imagem, áudio, vídeo), `false` caso contrário.
    pub fn has_media(&self) -> bool {
        match self {
            WebhookPayload::ChatGuru(p) => {
                p.media_url.is_some() || p.url_arquivo.is_some()
            },
            _ => false,
        }
    }

    /// Extrai URL da mídia anexada (se houver)
    ///
    /// # Retorno
    ///
    /// `Some(String)` com a URL da mídia, ou `None` se não houver.
    pub fn get_media_url(&self) -> Option<String> {
        match self {
            WebhookPayload::ChatGuru(p) => {
                p.media_url.clone().or_else(|| p.url_arquivo.clone())
            },
            _ => None,
        }
    }

    /// Extrai tipo da mídia anexada (se houver)
    ///
    /// # Retorno
    ///
    /// `Some(String)` com o tipo da mídia (ex: "image/jpeg", "audio/ogg"),
    /// ou `None` se não houver.
    pub fn get_media_type(&self) -> Option<String> {
        match self {
            WebhookPayload::ChatGuru(p) => {
                p.media_type.clone().or_else(|| {
                    // Tentar derivar do tipo_mensagem
                    p.tipo_mensagem.as_ref().map(|t| match t.as_str() {
                        "image" => "image/jpeg".to_string(),
                        "ptt" | "audio" => "audio/ogg".to_string(),
                        "video" => "video/mp4".to_string(),
                        "document" => "application/pdf".to_string(),
                        other => format!("application/{}", other),
                    })
                })
            },
            _ => None,
        }
    }
}
