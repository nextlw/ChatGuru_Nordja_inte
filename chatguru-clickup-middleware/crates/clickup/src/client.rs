//! Cliente HTTP para a API do ClickUp

use crate::error::{ClickUpError, Result};
use reqwest::{Client as HttpClient, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

/// Cliente para interagir com a API do ClickUp
///
/// Suporta ambas as versões da API (v2 e v3):
/// - v3: workspaces, groups, webhooks (workspace-centric)
/// - v2: tasks, lists, custom fields, folders, attachments
#[derive(Clone)]
pub struct ClickUpClient {
    http_client: HttpClient,
    api_token: String,
    base_url_v2: String,
    base_url_v3: String,
}

impl ClickUpClient {
    /// Cria um novo cliente ClickUp
    ///
    /// # Argumentos
    ///
    /// * `api_token` - Token de autenticação (Personal Token ou OAuth2)
    ///
    /// # Timeouts
    ///
    /// - Total: 30s
    /// - Connect: 5s
    pub fn new(api_token: impl Into<String>) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            api_token: api_token.into(),
            base_url_v2: "https://api.clickup.com/api/v2".to_string(),
            base_url_v3: "https://api.clickup.com/api/v3".to_string(),
        })
    }

    /// Cria um novo cliente com timeouts customizados
    pub fn with_timeouts(
        api_token: impl Into<String>,
        total_timeout_secs: u64,
        connect_timeout_secs: u64,
    ) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(total_timeout_secs))
            .connect_timeout(Duration::from_secs(connect_timeout_secs))
            .build()
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http_client,
            api_token: api_token.into(),
            base_url_v2: "https://api.clickup.com/api/v2".to_string(),
            base_url_v3: "https://api.clickup.com/api/v3".to_string(),
        })
    }

    /// Executa uma requisição GET (API v2)
    pub(crate) async fn get(&self, endpoint: &str) -> Result<Response> {
        self.get_v2(endpoint).await
    }

    /// Executa uma requisição GET e parseia JSON (API v2)
    pub(crate) async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.get_json_v2(endpoint).await
    }

    /// Executa uma requisição GET na API v2
    pub(crate) async fn get_v2(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v2, endpoint);

        tracing::debug!("GET v2 {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição GET na API v2 e parseia JSON
    pub(crate) async fn get_json_v2<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.get_v2(endpoint).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição GET na API v3
    pub(crate) async fn get_v3(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v3, endpoint);

        tracing::debug!("GET v3 {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição GET na API v3 e parseia JSON
    pub(crate) async fn get_json_v3<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.get_v3(endpoint).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição POST (API v2)
    pub(crate) async fn post(&self, endpoint: &str, body: &Value) -> Result<Response> {
        self.post_v2(endpoint, body).await
    }

    /// Executa uma requisição POST e parseia JSON (API v2)
    pub(crate) async fn post_json<T: DeserializeOwned>(&self, endpoint: &str, body: &Value) -> Result<T> {
        self.post_json_v2(endpoint, body).await
    }

    /// Executa uma requisição POST na API v2
    pub(crate) async fn post_v2(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v2, endpoint);

        tracing::debug!("POST v2 {} with body: {}", url, serde_json::to_string(body).unwrap_or_default());

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição POST na API v2 e parseia JSON
    pub(crate) async fn post_json_v2<T: DeserializeOwned>(&self, endpoint: &str, body: &Value) -> Result<T> {
        let response = self.post_v2(endpoint, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição POST na API v3
    pub(crate) async fn post_v3(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v3, endpoint);

        tracing::debug!("POST v3 {} with body: {}", url, serde_json::to_string(body).unwrap_or_default());

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição POST na API v3 e parseia JSON
    pub(crate) async fn post_json_v3<T: DeserializeOwned>(&self, endpoint: &str, body: &Value) -> Result<T> {
        let response = self.post_v3(endpoint, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição PUT (API v2)
    pub(crate) async fn put(&self, endpoint: &str, body: &Value) -> Result<Response> {
        self.put_v2(endpoint, body).await
    }

    /// Executa uma requisição PUT e parseia JSON (API v2)
    pub(crate) async fn put_json<T: DeserializeOwned>(&self, endpoint: &str, body: &Value) -> Result<T> {
        self.put_json_v2(endpoint, body).await
    }

    /// Executa uma requisição PUT na API v2
    pub(crate) async fn put_v2(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v2, endpoint);

        tracing::debug!("PUT v2 {}", url);

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição PUT na API v2 e parseia JSON
    pub(crate) async fn put_json_v2<T: DeserializeOwned>(&self, endpoint: &str, body: &Value) -> Result<T> {
        let response = self.put_v2(endpoint, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição DELETE (API v2)
    pub(crate) async fn delete(&self, endpoint: &str) -> Result<Response> {
        self.delete_v2(endpoint).await
    }

    /// Executa uma requisição DELETE na API v2
    pub(crate) async fn delete_v2(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url_v2, endpoint);

        tracing::debug!("DELETE v2 {}", url);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", &self.api_token)
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Processa a resposta HTTP e trata erros
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let status_code = status.as_u16();
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());

            tracing::error!("ClickUp API error ({}): {}", status_code, error_body);

            // Tentar extrair mensagem de erro do JSON
            let message = if let Ok(json) = serde_json::from_str::<Value>(&error_body) {
                json.get("err")
                    .or_else(|| json.get("error"))
                    .or_else(|| json.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(&error_body)
                    .to_string()
            } else {
                error_body
            };

            Err(ClickUpError::ApiError {
                status: status_code,
                message,
            })
        }
    }

    /// Obtém o token de autenticação
    pub fn token(&self) -> &str {
        &self.api_token
    }

    /// Obtém a URL base v2 (padrão para a maioria dos endpoints)
    pub fn base_url(&self) -> &str {
        &self.base_url_v2
    }

    /// Obtém a URL base v2
    pub fn base_url_v2(&self) -> &str {
        &self.base_url_v2
    }

    /// Obtém a URL base v3
    pub fn base_url_v3(&self) -> &str {
        &self.base_url_v3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ClickUpClient::new("test-token").unwrap();
        assert_eq!(client.token(), "test-token");
        assert_eq!(client.base_url(), "https://api.clickup.com/api/v2");
        assert_eq!(client.base_url_v2(), "https://api.clickup.com/api/v2");
        assert_eq!(client.base_url_v3(), "https://api.clickup.com/api/v3");
    }

    #[test]
    fn test_client_with_custom_timeouts() {
        let client = ClickUpClient::with_timeouts("test-token", 60, 10).unwrap();
        assert_eq!(client.token(), "test-token");
    }
}
