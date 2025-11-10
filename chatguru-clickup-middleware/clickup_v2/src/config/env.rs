use dotenv::dotenv;
use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use crate::error::{AuthError, AuthResult};

/// Configuração do ambiente de execução
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,  // Usa arquivo .env
    Production,   // Usa variáveis de ambiente do sistema/secrets
}

/// Gerenciador de variáveis de ambiente para OAuth2 do ClickUp
#[derive(Debug, Clone)]
pub struct EnvManager {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub api_base_url: String,
    pub callback_port: u16,
    pub environment: Environment,
}

impl EnvManager {
    /// Carrega as configurações detectando automaticamente o ambiente
    pub fn load() -> AuthResult<Self> {
        let environment = Self::detect_environment();

        match environment {
            Environment::Development => {
                // Carrega do arquivo .env apenas se não estiver em modo de teste
                // Durante testes, os testes devem configurar as variáveis diretamente
                if cfg!(not(test)) && Path::new(".env").exists() {
                    dotenv().map_err(|e| AuthError::config_error(&format!("Erro ao carregar .env: {}", e)))?;
                }
            },
            Environment::Production => {
                // Em produção, assume que as variáveis já estão definidas no ambiente
                log::info!("Ambiente de produção detectado, usando variáveis de ambiente do sistema");
            }
        }

        let client_id = Self::get_env_var("CLICKUP_CLIENT_ID")?;
        let client_secret = Self::get_env_var("CLICKUP_CLIENT_SECRET")?;
        
        // Para produção, usa uma das URLs configuradas
        let redirect_uri = match environment {
            Environment::Development => {
                env::var("CLICKUP_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:8888/callback".to_string())
            },
            Environment::Production => {
                // Prioriza variável específica, senão usa uma das URLs configuradas
                env::var("CLICKUP_REDIRECT_URI").unwrap_or_else(|_| {
                    // Usa a primeira URL disponível como padrão
                    "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback".to_string()
                })
            }
        };

        let api_base_url = env::var("CLICKUP_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.clickup.com/api/v2".to_string());
            
        let callback_port = env::var("CALLBACK_PORT")
            .unwrap_or_else(|_| match environment {
                Environment::Development => "8888".to_string(),
                Environment::Production => env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            })
            .parse()
            .unwrap_or(match environment {
                Environment::Development => 8888,
                Environment::Production => 8080,
            });

        Ok(Self {
            client_id,
            client_secret,
            redirect_uri,
            api_base_url,
            callback_port,
            environment,
        })
    }

    /// Detecta o ambiente de execução
    fn detect_environment() -> Environment {
        // Verifica indicadores de produção
        if env::var("GOOGLE_CLOUD_PROJECT").is_ok() 
            || env::var("K_SERVICE").is_ok()  // Cloud Run
            || env::var("GAE_APPLICATION").is_ok()  // App Engine
            || env::var("PRODUCTION").is_ok() 
        {
            Environment::Production
        } else {
            Environment::Development
        }
    }

    /// Obtém variável de ambiente obrigatória
    fn get_env_var(key: &str) -> AuthResult<String> {
        env::var(key).map_err(|_| AuthError::env_error(&format!("{} não encontrado", key)))
    }

    /// Obtém o token de acesso
    pub fn get_access_token() -> Option<String> {
        // Tenta carregar .env apenas em desenvolvimento e não em testes
        if cfg!(not(test)) && Self::detect_environment() == Environment::Development {
            dotenv().ok();
        }
        env::var("CLICKUP_ACCESS_TOKEN").ok().filter(|token| !token.is_empty())
    }

    /// Salva o token de acesso
    pub fn save_access_token(token: &str) -> AuthResult<()> {
        let environment = Self::detect_environment();

        // Durante testes, sempre usa variáveis de ambiente em vez de arquivo
        if cfg!(test) {
            env::set_var("CLICKUP_ACCESS_TOKEN", token);
            return Ok(());
        }

        match environment {
            Environment::Development => {
                // Em desenvolvimento, salva no arquivo .env
                Self::update_env_file("CLICKUP_ACCESS_TOKEN", token)
            },
            Environment::Production => {
                // Em produção, define na variável de ambiente do processo
                // Nota: Isso só afeta o processo atual, não persiste
                env::set_var("CLICKUP_ACCESS_TOKEN", token);
                log::info!("Token salvo na variável de ambiente (sessão atual)");
                Ok(())
            }
        }
    }

