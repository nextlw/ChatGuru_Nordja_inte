//! Cliente HTTP para a API do ClickUp

use crate::error::{ClickUpError, Result};
use reqwest::{Client as HttpClient, Response};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::time::Duration;

/// Cliente para interagir com a API do ClickUp (v2 apenas)
///
/// Usa apenas a API v2 que é estável e suporta todos os recursos necessários:
/// - tasks, lists, custom fields, folders, attachments, webhooks, spaces
#[derive(Clone, Debug)]
pub struct ClickUpClient {
    http_client: HttpClient,
    api_token: String,
    base_url: String,
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
            .map_err(|e| {
                ClickUpError::ConfigError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            http_client,
            api_token: api_token.into(),
            base_url: "https://api.clickup.com/api/v2".to_string(),
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
            .map_err(|e| {
                ClickUpError::ConfigError(format!("Failed to create HTTP client: {}", e))
            })?;

        Ok(Self {
            http_client,
            api_token: api_token.into(),
            base_url: "https://api.clickup.com/api/v2".to_string(),
        })
    }

    /// Executa uma requisição GET
    pub(crate) async fn get(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);

        tracing::debug!("GET {}", url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição GET e parseia JSON
    pub(crate) async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.get(endpoint).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição POST
    pub(crate) async fn post(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);

        tracing::debug!(
            "POST {} with body: {}",
            url,
            serde_json::to_string(body).unwrap_or_default()
        );

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.api_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição POST e parseia JSON
    pub(crate) async fn post_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
    ) -> Result<T> {
        let response = self.post(endpoint, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição PUT
    pub(crate) async fn put(&self, endpoint: &str, body: &Value) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);

        tracing::debug!("PUT {}", url);

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", &self.api_token))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição PUT e parseia JSON
    pub(crate) async fn put_json<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
    ) -> Result<T> {
        let response = self.put(endpoint, body).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Executa uma requisição DELETE
    ///
    /// Usado para deletar recursos: tasks, folders, lists, custom fields.
    #[allow(dead_code)] // Necessário para operações de deleção (ainda não implementadas)
    pub(crate) async fn delete(&self, endpoint: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);

        tracing::debug!("DELETE {}", url);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", &self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Executa uma requisição DELETE e parseia JSON
    pub(crate) async fn delete_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.delete(endpoint).await?;
        let json = response.json().await?;
        Ok(json)
    }

    /// Processa a resposta HTTP e trata erros
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            Ok(response)
        } else {
            let status_code = status.as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

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

    /// Obtém a URL base da API
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Busca uma lista por nome em uma pasta específica
    ///
    /// # Argumentos
    ///
    /// * `folder_id` - ID da pasta onde buscar a lista
    /// * `list_name` - Nome da lista a ser encontrada
    ///
    /// # Retorna
    ///
    /// `Ok(Some(list))` se a lista for encontrada
    /// `Ok(None)` se a lista não for encontrada
    /// `Err(...)` em caso de erro na API
    pub async fn find_list_by_name(
        &self,
        folder_id: &str,
        list_name: &str,
    ) -> Result<Option<ListInfo>> {
        let endpoint = format!("/folder/{}", folder_id);
        let folder_data: Value = self.get_json(&endpoint).await?;

        if let Some(lists) = folder_data["lists"].as_array() {
            for list in lists {
                if let Some(name) = list["name"].as_str() {
                    if name.eq_ignore_ascii_case(list_name) {
                        let id = list["id"]
                            .as_str()
                            .map(|s| s.to_string())
                            .or_else(|| list["id"].as_u64().map(|n| n.to_string()))
                            .unwrap_or_else(String::new);

                        return Ok(Some(ListInfo {
                            id,
                            name: name.to_string(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Cria uma nova lista em uma pasta
    ///
    /// # Argumentos
    ///
    /// * `folder_id` - ID da pasta onde criar a lista
    /// * `list_data` - Dados da lista a ser criada
    ///
    /// # Retorna
    ///
    /// A lista criada com o ID atribuído pela API
    pub async fn create_list(
        &self,
        folder_id: &str,
        list_data: &CreateListRequest,
    ) -> Result<ListInfo> {
        let endpoint = format!("/folder/{}/list", folder_id);

        let body = json!({
            "name": list_data.name,
            "content": list_data.content,
            "due_date": list_data.due_date,
            "priority": list_data.priority,
            "assignee": list_data.assignee,
            "status": list_data.status
        });

        let response: Value = self.post_json(&endpoint, &body).await?;

        let id = response["id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| response["id"].as_u64().map(|n| n.to_string()))
            .unwrap_or_default();

        let name = response["name"]
            .as_str()
            .unwrap_or(&list_data.name)
            .to_string();

        Ok(ListInfo { id, name })
    }
}

/// Informações básicas de uma lista
#[derive(Debug, Clone)]
pub struct ListInfo {
    pub id: String,
    pub name: String,
}

/// Dados para criar uma nova lista
#[derive(Debug, Clone)]
pub struct CreateListRequest {
    pub name: String,
    pub content: Option<String>,
    pub due_date: Option<i64>,
    pub priority: Option<u32>,
    pub assignee: Option<String>,
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ClickUpClient::new("test-token").unwrap();
        assert_eq!(client.token(), "test-token");
        assert_eq!(client.base_url(), "https://api.clickup.com/api/v2");
    }

    #[test]
    fn test_client_with_custom_timeouts() {
        let client = ClickUpClient::with_timeouts("test-token", 60, 10).unwrap();
        assert_eq!(client.token(), "test-token");
    }
}
