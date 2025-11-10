use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use warp::Filter;
use crate::error::{AuthError, AuthResult};

/// Servidor HTTP local para capturar o callback do OAuth2
pub struct CallbackServer {
    port: u16,
    state: String,
}

/// Resultado do callback OAuth2
#[derive(Debug)]
pub struct CallbackResult {
    pub code: String,
    pub state: String,
}

impl CallbackServer {
    /// Cria um novo servidor de callback
    pub fn new(port: u16, state: String) -> Self {
        Self { port, state }
    }

    /// Inicia o servidor e aguarda o callback
    pub async fn start_and_wait(self) -> AuthResult<CallbackResult> {
        let expected_state = self.state.clone();
        let (tx, rx) = oneshot::channel::<AuthResult<CallbackResult>>();
        let tx = Arc::new(Mutex::new(Some(tx)));

        // Rota para o callback OAuth2
        let callback_route = warp::path("callback")
            .and(warp::query::<HashMap<String, String>>())
            .and_then({
                let tx = tx.clone();
                let expected_state = expected_state.clone();
                move |params: HashMap<String, String>| {
                    let tx = tx.clone();
                    let expected_state = expected_state.clone();
                    
                    async move {
                        log::info!("Recebido callback OAuth2: {:?}", params);

                        let result = Self::process_callback(params, &expected_state);
                        let is_success = result.is_ok();

                        // Envia o resultado pelo channel
                        if let Ok(mut sender) = tx.lock() {
                            if let Some(tx) = sender.take() {
                                let _ = tx.send(result);
                            }
                        }

                        // Retorna uma página de resposta
                        let html_response = if is_success {
                            warp::reply::html(SUCCESS_PAGE)
                        } else {
                            warp::reply::html(ERROR_PAGE)
                        };
                        Ok::<_, warp::Rejection>(html_response)
                    }
                }
            });

        // Rota para servir uma página de status
        let status_route = warp::path::end()
            .map(|| warp::reply::html(WAITING_PAGE));

        // Combina as rotas
        let routes = callback_route
            .or(status_route)
            .with(warp::filters::log::log("callback_server"));

        // Inicia o servidor em uma task separada
        let addr = ([127, 0, 0, 1], self.port);

        let (actual_addr, server_future) = warp::serve(routes)
            .try_bind_ephemeral(addr)
            .map_err(|e| AuthError::callback_error(format!("Failed to bind server: {}", e)))?;

        log::info!("Servidor de callback iniciado em: http://{}", actual_addr);

        let server_task = tokio::spawn(server_future);

        // Aguarda o resultado do callback com timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minutos de timeout
            rx
        ).await;

        // Cancela o servidor
        server_task.abort();

        match result {
            Ok(Ok(callback_result)) => callback_result,
            Ok(Err(_)) => Err(AuthError::CallbackServerError("Canal de comunicação fechado".to_string())),
            Err(_) => Err(AuthError::Timeout),
        }
    }

    /// Processa os parâmetros do callback
    fn process_callback(
        params: HashMap<String, String>, 
        expected_state: &str
    ) -> AuthResult<CallbackResult> {
        // Verifica se há erro
        if let Some(error) = params.get("error") {
            match error.as_str() {
                "access_denied" => return Err(AuthError::AccessDenied),
                _ => return Err(AuthError::Generic(format!("Erro OAuth2: {}", error))),
            }
        }

        // Verifica o código de autorização
        let code = params.get("code")
            .ok_or_else(|| AuthError::InvalidCode("Código não encontrado no callback".to_string()))?;

        // Verifica o estado (proteção CSRF)
        let received_state = params.get("state")
            .ok_or_else(|| AuthError::InvalidState)?;

        if received_state != expected_state {
            return Err(AuthError::InvalidState);
        }

        log::info!("Callback OAuth2 processado com sucesso");

        Ok(CallbackResult {
            code: code.clone(),
            state: received_state.clone(),
        })
    }

    /// Gera um estado aleatório para proteção CSRF
    pub fn generate_state() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        // Simples geração de estado baseada em timestamp
        // Em produção, use uma biblioteca de geração de números aleatórios mais robusta
        format!("state_{}", timestamp)
    }

    /// Constrói a URL de redirecionamento
    pub fn build_redirect_url(port: u16) -> String {
        format!("http://localhost:{}/callback", port)
    }
}

