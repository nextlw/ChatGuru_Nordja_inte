use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::EnvManager;
use crate::error::{AuthError, AuthResult};

/// Estrutura para representar um token de acesso OAuth2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub created_at: u64,
}

impl AccessToken {
    /// Cria um novo token de acesso
    pub fn new(access_token: String, token_type: String, expires_in: Option<u64>) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            access_token,
            token_type,
            expires_in,
            created_at,
        }
    }

    /// Verifica se o token está expirado
    pub fn is_expired(&self) -> bool {
        if let Some(expires_in) = self.expires_in {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Adiciona uma margem de 5 minutos para renovação antecipada
            let expiry_time = self.created_at + expires_in - 300;
            current_time >= expiry_time
        } else {
            // Se não tem expiração definida, considera como não expirado
            false
        }
    }

    /// Retorna o tempo restante em segundos até a expiração
    pub fn time_to_expiry(&self) -> Option<u64> {
        if let Some(expires_in) = self.expires_in {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let expiry_time = self.created_at + expires_in;
            if current_time < expiry_time {
                Some(expiry_time - current_time)
            } else {
                Some(0)
            }
        } else {
            None
        }
    }

    /// Retorna o token no formato de autorização para requisições HTTP
    pub fn authorization_header(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

/// Resposta da API de token do ClickUp
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

impl From<TokenResponse> for AccessToken {
    fn from(response: TokenResponse) -> Self {
        Self::new(
            response.access_token,
            response.token_type,
            response.expires_in,
        )
    }
}

/// Gerenciador de tokens OAuth2
#[derive(Debug, Clone)]
pub struct TokenManager;

impl TokenManager {
    /// Cria uma nova instância do TokenManager
    pub fn new() -> Self {
        Self
    }

    /// Carrega o token do .env se existir e for válido
    pub fn get_token(&self) -> Option<String> {
        EnvManager::get_access_token()
    }

    /// Salva o token no .env
    pub fn save_token(&self, token: &str) -> AuthResult<()> {
        EnvManager::save_access_token(token)
    }

    /// Remove o token do .env
    pub fn remove_token(&self) -> AuthResult<()> {
        EnvManager::remove_access_token()
    }

    /// Carrega o token como AccessToken do .env se existir e for válido
    pub fn load_token() -> Option<AccessToken> {
        if let Some(token_str) = EnvManager::get_access_token() {
            // Para tokens simples, cria um AccessToken básico
            Some(AccessToken::new(
                token_str,
                "Bearer".to_string(),
                None, // ClickUp não retorna expires_in
            ))
        } else {
            None
        }
    }

    /// Salva o AccessToken no .env
    pub fn save_access_token(token: &AccessToken) -> AuthResult<()> {
        EnvManager::save_access_token(&token.access_token)
    }

    /// Remove o token do .env (método estático)
    pub fn remove_access_token() -> AuthResult<()> {
        EnvManager::remove_access_token()
    }

    /// Valida se um token é válido fazendo uma requisição de teste
    pub async fn validate_token(token: &AccessToken) -> AuthResult<bool> {
        let client = reqwest::Client::new();
        let env_config = EnvManager::load()?;
        
        let response = client
            .get(&env_config.get_api_url("team"))
            .header("Authorization", token.authorization_header())
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(true),
            reqwest::StatusCode::UNAUTHORIZED => Ok(false),
            status => {
                log::warn!("Resposta inesperada ao validar token: {}", status);
                Ok(false)
            }
        }
    }

    /// Troca código de autorização por token de acesso
    pub async fn exchange_code_for_token(
        code: &str,
        env_config: &EnvManager,
    ) -> AuthResult<AccessToken> {
        let client = reqwest::Client::new();
        let (_, token_url) = EnvManager::get_oauth_urls();

        let params = [
            ("client_id", env_config.client_id.as_str()),
            ("client_secret", env_config.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", env_config.redirect_uri.as_str()),
        ];

        let response = client
            .post(&token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AuthError::InvalidCode(format!(
                "Falha ao trocar código por token: {}", 
                error_text
            )));
        }

        let token_response: TokenResponse = response.json().await?;
        Ok(token_response.into())
    }

    /// Obtém ou renova um token válido
    pub async fn get_valid_token(force_refresh: bool) -> AuthResult<AccessToken> {
        // Tenta carregar token existente se não for para forçar renovação
        if !force_refresh {
            if let Some(token) = Self::load_token() {
                // Verifica se o token não está expirado
                if !token.is_expired() {
                    // Valida o token fazendo uma requisição
                    match Self::validate_token(&token).await {
                        Ok(true) => return Ok(token),
                        Ok(false) => {
                            log::info!("Token inválido, iniciando novo fluxo OAuth2");
                            Self::remove_access_token()?;
                        }
                        Err(e) => {
                            log::warn!("Erro ao validar token: {}", e);
                            Self::remove_access_token()?;
                        }
                    }
                } else {
                    log::info!("Token expirado, iniciando novo fluxo OAuth2");
                    Self::remove_access_token()?;
                }
            }
        } else {
            log::info!("Forçando renovação do token");
            Self::remove_access_token()?;
        }

        // Se chegou aqui, precisa de um novo token
        Err(AuthError::TokenExpired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_token_creation() {
        let token = AccessToken::new(
            "test_token".to_string(),
            "Bearer".to_string(),
            Some(3600),
        );

        assert_eq!(token.access_token, "test_token");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.expires_in, Some(3600));
        assert!(!token.is_expired()); // Recém criado não deve estar expirado
    }

    #[test]
    fn test_authorization_header() {
        let token = AccessToken::new(
            "test_token".to_string(),
            "Bearer".to_string(),
            None,
        );

        assert_eq!(token.authorization_header(), "Bearer test_token");
    }

    #[test]
    fn test_time_to_expiry() {
        let token = AccessToken::new(
            "test_token".to_string(),
            "Bearer".to_string(),
            Some(3600),
        );

        let time_left = token.time_to_expiry().unwrap();
        assert!(time_left <= 3600);
        assert!(time_left > 3500); // Deve estar próximo a 3600
    }
}