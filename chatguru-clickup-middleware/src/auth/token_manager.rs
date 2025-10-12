//! Token Manager
//!
//! Gerenciamento de tokens OAuth2: armazenamento, validação, e fornecimento

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::services::secrets::SecretManagerService;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use super::{OAuth2Config, OAuth2Client};

/// Cache de token em memória
#[derive(Debug, Clone)]
struct TokenCache {
    token: Option<String>,
    validated_at: Option<std::time::Instant>,
    ttl: std::time::Duration,
}

impl TokenCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            token: None,
            validated_at: None,
            ttl: std::time::Duration::from_secs(ttl_seconds),
        }
    }

    fn is_valid(&self) -> bool {
        if let (Some(_), Some(validated_at)) = (&self.token, self.validated_at) {
            validated_at.elapsed() < self.ttl
        } else {
            false
        }
    }

    fn set(&mut self, token: String) {
        self.token = Some(token);
        self.validated_at = Some(std::time::Instant::now());
    }

    fn clear(&mut self) {
        self.token = None;
        self.validated_at = None;
    }
}

/// Gerenciador de tokens OAuth2
pub struct TokenManager {
    config: OAuth2Config,
    oauth_client: OAuth2Client,
    secret_manager: SecretManagerService,
    cache: Arc<RwLock<TokenCache>>,
}

impl TokenManager {
    /// Criar novo gerenciador de tokens
    pub async fn new(
        config: OAuth2Config,
    ) -> Result<Self, String> {
        let oauth_client = OAuth2Client::new(config.clone());
        let secret_manager = SecretManagerService::new()
            .await
            .map_err(|e| format!("Failed to initialize Secret Manager: {}", e))?;

        Ok(Self {
            config,
            oauth_client,
            secret_manager,
            cache: Arc::new(RwLock::new(TokenCache::new(3600))), // 1 hora de cache
        })
    }

    /// Obter token válido (cache → Secret Manager → erro)
    ///
    /// # Retorno
    /// - `Ok(String)`: Token OAuth2 válido
    /// - `Err(AppError)`: Nenhum token válido disponível
    pub async fn get_valid_token(&self) -> AppResult<String> {
        // 1. Verificar cache em memória
        {
            let cache = self.cache.read().await;
            if cache.is_valid() {
                if let Some(token) = &cache.token {
                    log_info("✅ [TokenManager] Token obtido do cache em memória");
                    return Ok(token.clone());
                }
            }
        }

        log_info("🔄 [TokenManager] Cache expirado, consultando Secret Manager...");

        // 2. Buscar no Secret Manager
        let token = self.secret_manager
            .get_secret_value(&self.config.token_secret_name)
            .await
            .map_err(|e| {
                log_error(&format!("❌ [TokenManager] Token não encontrado no Secret Manager: {}", e));
                AppError::ConfigError("OAuth2 token não configurado. Execute /auth/clickup para autorizar.".to_string())
            })?;

        // 3. Validar token com análise detalhada
        let validation_result = self.oauth_client.validate_token_detailed(&token).await;

        if !validation_result.is_valid {
            log_error(&format!("❌ [TokenManager] Token inválido - Código: {:?}, Status: {:?}",
                validation_result.error_code, validation_result.status_code));

            // Invalidar cache se necessário
            if validation_result.should_invalidate_cache {
                log_info("🗑️ [TokenManager] Invalidando cache devido ao erro OAuth");
                self.cache.write().await.clear();
            }

            // Tratamento específico por código de erro
            let error_message = match validation_result.error_code.as_deref() {
                Some("OAUTH_025") => {
                    log_error("🚨 [TokenManager] Token expirado detectado (OAUTH_025). Reautorização necessária.");
                    "OAuth2 token expirado. Execute /auth/clickup para renovar a autorização."
                },
                Some("OAUTH_027") => {
                    log_error("🚨 [TokenManager] Team não autorizado (OAUTH_027). Verifique permissões.");
                    "Team não autorizado no ClickUp. Verifique as permissões do workspace."
                },
                Some("OAUTH_019") => {
                    log_error("🚨 [TokenManager] Erro de autorização genérico (OAUTH_019).");
                    "Erro de autorização no ClickUp. Execute /auth/clickup para re-autorizar."
                },
                Some("NETWORK_ERROR") => {
                    log_error("🌐 [TokenManager] Erro de rede ao validar token.");
                    "Erro de conectividade. Tente novamente em alguns momentos."
                },
                _ => {
                    log_error("❓ [TokenManager] Erro OAuth desconhecido.");
                    "OAuth2 token inválido ou expirado. Execute /auth/clickup para re-autorizar."
                }
            };

            return Err(AppError::ConfigError(error_message.to_string()));
        }

        // 4. Atualizar cache
        self.cache.write().await.set(token.clone());

        log_info("✅ [TokenManager] Token validado e atualizado no cache");

        Ok(token)
    }

    /// Salvar novo token no Secret Manager
    ///
    /// # Parâmetros
    /// - `token`: Novo access token OAuth2
    ///
    /// # Retorno
    /// - `Ok(())`: Token salvo com sucesso
    /// - `Err(AppError)`: Erro ao salvar token
    pub async fn save_token(&self, token: &str) -> AppResult<()> {
        log_info("💾 [TokenManager] Salvando novo token no Secret Manager...");

        // 1. Validar token com análise detalhada antes de salvar
        let validation_result = self.oauth_client.validate_token_detailed(token).await;

        if !validation_result.is_valid {
            log_error(&format!("❌ [TokenManager] Token inválido, não será salvo - Código: {:?}",
                validation_result.error_code));
            
            let error_message = match validation_result.error_code.as_deref() {
                Some("OAUTH_025") => "Token expirado ou inválido",
                Some("OAUTH_027") => "Team não autorizado para este token",
                Some("OAUTH_019") => "Erro de autorização OAuth",
                Some("NETWORK_ERROR") => "Erro de conectividade ao validar token",
                _ => "Token fornecido é inválido"
            };
            
            return Err(AppError::ConfigError(error_message.to_string()));
        }

        // 2. Salvar no Secret Manager
        self.secret_manager
            .create_or_update_secret(&self.config.token_secret_name, token)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to save token: {}", e)))?;

        // 3. Atualizar cache
        self.cache.write().await.set(token.to_string());

        log_info("✅ [TokenManager] Token salvo e validado com sucesso");

        Ok(())
    }

    /// Limpar cache de token (força revalidação)
    pub async fn invalidate_cache(&self) {
        log_info("🗑️  [TokenManager] Cache de token invalidado");
        self.cache.write().await.clear();
    }

    /// Verificar workspaces autorizados
    pub async fn get_authorized_workspaces(&self) -> AppResult<Vec<(String, String)>> {
        let token = self.get_valid_token().await?;

        let teams = self.oauth_client.get_authorized_teams(&token).await?;

        Ok(teams.into_iter().map(|t| (t.id, t.name)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_cache_expiration() {
        let mut cache = TokenCache::new(1); // 1 segundo
        assert!(!cache.is_valid());

        cache.set("test_token".to_string());
        assert!(cache.is_valid());

        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(!cache.is_valid());
    }
}