// Páginas HTML para o servidor de callback
const WAITING_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ClickUp OAuth2 - Aguardando Autorização</title>
    <meta charset="UTF-8">
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: #f5f5f5; 
            text-align: center;
        }
        .container { 
            max-width: 600px; 
            margin: 50px auto; 
            background: white; 
            padding: 30px; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #7b68ee; }
        .spinner {
            border: 4px solid #f3f3f3;
            border-top: 4px solid #7b68ee;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🔐 ClickUp OAuth2</h1>
        <div class="spinner"></div>
        <h2>Aguardando autorização...</h2>
        <p>Por favor, complete o processo de autorização no ClickUp.</p>
        <p>Esta página será atualizada automaticamente quando a autorização for concluída.</p>
    </div>
</body>
</html>
"#;

const SUCCESS_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ClickUp OAuth2 - Autorização Concluída</title>
    <meta charset="UTF-8">
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: #f5f5f5; 
            text-align: center;
        }
        .container { 
            max-width: 600px; 
            margin: 50px auto; 
            background: white; 
            padding: 30px; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #28a745; }
        .success-icon {
            font-size: 64px;
            color: #28a745;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="success-icon">✅</div>
        <h1>Autorização Concluída!</h1>
        <p>Autorização do ClickUp realizada com sucesso!</p>
        <p>Você pode fechar esta janela e retornar à aplicação.</p>
        <p>O token de acesso foi salvo automaticamente.</p>
    </div>
    <script>
        setTimeout(() => {
            window.close();
        }, 3000);
    </script>
</body>
</html>
"#;

const ERROR_PAGE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ClickUp OAuth2 - Erro na Autorização</title>
    <meta charset="UTF-8">
    <style>
        body { 
            font-family: Arial, sans-serif; 
            margin: 0; 
            padding: 20px; 
            background: #f5f5f5; 
            text-align: center;
        }
        .container { 
            max-width: 600px; 
            margin: 50px auto; 
            background: white; 
            padding: 30px; 
            border-radius: 8px; 
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 { color: #dc3545; }
        .error-icon {
            font-size: 64px;
            color: #dc3545;
            margin: 20px 0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="error-icon">❌</div>
        <h1>Erro na Autorização</h1>
        <p>Ocorreu um erro durante o processo de autorização do ClickUp.</p>
        <p>Verifique a configuração e tente novamente.</p>
        <p>Você pode fechar esta janela.</p>
    </div>
</body>
</html>
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_state() {
        let state1 = CallbackServer::generate_state();
        let state2 = CallbackServer::generate_state();
        
        assert!(state1.starts_with("state_"));
        assert!(state2.starts_with("state_"));
        assert_ne!(state1, state2); // Estados devem ser diferentes
    }

    #[test]
    fn test_build_redirect_url() {
        let url = CallbackServer::build_redirect_url(8888);
        assert_eq!(url, "http://localhost:8888/callback");
    }

    #[test]
    fn test_process_callback_success() {
        let mut params = HashMap::new();
        params.insert("code".to_string(), "test_code".to_string());
        params.insert("state".to_string(), "test_state".to_string());

        let result = CallbackServer::process_callback(params, "test_state");
        assert!(result.is_ok());
        
        let callback_result = result.unwrap();
        assert_eq!(callback_result.code, "test_code");
        assert_eq!(callback_result.state, "test_state");
    }

    #[test]
    fn test_process_callback_invalid_state() {
        let mut params = HashMap::new();
        params.insert("code".to_string(), "test_code".to_string());
        params.insert("state".to_string(), "wrong_state".to_string());

        let result = CallbackServer::process_callback(params, "test_state");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidState));
    }

    #[test]
    fn test_process_callback_access_denied() {
        let mut params = HashMap::new();
        params.insert("error".to_string(), "access_denied".to_string());

        let result = CallbackServer::process_callback(params, "test_state");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::AccessDenied));
    }
}