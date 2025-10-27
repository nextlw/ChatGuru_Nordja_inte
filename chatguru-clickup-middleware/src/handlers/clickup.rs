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

    // ✅ Usa TaskManager do crate em vez de HTTP direto
    match state.clickup.get_tasks_in_list(None).await {
        Ok(tasks) => {
            log_info(&format!("✅ Listadas {} tasks", tasks.len()));
            Ok(Json(json!({
                "success": true,
                "tasks": tasks,
                "count": tasks.len(),
                "list_id": state.settings.clickup.list_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            log_clickup_api_error("get_tasks_in_list", None, &e.to_string());
            Err(AppError::ClickUpApi(e.to_string()))
        }
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

// ❌ REMOVIDO: test_clickup_connection
// Este endpoint era redundante com /ready que já faz o mesmo teste.
// Use /ready para health checks que testam conectividade com ClickUp.