use crate::utils::AppResult;
use crate::utils::logging::*;
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use crate::utils::AppError;

#[derive(Clone)]
pub struct ChatGuruApiService {
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

impl ChatGuruApiService {
    pub fn new(api_token: String, api_endpoint: String, account_id: String) -> Self {
        // OTIMIZAÇÃO FASE 1: Cliente HTTP com timeout de 5s para ChatGuru
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        log_info("⚡ ChatGuru client configured with 10s timeout");
        
        Self {
            client,
            api_token,
            api_endpoint,
            account_id,
            _message_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adicionar anotação ao chat no ChatGuru
    /// Usa a API do ChatGuru para adicionar uma nota/anotação visível no chat
    pub async fn add_annotation(
        &self,
        chat_id: &str,
        phone_number: &str, 
        annotation_text: &str
    ) -> AppResult<()> {
        // Construir URL com parâmetros
        let phone_id_value = "62558780e2923cc4705beee1"; // Phone ID padrão do sistema
        
        // Limpar número de telefone (remover caracteres especiais)
        let clean_phone = phone_number.chars()
            .filter(|c| c.is_numeric())
            .collect::<String>();
        
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
        
        log_info(&format!(
            "Adding annotation to chat {}: {}",
            chat_id, annotation_text
        ));
        
        // Fazer a requisição POST
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to add annotation: {}", e)))?;
        
        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();
        
        if status.is_success() || status.as_u16() == 201 {
            log_info(&format!(
                "Annotation added successfully to chat {}: {}",
                chat_id, response_text
            ));
            
            // Logar como o legado
            log_info(&format!("Mensagem enviada com sucesso: {}", annotation_text));
            
            Ok(())
        } else {
            // Apenas logar warning se for erro de chat não encontrado
            if response_text.contains("Chat não encontrado") || response_text.contains("Chat n") {
                log_warning(&format!(
                    "Chat not found for annotation (phone: {}). This is normal for inactive chats.",
                    phone_number
                ));
            } else {
                log_error(&format!(
                    "Failed to add annotation. Status: {}, Response: {}",
                    status, response_text
                ));
            }
            
            // Não falhar o processo se a anotação falhar
            Ok(())
        }
    }
    
    /// Enviar mensagem de confirmação "Ok" via WhatsApp
    /// Usa a API do ChatGuru para enviar mensagem direta ao usuário
    /// NOTA: Só funciona se já existe um chat ativo com o número
    pub async fn send_confirmation_message(
        &self, 
        phone_number: &str,
        phone_id: Option<&str>,
        message: &str
    ) -> AppResult<()> {
        // Construir URL com parâmetros
        let phone_id_value = phone_id.unwrap_or("62558780e2923cc4705beee1");
        
        // Limpar número de telefone (remover caracteres especiais)
        let clean_phone = phone_number.chars()
            .filter(|c| c.is_numeric())
            .collect::<String>();
        
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
        
        log_info(&format!(
            "Sending confirmation message to {}: {}",
            phone_number, message
        ));
        
        // Fazer a requisição POST
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to send message: {}", e)))?;
        
        let status = response.status();
        let response_text = response.text().await.unwrap_or_default();
        
        if status.is_success() || status.as_u16() == 201 {
            log_info(&format!(
                "Confirmation message sent successfully to {}: {}",
                phone_number, response_text
            ));
            
            // Logar como o legado
            log_info(&format!("Mensagem enviada com sucesso: {}", message));
            
            Ok(())
        } else {
            // Apenas logar warning se for erro de chat não encontrado
            if response_text.contains("Chat não existe") || response_text.contains("Chat n") {
                log_warning(&format!(
                    "Chat not found for message (phone: {}). This is normal - user may not have active chat.",
                    phone_number
                ));
            } else {
                log_error(&format!(
                    "Failed to send confirmation message. Status: {}, Response: {}",
                    status, response_text
                ));
            }
            
            // Não falhar o processo se o envio falhar
            Ok(())
        }
    }
}