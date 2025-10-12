//! OAuth2 Configuration
//!
//! Centraliza todas as configurações necessárias para OAuth2 do ClickUp

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID fornecido pelo ClickUp
    pub client_id: String,

    /// Client Secret fornecido pelo ClickUp
    pub client_secret: String,

    /// URL de callback registrada no ClickUp App
    pub redirect_uri: String,

    /// Nome do secret no Google Secret Manager para armazenar o token
    pub token_secret_name: String,
}

impl OAuth2Config {
    /// Criar configuração a partir de variáveis de ambiente
    pub fn from_env() -> Result<Self, String> {
        let client_id = std::env::var("CLICKUP_CLIENT_ID")
            .map_err(|_| "CLICKUP_CLIENT_ID não configurado".to_string())?;

        let client_secret = std::env::var("CLICKUP_CLIENT_SECRET")
            .map_err(|_| "CLICKUP_CLIENT_SECRET não configurado".to_string())?;

        let redirect_uri = std::env::var("CLICKUP_REDIRECT_URI")
            .unwrap_or_else(|_| {
                "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/auth/clickup/callback"
                    .to_string()
            });

        let token_secret_name = std::env::var("CLICKUP_OAUTH_SECRET_NAME")
            .unwrap_or_else(|_| "clickup-oauth-token".to_string());

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            token_secret_name,
        })
    }

    /// Gerar URL de autorização do ClickUp
    pub fn authorization_url(&self) -> String {
        format!(
            "https://app.clickup.com/api?client_id={}&redirect_uri={}",
            self.client_id,
            urlencoding::encode(&self.redirect_uri)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_url() {
        let config = OAuth2Config {
            client_id: "test_client_id".to_string(),
            client_secret: "test_secret".to_string(),
            redirect_uri: "https://example.com/callback".to_string(),
            token_secret_name: "test-secret".to_string(),
        };

        let url = config.authorization_url();
        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
    }
}
