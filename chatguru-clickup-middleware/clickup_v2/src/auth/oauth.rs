use std::collections::HashMap;
use std::time::Duration;
use oauth2::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    AuthUrl, TokenUrl, basic::BasicClient,
    reqwest::async_http_client, TokenResponse,
};
use crate::config::EnvManager;
use crate::auth::callback::CallbackServer;
use crate::auth::token::TokenManager;
use crate::client::api::ClickUpClient;
use crate::error::{AuthError, AuthResult};

/// Gerenciador do fluxo OAuth2 do ClickUp
#[derive(Debug)]
pub struct OAuthFlow {
    env_manager: EnvManager,
    token_manager: TokenManager,
}

impl OAuthFlow {
    /// Cria uma nova instância do fluxo OAuth
    pub fn new() -> AuthResult<Self> {
        let env_manager = EnvManager::load()?;
        env_manager.validate()?;
        
        let token_manager = TokenManager::new();
        
        Ok(Self {
            env_manager,
            token_manager,
        })
    }

    /// Executa o fluxo de autenticação completo
    pub async fn authenticate(&self) -> AuthResult<String> {
        log::info!("🔑 Iniciando processo de autenticação OAuth2...");
        log::info!("📍 {}", self.env_manager.environment_info());

        // 1. Verifica se já existe um token válido
        if let Some(token) = self.token_manager.get_token() {
            log::info!("🔍 Token encontrado, validando...");
            
            if self.validate_token(&token).await? {
                log::info!("✅ Token válido! Autenticação concluída.");
                return Ok(token);
            } else {
                log::warn!("❌ Token inválido, iniciando novo fluxo OAuth2...");
                self.token_manager.remove_token()?;
            }
        } else {
            log::info!("🆕 Nenhum token encontrado, iniciando fluxo OAuth2...");
        }

        // 2. Inicia o fluxo OAuth2
        let token = self.execute_oauth_flow().await?;
        
        // 3. Valida o token obtido
        if self.validate_token(&token).await? {
            // 4. Salva o token
            self.token_manager.save_token(&token)?;
            log::info!("✅ Autenticação OAuth2 concluída com sucesso!");
            Ok(token)
        } else {
            Err(AuthError::oauth_error("Token obtido é inválido"))
        }
    }

    /// Executa o fluxo OAuth2 completo
    async fn execute_oauth_flow(&self) -> AuthResult<String> {
        match self.env_manager.environment {
            crate::config::env::Environment::Development => {
                self.execute_local_oauth_flow().await
            },
            crate::config::env::Environment::Production => {
                self.execute_production_oauth_flow().await
            }
        }
    }

    /// Executa o fluxo OAuth2 para ambiente de desenvolvimento (local)
    async fn execute_local_oauth_flow(&self) -> AuthResult<String> {
        log::info!("🏠 Executando fluxo OAuth2 para desenvolvimento local...");

        // 1. Configura o cliente OAuth2
        let (auth_url, token_url) = EnvManager::get_oauth_urls();
        let client = self.create_oauth_client(&auth_url, &token_url)?;

        // 2. Gera URL de autorização
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            .url();

        log::info!("🌐 URL de autorização gerada: {}", auth_url);

        // 3. Inicia o servidor callback local
        let state = csrf_token.secret().clone();
        let callback_server = CallbackServer::new(self.env_manager.callback_port, state.clone());

        // 4. Inicia o servidor e aguarda o resultado
        let server_handle = tokio::spawn(async move {
            callback_server.start_and_wait().await
        });

        // 5. Abre o navegador
        if let Err(e) = webbrowser::open(auth_url.as_str()) {
            log::warn!("⚠️ Não foi possível abrir o navegador automaticamente: {}", e);
            log::info!("🔗 Abra manualmente o link: {}", auth_url);
        } else {
            log::info!("🌐 Navegador aberto automaticamente");
        }

        log::info!("⏳ Aguardando autorização do usuário...");

        // 6. Aguarda o callback
        let callback_result = tokio::time::timeout(
            Duration::from_secs(300), // 5 minutos
            server_handle
        )
        .await
        .map_err(|_| AuthError::Timeout)?
        .map_err(|e| AuthError::generic(format!("Erro na thread do servidor: {}", e)))?
        .map_err(|e| AuthError::generic(format!("Erro no callback: {}", e)))?;

        // 7. O CSRF token já foi validado pelo CallbackServer
        let code = callback_result.code;

        log::info!("✅ Código de autorização recebido");

        // 9. Troca código por token
        self.exchange_code_for_token(client, code).await
    }

