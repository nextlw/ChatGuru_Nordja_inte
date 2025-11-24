//! OAuth2 HTTP Client
//!
//! Cliente HTTP isolado para comunicação com ClickUp OAuth2 API

use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use super::OAuth2Config;

/// Resposta da API de troca de token
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
}

/// Informação de um workspace autorizado
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorizedTeam {
    pub id: String,
    pub name: String,
}

/// Resposta da API de workspaces autorizados
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorizedTeamsResponse {
    pub teams: Vec<AuthorizedTeam>,
}

/// Resultado detalhado da validação de token
#[derive(Debug)]
pub struct TokenValidationResult {
    pub is_valid: bool,
    pub error_code: Option<String>,
    pub status_code: Option<u16>,
    pub should_invalidate_cache: bool,
    pub should_reauthorize: bool,
}

impl Default for TokenValidationResult {
    fn default() -> Self {
        Self {
            is_valid: false,
            error_code: None,
            status_code: None,
            should_invalidate_cache: false,
            should_reauthorize: false,
        }
    }
}

/// Cliente OAuth2 para ClickUp
pub struct OAuth2Client {
    config: OAuth2Config,
    http_client: Client,
}

impl OAuth2Client {
    /// Criar novo cliente OAuth2
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    /// Trocar authorization code por access token
    ///
    /// # Parâmetros
    /// - `code`: Authorization code recebido do callback
    ///
    /// # Retorno
    /// - `Ok(TokenResponse)`: Token obtido com sucesso
    /// - `Err(AppError)`: Erro na troca do token
    pub async fn exchange_code_for_token(&self, code: &str) -> AppResult<TokenResponse> {
        log_info("🔐 [OAuth2] Trocando authorization code por access token...");

        let url = "https://api.clickup.com/api/v2/oauth/token";

        let body = serde_json::json!({
            "client_id": self.config.client_id,
            "client_secret": self.config.client_secret,
            "code": code
        });

        use crate::utils::truncate_safe;
        log_info(&format!("📤 [OAuth2] POST {} - client_id: {}, code: {}...",
            url, &self.config.client_id, truncate_safe(&code, 10)));

        let response = self.http_client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ClickUpApi(format!("Falha ao conectar com ClickUp OAuth API: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            log_error(&format!("❌ [OAuth2] Token exchange failed: {} - {}", status, error_text));
            return Err(AppError::ClickUpApi(format!("OAuth token exchange failed [{}]: {}", status, error_text)));
        }

        let token_response: TokenResponse = response.json().await
            .map_err(|e| AppError::ClickUpApi(format!("Falha ao parsear resposta do token: {}", e)))?;

        use crate::utils::truncate_safe;
        log_info(&format!("✅ [OAuth2] Access token obtido: {}...", truncate_safe(&token_response.access_token, 20)));

        Ok(token_response)
    }

    /// Verificar quais workspaces foram autorizados
    ///
    /// # Parâmetros
    /// - `access_token`: Access token OAuth2 válido
    ///
    /// # Retorno
    /// - `Ok(Vec<AuthorizedTeam>)`: Lista de workspaces autorizados
    /// - `Err(AppError)`: Erro ao consultar API
    pub async fn get_authorized_teams(&self, access_token: &str) -> AppResult<Vec<AuthorizedTeam>> {
        log_info("🔍 [OAuth2] Consultando workspaces autorizados...");

        let url = "https://api.clickup.com/api/v2/team";

        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(|e| AppError::ClickUpApi(format!("Falha ao consultar teams: {}", e)))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            log_error(&format!("❌ [OAuth2] Failed to get authorized teams: {} - {}", status, error_text));
            return Err(AppError::ClickUpApi(format!("Failed to get teams [{}]: {}", status, error_text)));
        }

        let teams_response: AuthorizedTeamsResponse = response.json().await
            .map_err(|e| AppError::ClickUpApi(format!("Falha ao parsear teams: {}", e)))?;

        log_info(&format!("✅ [OAuth2] {} workspaces autorizados", teams_response.teams.len()));

        for team in &teams_response.teams {
            log_info(&format!("  ├─ {} (ID: {})", team.name, team.id));
        }

        Ok(teams_response.teams)
    }

    /// Validar se um access token é válido
    ///
    /// # Parâmetros
    /// - `access_token`: Token a ser validado
    ///
    /// # Retorno
    /// - `true`: Token válido
    /// - `false`: Token inválido ou expirado
    pub async fn validate_token(&self, access_token: &str) -> bool {
        match self.get_authorized_teams(access_token).await {
            Ok(_) => {
                log_info("✅ [OAuth2] Token validado com sucesso");
                true
            }
            Err(e) => {
                log_warning(&format!("⚠️ [OAuth2] Token inválido: {}", e));
                false
            }
        }
    }

