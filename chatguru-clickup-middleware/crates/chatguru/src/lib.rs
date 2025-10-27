//! Cliente completo da API ChatGuru
//!
//! Este crate fornece uma interface tipo-segura e ergonômica para interagir com a API do ChatGuru,
//! incluindo funcionalidades como:
//!
//! - Cliente HTTP para adicionar anotações aos chats
//! - Cliente HTTP para enviar mensagens de confirmação via WhatsApp
//! - Tipos de webhook flexíveis (ChatGuru, EventType, Generic)
//! - Normalização automática de campos de mídia
//! - Tratamento de erros específico para ChatGuru
//!
//! # Arquitetura da API ChatGuru
//!
//! A API do ChatGuru usa autenticação via token e permite:
//! - **Adicionar anotações**: Notas visíveis no chat para o atendente
//! - **Enviar mensagens**: Mensagens enviadas diretamente ao cliente via WhatsApp
//!
//! ## Endpoints Implementados
//!
//! ### Adicionar Anotação
//! ```text
//! POST {api_endpoint}?key={token}&account_id={id}&phone_id={phone_id}
//!      &action=note_add&note_text={text}&chat_number={number}
//! ```
//!
//! ### Enviar Mensagem
//! ```text
//! POST {api_endpoint}?key={token}&account_id={id}&phone_id={phone_id}
//!      &action=message_send&text={text}&chat_number={number}
//! ```
//!
//! # Exemplo Básico
//!
//! ```rust,ignore
//! use chatguru::{ChatGuruClient, types::WebhookPayload};
//!
//! #[tokio::main]
//! async fn main() -> chatguru::Result<()> {
//!     // IMPORTANTE: Ler de variáveis de ambiente (NUNCA hardcode!)
//!     let api_token = std::env::var("CHATGURU_API_TOKEN")
//!         .expect("CHATGURU_API_TOKEN não configurado");
//!     let api_endpoint = "https://api.chatguru.app/api/v1";
//!     let account_id = std::env::var("CHATGURU_ACCOUNT_ID")
//!         .expect("CHATGURU_ACCOUNT_ID não configurado");
//!
//!     let client = ChatGuruClient::new(
//!         api_token,
//!         api_endpoint.to_string(),
//!         account_id
//!     );
//!
//!     // Adicionar anotação ao chat
//!     client.add_annotation(
//!         "chat_123",
//!         "5511999999999",
//!         "Tarefa criada no ClickUp: TASK-456"
//!     ).await?;
//!
//!     // Enviar mensagem de confirmação
//!     client.send_confirmation_message(
//!         "5511999999999",
//!         None,
//!         "✅ Sua solicitação foi registrada!"
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Processamento de Webhooks
//!
//! O crate suporta múltiplos formatos de webhook através do enum `WebhookPayload`:
//!
//! ```rust,ignore
//! use chatguru::types::WebhookPayload;
//!
//! let payload_json = r#"{"campanha_id": "123", "nome": "João", ...}"#;
//! let payload: WebhookPayload = serde_json::from_str(payload_json)?;
//!
//! match payload {
//!     WebhookPayload::ChatGuru(p) => {
//!         println!("ChatGuru webhook: {}", p.nome);
//!         println!("Mensagem: {}", p.texto_mensagem);
//!     },
//!     WebhookPayload::EventType(p) => {
//!         println!("Event webhook: {}", p.event_type);
//!     },
//!     WebhookPayload::Generic(p) => {
//!         println!("Generic webhook: {:?}", p.nome);
//!     }
//! }
//! ```
//!
//! # Normalização de Mídia
//!
//! O ChatGuru pode enviar mídia em diferentes formatos. Use `normalize_media_fields()`
//! para garantir compatibilidade:
//!
//! ```rust,ignore
//! let mut payload = ChatGuruPayload { ... };
//! payload.normalize_media_fields();
//!
//! if let Some(media_url) = payload.media_url {
//!     println!("Mídia anexada: {}", media_url);
//! }
//! ```
//!
//! # Configuração
//!
//! Configure através de variáveis de ambiente:
//!
//! - `CHATGURU_API_TOKEN`: Token de autenticação da API
//! - `CHATGURU_API_ENDPOINT`: URL base da API (padrão: `https://api.chatguru.app/api/v1`)
//! - `CHATGURU_ACCOUNT_ID`: ID da conta ChatGuru
//!
//! # Tratamento de Erros
//!
//! Todos os métodos retornam `chatguru::Result<T>`, que é um alias para `Result<T, ChatGuruError>`.
//!
//! Os erros são categorizados em:
//! - `NetworkError`: Falhas de rede/HTTP
//! - `ApiError`: Erros retornados pela API
//! - `SerializationError`: Erros de serialização/deserialização JSON
//! - `ValidationError`: Dados inválidos
//! - `InternalError`: Erros internos do cliente

// Módulos públicos
pub mod client;
pub mod error;
pub mod types;

// Re-exports principais
pub use client::ChatGuruClient;
pub use error::{ChatGuruError, Result};

// Re-exports de types para conveniência
pub use types::{
    ChatGuruPayload, BotContext,
    EventTypePayload, EventData, GenericPayload,
    WebhookPayload,
};
