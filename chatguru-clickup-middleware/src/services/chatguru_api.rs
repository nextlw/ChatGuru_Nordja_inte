use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct ChatGuruApiService {
    client: Client,
    api_token: String,
    api_endpoint: String,
    account_id: String,
}

impl ChatGuruApiService {
    pub fn new(api_token: String, api_endpoint: String, account_id: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
            api_endpoint,
            account_id,
        }
    }

    /// Envia anotação de volta para o ChatGuru após processar a mensagem
    /// Similar ao que o sistema legado faz após retornar Success
    pub async fn send_annotation(&self, phone: &str, annotation: &str) -> AppResult<()> {
        // Usa o endpoint de mensagens do ChatGuru
        let url = format!("{}/messages/send-text", self.api_endpoint);
        
        let body = json!({
            "account_id": self.account_id,
            "phone": phone,
            "message": annotation
        });

        log_info(&format!("Sending annotation to ChatGuru for phone {}: {}", phone, annotation));
        
        let response = self.client
            .post(&url)
            .header("X-API-KEY", &self.api_token)  // ChatGuru usa X-API-KEY
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            log_info("ChatGuru annotation sent successfully");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            log_error(&format!("Failed to send ChatGuru annotation: {} - {}", status, error_text));
            Err(AppError::InternalError(format!("ChatGuru API error: {}", error_text)))
        }
    }

}