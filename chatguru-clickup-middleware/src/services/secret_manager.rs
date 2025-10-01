use anyhow::Result;
use std::env;
use google_cloud_secretmanager_v1::client::SecretManagerService as GcpSecretClient;

pub struct SecretManagerService {
    client: Option<GcpSecretClient>,
    project_id: String,
}

impl SecretManagerService {
    pub async fn new() -> Result<Self> {
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
            let secret_name = format!(
                "projects/{}/secrets/clickup-api-token/versions/latest",
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
                            Ok(token) => {
                                tracing::info!("ClickUp API token recuperado do Secret Manager");
                                return Ok(token);
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
        if let Ok(token) = env::var("CLICKUP_API_TOKEN") {
            tracing::debug!("Usando CLICKUP_API_TOKEN da variável de ambiente");
            return Ok(token);
        }

        Err(anyhow::anyhow!(
            "CLICKUP_API_TOKEN não encontrado no Secret Manager nem no ambiente"
        ))
    }

    pub async fn get_clickup_list_id(&self) -> Result<String> {
        // Tenta buscar do Secret Manager primeiro (se disponível)
        if let Some(client) = &self.client {
            let secret_name = format!(
                "projects/{}/secrets/clickup-list-id/versions/latest",
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
                            Ok(list_id) => {
                                tracing::info!("ClickUp List ID recuperado do Secret Manager");
                                return Ok(list_id);
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
        if let Ok(list_id) = env::var("CLICKUP_LIST_ID") {
            tracing::debug!("Usando CLICKUP_LIST_ID da variável de ambiente");
            return Ok(list_id);
        }

        // Fallback final: valor padrão
        tracing::info!("Usando valor padrão para CLICKUP_LIST_ID: 901300373349");
        Ok("901300373349".to_string())
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
        // Primeiro limpa qualquer variável residual
        env::remove_var("CLICKUP_API_TOKEN");
        env::remove_var("CLICKUP_LIST_ID");
        
        // Configura as variáveis de teste
        env::set_var("CLICKUP_API_TOKEN", "test-token-from-env");
        env::set_var("CLICKUP_LIST_ID", "test-list-from-env");

        let service = SecretManagerService::new().await.unwrap();
        
        let token = service.get_clickup_api_token().await.unwrap();
        assert_eq!(token, "test-token-from-env");
        
        let list_id = service.get_clickup_list_id().await.unwrap();
        assert_eq!(list_id, "test-list-from-env");

        // Limpa as variáveis após o teste
        env::remove_var("CLICKUP_API_TOKEN");
        env::remove_var("CLICKUP_LIST_ID");
    }

    #[tokio::test] 
    async fn test_list_id_fallback() {
        // Garante que não há variável de ambiente residual
        env::remove_var("CLICKUP_API_TOKEN");
        env::remove_var("CLICKUP_LIST_ID");
        
        let service = SecretManagerService::new().await.unwrap();
        let list_id = service.get_clickup_list_id().await.unwrap();
        
        assert_eq!(list_id, "901300373349");
    }
}