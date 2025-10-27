// ============================================================================
// Handlers para endpoints administrativos do ClickUp
// ============================================================================
//
// Este módulo contém handlers HTTP para debug e administração da API do ClickUp.
// Estes endpoints NÃO fazem parte do fluxo principal de webhook/worker, mas sim
// são ferramentas auxiliares para:
//
// 1. Listar tarefas existentes em uma lista específica
// 2. Obter informações sobre uma lista (nome, status, campos customizados)
// 3. Testar conectividade e autenticação com a API do ClickUp
//
// SEGURANÇA: Estes endpoints devem ser protegidos em produção (ex: API key,
// autenticação OAuth2, ou restringir por IP/VPC).
//
// IMPORTANTE: Estes handlers fazem chamadas síncronas diretas à API do ClickUp,
// ao contrário do fluxo worker que usa Pub/Sub assíncrono.

use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use chatguru_clickup_middleware::utils::AppError;
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;

/// Handler HTTP para listar tarefas de uma lista específica do ClickUp
///
/// # Endpoint
/// `GET /clickup/tasks`
///
/// # Descrição
/// Faz uma chamada direta à API do ClickUp para listar todas as tarefas
/// de uma lista específica (configurada em settings.clickup.list_id).
///
/// # Fluxo
/// 1. Extrai list_id da configuração do AppState
/// 2. Constrói URL da API do ClickUp: `/api/v2/list/{list_id}/task`
/// 3. Envia requisição GET com token de autenticação no header
/// 4. Processa resposta e retorna tarefas em formato JSON
///
/// # Resposta de Sucesso
/// ```json
/// {
///   "success": true,
///   "tasks": [...],           // Array de tarefas da API do ClickUp
///   "list_id": "123456789",   // ID da lista consultada
///   "timestamp": "2025-10-14T16:20:00Z"
/// }
/// ```
///
/// # Resposta de Erro
/// Retorna `AppError::ClickUpApi` com código HTTP e mensagem de erro
///
/// # Uso
/// Útil para debug, verificar se tarefas foram criadas corretamente,
/// ou auditar conteúdo de uma lista específica.
pub async fn list_clickup_tasks(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/tasks", "GET");

    // Construir URL da API do ClickUp para listar tarefas
    // Formato: https://api.clickup.com/api/v2/list/{list_id}/task
    let url = format!("https://api.clickup.com/api/v2/list/{}/task", state.settings.clickup.list_id);

    // Fazer requisição HTTP GET à API do ClickUp
    // Authorization: Bearer token (Personal Token ou OAuth2 Access Token)
    let response = state.clickup_client
        .get(&url)
        .header("Authorization", format!("Bearer {}", &state.settings.clickup.token))
        .send()
        .await?;

    let status = response.status();

    // Processar resposta com base no status HTTP
    if status.is_success() {
        // Sucesso: extrair array de tarefas do JSON retornado
        let tasks: Value = response.json().await?;
        Ok(Json(json!({
            "success": true,
            "tasks": tasks.get("tasks").unwrap_or(&json!([])),  // Extrair campo "tasks" ou retornar []
            "list_id": state.settings.clickup.list_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })))
    } else {
        // Erro: logar e retornar erro estruturado
        let error_text = response.text().await.unwrap_or_default();
        log_clickup_api_error(&url, Some(status.as_u16()), &error_text);
        Err(AppError::ClickUpApi(format!("Status: {} - {}", status, error_text)))
    }
}

