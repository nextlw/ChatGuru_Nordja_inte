use anyhow::Result;
use std::env;
use google_cloud_secretmanager_v1::client::SecretManagerService as GcpSecretClient;
use crate::config::settings::Settings;
use crate::utils::error::SecretsError;

pub struct SecretManagerService {
    client: Option<GcpSecretClient>,
    project_id: String,
    settings: Settings,
}

impl SecretManagerService {
    pub async fn new() -> Result<Self> {
        // Carrega as configurações
        let settings = Settings::new().map_err(|e| anyhow::anyhow!("Falha ao carregar configurações: {}", e))?;
        
        // Tenta obter o project_id do ambiente
        let project_id = Self::get_project_id().await?;

        // Tenta criar o cliente do Secret Manager (apenas em produção)
        let client = if cfg!(debug_assertions) {
            tracing::debug!("Modo desenvolvimento: usando variáveis de ambiente");
            None
        } else {
            match GcpSecretClient::builder().build().await {
                Ok(client) => {
                    tracing::info!("Secret Manager client inicializado com sucesso");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("Falha ao criar Secret Manager client: {}. Usando fallback para variáveis de ambiente.", e);
                    None
                }
            }
        };

        tracing::info!("Secret Manager Service inicializado para projeto: {}", project_id);

        Ok(Self {
            project_id,
            client,
            settings,
        })
    }

    async fn get_project_id() -> Result<String> {
        // Primeiro tenta do ambiente
        if let Ok(project_id) = env::var("GCP_PROJECT_ID") {
            return Ok(project_id);
        }
        
        if let Ok(project_id) = env::var("GOOGLE_CLOUD_PROJECT") {
            return Ok(project_id);
        }

        // Fallback para o projeto buzzlightear
        Ok("buzzlightear".to_string())
    }

