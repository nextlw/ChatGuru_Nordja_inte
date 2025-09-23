use anyhow::Result;
use std::env;

// Estrutura simplificada por enquanto - vamos fazer funcionar primeiro
pub struct SecretManagerService {
    project_id: String,
}

impl SecretManagerService {
    pub async fn new() -> Result<Self> {
        // Tenta obter o project_id do ambiente
        let project_id = Self::get_project_id().await?;
        
        tracing::info!("Secret Manager Service inicializado para projeto: {}", project_id);

        Ok(Self {
            project_id,
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

        // Fallback para o projeto conhecido
        Ok("chatguru-integration".to_string())
    }

    pub async fn get_clickup_api_token(&self) -> Result<String> {
        // Por enquanto, apenas usa variáveis de ambiente
        // Implementação completa do Secret Manager será adicionada depois
        if let Ok(token) = env::var("CLICKUP_API_TOKEN") {
            tracing::debug!("Usando CLICKUP_API_TOKEN da variável de ambiente");
            return Ok(token);
        }

        // TODO: Implementar chamada real ao Secret Manager
        tracing::warn!(
            "Secret Manager ainda não implementado completamente. \
            Por favor, configure CLICKUP_API_TOKEN como variável de ambiente."
        );
        
        Err(anyhow::anyhow!(
            "CLICKUP_API_TOKEN não encontrado no ambiente"
        ))
    }

    pub async fn get_clickup_list_id(&self) -> Result<String> {
        // Por enquanto, apenas usa variáveis de ambiente
        if let Ok(list_id) = env::var("CLICKUP_LIST_ID") {
            tracing::debug!("Usando CLICKUP_LIST_ID da variável de ambiente");
            return Ok(list_id);
        }

        // TODO: Implementar chamada real ao Secret Manager
        tracing::info!(
            "CLICKUP_LIST_ID não encontrado no ambiente. \
            Usando valor padrão: 901300373349"
        );

        // Fallback para o valor padrão usado anteriormente
        Ok("901300373349".to_string())
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