/// Handler HTTP para obter informações detalhadas sobre uma lista do ClickUp
///
/// # Endpoint
/// `GET /clickup/list`
///
/// # Descrição
/// Delega a chamada para `ClickUpService::get_list_info()`, que retorna
/// metadados sobre a lista configurada, incluindo:
/// - Nome da lista
/// - Status disponíveis
/// - Campos customizados (custom fields) e seus IDs
/// - Configurações da lista
///
/// # Fluxo
/// 1. Chama `state.clickup.get_list_info()` (método do ClickUpService)
/// 2. Retorna informações estruturadas em JSON
///
/// # Resposta de Sucesso
/// ```json
/// {
///   "success": true,
///   "list": {
///     "id": "123456789",
///     "name": "OUTUBRO 2025",
///     "status": [...],
///     "custom_fields": [...]
///   },
///   "timestamp": "2025-10-14T16:20:00Z"
/// }
/// ```
///
/// # Resposta de Erro
/// Retorna `AppError::ClickUpApi` se a API falhar
///
/// # Uso
/// Útil para:
/// - Debug: verificar se a lista está corretamente configurada
/// - Mapeamento: obter IDs de custom fields para usar no payload
/// - Auditoria: verificar status disponíveis na lista
pub async fn get_clickup_list_info(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/list", "GET");

    // Delegar para o serviço ClickUp, que encapsula a lógica de chamada à API
    match state.clickup.get_list_info(None).await {
        Ok(list_info) => {
            // Sucesso: retornar informações da lista em formato estruturado
            Ok(Json(json!({
                "success": true,
                "list": list_info,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            // Erro: logar e propagar erro para o cliente
            log_clickup_api_error("get_list_info", None, &e.to_string());
            Err(AppError::ClickUpApi(e.to_string()))
        }
    }
}

/// Handler HTTP para testar conectividade e autenticação com a API do ClickUp
///
/// # Endpoint
/// `GET /clickup/test`
///
/// # Descrição
/// Executa um teste de conectividade chamando a API do ClickUp para validar:
/// - Token de autenticação é válido
/// - Permissões estão corretas
/// - API está acessível
/// - Retorna informações do usuário autenticado
///
/// # Fluxo
/// 1. Chama `state.clickup.test_connection()` (método do ClickUpService)
/// 2. Tenta fazer uma chamada simples à API (ex: GET /api/v2/user)
/// 3. Retorna sucesso + informações do usuário OU erro detalhado
///
/// # Resposta de Sucesso
/// ```json
/// {
///   "success": true,
///   "message": "ClickUp connection successful",
///   "user": {
///     "id": 123456,
///     "username": "user@example.com",
///     "email": "user@example.com"
///   },
///   "list_id": "901321080769",
///   "timestamp": "2025-10-14T16:20:00Z"
/// }
/// ```
///
/// # Resposta de Erro (HTTP 200 com success: false)
/// ```json
/// {
///   "success": false,
///   "message": "ClickUp connection failed",
///   "error": "Invalid token or insufficient permissions",
///   "list_id": "901321080769",
///   "timestamp": "2025-10-14T16:20:00Z"
/// }
/// ```
///
/// # Uso
/// Útil para:
/// - Verificar se o token OAuth2 ou Personal Token está válido
/// - Debug de problemas de autenticação antes de processar webhooks
/// - Health check externo do sistema
/// - Monitoramento: pode ser usado em scripts de monitoramento
///
/// # IMPORTANTE
/// Este endpoint SEMPRE retorna HTTP 200, mesmo em caso de falha.
/// O campo "success" no JSON indica se o teste passou ou falhou.
pub async fn test_clickup_connection(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    log_request_received("/clickup/test", "GET");

    // Delegar teste de conexão para o serviço ClickUp
    match state.clickup.test_connection().await {
        Ok(user_info) => {
            // Sucesso: token válido, API acessível, retornar info do usuário
            Ok(Json(json!({
                "success": true,
                "message": "ClickUp connection successful",
                "user": user_info,
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            // Falha: token inválido, sem permissão, ou API inacessível
            // IMPORTANTE: Retorna HTTP 200 com success: false (não propaga erro)
            log_clickup_api_error("test_connection", None, &e.to_string());
            Ok(Json(json!({
                "success": false,
                "message": "ClickUp connection failed",
                "error": e.to_string(),
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
    }
}