    /// Validar token com análise detalhada de erros OAuth
    ///
    /// # Parâmetros
    /// - `access_token`: Token a ser validado
    ///
    /// # Retorno
    /// - `TokenValidationResult`: Resultado detalhado da validação com códigos de erro
    pub async fn validate_token_detailed(&self, access_token: &str) -> TokenValidationResult {
        log_info("🔍 [OAuth2] Validando token com análise detalhada...");

        let url = "https://api.clickup.com/api/v2/user";

        let response = self.http_client
            .get(url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status_code = resp.status().as_u16();

                if resp.status().is_success() {
                    log_info("✅ [OAuth2] Token válido confirmado");
                    return TokenValidationResult {
                        is_valid: true,
                        error_code: None,
                        status_code: Some(status_code),
                        should_invalidate_cache: false,
                        should_reauthorize: false,
                    };
                }

                // Tentar extrair o corpo da resposta para análise de erro
                let error_body = resp.text().await.unwrap_or_default();
                let error_code = self.extract_oauth_error_code(&error_body);

                log_warning(&format!("❌ [OAuth2] Token inválido - Status: {}, Body: {}", status_code, error_body));

                match error_code.as_deref() {
                    Some("OAUTH_025") => {
                        // Token inválido/expirado - invalidar cache e reautorizar
                        log_warning("🚨 [OAuth2] Token inválido detectado (OAUTH_025). Cache será invalidado.");
                        TokenValidationResult {
                            is_valid: false,
                            error_code: Some("OAUTH_025".to_string()),
                            status_code: Some(status_code),
                            should_invalidate_cache: true,
                            should_reauthorize: true,
                        }
                    }
                    Some("OAUTH_027") => {
                        // Team não autorizado - não invalidar cache, mas reautorizar
                        log_warning("🚨 [OAuth2] Team não autorizado detectado (OAUTH_027). Reautorização necessária.");
                        TokenValidationResult {
                            is_valid: false,
                            error_code: Some("OAUTH_027".to_string()),
                            status_code: Some(status_code),
                            should_invalidate_cache: false,
                            should_reauthorize: true,
                        }
                    }
                    Some("OAUTH_019") => {
                        // Erro de autorização genérico
                        log_warning("🚨 [OAuth2] Erro de autorização detectado (OAUTH_019).");
                        TokenValidationResult {
                            is_valid: false,
                            error_code: Some("OAUTH_019".to_string()),
                            status_code: Some(status_code),
                            should_invalidate_cache: true,
                            should_reauthorize: true,
                        }
                    }
                    _ => {
                        // Erro desconhecido - comportamento conservador
                        log_warning(&format!("⚠️ [OAuth2] Erro OAuth desconhecido. Status: {}, Body: {}", status_code, error_body));
                        TokenValidationResult {
                            is_valid: false,
                            error_code,
                            status_code: Some(status_code),
                            should_invalidate_cache: status_code == 401 || status_code == 403,
                            should_reauthorize: status_code == 401 || status_code == 403,
                        }
                    }
                }
            }
            Err(e) => {
                log_error(&format!("❌ [OAuth2] Erro de rede ao validar token: {}", e));
                TokenValidationResult {
                    is_valid: false,
                    error_code: Some("NETWORK_ERROR".to_string()),
                    status_code: None,
                    should_invalidate_cache: false,
                    should_reauthorize: false,
                }
            }
        }
    }

    /// Extrai código de erro OAuth do corpo da resposta
    ///
    /// # Parâmetros
    /// - `body`: Corpo da resposta HTTP
    ///
    /// # Retorno
    /// - `Some(String)`: Código de erro encontrado
    /// - `None`: Nenhum código de erro identificado
    fn extract_oauth_error_code(&self, body: &str) -> Option<String> {
        // Tentar parsing como JSON primeiro
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
            // Padrões comuns da API ClickUp
            if let Some(error_code) = json_value.get("ECODE").and_then(|v| v.as_str()) {
                return Some(error_code.to_string());
            }
            if let Some(error_code) = json_value.get("error").and_then(|v| v.get("code")).and_then(|v| v.as_str()) {
                return Some(error_code.to_string());
            }
            if let Some(error_code) = json_value.get("err").and_then(|v| v.as_str()) {
                return Some(error_code.to_string());
            }
        }

        // Fallback: busca por regex se JSON parsing falhar
        let oauth_patterns = [
            "OAUTH_025",
            "OAUTH_027",
            "OAUTH_019",
            "Invalid token",
            "Token expired",
        ];

        for pattern in &oauth_patterns {
            if body.contains(pattern) {
                return Some(pattern.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_client_creation() {
        let config = OAuth2Config {
            client_id: "test_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "https://example.com/callback".to_string(),
            token_secret_name: "test-secret".to_string(),
        };

        let client = OAuth2Client::new(config);
        assert_eq!(client.config.client_id, "test_id");
    }
}