    /// Remove o token de acesso
    pub fn remove_access_token() -> AuthResult<()> {
        let environment = Self::detect_environment();

        // Durante testes, sempre usa variáveis de ambiente em vez de arquivo
        if cfg!(test) {
            env::remove_var("CLICKUP_ACCESS_TOKEN");
            return Ok(());
        }

        match environment {
            Environment::Development => {
                // Em desenvolvimento, remove do arquivo .env
                Self::update_env_file("CLICKUP_ACCESS_TOKEN", "")
            },
            Environment::Production => {
                // Em produção, remove da variável de ambiente do processo
                env::remove_var("CLICKUP_ACCESS_TOKEN");
                log::info!("Token removido da variável de ambiente");
                Ok(())
            }
        }
    }

    /// Atualiza uma variável no arquivo .env (apenas para desenvolvimento)
    fn update_env_file(key: &str, value: &str) -> AuthResult<()> {
        let env_path = ".env";
        
        // Lê o conteúdo atual do arquivo .env
        let mut lines = Vec::new();
        let mut key_found = false;

        if let Ok(file) = std::fs::File::open(env_path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.starts_with(&format!("{}=", key)) {
                    if !value.is_empty() {
                        lines.push(format!("{}={}", key, value));
                    }
                    key_found = true;
                } else {
                    lines.push(line);
                }
            }
        }

        // Se não encontrou a linha do token e o valor não está vazio, adiciona uma nova
        if !key_found && !value.is_empty() {
            lines.push(format!("{}={}", key, value));
        }

        // Escreve o arquivo atualizado
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(env_path)?;

        for line in lines {
            writeln!(file, "{}", line)?;
        }