    pub async fn get_clickup_api_token(&self) -> Result<String> {
        // Tenta buscar do Secret Manager primeiro (se disponível)
        if let Some(client) = &self.client {
            // PRIORIDADE 1: OAuth2 token (mais permissões, pode criar folders/spaces)
            let oauth_secret_name = format!(
                "projects/{}/secrets/clickup-oauth-token/versions/latest",
                self.project_id
            );

            match client
                .access_secret_version()
                .set_name(oauth_secret_name)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(payload) = response.payload {
                        match String::from_utf8(payload.data.to_vec()) {
                            Ok(token) => {
                                tracing::info!("✅ ClickUp OAuth2 token recuperado do Secret Manager");
                                return Ok(token);
                            }
                            Err(e) => {
                                tracing::error!("Erro ao decodificar OAuth2 token: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("OAuth2 token não encontrado ({}), tentando Personal Token...", e);
                }
            }

            // PRIORIDADE 2: Personal Token (fallback)
            let api_secret_name = format!(
                "projects/{}/secrets/clickup-api-token/versions/latest",
                self.project_id
            );

            match client
                .access_secret_version()
                .set_name(api_secret_name)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(payload) = response.payload {
                        match String::from_utf8(payload.data.to_vec()) {
                            Ok(token) => {
                                tracing::info!("ClickUp Personal Token recuperado do Secret Manager");
                                return Ok(token);
                            }
                            Err(e) => {
                                tracing::error!("Erro ao decodificar Personal Token: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Personal Token não encontrado no Secret Manager: {}", e);
                }
            }
        }

        // PRIORIDADE 3: variável de ambiente
        if let Ok(token) = env::var("clickup_api_token") {
            tracing::debug!("Usando clickup_api_token da variável de ambiente");
            return Ok(token);
        }

        Err(anyhow::anyhow!(
            "clickup_api_token não encontrado no Secret Manager (OAuth2 ou Personal) nem no ambiente"
        ))
    }

    /// Get ClickUp List ID with fallback hierarchy:
    /// 1. Secret Manager value (if available)
    /// 2. Environment variable CLICKUP_LIST_ID
    /// 3. Default from settings
    pub async fn get_clickup_list_id(&self) -> Result<String, SecretsError> {
        // First try Secret Manager (highest priority)
        if let Some(ref _client) = self.client {
            if let Ok(secret_value) = self.get_secret("clickup-list-id").await {
                if !secret_value.trim().is_empty() {
                    return Ok(secret_value);
                }
            }
        }

        // Then try environment variable
        if let Ok(list_id) = env::var("CLICKUP_LIST_ID") {
            if !list_id.trim().is_empty() {
                return Ok(list_id);
            }
        }

        // Finally, fallback to default
        Ok(self.settings.clickup.list_id.clone())
    }

    /// Public method to get a secret value from Secret Manager
    pub async fn get_secret_value(&self, secret_name: &str) -> Result<String, SecretsError> {
        self.get_secret(secret_name).await
    }

    /// Create or update a secret in Secret Manager
    pub async fn create_or_update_secret(&self, secret_name: &str, _value: &str) -> Result<(), SecretsError> {
        if let Some(_client) = &self.client {
            // Para simplificar, vamos apenas adicionar uma nova versão ao secret existente
            // Se o secret não existir, retorna erro
            tracing::info!("Atualizando secret '{}' no Secret Manager", secret_name);

            // Por enquanto, retorna sucesso (implementação completa requer google_cloud_secretmanager_v1::client methods)
            // A implementação real exigiria:
            // 1. Tentar adicionar versão (add_secret_version)
            // 2. Se falhar com NOT_FOUND, criar o secret primeiro
            tracing::warn!("create_or_update_secret ainda não implementado completamente. Use gcloud CLI para atualizar manualmente.");
            Ok(())
        } else {
            Err(SecretsError::ClientNotAvailable)
        }
    }

    /// Helper method to get a secret from Secret Manager
    async fn get_secret(&self, secret_name: &str) -> Result<String, SecretsError> {
        if let Some(client) = &self.client {
            let full_secret_name = format!(
                "projects/{}/secrets/{}/versions/latest",
                self.project_id, secret_name
            );

            match client
                .access_secret_version()
                .set_name(full_secret_name)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(payload) = response.payload {
                        match String::from_utf8(payload.data.to_vec()) {
                            Ok(value) => {
                                tracing::info!("Secret '{}' recuperado do Secret Manager", secret_name);
                                return Ok(value);
                            }
                            Err(e) => {
                                return Err(SecretsError::DecodingError(e.to_string()));
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(SecretsError::AccessError(e.to_string()));
                }
            }
        }
        
        Err(SecretsError::ClientNotAvailable)
    }

    pub async fn get_openai_api_key(&self) -> Result<String> {
        // Tenta buscar do Secret Manager primeiro (se disponível)
        if let Some(client) = &self.client {
            let secret_name = format!(
                "projects/{}/secrets/openai-api-key/versions/latest",
                self.project_id
            );

            match client
                .access_secret_version()
                .set_name(secret_name)
                .send()
                .await
            {
                Ok(response) => {
                    if let Some(payload) = response.payload {
                        match String::from_utf8(payload.data.to_vec()) {
                            Ok(api_key) => {
                                tracing::info!("OpenAI API key recuperada do Secret Manager");
                                return Ok(api_key);
                            }
                            Err(e) => {
                                tracing::error!("Erro ao decodificar secret do Secret Manager: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Falha ao acessar secret no Secret Manager: {}. Tentando variável de ambiente.", e);
                }
            }
        }

        // Fallback: variável de ambiente
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            tracing::debug!("Usando OPENAI_API_KEY da variável de ambiente");
            return Ok(api_key);
        }

        Err(anyhow::anyhow!(
            "OPENAI_API_KEY não encontrado no Secret Manager nem no ambiente"
        ))
    }
}

// Teste básico
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_manager_creation() {
        // Este teste só verifica se consegue criar a instância
        let service = SecretManagerService::new().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_env_var_priority() {
        // Salva o estado atual das variáveis
        let original_token = env::var("clickup_api_token").ok();
        let original_list_id = env::var("CLICKUP_LIST_ID").ok();

        // Remove variáveis para garantir teste limpo
        env::remove_var("clickup_api_token");
        env::remove_var("CLICKUP_LIST_ID");

        // Configura as variáveis de teste
        env::set_var("clickup_api_token", "test-token-from-env");
        env::set_var("CLICKUP_LIST_ID", "test-list-from-env");

        // Cria service sem Secret Manager (client = None) para forçar uso de env vars
        let settings = crate::config::settings::Settings::new().unwrap();
        let service = SecretManagerService {
            client: None, // Força não usar Secret Manager nos testes
            project_id: settings.gcp.project_id.clone(),
            settings,
        };

        // Em ambiente de teste sem Secret Manager, deve usar a env var como fallback.
        let token = service.get_clickup_api_token().await.unwrap();
        assert_eq!(token, "test-token-from-env");

        let list_id = service.get_clickup_list_id().await.unwrap();
        assert_eq!(list_id, "test-list-from-env");

        // Restaura o estado original das variáveis
        env::remove_var("clickup_api_token");
        env::remove_var("CLICKUP_LIST_ID");

        if let Some(token) = original_token {
            env::set_var("clickup_api_token", token);
        }

        if let Some(list_id) = original_list_id {
            env::set_var("CLICKUP_LIST_ID", list_id);
        }
    }

    #[tokio::test]
    async fn test_list_id_fallback() {
        // Salva o estado atual das variáveis
        let original_list_id = env::var("CLICKUP_LIST_ID").ok();
        
        // Garante que não há variável de ambiente (deve usar valor padrão)
        env::remove_var("CLICKUP_LIST_ID");
        
        let service = SecretManagerService::new().await.unwrap();
        let list_id = service.get_clickup_list_id().await.unwrap();
        
        // Deve retornar o valor padrão das configurações
        assert_eq!(list_id, service.settings.clickup.list_id);
        
        // Restaura a variável se existia
        if let Some(list_id) = original_list_id {
            env::set_var("CLICKUP_LIST_ID", list_id);
        }
    }
}