//! OAuth2 HTTP Handlers
//!
//! Endpoints HTTP para iniciar e completar o fluxo OAuth2

use axum::{
    extract::{Query, State},
    response::{Html, Redirect},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::utils::logging::*;
use super::{OAuth2Config, OAuth2Client, TokenManager};

/// Parâmetros do callback OAuth2
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackParams {
    /// Authorization code retornado pelo ClickUp
    code: Option<String>,
    /// Erro retornado pelo ClickUp (se houver)
    error: Option<String>,
}

/// State compartilhado para os handlers OAuth2
pub struct OAuth2State {
    pub config: OAuth2Config,
    pub token_manager: Arc<TokenManager>,
}

/// GET /auth/clickup
///
/// Inicia o fluxo OAuth2 redirecionando o usuário para a página de autorização do ClickUp
///
/// # Retorno
/// - `Ok(Redirect)`: Redireciona para ClickUp
/// - `Err`: Erro na configuração
pub async fn start_oauth_flow(
    State(oauth_state): State<Arc<OAuth2State>>,
) -> Result<Redirect, (StatusCode, String)> {
    log_info("🚀 [OAuth2] Iniciando fluxo de autorização...");

    let auth_url = oauth_state.config.authorization_url();

    log_info(&format!("↗️  [OAuth2] Redirecionando para: {}", auth_url));

    Ok(Redirect::to(&auth_url))
}

/// GET /auth/clickup/callback?code=XXX
///
/// Recebe o callback OAuth2 do ClickUp e troca o code por access token
///
/// # Parâmetros
/// - `code`: Authorization code (sucesso)
/// - `error`: Erro (falha na autorização)
///
/// # Retorno
/// - `Ok(Html)`: Página de sucesso ou erro
/// - `Err`: Erro interno
pub async fn handle_oauth_callback(
    State(oauth_state): State<Arc<OAuth2State>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<Html<String>, (StatusCode, String)> {
    log_info("📥 [OAuth2] Callback recebido");

    // Verificar se houve erro na autorização
    if let Some(error) = params.error {
        log_error(&format!("❌ [OAuth2] Erro na autorização: {}", error));
        return Ok(render_error_page(&error));
    }

    // Obter authorization code
    let code = params.code.ok_or_else(|| {
        log_error("❌ [OAuth2] Code não recebido no callback");
        (StatusCode::BAD_REQUEST, "Missing code parameter".to_string())
    })?;

    use chatguru_clickup_middleware::utils::truncate_safe;
    log_info(&format!("🔑 [OAuth2] Code recebido: {}...", truncate_safe(&code, 10)));

    // Trocar code por access token
    let oauth_client = OAuth2Client::new(oauth_state.config.clone());

    let token_response = oauth_client
        .exchange_code_for_token(&code)
        .await
        .map_err(|e| {
            log_error(&format!("❌ [OAuth2] Falha ao obter token: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to exchange code: {}", e))
        })?;

    let access_token = token_response.access_token;

    log_info(&format!("✅ [OAuth2] Token obtido: {}...", truncate_safe(&access_token, 20)));

    // Verificar workspaces autorizados
    let teams = oauth_client
        .get_authorized_teams(&access_token)
        .await
        .map_err(|e| {
            log_warning(&format!("⚠️  [OAuth2] Não foi possível verificar teams: {}", e));
            e
        })
        .ok();

    // Salvar token no Secret Manager
    match oauth_state.token_manager.save_token(&access_token).await {
        Ok(_) => {
            log_info("✅ [OAuth2] Token salvo no Secret Manager com sucesso");
            Ok(render_success_page(&access_token, teams.as_deref()))
        }
        Err(e) => {
            log_error(&format!("❌ [OAuth2] Erro ao salvar token: {}", e));
            // Mesmo com erro, exibir token para cópia manual
            Ok(render_success_with_warning(&access_token, &e.to_string(), teams.as_deref()))
        }
    }
}

/// Renderizar página de sucesso
fn render_success_page(token: &str, teams: Option<&[super::client::AuthorizedTeam]>) -> Html<String> {
    let teams_html = if let Some(teams) = teams {
        let items: Vec<String> = teams
            .iter()
            .map(|t| format!("<li><strong>{}</strong> (ID: {})</li>", t.name, t.id))
            .collect();

        format!(
            r#"
            <div class="teams-box">
                <h3>✅ Workspaces Autorizados:</h3>
                <ul>{}</ul>
            </div>
            "#,
            items.join("\n")
        )
    } else {
        String::new()
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ClickUp OAuth - Sucesso</title>
            <meta charset="UTF-8">
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
                       max-width: 900px; margin: 50px auto; padding: 20px; background: #f5f5f5; }}
                .container {{ background: white; padding: 30px; border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                .success {{ background: #d4edda; border: 2px solid #28a745; padding: 20px; border-radius: 8px; margin-bottom: 20px; }}
                .token-box {{ background: #fff3cd; padding: 20px; border: 2px solid #ffc107; border-radius: 8px; margin: 20px 0; }}
                .teams-box {{ background: #d1ecf1; padding: 20px; border: 2px solid #17a2b8; border-radius: 8px; margin: 20px 0; }}
                textarea {{ width: 100%; padding: 12px; font-family: 'Courier New', monospace;
                           font-size: 11px; border: 1px solid #ddd; border-radius: 4px; resize: vertical; }}
                button {{ background: #28a745; color: white; padding: 12px 24px; border: none;
                         border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: bold; }}
                button:hover {{ background: #218838; }}
                h1 {{ color: #28a745; margin-top: 0; }}
                h3 {{ margin-top: 0; color: #333; }}
                ul {{ padding-left: 20px; }}
                li {{ margin: 8px 0; }}
                .footer {{ text-align: center; margin-top: 30px; color: #666; font-size: 12px; }}
            </style>
            <script>
                function copyToken() {{
                    const textarea = document.getElementById('token');
                    textarea.select();
                    navigator.clipboard.writeText(textarea.value);
                    const btn = document.getElementById('copyBtn');
                    btn.textContent = '✅ Copiado!';
                    setTimeout(() => {{ btn.textContent = '📋 Copiar Token'; }}, 2000);
                }}
            </script>
        </head>
        <body>
            <div class="container">
                <div class="success">
                    <h1>✅ Autorização OAuth2 Concluída!</h1>
                    <p>Seu token foi gerado e salvo automaticamente no Secret Manager.</p>
                </div>

                {}

                <div class="token-box">
                    <h3>🔑 Access Token (para referência):</h3>
                    <textarea id="token" rows="4" readonly>{}</textarea>
                    <button id="copyBtn" onclick="copyToken()">📋 Copiar Token</button>
                </div>

                <div class="footer">
                    <p>✅ Sistema pronto para usar! O middleware já pode criar folders e spaces no ClickUp.</p>
                    <p>Você pode fechar esta janela.</p>
                </div>
            </div>
        </body>
        </html>
        "#,
        teams_html, token
    ))
}

/// Renderizar página de sucesso com aviso
fn render_success_with_warning(token: &str, warning: &str, teams: Option<&[super::client::AuthorizedTeam]>) -> Html<String> {
    let teams_html = if let Some(teams) = teams {
        let items: Vec<String> = teams
            .iter()
            .map(|t| format!("<li><strong>{}</strong> (ID: {})</li>", t.name, t.id))
            .collect();

        format!(
            r#"
            <div class="teams-box">
                <h3>✅ Workspaces Autorizados:</h3>
                <ul>{}</ul>
            </div>
            "#,
            items.join("\n")
        )
    } else {
        String::new()
    };

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ClickUp OAuth - Atenção</title>
            <meta charset="UTF-8">
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
                       max-width: 900px; margin: 50px auto; padding: 20px; background: #f5f5f5; }}
                .container {{ background: white; padding: 30px; border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                .warning {{ background: #fff3cd; border: 2px solid #ffc107; padding: 20px; border-radius: 8px; margin-bottom: 20px; }}
                .token-box {{ background: #f8d7da; padding: 20px; border: 2px solid #dc3545; border-radius: 8px; margin: 20px 0; }}
                .teams-box {{ background: #d1ecf1; padding: 20px; border: 2px solid #17a2b8; border-radius: 8px; margin: 20px 0; }}
                textarea {{ width: 100%; padding: 12px; font-family: 'Courier New', monospace;
                           font-size: 11px; border: 1px solid #ddd; border-radius: 4px; resize: vertical; }}
                button {{ background: #dc3545; color: white; padding: 12px 24px; border: none;
                         border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: bold; }}
                button:hover {{ background: #c82333; }}
                h1 {{ color: #856404; margin-top: 0; }}
                h3 {{ margin-top: 0; color: #333; }}
                ul {{ padding-left: 20px; }}
                li {{ margin: 8px 0; }}
            </style>
            <script>
                function copyToken() {{
                    const textarea = document.getElementById('token');
                    textarea.select();
                    navigator.clipboard.writeText(textarea.value);
                    const btn = document.getElementById('copyBtn');
                    btn.textContent = '✅ Copiado!';
                    setTimeout(() => {{ btn.textContent = '📋 Copiar Token'; }}, 2000);
                }}
            </script>
        </head>
        <body>
            <div class="container">
                <div class="warning">
                    <h1>⚠️  Token Obtido, mas com Aviso</h1>
                    <p>O token foi gerado com sucesso, mas houve um problema ao salvar automaticamente:</p>
                    <p><strong>{}</strong></p>
                    <p>Por favor, copie o token abaixo e salve manualmente usando o comando:</p>
                    <pre>gcloud secrets versions add clickup-oauth-token --data-file=-</pre>
                </div>

                {}

                <div class="token-box">
                    <h3>🔑 Access Token:</h3>
                    <textarea id="token" rows="4" readonly>{}</textarea>
                    <button id="copyBtn" onclick="copyToken()">📋 Copiar Token</button>
                </div>
            </div>
        </body>
        </html>
        "#,
        warning, teams_html, token
    ))
}

/// Renderizar página de erro
fn render_error_page(error: &str) -> Html<String> {
    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>ClickUp OAuth - Erro</title>
            <meta charset="UTF-8">
            <style>
                body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
                       max-width: 600px; margin: 50px auto; padding: 20px; background: #f5f5f5; }}
                .container {{ background: white; padding: 30px; border-radius: 12px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                .error {{ background: #f8d7da; border: 2px solid #dc3545; padding: 20px; border-radius: 8px; }}
                h1 {{ color: #721c24; margin-top: 0; }}
                a {{ color: #007bff; text-decoration: none; font-weight: bold; }}
                a:hover {{ text-decoration: underline; }}
            </style>
        </head>
        <body>
            <div class="container">
                <div class="error">
                    <h1>❌ Erro na Autorização</h1>
                    <p><strong>Erro:</strong> {}</p>
                    <p><a href="/auth/clickup">← Tentar novamente</a></p>
                </div>
            </div>
        </body>
        </html>
        "#,
        error
    ))
}
