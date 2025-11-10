use crate::error::{ChatGuruError, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Cliente HTTP para a API do ChatGuru
///
/// Fornece métodos para adicionar anotações e enviar mensagens de confirmação
/// via WhatsApp através da API do ChatGuru.
///
/// # Exemplo
///
/// ```rust,ignore
/// use chatguru::ChatGuruClient;
///
/// let client = ChatGuruClient::new(
///     "api_token".to_string(),
///     "https://api.chatguru.app/api/v1".to_string(),
///     "account_id".to_string()
/// );
///
/// client.add_annotation(
///     "chat_123",
///     "5511999999999",
///     "Tarefa criada no ClickUp"
/// ).await?;
/// ```
#[derive(Clone)]
pub struct ChatGuruClient {
    client: Client,
    api_token: String,
    api_endpoint: String,
    account_id: String,
    _message_states: Arc<RwLock<HashMap<String, MessageState>>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct MessageState {
    phone: String,
    chat_id: Option<String>,
    annotation: String,
    timestamp: DateTime<Utc>,
    sent: bool,
}

impl ChatGuruClient {
    /// Cria uma nova instância do cliente ChatGuru
    ///
    /// # Parâmetros
    ///
    /// * `api_token` - Token de autenticação da API ChatGuru
    /// * `api_endpoint` - URL base da API (ex: `https://api.chatguru.app/api/v1`)
    /// * `account_id` - ID da conta ChatGuru
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// let client = ChatGuruClient::new(
    ///     std::env::var("CHATGURU_API_TOKEN")?,
    ///     "https://api.chatguru.app/api/v1".to_string(),
    ///     std::env::var("CHATGURU_ACCOUNT_ID")?
    /// );
    /// ```
    pub fn new(api_token: String, api_endpoint: String, account_id: String) -> Self {
        // Cliente HTTP com timeout de 10s para ChatGuru
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());

        tracing::info!("⚡ ChatGuru client configured with 10s timeout");

        Self {
            client,
            api_token,
            api_endpoint,
            account_id,
            _message_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adiciona uma anotação ao chat no ChatGuru
    ///
    /// Usa a API do ChatGuru para adicionar uma nota/anotação visível no chat.
    ///
    /// # Parâmetros
    ///
    /// * `chat_id` - ID do chat onde adicionar a anotação
    /// * `phone_number` - Número de telefone do contato (com código do país)
    /// * `annotation_text` - Texto da anotação a ser adicionada
    ///
    /// # Retorno
    ///
    /// Retorna `Ok(())` se a anotação foi adicionada com sucesso, ou um erro caso contrário.
    /// Nota: Erros de "chat não encontrado" são logados como warning mas não falham o processo.
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// client.add_annotation(
    ///     "chat_abc123",
    ///     "5511999999999",
    ///     "Tarefa criada no ClickUp: TASK-456"
    /// ).await?;
    /// ```
    pub async fn add_annotation(
        &self,
        chat_id: &str,
        phone_number: &str,
        annotation_text: &str
    ) -> Result<()> {
        // Construir URL com parâmetros
        let phone_id_value = "62558780e2923cc4705beee1"; // Phone ID padrão do sistema

        // Formato E.164: +<country_code><number>
        // Remove caracteres especiais mas mantém o "+" se existir
        let mut clean_phone = phone_number.chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect::<String>();

        // Adiciona "+" no início se não tiver
        if !clean_phone.starts_with('+') && !clean_phone.is_empty() {
            clean_phone = format!("+{}", clean_phone);
        }

        // Construir URL com query params para adicionar anotação
        let base_url = if self.api_endpoint.ends_with("/api/v1") {
            self.api_endpoint.clone()
        } else if self.api_endpoint.ends_with("/") {
            format!("{}api/v1", self.api_endpoint)
        } else {
            format!("{}/api/v1", self.api_endpoint)
        };

        let url = format!(
            "{}?key={}&account_id={}&phone_id={}&action=note_add&note_text={}&chat_number={}",
            base_url,
            self.api_token,
            self.account_id,
            phone_id_value,
            urlencoding::encode(annotation_text),
            clean_phone
        );

        tracing::info!(
            "Adding annotation to chat {}: {}",
            chat_id, annotation_text
        );

        // Fazer a requisição POST
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| ChatGuruError::NetworkError(format!("Failed to add annotation: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if status.is_success() || status.as_u16() == 201 {
            tracing::info!(
                "Annotation added successfully to chat {}: {}",
                chat_id, response_text
            );

            // Logar como o legado
            tracing::info!("Mensagem enviada com sucesso: {}", annotation_text);

            Ok(())
        } else {
            // Apenas logar warning se for erro de chat não encontrado
            if response_text.contains("Chat não encontrado") || response_text.contains("Chat n") {
                tracing::warn!(
                    "Chat not found for annotation (phone: {}). This is normal for inactive chats.",
                    phone_number
                );
            } else {
                tracing::error!(
                    "Failed to add annotation. Status: {}, Response: {}",
                    status, response_text
                );
            }

            // Não falhar o processo se a anotação falhar
            Ok(())
        }
    }

    /// Envia uma mensagem de confirmação via WhatsApp
    ///
    /// Usa a API do ChatGuru para enviar mensagem direta ao usuário.
    ///
    /// **NOTA**: Só funciona se já existe um chat ativo com o número.
    ///
    /// # Parâmetros
    ///
    /// * `phone_number` - Número de telefone do destinatário (com código do país)
    /// * `phone_id` - ID do telefone ChatGuru (opcional, usa padrão se None)
    /// * `message` - Texto da mensagem a ser enviada
    ///
    /// # Retorno
    ///
    /// Retorna `Ok(())` se a mensagem foi enviada com sucesso, ou um erro caso contrário.
    /// Nota: Erros de "chat não existe" são logados como warning mas não falham o processo.
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// client.send_confirmation_message(
    ///     "5511999999999",
    ///     None,
    ///     "✅ Sua solicitação foi registrada com sucesso!"
    /// ).await?;
    /// ```
    pub async fn send_confirmation_message(
        &self,
        phone_number: &str,
        phone_id: Option<&str>,
        message: &str
    ) -> Result<()> {
        // Construir URL com parâmetros
        let phone_id_value = phone_id.unwrap_or("62558780e2923cc4705beee1");

        // Formato E.164: +<country_code><number>
        // Remove caracteres especiais mas mantém o "+" se existir
        let mut clean_phone = phone_number.chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect::<String>();

        // Adiciona "+" no início se não tiver
        if !clean_phone.starts_with('+') && !clean_phone.is_empty() {
            clean_phone = format!("+{}", clean_phone);
        }

        // Construir URL com query params
        // Se api_endpoint já contém /api/v1, não adicionar novamente
        let base_url = if self.api_endpoint.ends_with("/api/v1") {
            self.api_endpoint.clone()
        } else if self.api_endpoint.ends_with("/") {
            format!("{}api/v1", self.api_endpoint)
        } else {
            format!("{}/api/v1", self.api_endpoint)
        };

        // Enviar mensagem imediatamente (sem agendamento)
        // Removido send_date para envio imediato
        let url = format!(
            "{}?key={}&account_id={}&phone_id={}&action=message_send&text={}&chat_number={}",
            base_url,
            self.api_token,
            self.account_id,
            phone_id_value,
            urlencoding::encode(message),
            clean_phone
        );

        tracing::info!(
            "Sending confirmation message to {}: {}",
            phone_number, message
        );

        // Fazer a requisição POST
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| ChatGuruError::NetworkError(format!("Failed to send message: {}", e)))?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();

        if status.is_success() || status.as_u16() == 201 {
            tracing::info!(
                "Confirmation message sent successfully to {}: {}",
                phone_number, response_text
            );

            // Logar como o legado
            tracing::info!("Mensagem enviada com sucesso: {}", message);

            Ok(())
        } else {
            // Apenas logar warning se for erro de chat não encontrado
            if response_text.contains("Chat não existe") || response_text.contains("Chat n") {
                tracing::warn!(
                    "Chat not found for message (phone: {}). This is normal - user may not have active chat.",
                    phone_number
                );
            } else {
                tracing::error!(
                    "Failed to send confirmation message. Status: {}, Response: {}",
                    status, response_text
                );
            }

            // Não falhar o processo se o envio falhar
            Ok(())
        }
    }
}