    /// Executa o fluxo OAuth2 para ambiente de produção
    async fn execute_production_oauth_flow(&self) -> AuthResult<String> {
        log::info!("☁️ Executando fluxo OAuth2 para produção...");

        // Em produção, assumimos que o token será fornecido via variável de ambiente
        // ou através de um processo externo (como secrets manager)
        
        // 1. Verifica se há um token na variável de ambiente
        if let Some(token) = std::env::var("CLICKUP_ACCESS_TOKEN").ok().filter(|t| !t.is_empty()) {
            log::info!("🔑 Token encontrado nas variáveis de ambiente");
            return Ok(token);
        }

        // 2. Para produção, você pode implementar diferentes estratégias:
        // - Integração com Google Secret Manager
        // - Endpoint para receber token via API
        // - Interface web para autorização
        
        // Por enquanto, retorna erro solicitando configuração manual
        Err(AuthError::config_error(
            "Em ambiente de produção, configure CLICKUP_ACCESS_TOKEN nas variáveis de ambiente. \
            Para obter o token, execute o fluxo OAuth2 em ambiente de desenvolvimento primeiro."
        ))
    }

    /// Cria o cliente OAuth2
    fn create_oauth_client(&self, auth_url: &str, token_url: &str) -> AuthResult<BasicClient> {
        let client_id = ClientId::new(self.env_manager.client_id.clone());
        let client_secret = ClientSecret::new(self.env_manager.client_secret.clone());
        let auth_url = AuthUrl::new(auth_url.to_string())
            .map_err(|e| AuthError::oauth_error(&format!("URL de autorização inválida: {}", e)))?;
        let token_url = TokenUrl::new(token_url.to_string())
            .map_err(|e| AuthError::oauth_error(&format!("URL de token inválida: {}", e)))?;

        let redirect_url = RedirectUrl::new(self.env_manager.get_callback_url())
            .map_err(|e| AuthError::oauth_error(&format!("URL de redirecionamento inválida: {}", e)))?;

        Ok(BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url,
            Some(token_url),
        ).set_redirect_uri(redirect_url))
    }

    /// Troca o código de autorização por um token de acesso
    async fn exchange_code_for_token(&self, client: BasicClient, code: String) -> AuthResult<String> {
        log::info!("🔄 Trocando código de autorização por token...");

        let auth_code = AuthorizationCode::new(code);

        let token_response = client
            .exchange_code(auth_code)
            .request_async(async_http_client)
            .await
            .map_err(|e| AuthError::oauth_error(&format!("Falha ao trocar código por token: {}", e)))?;

        let access_token = token_response.access_token().secret().clone();
        
        log::info!("✅ Token de acesso obtido com sucesso");
        Ok(access_token)
    }

    /// Valida se o token de acesso é válido fazendo uma requisição de teste
    async fn validate_token(&self, token: &str) -> AuthResult<bool> {
        log::info!("🔍 Validando token de acesso...");

        let client = ClickUpClient::new(token.to_string(), self.env_manager.api_base_url.clone());
        
        match client.get_authorized_user().await {
            Ok(_) => {
                log::info!("✅ Token validado com sucesso");
                Ok(true)
            },
            Err(e) => {
                log::warn!("❌ Token inválido: {}", e);
                Ok(false)
            }
        }
    }

    /// Força uma nova autenticação removendo o token atual
    pub async fn force_reauth(&self) -> AuthResult<String> {
        log::info!("🔄 Forçando nova autenticação...");
        
        // Remove o token atual
        self.token_manager.remove_token()?;
        
        // Executa nova autenticação
        self.authenticate().await
    }

    /// Verifica se o usuário está autenticado
    pub async fn is_authenticated(&self) -> bool {
        if let Some(token) = self.token_manager.get_token() {
            self.validate_token(&token).await.unwrap_or(false)
        } else {
            false
        }
    }

    /// Obtém informações do usuário autenticado
    pub async fn get_user_info(&self) -> AuthResult<serde_json::Value> {
        let token = self.authenticate().await?;
        let client = ClickUpClient::new(token, self.env_manager.api_base_url.clone());
        client.get_authorized_user().await
    }

    /// Obtém as equipes autorizadas
    pub async fn get_authorized_teams(&self) -> AuthResult<serde_json::Value> {
        let token = self.authenticate().await?;
        let client = ClickUpClient::new(token, self.env_manager.api_base_url.clone());
        client.get_authorized_teams().await
    }

    /// Revoga o token de acesso
    pub async fn revoke_token(&self) -> AuthResult<()> {
        if let Some(_token) = self.token_manager.get_token() {
            log::info!("🗑️ Revogando token de acesso...");

            // O ClickUp não possui endpoint público de revogação
            // Apenas removemos o token localmente
            self.token_manager.remove_token()?;

            log::info!("✅ Token removido localmente");
        }

        Ok(())
    }

    /// Obtém o token atual sem validação
    pub fn get_current_token(&self) -> Option<String> {
        self.token_manager.get_token()
    }

    /// Define um token manualmente (útil para testes)
    pub fn set_token(&self, token: &str) -> AuthResult<()> {
        self.token_manager.save_token(token)
    }

    /// Retorna informações sobre a configuração atual
    pub fn get_config_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("environment".to_string(), self.env_manager.environment_info());
        info.insert("api_base_url".to_string(), self.env_manager.api_base_url.clone());
        info.insert("callback_url".to_string(), self.env_manager.get_callback_url());
        info.insert("has_token".to_string(), self.token_manager.get_token().is_some().to_string());
        
        let details = self.env_manager.get_environment_details();
        info.insert("environment_details".to_string(), details);
        
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[tokio::test]
    async fn test_oauth_flow_creation() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
            ],
            || {
                let result = OAuthFlow::new();
                assert!(result.is_ok());

                let oauth = result.unwrap();
                assert_eq!(oauth.env_manager.client_id, "test_client_id");
                assert_eq!(oauth.env_manager.client_secret, "test_client_secret");
            },
        );
    }

    #[tokio::test]
    async fn test_oauth_flow_creation_missing_env() {
        temp_env::with_vars_unset(vec!["CLICKUP_CLIENT_ID", "CLICKUP_CLIENT_SECRET"], || {
            let result = OAuthFlow::new();
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_config_info() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
            ],
            || {
                let oauth = OAuthFlow::new().unwrap();
                let info = oauth.get_config_info();

                assert!(info.contains_key("environment"));
                assert!(info.contains_key("has_token"));
                assert!(info.contains_key("api_base_url"));
                assert!(info.contains_key("callback_url"));
                assert_eq!(info.get("has_token").unwrap(), "false");
            },
        );
    }

    #[tokio::test]
    async fn test_is_authenticated_without_token() {
        // Set env vars directly for async test
        std::env::set_var("CLICKUP_CLIENT_ID", "test_client_id");
        std::env::set_var("CLICKUP_CLIENT_SECRET", "test_client_secret");
        std::env::remove_var("CLICKUP_ACCESS_TOKEN");

        let oauth = OAuthFlow::new().unwrap();
        assert!(!oauth.is_authenticated().await);

        // Cleanup
        std::env::remove_var("CLICKUP_CLIENT_ID");
        std::env::remove_var("CLICKUP_CLIENT_SECRET");
    }

    #[test]
    fn test_get_current_token_none() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
                ("CLICKUP_ACCESS_TOKEN", None),
            ],
            || {
                let oauth = OAuthFlow::new().unwrap();
                assert!(oauth.get_current_token().is_none());
            },
        );
    }

    #[test]
    fn test_set_and_get_token() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
            ],
            || {
                let oauth = OAuthFlow::new().unwrap();

                // Define um token
                let test_token = "test_access_token_12345";
                let result = oauth.set_token(test_token);
                assert!(result.is_ok());

                // Verifica se o token foi salvo
                let current_token = oauth.get_current_token();
                assert!(current_token.is_some());
                assert_eq!(current_token.unwrap(), test_token);
            },
        );
    }

    #[tokio::test]
    async fn test_revoke_token() {
        // Set env vars directly for async test
        std::env::set_var("CLICKUP_CLIENT_ID", "test_client_id");
        std::env::set_var("CLICKUP_CLIENT_SECRET", "test_client_secret");

        let oauth = OAuthFlow::new().unwrap();

        // Define um token
        oauth.set_token("test_token_to_revoke").unwrap();
        assert!(oauth.get_current_token().is_some());

        // Revoga o token
        let result = oauth.revoke_token().await;
        assert!(result.is_ok());

        // Verifica se o token foi removido
        assert!(oauth.get_current_token().is_none());

        // Cleanup
        std::env::remove_var("CLICKUP_CLIENT_ID");
        std::env::remove_var("CLICKUP_CLIENT_SECRET");
        std::env::remove_var("CLICKUP_ACCESS_TOKEN");
    }

    #[test]
    fn test_create_oauth_client() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
            ],
            || {
                let oauth = OAuthFlow::new().unwrap();
                let (auth_url, token_url) = EnvManager::get_oauth_urls();

                let client = oauth.create_oauth_client(&auth_url, &token_url);
                assert!(client.is_ok());
            },
        );
    }

    #[test]
    fn test_create_oauth_client_invalid_urls() {
        temp_env::with_vars(
            vec![
                ("CLICKUP_CLIENT_ID", Some("test_client_id")),
                ("CLICKUP_CLIENT_SECRET", Some("test_client_secret")),
            ],
            || {
                let oauth = OAuthFlow::new().unwrap();

                // Testa com URL inválida
                let client = oauth.create_oauth_client("not-a-url", "also-not-a-url");
                assert!(client.is_err());
            },
        );
    }

    #[tokio::test]
    async fn test_production_oauth_flow_with_env_token() {
        // Set env vars directly for async test
        std::env::set_var("CLICKUP_CLIENT_ID", "test_client_id");
        std::env::set_var("CLICKUP_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("PRODUCTION", "true");
        std::env::set_var("CLICKUP_ACCESS_TOKEN", "production_token_123");

        let oauth = OAuthFlow::new().unwrap();
        let result = oauth.execute_production_oauth_flow().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "production_token_123");

        // Cleanup
        std::env::remove_var("CLICKUP_CLIENT_ID");
        std::env::remove_var("CLICKUP_CLIENT_SECRET");
        std::env::remove_var("PRODUCTION");
        std::env::remove_var("CLICKUP_ACCESS_TOKEN");
    }

    #[tokio::test]
    async fn test_production_oauth_flow_without_token() {
        // Set env vars directly for async test
        std::env::set_var("CLICKUP_CLIENT_ID", "test_client_id");
        std::env::set_var("CLICKUP_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("PRODUCTION", "true");
        std::env::remove_var("CLICKUP_ACCESS_TOKEN");

        let oauth = OAuthFlow::new().unwrap();
        let result = oauth.execute_production_oauth_flow().await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("configure CLICKUP_ACCESS_TOKEN"));

        // Cleanup
        std::env::remove_var("CLICKUP_CLIENT_ID");
        std::env::remove_var("CLICKUP_CLIENT_SECRET");
        std::env::remove_var("PRODUCTION");
    }
}