        log::info!("Variável {} atualizada no arquivo .env", key);
        Ok(())
    }

    /// Valida se todas as configurações obrigatórias estão presentes
    pub fn validate(&self) -> AuthResult<()> {
        if self.client_id.is_empty() {
            return Err(AuthError::config_error("CLICKUP_CLIENT_ID é obrigatório"));
        }

        if self.client_secret.is_empty() {
            return Err(AuthError::config_error("CLICKUP_CLIENT_SECRET é obrigatório"));
        }

        if self.redirect_uri.is_empty() {
            return Err(AuthError::config_error("CLICKUP_REDIRECT_URI é obrigatório"));
        }

        // Valida formato da redirect URI
        if !self.redirect_uri.starts_with("http://") && !self.redirect_uri.starts_with("https://") {
            return Err(AuthError::config_error("CLICKUP_REDIRECT_URI deve ser uma URL válida"));
        }

        // Valida se a redirect URI está na lista de URLs permitidas
        if !Self::is_valid_redirect_url(&self.redirect_uri) {
            return Err(AuthError::config_error(&format!(
                "CLICKUP_REDIRECT_URI não está na lista de URLs permitidas: {}", 
                self.redirect_uri
            )));
        }

        Ok(())
    }

    /// Obtém a URL base da API do ClickUp
    pub fn get_api_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.api_base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'))
    }

    /// Obtém as URLs do OAuth2 do ClickUp
    pub fn get_oauth_urls() -> (String, String) {
        let auth_url = "https://app.clickup.com/api".to_string();
        let token_url = "https://api.clickup.com/api/v2/oauth/token".to_string();
        (auth_url, token_url)
    }

    /// Retorna todas as URLs de redirecionamento válidas
    pub fn get_valid_redirect_urls() -> Vec<String> {
        vec![
            "http://localhost:8888/callback".to_string(),
            "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback".to_string(),
            "https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/callback".to_string(),
            "https://voila-422100.rj.r.appspot.com/callback".to_string(),
        ]
    }

    /// Verifica se uma URL de redirecionamento é válida
    pub fn is_valid_redirect_url(url: &str) -> bool {
        Self::get_valid_redirect_urls().contains(&url.to_string())
    }

    /// Retorna informações sobre o ambiente
    pub fn environment_info(&self) -> String {
        match self.environment {
            Environment::Development => {
                format!("Desenvolvimento (porta: {}, redirect: {})", 
                    self.callback_port, self.redirect_uri)
            },
            Environment::Production => {
                format!("Produção (porta: {}, redirect: {})", 
                    self.callback_port, self.redirect_uri)
            }
        }
    }

    /// Cria um arquivo .env padrão se não existir (apenas em desenvolvimento)
    pub fn create_env_file_if_not_exists() -> AuthResult<()> {
        if Self::detect_environment() == Environment::Development && !std::path::Path::new(".env").exists() {
            let default_content = format!(r#"# ClickUp OAuth2 Credentials
CLICKUP_CLIENT_ID=your_client_id_here
CLICKUP_CLIENT_SECRET=your_client_secret_here
CLICKUP_REDIRECT_URI=http://localhost:8888/callback

# Token (será preenchido automaticamente)
CLICKUP_ACCESS_TOKEN=

# Configurações opcionais
CLICKUP_API_BASE_URL=https://api.clickup.com/api/v2
CALLBACK_PORT=8888

# URLs de redirecionamento válidas (para referência):
# - http://localhost:8888/callback (desenvolvimento)
# - https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback
# - https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/callback  
# - https://voila-422100.rj.r.appspot.com/callback
"#);

            std::fs::write(".env", default_content)?;
            log::info!("Arquivo .env criado com configurações padrão");
        }

        Ok(())
    }

    /// Obtém a URL de callback baseada no ambiente
    pub fn get_callback_url(&self) -> String {
        match self.environment {
            Environment::Development => {
                format!("http://localhost:{}/callback", self.callback_port)
            },
            Environment::Production => {
                self.redirect_uri.clone()
            }
        }
    }

    /// Verifica se está rodando em ambiente Cloud Run
    pub fn is_cloud_run() -> bool {
        env::var("K_SERVICE").is_ok()
    }

    /// Verifica se está rodando em ambiente App Engine
    pub fn is_app_engine() -> bool {
        env::var("GAE_APPLICATION").is_ok()
    }

    /// Retorna informações detalhadas do ambiente
    pub fn get_environment_details(&self) -> String {
        let mut details = vec![];
        
        details.push(format!("Ambiente: {:?}", self.environment));
        details.push(format!("API Base: {}", self.api_base_url));
        details.push(format!("Callback Port: {}", self.callback_port));
        details.push(format!("Redirect URI: {}", self.redirect_uri));
        
        if Self::is_cloud_run() {
            details.push("Plataforma: Google Cloud Run".to_string());
        } else if Self::is_app_engine() {
            details.push("Plataforma: Google App Engine".to_string());
        }
        
        if let Ok(project) = env::var("GOOGLE_CLOUD_PROJECT") {
            details.push(format!("Projeto GCP: {}", project));
        }
        
        details.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_environment_detection() {
        // Testa detecção de desenvolvimento (padrão sem variáveis)
        temp_env::with_vars_unset(vec!["GOOGLE_CLOUD_PROJECT", "K_SERVICE", "GAE_APPLICATION", "PRODUCTION"], || {
            assert_eq!(EnvManager::detect_environment(), Environment::Development);
        });

        // Testa detecção de produção com Cloud Run
        temp_env::with_var("K_SERVICE", Some("test-service"), || {
            assert_eq!(EnvManager::detect_environment(), Environment::Production);
        });

        // Testa detecção de produção com App Engine
        temp_env::with_var("GAE_APPLICATION", Some("test-app"), || {
            assert_eq!(EnvManager::detect_environment(), Environment::Production);
        });

        // Testa detecção de produção com variável PRODUCTION
        temp_env::with_var("PRODUCTION", Some("true"), || {
            assert_eq!(EnvManager::detect_environment(), Environment::Production);
        });
    }

    #[test]
    fn test_valid_redirect_urls() {
        let urls = EnvManager::get_valid_redirect_urls();
        assert_eq!(urls.len(), 4);
        assert!(urls.contains(&"http://localhost:8888/callback".to_string()));
        assert!(urls.contains(&"https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback".to_string()));

        // Testa validação
        assert!(EnvManager::is_valid_redirect_url("http://localhost:8888/callback"));
        assert!(EnvManager::is_valid_redirect_url("https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback"));
        assert!(!EnvManager::is_valid_redirect_url("https://malicious-site.com/callback"));
        assert!(!EnvManager::is_valid_redirect_url(""));
    }

    #[test]
    fn test_callback_url_generation() {
        let env_dev = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert_eq!(env_dev.get_callback_url(), "http://localhost:8888/callback");

        let env_prod = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8080,
            environment: Environment::Production,
        };

        assert_eq!(env_prod.get_callback_url(), "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/callback");
    }

    #[test]
    fn test_api_url_generation() {
        let env_manager = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert_eq!(env_manager.get_api_url("user"), "https://api.clickup.com/api/v2/user");
        assert_eq!(env_manager.get_api_url("/user"), "https://api.clickup.com/api/v2/user");

        // Testa com URL base com trailing slash
        let env_manager_trailing = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2/".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert_eq!(env_manager_trailing.get_api_url("user"), "https://api.clickup.com/api/v2/user");
    }

    #[test]
    fn test_validate_configuration() {
        // Testa configuração válida
        let valid_config = EnvManager {
            client_id: "valid_id".to_string(),
            client_secret: "valid_secret".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(valid_config.validate().is_ok());

        // Testa configuração com client_id vazio
        let invalid_client_id = EnvManager {
            client_id: "".to_string(),
            client_secret: "valid_secret".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(invalid_client_id.validate().is_err());

        // Testa configuração com client_secret vazio
        let invalid_client_secret = EnvManager {
            client_id: "valid_id".to_string(),
            client_secret: "".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(invalid_client_secret.validate().is_err());

        // Testa configuração com redirect_uri inválido
        let invalid_redirect = EnvManager {
            client_id: "valid_id".to_string(),
            client_secret: "valid_secret".to_string(),
            redirect_uri: "invalid-url".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(invalid_redirect.validate().is_err());

        // Testa configuração com redirect_uri não permitido
        let unauthorized_redirect = EnvManager {
            client_id: "valid_id".to_string(),
            client_secret: "valid_secret".to_string(),
            redirect_uri: "https://malicious.com/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(unauthorized_redirect.validate().is_err());
    }

    #[test]
    fn test_oauth_urls() {
        let (auth_url, token_url) = EnvManager::get_oauth_urls();
        assert_eq!(auth_url, "https://app.clickup.com/api");
        assert_eq!(token_url, "https://api.clickup.com/api/v2/oauth/token");
    }

    #[test]
    fn test_environment_info() {
        let env_dev = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        assert!(env_dev.environment_info().contains("Desenvolvimento"));
        assert!(env_dev.environment_info().contains("8888"));

        let env_prod = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "https://app.com/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8080,
            environment: Environment::Production,
        };

        assert!(env_prod.environment_info().contains("Produção"));
        assert!(env_prod.environment_info().contains("8080"));
    }

    #[test]
    fn test_cloud_run_detection() {
        temp_env::with_var_unset("K_SERVICE", || {
            assert!(!EnvManager::is_cloud_run());
        });

        temp_env::with_var("K_SERVICE", Some("test-service"), || {
            assert!(EnvManager::is_cloud_run());
        });
    }

    #[test]
    fn test_app_engine_detection() {
        temp_env::with_var_unset("GAE_APPLICATION", || {
            assert!(!EnvManager::is_app_engine());
        });

        temp_env::with_var("GAE_APPLICATION", Some("test-app"), || {
            assert!(EnvManager::is_app_engine());
        });
    }

    #[test]
    fn test_get_access_token() {
        // Testa sem token definido
        temp_env::with_var_unset("CLICKUP_ACCESS_TOKEN", || {
            assert!(EnvManager::get_access_token().is_none());
        });

        // Testa com token definido
        temp_env::with_var("CLICKUP_ACCESS_TOKEN", Some("test_token_123"), || {
            assert_eq!(EnvManager::get_access_token(), Some("test_token_123".to_string()));
        });

        // Testa com token vazio (deve retornar None)
        temp_env::with_var("CLICKUP_ACCESS_TOKEN", Some(""), || {
            assert!(EnvManager::get_access_token().is_none());
        });
    }

    #[test]
    fn test_save_and_remove_access_token_in_production() {
        // Simula ambiente de produção
        temp_env::with_var("PRODUCTION", Some("true"), || {
            // Salva token
            let result = EnvManager::save_access_token("prod_token_123");
            assert!(result.is_ok());
            assert_eq!(env::var("CLICKUP_ACCESS_TOKEN").ok(), Some("prod_token_123".to_string()));

            // Remove token
            let result = EnvManager::remove_access_token();
            assert!(result.is_ok());
            assert!(env::var("CLICKUP_ACCESS_TOKEN").is_err());
        });
    }

    #[test]
    fn test_get_environment_details() {
        let env_manager = EnvManager {
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            redirect_uri: "http://localhost:8888/callback".to_string(),
            api_base_url: "https://api.clickup.com/api/v2".to_string(),
            callback_port: 8888,
            environment: Environment::Development,
        };

        let details = env_manager.get_environment_details();
        assert!(details.contains("Ambiente: Development"));
        assert!(details.contains("API Base: https://api.clickup.com/api/v2"));
        assert!(details.contains("Callback Port: 8888"));
        assert!(details.contains("Redirect URI: http://localhost:8888/callback"));

        // Testa com Cloud Run
        temp_env::with_var("K_SERVICE", Some("test-service"), || {
            temp_env::with_var("GOOGLE_CLOUD_PROJECT", Some("test-project"), || {
                let details = env_manager.get_environment_details();
                assert!(details.contains("Plataforma: Google Cloud Run"));
                assert!(details.contains("Projeto GCP: test-project"));
            });
        });
    }
}