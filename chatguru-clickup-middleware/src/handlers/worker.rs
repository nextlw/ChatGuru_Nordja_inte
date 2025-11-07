/// Worker Handler: Processa mensagens do Pub/Sub
///
/// 🔄 **HANDLER ATUALIZADO (Novembro 2025)**: Integrado com nova implementação
/// de busca direta por nome de pasta/lista, eliminando uso do campo "Cliente Solicitante"
/// no fluxo principal de processamento.
///
/// ## MUDANÇAS NO FLUXO:
/// - ✅ Utiliza workspace_hierarchy service atualizado
/// - ✅ Busca estrutura organizacional por nome diretamente
/// - ✅ Mantém compatibilidade com todo pipeline existente
/// - ✅ Melhoria na confiabilidade de criação de tarefas
///
/// Arquitetura:
/// 1. Recebe payload RAW do Pub/Sub via HTTP POST
/// 2. Processa com OpenAI para classificação
/// 3. Se for atividade, cria tarefa no ClickUp (usando busca por nome)
/// 4. Envia anotação de volta ao ChatGuru
///
/// Este endpoint é chamado automaticamente pelo Cloud Tasks
/// Headers esperados:
/// - X-CloudTasks-TaskName: Nome da task
/// - X-CloudTasks-QueueName: Nome da fila
/// - X-CloudTasks-TaskRetryCount: Número de tentativas (0-indexed)

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
    http::StatusCode,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;
use base64::{Engine as _, engine::general_purpose};
use chrono::Datelike;

use chatguru_clickup_middleware::models::payload::WebhookPayload;
use chatguru_clickup_middleware::utils::{AppResult, AppError};
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;
use chatguru_clickup_middleware::services; // Para SecretsService
// Usar services do crate clickup ao invés de duplicar no main project
use clickup::assignees::SmartAssigneeFinder;
// REMOVIDO: use clickup::fields::CustomFieldManager;
// Motivo: Eliminação da lógica do campo "Cliente Solicitante"

/// 🏗️ ESTRUTURA: Contexto organizacional para enriquecer classificação IA
///
/// OBJETIVO: Encapsular informações de estrutura organizacional (folder/list) 
/// para fornecer contexto rico à IA na classificação de tarefas
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct OrganizationalContext {
    /// ID da pasta no ClickUp
    folder_id: String,
    /// Nome da pasta para contexto
    folder_name: String,
    /// ID da lista no ClickUp
    list_id: String,
    /// Nome da lista para contexto
    list_name: String,
}

/// 🧠 RESULTADO DA CLASSIFICAÇÃO IA
///
/// Estrutura que armazena o resultado detalhado da análise IA sobre o conteúdo
#[derive(Debug, Clone)]
struct AiClassificationResult {
    /// Se o conteúdo é uma task válida
    is_task: bool,
    /// Nível de confiança da classificação (0.0 a 1.0)
    confidence: f32,
    /// Razão para a classificação
    reason: String,
    /// Campanha identificada (opcional)
    campanha: Option<String>,
    /// Sub-categoria da atividade (opcional)
    sub_categoria: Option<String>,
    /// Prioridade sugerida (1-4, sendo 1 mais urgente)
    priority: Option<u8>,
    /// Se contexto organizacional foi usado na análise
    organizational_context_used: bool,
}

// Configuração de retry
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// 🤖 FUNÇÃO PRINCIPAL: Classificação IA de conteúdo
///
/// OBJETIVO: Determinar se o conteúdo recebido constitui uma task válida para criação no ClickUp
/// BENEFÍCIO: Automatiza a triagem de mensagens, reduzindo ruído e melhorando qualidade das tarefas
///
/// PARÂMETROS:
/// - payload: Payload completo do ChatGuru com todo contexto da conversa
/// - organizational_context: Contexto organizacional opcional (folder/list) para enriquecer análise
///
/// RETORNO:
/// - Ok(AiClassificationResult): Resultado detalhado da classificação
/// - Err(AppError): Erro na comunicação com IA ou processamento
///
/// INTEGRAÇÃO: Utiliza serviços OpenAI via SecretsService para análise de texto
async fn classify_content_with_ai(
    payload: &WebhookPayload,
    organizational_context: Option<&OrganizationalContext>,
) -> Result<AiClassificationResult, AppError> {
    // 📋 LOG DE INÍCIO DA CLASSIFICAÇÃO IA
    log_info("🤖 INICIANDO CLASSIFICAÇÃO IA DE CONTEÚDO");
    
    // 🔑 OBTENÇÃO DE CREDENCIAIS OPENAI
    let secrets_service = match services::SecretManagerService::new().await {
        Ok(service) => service,
        Err(e) => {
            log_error(&format!("❌ Falha ao inicializar SecretsService: {}", e));
            return Err(AppError::ConfigError(format!("Secrets service error: {}", e)));
        }
    };

    let openai_api_key = match secrets_service.get_openai_api_key().await {
        Ok(key) => key,
        Err(e) => {
            log_error(&format!("❌ Falha ao obter chave OpenAI: {}", e));
            return Err(AppError::ConfigError(format!("OpenAI key error: {}", e)));
        }
    };

    // 📝 EXTRAÇÃO DE CONTEXTO PARA IA
    // 📝 EXTRAÇÃO DE CONTEXTO PARA IA - baseado na estrutura correta do WebhookPayload
    let (message_content, client_name, attendant_name) = match payload {
        WebhookPayload::ChatGuru(chatguru_payload) => {
            let message = if !chatguru_payload.texto_mensagem.is_empty() {
                chatguru_payload.texto_mensagem.clone()
            } else {
                "[Conteúdo não disponível]".to_string()
            };
            
            let cliente = chatguru_payload.campos_personalizados
                .get("Info_2")
                .and_then(|v| v.as_str())
                .unwrap_or("[Cliente não identificado]")
                .to_string();
                
            let atendente = chatguru_payload.campos_personalizados
                .get("Info_1")
                .and_then(|v| v.as_str())
                .unwrap_or("[Atendente não identificado]")
                .to_string();
                
            (message, cliente, atendente)
        },
        WebhookPayload::EventType(event_payload) => {
            let message = event_payload.data.annotation
                .as_ref()
                .unwrap_or(&"[Conteúdo não disponível]".to_string())
                .clone();
            let cliente = event_payload.data.lead_name
                .as_ref()
                .unwrap_or(&"[Cliente não identificado]".to_string())
                .clone();
            let atendente = "[Atendente não identificado]".to_string();
            (message, cliente, atendente)
        },
        WebhookPayload::Generic(generic_payload) => {
            let message = generic_payload.mensagem
                .as_ref()
                .unwrap_or(&"[Conteúdo não disponível]".to_string())
                .clone();
            let cliente = generic_payload.nome
                .as_ref()
                .unwrap_or(&"[Cliente não identificado]".to_string())
                .clone();
            let atendente = "[Atendente não identificado]".to_string();
            (message, cliente, atendente)
        }
    };

    // 🏢 PREPARAÇÃO DO CONTEXTO ORGANIZACIONAL
    let context_info = if let Some(ctx) = organizational_context {
        format!(
            "\n📁 CONTEXTO ORGANIZACIONAL:\n- Pasta: {} ({})\n- Lista: {} ({})",
            ctx.folder_name, ctx.folder_id, ctx.list_name, ctx.list_id
        )
    } else {
        "\n⚠️ Sem contexto organizacional específico".to_string()
    };

    // 🧠 CONSTRUÇÃO DO PROMPT PARA IA
    let ai_prompt = format!(
        r#"ANÁLISE DE CLASSIFICAÇÃO DE TASK - ChatGuru ClickUp Integration

OBJETIVO: Determinar se o conteúdo é uma TASK VÁLIDA para ClickUp.

CONTEÚDO DA MENSAGEM:
"{}"

CONTEXTO:
- Cliente: {}
- Atendente: {}{}

CRITÉRIOS PARA SER TASK:
✅ SIM se contém:
- Solicitação de trabalho específico
- Ação concreta a ser executada
- Demanda de entrega/resultado
- Pedido de desenvolvimento, design, análise
- Briefing de projeto ou campanha

❌ NÃO se contém apenas:
- Saudações e conversas casuais
- Dúvidas simples ou perguntas
- Agradecimentos ou confirmações
- Informações sem ação requerida
- Conversas administrativas

RESPONDA EM JSON:
{{
  "is_task": boolean,
  "confidence": 0.0-1.0,
  "reason": "explicação clara da decisão",
  "campanha": "nome da campanha se identificada ou null",
  "sub_categoria": "categoria da atividade ou null",
  "priority": 1-4 ou null (1=urgente, 4=baixa)
}}

Seja rigoroso: apenas conteúdo que realmente demanda execução deve ser classificado como task."#,
        message_content, client_name, attendant_name, context_info
    );

    // 📡 CHAMADA PARA OPENAI
    log_info("📡 Enviando solicitação para OpenAI...");
    
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "system",
                    "content": "Você é um especialista em classificação de tarefas para sistemas de gestão de projetos. Responda sempre em JSON válido conforme solicitado."
                },
                {
                    "role": "user",
                    "content": ai_prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 500
        }))
        .send()
        .await
        .map_err(|e| AppError::HttpError(e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        log_error(&format!("❌ OpenAI API error: {} - {}", status, error_text));
        return Err(AppError::InternalError(format!("OpenAI API error: {}", error_text)));
    }

    // 🔍 PROCESSAMENTO DA RESPOSTA
    let openai_response: serde_json::Value = response.json().await
        .map_err(|e| AppError::HttpError(e))?;

    let ai_content = openai_response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| AppError::InternalError("Invalid OpenAI response format".to_string()))?;

    // 📊 PARSING DO RESULTADO JSON
    let ai_result: serde_json::Value = serde_json::from_str(ai_content.trim())
        .map_err(|e| AppError::JsonError(e))?;

    let classification_result = AiClassificationResult {
        is_task: ai_result["is_task"].as_bool().unwrap_or(false),
        confidence: ai_result["confidence"].as_f64().unwrap_or(0.0) as f32,
        reason: ai_result["reason"].as_str().unwrap_or("Não especificado").to_string(),
        campanha: ai_result["campanha"].as_str().map(|s| s.to_string()),
        sub_categoria: ai_result["sub_categoria"].as_str().map(|s| s.to_string()),
        priority: ai_result["priority"].as_u64().map(|p| p as u8),
        organizational_context_used: organizational_context.is_some(),
    };

    // ✅ LOG DO RESULTADO
    log_info(&format!(
        "🎯 CLASSIFICAÇÃO IA CONCLUÍDA - É Task: {} | Confiança: {:.1}% | Razão: {}",
        classification_result.is_task,
        classification_result.confidence * 100.0,
        classification_result.reason
    ));

    if let Some(campanha) = &classification_result.campanha {
        log_info(&format!("🎪 Campanha identificada: {}", campanha));
    }

    Ok(classification_result)
}

/// 🏗️ FUNÇÃO AUXILIAR: Extração de contexto organizacional
///
/// OBJETIVO: Extrair informações de estrutura organizacional (folder/list) do payload
/// para enriquecer a análise IA com contexto específico do cliente/atendente
///
/// PARÂMETROS:
/// - payload: Payload completo do ChatGuru
///
/// RETORNO:
/// - Ok(Some(OrganizationalContext)): Contexto organizacional encontrado
/// - Ok(None): Sem contexto organizacional disponível
/// - Err(AppError): Erro ao processar contexto
async fn extract_organizational_context(payload: &WebhookPayload) -> Result<Option<OrganizationalContext>, AppError> {
    // Extrair cliente e atendente do payload
    let (cliente, atendente) = match payload {
        WebhookPayload::ChatGuru(chatguru_payload) => {
            let info_1 = chatguru_payload.campos_personalizados
                .get("Info_1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            let info_2 = chatguru_payload.campos_personalizados
                .get("Info_2")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
                
            (info_1, info_2)
        },
        _ => (None, None)
    };

    // Se não temos cliente ou atendente, não há contexto organizacional
    let (cliente_name, atendente_name) = match (cliente, atendente) {
        (Some(c), Some(a)) => (c, a),
        _ => {
            log_info("ℹ️ Contexto organizacional incompleto (faltam info_1 ou info_2)");
            return Ok(None);
        }
    };

    log_info(&format!("🔍 Buscando contexto organizacional para Cliente: '{}' | Atendente: '{}'",
        cliente_name, atendente_name));

    // TODO: Aqui deveria integrar com o workspace_hierarchy service para buscar
    // a estrutura organizacional real. Por enquanto, retorna None até a integração
    // completa estar disponível.
    //
    // INTEGRAÇÃO FUTURA:
    // let workspace_service = services::workspace_hierarchy::WorkspaceHierarchyService::new();
    // let structure = workspace_service.resolve_structure(&cliente_name, &atendente_name).await?;
    
    log_info("⚠️ Integração com workspace_hierarchy service pendente - retornando None por enquanto");
    
    // Retorna None por enquanto (implementação completa virá em próxima iteração)
    Ok(None)
}

/// Handler do worker
/// Retorna 200 OK se processado com sucesso
/// Retorna 4xx se erro não recuperável (não faz retry)
/// Retorna 5xx se erro recuperável (Pub/Sub faz retry até MAX_RETRY_ATTEMPTS)
pub async fn handle_worker(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // CORREÇÃO CRÍTICA: Validação preventiva antes de processar
    // Verificar headers básicos para detectar problemas early
    let content_type = request.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
        
    let content_length = request.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    
    // Log de diagnóstico para headers críticos
    log_info(&format!(
        "🔍 WORKER REQUEST HEADERS - Content-Type: '{}' | Content-Length: {} | Headers: {}",
        content_type,
        content_length,
        request.headers().len()
    ));
    
    // Validação preventiva de content-type (Pub/Sub deve ser application/json)
    if !content_type.is_empty() && !content_type.contains("application/json") {
        log_error(&format!("❌ INVALID CONTENT-TYPE - Expected JSON, got: '{}'", content_type));
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid Content-Type, expected application/json",
                "received_content_type": content_type,
                "status": "invalid_request"
            }))
        ));
    }
    
    // Validação preventiva de tamanho (máx 50MB para Pub/Sub)
    if content_length > 50_000_000 {
        log_error(&format!("❌ PAYLOAD TOO LARGE - Size: {} bytes (max: 50MB)", content_length));
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "error": "Payload too large",
                "size_bytes": content_length,
                "max_size_bytes": 50_000_000,
                "status": "payload_too_large"
            }))
        ));
    }
    
    // Timeout global reduzido para detectar problemas mais rapidamente
    let global_timeout = std::time::Duration::from_secs(45);
    
    match tokio::time::timeout(global_timeout, handle_worker_internal(state, request)).await {
        Ok(result) => result,
        Err(_) => {
            log_error("❌ TIMEOUT GLOBAL - Worker excedeu 45 segundos, forçando término");
            Err((
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({
                    "error": "Worker timeout - processing exceeded 45 seconds",
                    "status": "timeout",
                    "timeout_seconds": 45
                }))
            ))
        }
    }
}

/// Implementação interna do worker com timeouts detalhados
async fn handle_worker_internal(
    state: Arc<AppState>,
    request: Request<Body>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let start_time = Instant::now();
    
    // Log de início com informações de request
    log_info(&format!(
        "🚀 WORKER INICIADO - Start time: {:?} | Headers count: {}",
        start_time,
        request.headers().len()
    ));
    
    log_request_received("/worker/process", "POST");

    // Primeiro, extrair headers antes de consumir o request
    let retry_count = request
        .headers()
        .get("googclient_deliveryattempt")
        .or_else(|| request.headers().get("x-goog-delivery-attempt"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1); // Pub/Sub starts at 1, not 0

    let message_id = request
        .headers()
        .get("x-cloudtasks-taskname")
        .or_else(|| request.headers().get("x-pubsub-messageid"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    log_info(&format!("🔄 Tentativa {} de {} (header: googclient_deliveryattempt), messageId: {}", retry_count, MAX_RETRY_ATTEMPTS, message_id));

    // Se excedeu o limite, retornar 200 para evitar loop infinito
    if retry_count > MAX_RETRY_ATTEMPTS {
        log_error(&format!("❌ Limite de tentativas excedido ({}/{}), descartando mensagem",
            retry_count, MAX_RETRY_ATTEMPTS));
        return Ok(Json(json!({
            "status": "discarded",
            "reason": "Max retry attempts exceeded",
            "retry_count": retry_count
        })));
    }

    // CORREÇÃO CRÍTICA: Timeout muito baixo para detectar problemas rapidamente
    let body_limit = 50_000_000; // 50MB máximo (Pub/Sub pode ser grande)
    let body_timeout = std::time::Duration::from_secs(5); // Reduzido de 10s para 5s
    
    log_info(&format!("📦 Reading body with timeout: {}s, limit: {}MB",
        body_timeout.as_secs(), body_limit / 1_000_000));
    
    let body_bytes = match tokio::time::timeout(
        body_timeout,
        axum::body::to_bytes(request.into_body(), body_limit)
    ).await {
        Ok(Ok(bytes)) => {
            log_info(&format!("✅ Body read successfully: {} bytes", bytes.len()));
            bytes
        },
        Ok(Err(e)) => {
            log_error(&format!("❌ BODY READ ERROR - {}", e));
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(json!({
                    "error": "Request body too large or invalid",
                    "limit_mb": body_limit / 1_000_000,
                    "details": e.to_string(),
                    "status": "body_read_error"
                }))
            ));
        },
        Err(_) => {
            log_error(&format!("❌ BODY TIMEOUT - Failed to read body within {}s", body_timeout.as_secs()));
            return Err((
                StatusCode::REQUEST_TIMEOUT,
                Json(json!({
                    "error": "Timeout reading request body",
                    "timeout_seconds": body_timeout.as_secs(),
                    "status": "body_timeout"
                }))
            ));
        }
    };

    // Validar se o body não está vazio
    if body_bytes.is_empty() {
        log_error("Request body is empty");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Empty request body"}))
        ));
    }

    let body_str = match String::from_utf8(body_bytes.to_vec()) {
        Ok(s) => s,
        Err(e) => {
            log_error(&format!("Invalid UTF-8: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid UTF-8"}))
            ));
        }
    };

    // CORREÇÃO: Parsing JSON mais robusto
    let envelope: Value = match serde_json::from_str::<Value>(&body_str) {
        Ok(v) => {
            log_info(&format!("✅ JSON parsed successfully: {} fields",
                v.as_object().map_or(0, |o| o.len())));
            v
        },
        Err(e) => {
            log_error(&format!("❌ JSON PARSE ERROR - {} | Body preview: {}",
                e,
                if body_str.len() > 200 {
                    format!("{}...", &body_str[..200])
                } else {
                    body_str.clone()
                }
            ));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid JSON format",
                    "details": e.to_string(),
                    "status": "json_parse_error",
                    "body_preview": if body_str.len() > 200 {
                        format!("{}...", &body_str[..200])
                    } else {
                        body_str
                    }
                }))
            ));
        }
    };

    // Extrair e decodificar payload do Pub/Sub
    // Formato completo do payload vindo do ChatGuru (via Pub/Sub):
    // {
    //   "message": {
    //     "data": "base64_encoded_json",
    //     "messageId": "12345678",
    //     "publishTime": "2025-01-01T00:00:00.000Z"
    //   },
    //   "subscription": "projects/PROJECT_ID/subscriptions/SUBSCRIPTION_NAME"
    // }
    //
    // Onde "data" (decodificado) contém envelope interno:
    // {
    //   "raw_payload": "{\"id_chatguru\":\"...\",\"texto_mensagem\":\"...\",\"celular\":\"...\",\"nome\":\"...\",\"media_url\":\"...\",\"media_type\":\"...\",...}"
    // }
    //
    // E raw_payload (decodificado) contém o payload real do ChatGuru:
    // {
    //   "campanha_id": "123",
    //   "campanha_nome": "WhatsApp",
    //   "origem": "whatsapp",
    //   "email": "cliente@example.com",
    //   "nome": "João Silva",
    //   "tags": ["tag1", "tag2"],
    //   "texto_mensagem": "Preciso de um motoboy",
    //   "media_url": "https://...",
    //   "media_type": "audio/ogg",
    //   "campos_personalizados": {},
    //   "bot_context": { "ChatGuru": true },
    //   "responsavel_nome": "Atendente",
    //   "responsavel_email": "atendente@example.com",
    //   "link_chat": "https://...",
    //   "celular": "5511999999999",
    //   "phone_id": "phone123",
    //   "chat_id": "chat123",
    //   "chat_created": "2025-01-01T00:00:00Z"
    // }
    let raw_payload_str = if let Some(message) = envelope.get("message") {
        // Formato padrão do Pub/Sub Push
        if let Some(data_b64) = message.get("data").and_then(|v| v.as_str()) {
            // Decodificar base64
            match general_purpose::STANDARD.decode(data_b64) {
                Ok(decoded_bytes) => {
                    match String::from_utf8(decoded_bytes) {
                        Ok(s) => s,
                        Err(e) => {
                            log_error(&format!("Invalid UTF-8 in Pub/Sub data: {}", e));
                            return Err((
                                StatusCode::BAD_REQUEST,
                                Json(json!({"error": "Invalid UTF-8 in message data"}))
                            ));
                        }
                    }
                },
                Err(e) => {
                    log_error(&format!("Failed to decode base64: {}", e));
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": "Invalid base64 encoding"}))
                    ));
                }
            }
        } else {
            log_error("Missing 'data' field in Pub/Sub message");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing data in message"}))
            ));
        }
    } else if let Some(raw_payload) = envelope.get("raw_payload").and_then(|v| v.as_str()) {
        // Formato direto (para testes)
        raw_payload.to_string()
    } else {
        log_error("Missing 'message' or 'raw_payload' in envelope");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid envelope format"}))
        ));
    };

    // Parsear o envelope que contém o raw_payload
    // O formato esperado após decodificar base64 é:
    // { "raw_payload": "{...chatguru payload...}", "received_at": "...", "source": "...", ... }
    let inner_envelope: Value = match serde_json::from_str(&raw_payload_str) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("Failed to parse envelope: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid envelope format"}))
            ));
        }
    };

    // Extrair o raw_payload do envelope (ou usar o próprio envelope se não tiver esse campo)
    let chatguru_payload_str = if let Some(raw_payload) = inner_envelope.get("raw_payload").and_then(|v| v.as_str()) {
        // Formato esperado: envelope tem campo raw_payload (string JSON)
        raw_payload.to_string()
    } else {
        // Fallback: o próprio envelope já é o payload do ChatGuru (para compatibilidade)
        log_warning("⚠️  Envelope sem campo 'raw_payload', usando envelope completo como payload");
        raw_payload_str.clone()
    };

    // Validar que o payload não está vazio
    if chatguru_payload_str.trim().is_empty() {
        log_error("Payload do ChatGuru está vazio");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Empty payload"}))
        ));
    }

    // Parsear payload do ChatGuru
    let mut payload: WebhookPayload = match serde_json::from_str(&chatguru_payload_str) {
        Ok(p) => p,
        Err(e) => {
            log_error(&format!("Failed to parse ChatGuru payload: {}", e));
            // Erro não recuperável - não fazer retry
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid ChatGuru payload"}))
            ));
        }
    };

    // Extrair informações básicas para logging detalhado
    let sender_name = extract_nome_from_payload(&payload);
    let message_text = extract_message_from_payload(&payload);
    let phone = extract_phone_from_payload(&payload);
    let chat_id = extract_chat_id_from_payload(&payload);
    
    // Log detalhado do worker iniciando processamento
    log_info(&format!(
        "🔧 WORKER INICIANDO PROCESSAMENTO - MessageID: {} | Tentativa: {}/{} | Sender: {} | Phone: {} | ChatID: {} | Size: {} chars",
        message_id,
        retry_count,
        MAX_RETRY_ATTEMPTS,
        sender_name,
        phone.as_deref().unwrap_or("N/A"),
        chat_id.as_deref().unwrap_or("N/A"),
        message_text.len()
    ));

    // Log do payload para debug (versão resumida)
    log_info(&format!("📦 Payload processado com sucesso ({} bytes)",
        serde_json::to_string(&payload).unwrap_or_default().len()
    ));

    // Clonar payload antes de fazer pattern matching para evitar conflitos de empréstimo
    let payload_clone = payload.clone();
    
    // Processar mídia (áudio/imagem) se houver
    if let WebhookPayload::ChatGuru(ref mut chatguru_payload) = payload {
        // IMPORTANTE: Normalizar campos de mídia do ChatGuru
        // Converte tipo_mensagem + url_arquivo → media_type + media_url
        chatguru_payload.normalize_media_fields();

        // Log dos campos de mídia (após normalização)
        log_info(&format!("🔍 Debug mídia - media_url: {:?}, media_type: {:?}, tipo_mensagem: {:?}, url_arquivo: {:?}, texto_mensagem: {:?}",
            chatguru_payload.media_url,
            chatguru_payload.media_type,
            chatguru_payload.tipo_mensagem,
            chatguru_payload.url_arquivo,
            chatguru_payload.texto_mensagem
        ));

        // Verificar se tem media_url e media_type
        if let (Some(media_url), Some(media_type)) = (&chatguru_payload.media_url, &chatguru_payload.media_type) {
            // Verificar se é tipo de mídia suportado (áudio, imagem, PDF)
            let is_supported = media_type.contains("audio") || media_type.contains("image") || media_type.contains("pdf");
            if is_supported {
                let processing_type = if media_type.contains("audio") {
                    "audio"
                } else if media_type.contains("image") {
                    "image"
                } else {
                    "pdf"
                };

                log_info(&format!("📎 Mídia detectada ({}: {}), iniciando processamento: {}",
                    processing_type, media_type, media_url));

                // Processar mídia com anotação usando IaService
                let (final_result, _annotation_opt) = if let Some(ref ia_service) = state.ia_service {
                    match processing_type {
                        "audio" => {
                            log_info("🎵 Processando áudio com transcrição + anotação");
                            // Timeout e limite de tamanho para áudio (máx 5MB, 10s)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(10),
                                ia_service.download_file(media_url, "Áudio")
                            ).await {
                                Ok(Ok(audio_bytes)) if audio_bytes.len() <= 5_000_000 => {
                                    let extension = media_url
                                        .split('.')
                                        .last()
                                        .and_then(|ext| ext.split('?').next())
                                        .unwrap_or("ogg");
                                    let filename = format!("audio.{}", extension);

                                    // Timeout para processamento de áudio (máx 15s)
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(15),
                                        ia_service.process_audio_with_annotation(&audio_bytes, &filename)
                                    ).await {
                                        Ok(Ok(result)) => {
                                            // Loga o tamanho da transcrição gerada
                                            log_info(&format!("✅ Áudio processado: {} caracteres", result.extracted_content.len()));
                                        
                                            // Monta a mensagem de transcrição no formato solicitado (sem emojis)
                                            let message = format!("estamos transcrevendo sua mensagem: {}", result.extracted_content);
                                        
                                            // Envia a mensagem de transcrição ao usuário via WhatsApp
                                            // ✅ Usa o cliente ChatGuru centralizado do AppState
                                            if let Some(phone) = extract_phone_from_payload(&payload_clone) {
                                                if let Err(e) = state.chatguru_client.send_confirmation_message(&phone, None, &message).await {
                                                    log_warning(&format!("⚠️ Falha ao enviar mensagem de transcrição: {}", e));
                                                } else {
                                                    log_info(&format!("✅ Mensagem de transcrição enviada via WhatsApp para {}", phone));
                                                }
                                            } else {
                                                log_warning("⚠️ Número de telefone não encontrado no payload, não foi possível enviar mensagem de transcrição");
                                            }
                                        
                                            // NÃO envia anotação separada para áudio, apenas a mensagem
                                            // A transcrição continua sendo usada para o batch de classificação normalmente
                                            (Some(result.extracted_content), None)
                                        }
                                        Ok(Err(e)) => {
                                            log_error(&format!("❌ Erro ao processar áudio: {}", e));
                                            (None, None)
                                        }
                                        Err(_) => {
                                            log_error("❌ Timeout ao processar áudio (15s)");
                                            (None, None)
                                        }
                                    }
                                }
                                Ok(Ok(_)) => {
                                    log_error("❌ Arquivo de áudio muito grande (>5MB), ignorando");
                                    (None, None)
                                }
                                Ok(Err(e)) => {
                                    log_error(&format!("❌ Erro ao baixar áudio: {}", e));
                                    (None, None)
                                }
                                Err(_) => {
                                    log_error("❌ Timeout ao baixar áudio (10s)");
                                    (None, None)
                                }
                            }
                        }
                        "image" => {
                            log_info("🖼️ Processando imagem com descrição + anotação");
                            // Timeout e limite para imagem (máx 3MB, 8s download, 10s processing)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(8),
                                ia_service.download_file(media_url, "Imagem")
                            ).await {
                                Ok(Ok(image_bytes)) if image_bytes.len() <= 3_000_000 => {
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(10),
                                        ia_service.process_image_with_annotation(&image_bytes)
                                    ).await {
                                        Ok(Ok(result)) => {
                                            log_info(&format!("✅ Imagem processada: {} caracteres", result.extracted_content.len()));
                                            (Some(result.extracted_content), result.annotation)
                                        }
                                        Ok(Err(e)) => {
                                            log_error(&format!("❌ Erro ao processar imagem: {}", e));
                                            (None, None)
                                        }
                                        Err(_) => {
                                            log_error("❌ Timeout ao processar imagem (10s)");
                                            (None, None)
                                        }
                                    }
                                }
                                Ok(Ok(_)) => {
                                    log_error("❌ Arquivo de imagem muito grande (>3MB), ignorando");
                                    (None, None)
                                }
                                Ok(Err(e)) => {
                                    log_error(&format!("❌ Erro ao baixar imagem: {}", e));
                                    (None, None)
                                }
                                Err(_) => {
                                    log_error("❌ Timeout ao baixar imagem (8s)");
                                    (None, None)
                                }
                            }
                        }
                        "pdf" => {
                            log_info("📄 Processando PDF com extração + anotação");
                            // Timeout e limite para PDF (máx 10MB, 15s download, 20s processing)
                            match tokio::time::timeout(
                                std::time::Duration::from_secs(15),
                                ia_service.download_file(media_url, "PDF")
                            ).await {
                                Ok(Ok(pdf_bytes)) if pdf_bytes.len() <= 10_000_000 => {
                                    match tokio::time::timeout(
                                        std::time::Duration::from_secs(20),
                                        ia_service.process_pdf_with_annotation(&pdf_bytes)
                                    ).await {
                                        Ok(Ok(result)) => {
                                            log_info(&format!("✅ PDF processado: {} caracteres", result.extracted_content.len()));
                                            (Some(result.extracted_content), result.annotation)
                                        }
                                        Ok(Err(e)) => {
                                            log_error(&format!("❌ Erro ao processar PDF: {}", e));
                                            (None, None)
                                        }
                                        Err(_) => {
                                            log_error("❌ Timeout ao processar PDF (20s)");
                                            (None, None)
                                        }
                                    }
                                }
                                Ok(Ok(_)) => {
                                    log_error("❌ Arquivo PDF muito grande (>10MB), ignorando");
                                    (None, None)
                                }
                                Ok(Err(e)) => {
                                    log_error(&format!("❌ Erro ao baixar PDF: {}", e));
                                    (None, None)
                                }
                                Err(_) => {
                                    log_error("❌ Timeout ao baixar PDF (15s)");
                                    (None, None)
                                }
                            }
                        }
                        _ => (None, None)
                    }
                } else {
                    log_error("❌ IaService não está disponível no AppState");
                    (None, None)
                };

                // Atualizar payload com resultado PRIMEIRO
                if let Some(result_text) = final_result {
                    let label = match processing_type {
                        "audio" => "Transcrição do áudio",
                        "image" => "Descrição da imagem",
                        "pdf" => "Conteúdo do PDF",
                        _ => "Descrição da mídia",
                    };

                    if !chatguru_payload.texto_mensagem.is_empty() {
                        chatguru_payload.texto_mensagem = format!(
                            "{}\n\n[{}]: {}",
                            chatguru_payload.texto_mensagem,
                            label,
                            result_text
                        );
                    } else {
                        chatguru_payload.texto_mensagem = result_text;
                    }

                    log_info(&format!("📝 Payload enriquecido com {}", label));
                } else {
                    log_warning("⚠️ Nenhum resultado de processamento de mídia disponível");
                }

                // [REMOVIDO] Não enviar mais anotação de mídia (descrição de imagem/arquivo) imediatamente após processamento.
                // O enriquecimento do payload permanece, mas o envio da anotação foi removido conforme solicitado.
            }
        }
    }

    // Extrair force_classification se presente
    let force_classification = envelope.get("force_classification");

    // 🧠 ETAPA CRÍTICA: CLASSIFICAÇÃO IA DO CONTEÚDO
    //
    // OBJETIVO: Determinar automaticamente se o conteúdo recebido constitui uma task válida
    // antes de prosseguir com o processo de criação no ClickUp
    //
    // BENEFÍCIOS:
    // - Reduz ruído no ClickUp (evita tasks desnecessárias)
    // - Melhora qualidade das tarefas criadas
    // - Fornece rastreabilidade da decisão
    // - Enriquece contexto para próximas etapas

    log_info("🤖 INICIANDO ANÁLISE IA - Classificação de conteúdo");
    
    // Extrair contexto organizacional se disponível (para enriquecer análise IA)
    let organizational_context = match extract_organizational_context(&payload).await {
        Ok(Some(ctx)) => {
            log_info(&format!("📁 Contexto organizacional extraído: {} / {}",
                ctx.folder_name, ctx.list_name));
            Some(ctx)
        },
        Ok(None) => {
            log_info("ℹ️ Sem contexto organizacional específico disponível");
            None
        },
        Err(e) => {
            log_warning(&format!("⚠️ Erro ao extrair contexto organizacional: {}", e));
            None
        }
    };

    // Realizar classificação IA com contexto organizacional
    let ai_classification = match classify_content_with_ai(&payload, organizational_context.as_ref()).await {
        Ok(result) => {
            log_info(&format!(
                "🎯 CLASSIFICAÇÃO IA CONCLUÍDA - É Task: {} | Confiança: {:.1}% | Razão: {}",
                result.is_task,
                result.confidence * 100.0,
                result.reason
            ));
            Some(result)
        },
        Err(e) => {
            log_error(&format!("❌ ERRO NA CLASSIFICAÇÃO IA: {} - Continuando sem classificação", e));
            // Em caso de erro na IA, continua processamento sem classificação
            None
        }
    };

    // Armazenar resultado da classificação para uso nas próximas etapas
    // (será usado em process_message para decidir se criar task ou apenas anotar)
    let classification_result = ai_classification.clone();

    // Log detalhado do resultado da classificação para rastreabilidade
    if let Some(ref classification) = classification_result {
        log_info(&format!(
            "📊 RESULTADO CLASSIFICAÇÃO ARMAZENADO - Task: {} | Confiança: {:.1}% | Contexto Org: {}",
            classification.is_task,
            classification.confidence * 100.0,
            classification.organizational_context_used
        ));
        
        if let Some(campanha) = &classification.campanha {
            log_info(&format!("🎪 Campanha identificada pela IA: {}", campanha));
        }
        
        if let Some(prioridade) = &classification.priority {
            log_info(&format!("📈 Prioridade sugerida pela IA: {}", prioridade));
        }
    }

// Processar mensagem com tratamento robusto de resposta
// TODO: Integrar classification_result no process_message para usar na decisão de criação
match process_message(&state, &payload, force_classification).await {
    Ok(result) => {
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        log_info(&format!(
            "✅ WORKER PROCESSAMENTO CONCLUÍDO - Time: {}ms | Status: success",
            processing_time
        ));
        
        log_request_processed("/worker/process", 200, processing_time);
        
        // Garantir que a resposta é válida e não está vazia
        let response = if result.is_null() {
            json!({
                "status": "processed",
                "processing_time_ms": processing_time,
                "result": "empty_payload"
            })
        } else {
            result
        };
        
        Ok(Json(response))
    }
    Err(e) => {
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        log_error(&format!(
            "❌ WORKER ERROR - Time: {}ms | Attempt: {}/{} | Error: {}",
            processing_time, retry_count, MAX_RETRY_ATTEMPTS, e
        ));

        // Classificar erro: recuperável vs não-recuperável
        let is_recoverable = match &e {
            // Erros de API externa (ClickUp, HTTP, Timeout) - recuperável
            AppError::ClickUpApi(_) => retry_count < MAX_RETRY_ATTEMPTS,
            AppError::HttpError(_) => retry_count < MAX_RETRY_ATTEMPTS,
            AppError::Timeout(_) => retry_count < MAX_RETRY_ATTEMPTS,
            AppError::PubSubError(_) => retry_count < MAX_RETRY_ATTEMPTS,

            // Erros de configuração/validação - NÃO recuperável
            AppError::ConfigError(_) => false,
            AppError::ValidationError(_) => false,
            AppError::JsonError(_) => false,

            // Estrutura não encontrada - NÃO recuperável (já tratado internamente)
            AppError::StructureNotFound(_) => false,

            // Database error - NÃO recuperável (indica problema de configuração)
            AppError::DatabaseError(_) => false,

            // Outros erros internos - permitir retry limitado
            AppError::InternalError(_) => retry_count < MAX_RETRY_ATTEMPTS,
        };

        if is_recoverable {
            // Erro recuperável - Pub/Sub vai fazer retry
            log_warning(&format!("⚠️ Erro recuperável, Pub/Sub fará retry (tentativa {}/{})",
                retry_count, MAX_RETRY_ATTEMPTS));
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": e.to_string(),
                    "retry_count": retry_count,
                    "max_retries": MAX_RETRY_ATTEMPTS,
                    "will_retry": true,
                    "processing_time_ms": processing_time
                }))
            ))
        } else {
            // Erro não recuperável - retornar 200 para evitar retry
            log_error(&format!("❌ Erro não recuperável ou limite de tentativas atingido, descartando mensagem: {}", e));
            
            // Retornar 200 OK com status de erro para evitar retry infinito
            Ok(Json(json!({
                "status": "failed",
                "error": e.to_string(),
                "retry_count": retry_count,
                "max_retries": MAX_RETRY_ATTEMPTS,
                "reason": "Non-recoverable error or max retries exceeded",
                "processing_time_ms": processing_time,
                "discarded": true
            })))
        }
    }
}

}

/// Processa uma mensagem do ChatGuru
async fn process_message(state: &Arc<AppState>, payload: &WebhookPayload, force_classification: Option<&Value>) -> AppResult<Value> {
    // Filtrar eventos que não devem ser processados
    if let WebhookPayload::EventType(event_payload) = payload {
        if event_payload.event_type == "annotation.added" {
            log_info("⏭️  Ignorando evento annotation.added (gerado pelo sistema)");
            return Ok(json!({
                "status": "skipped",
                "reason": "annotation.added event"
            }));
        }
    }

    // Extrair dados básicos
    let nome = extract_nome_from_payload(payload);
    let message = extract_message_from_payload(payload);
    let phone = extract_phone_from_payload(payload);
    let chat_id = extract_chat_id_from_payload(payload);

    // CORREÇÃO: Usar char_indices para evitar panic com UTF-8 multi-byte
    let message_preview = if message.chars().count() > 150 {
        let mut char_count = 0;
        let mut byte_end = 0;
        for (byte_idx, _) in message.char_indices() {
            if char_count >= 150 {
                byte_end = byte_idx;
                break;
            }
            char_count += 1;
        }
        if byte_end > 0 {
            format!("{}...", &message[..byte_end])
        } else {
            format!("{}...", message.chars().take(150).collect::<String>())
        }
    } else {
        message.clone()
    };

    log_info(&format!(
        "💬 PROCESSANDO MENSAGEM - Sender: {} | ChatID: {} | Phone: {} | Message: \"{}\"",
        if !nome.is_empty() { nome.clone() } else { "Desconhecido".to_string() },
        chat_id.as_deref().unwrap_or("N/A"),
        phone.as_deref().unwrap_or("N/A"),
        message_preview
    ));

    // Carregar configuração de prompt (necessária para ambos os cenários: forçado e IA)
    use chatguru_clickup_middleware::services::prompts::AiPromptConfig;
    let prompt_config = AiPromptConfig::load_default().await
        .map_err(|e| AppError::InternalError(format!("Failed to load prompt config: {}", e)))?;

    // Verificar se há classificação forçada (bypass OpenAI)
    let classification = if let Some(forced) = force_classification {
        log_info("🔧 Usando classificação forçada (bypass OpenAI)");

        use crate::services::OpenAIClassification;
        OpenAIClassification {
            reason: forced.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("Classificação manual")
                .to_string(),
            is_activity: forced.get("is_task_worthy")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            category: forced.get("campanha")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            campanha: forced.get("campanha")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: forced.get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            space_name: forced.get("space_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            folder_name: forced.get("folder_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            list_name: forced.get("list_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            info_1: forced.get("info_1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            info_2: forced.get("info_2")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            tipo_atividade: None,
            sub_categoria: None,
            subtasks: vec![],
            status_back_office: None,
        }
    } else {
        // 🤖 CLASSIFICAÇÃO IA MELHORADA: Utilizando contextos extraídos (info_2, folder_id, list_id)
        //
        // OBJETIVO: Implementar classificação automatizada que aproveita os contextos já extraídos
        // para fornecer informações mais ricas à IA, melhorando a precisão da classificação.
        //
        // MELHORIAS IMPLEMENTADAS:
        // 1. Pré-validação e extração de contextos organizacionais (folder/list)
        // 2. Enriquecimento do prompt com informações estruturadas do ClickUp
        // 3. Logs detalhados para rastreabilidade completa do processo
        // 4. Armazenamento estruturado do resultado para etapas posteriores
        //
        // FLUXO:
        // 1. Extrai info_2 e valida disponibilidade
        // 2. Busca contexto organizacional (folder_id, list_id) via WorkspaceHierarchyService
        // 3. Enriquece o prompt com contextos estruturados
        // 4. Executa classificação IA com timeout otimizado
        // 5. Armazena resultado estruturado para próximas etapas
        
        log_info(&format!(
            "🤖 INICIANDO CLASSIFICAÇÃO IA MELHORADA - ChatID: {} | Sender: {}",
            chat_id.as_deref().unwrap_or("N/A"),
            nome
        ));
        
        // 🔍 EXTRAÇÃO E VALIDAÇÃO DE CONTEXTOS OBRIGATÓRIOS
        let info_2 = extract_info_2_from_payload(payload).unwrap_or_default();
        
        log_info(&format!(
            "🔍 CONTEXTO EXTRAÍDO - Info_2: '{}' | Chat: {} | Telefone: {}",
            info_2,
            chat_id.as_deref().unwrap_or("N/A"),
            phone.as_deref().unwrap_or("N/A")
        ));
        
        // VALIDAÇÃO OBRIGATÓRIA: info_2 é essencial para o processamento
        if info_2.is_empty() {
            log_warning(&format!(
                "⚠️ CAMPO INFO_2 VAZIO - ChatID: {} | Sender: {} | Processamento cancelado",
                chat_id.as_deref().unwrap_or("N/A"),
                nome
            ));
            return Ok(json!({
                "status": "skipped",
                "reason": "info_2_not_found",
                "message": "Campo Info_2 é obrigatório para processar tarefas"
            }));
        }

        // 🏗️ PRÉ-BUSCA DE CONTEXTO ORGANIZACIONAL (folder_id, list_id)
        //
        // RAZÃO: Fornecer contexto organizacional à IA para melhor classificação
        // BENEFÍCIO: IA pode considerar a estrutura organizacional ao determinar se é uma task
        let mut organizational_context = String::new();
        let mut folder_context_info: Option<OrganizationalContext> = None;
        
        log_info(&format!(
            "🏗️ INICIANDO PRÉ-BUSCA DE CONTEXTO ORGANIZACIONAL para Info_2: '{}'",
            info_2
        ));

        // Tentar obter contexto organizacional com fallback automático
        let fallback_enabled = std::env::var("ENABLE_ORGANIZATIONAL_FALLBACK")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        match execute_with_fallback(
            || async {
                match get_organizational_context_for_ai(&info_2).await {
                    Ok(context_info) => {
                        if let Some(ref ctx) = context_info {
                            log_info(&format!(
                                "✅ CONTEXTO ORGANIZACIONAL OBTIDO - Pasta: '{}' | Lista: '{}'",
                                ctx.folder_name, ctx.list_name
                            ));
                            Ok(serde_json::json!({
                                "success": true,
                                "context": context_info
                            }))
                        } else {
                            log_info("ℹ️ CONTEXTO ORGANIZACIONAL: Cliente não mapeado");
                            Ok(serde_json::json!({
                                "success": true,
                                "context": null
                            }))
                        }
                    },
                    Err(e) => {
                        // Verificar se é elegível para fallback
                        if is_fallback_eligible_error(&e) {
                            log_warning(&format!(
                                "⚠️ ERRO ELEGÍVEL PARA FALLBACK na busca organizacional: {}",
                                e
                            ));
                            return Err(e);
                        } else {
                            log_warning(&format!(
                                "⚠️ ERRO NÃO ELEGÍVEL na busca organizacional: {} | Prosseguindo",
                                e
                            ));
                            Ok(serde_json::json!({
                                "success": false,
                                "error": e.to_string()
                            }))
                        }
                    }
                }
            },
            "busca de contexto organizacional",
            payload,
            fallback_enabled,
        ).await {
            Ok(result) => {
                if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some(context_value) = result.get("context") {
                        if !context_value.is_null() {
                            // Deserializar context_info de volta
                            folder_context_info = serde_json::from_value(context_value.clone()).ok();
                            if let Some(ref ctx) = folder_context_info {
                                organizational_context = format!(
                                    "\nContexto Organizacional:\n- Pasta: {} (ID: {})\n- Lista: {} (ID: {})",
                                    ctx.folder_name, ctx.folder_id, ctx.list_name, ctx.list_id
                                );
                            }
                        } else {
                            organizational_context = "\nContexto Organizacional: Cliente não mapeado".to_string();
                        }
                    }
                } else {
                    // Se retornou do AppEngine via fallback, não temos contexto específico
                    organizational_context = "\nContexto Organizacional: Processado via fallback".to_string();
                }
            },
            Err(e) => {
                log_error(&format!(
                    "❌ FALHA CRÍTICA na busca de contexto organizacional: {}",
                    e
                ));
                organizational_context = "\nContexto Organizacional: Erro crítico".to_string();
            }
        }
        
        let responsavel_nome = extract_responsavel_nome_from_payload(payload).unwrap_or_default();
        
        // 🔤 MONTAGEM DE CONTEXTO ENRIQUECIDO PARA IA
        //
        // ESTRUTURA MELHORADA: Inclui contextos organizacionais e estruturados
        // OBJETIVO: Fornecer máximo contexto possível para classificação precisa
        let enriched_context = format!(
            "Dados da Conversa:\n- Origem: WhatsApp\n- Nome: {}\n- Telefone: {}\n- Mensagem: {}\n\nDados Organizacionais:\n- Cliente (Info_2): {}\n- Responsável: {}{}",
            nome,
            phone.as_deref().unwrap_or("N/A"),
            message,
            info_2,
            responsavel_nome,
            organizational_context
        );

        // Gerar prompt usando a configuração
        let formatted_prompt = prompt_config.generate_prompt(&enriched_context);

        log_info(&format!(
            "📝 PROMPT ENRIQUECIDO GERADO - ChatID: {} | Context size: {} chars | Prompt size: {} chars",
            chat_id.as_deref().unwrap_or("N/A"),
            enriched_context.len(),
            formatted_prompt.len()
        ));

        log_info(&format!(
            "🔍 CONTEXTO DETALHADO - Cliente: '{}' | Org Context: {} | Message Preview: '{}'",
            info_2,
            if folder_context_info.is_some() { "Disponível" } else { "Não disponível" },
            if message.len() > 100 {
                format!("{}...", &message[..100])
            } else {
                message.clone()
            }
        ));

        // 🤖 EXECUÇÃO DA CLASSIFICAÇÃO IA MELHORADA COM NOVA IMPLEMENTAÇÃO
        // INTEGRAÇÃO: Utilizando classify_content_with_ai() para classificação aprimorada
        // CONTEXTO: Aproveita todos os contextos organizacionais já extraídos
        log_info("🚀 EXECUTANDO CLASSIFICAÇÃO IA APRIMORADA com contexto organizacional...");
        
        // Preparar contexto organizacional para a nova função
        let organizational_context = folder_context_info.as_ref().map(|ctx|
            OrganizationalContext {
                folder_id: ctx.folder_id.clone(),
                folder_name: ctx.folder_name.clone(),
                list_id: ctx.list_id.clone(),
                list_name: ctx.list_name.clone(),
            }
        );
        
        // 🎯 EXECUÇÃO DA NOVA CLASSIFICAÇÃO IA COM TIMEOUT OTIMIZADO
        match tokio::time::timeout(
            std::time::Duration::from_secs(8),
            classify_content_with_ai(payload, organizational_context.as_ref())
        ).await {
            Ok(Ok(ai_result)) => {
                log_info(&format!(
                    "✅ CLASSIFICAÇÃO IA APRIMORADA CONCLUÍDA - ChatID: {} | Is_task: {} | Categoria: {} | Cliente: '{}'",
                    chat_id.as_deref().unwrap_or("N/A"),
                    ai_result.is_task,
                    ai_result.campanha.as_deref().unwrap_or("N/A"),
                    info_2
                ));
                
                // 📊 LOG DETALHADO PARA RASTREABILIDADE COMPLETA
                log_info("📊 RESULTADO DETALHADO DA CLASSIFICAÇÃO APRIMORADA:");
                log_info(&format!(
                    "   🎯 É tarefa: {} | Categoria: {} | Confiança: {:.2}%",
                    ai_result.is_task,
                    ai_result.campanha.as_deref().unwrap_or("N/A"),
                    ai_result.confidence
                ));
                log_info(&format!(
                    "   📝 Razão: '{}'",
                    ai_result.reason
                ));
                log_info(&format!(
                    "   🏢 Contexto organizacional utilizado: {}",
                    if organizational_context.is_some() { "Sim" } else { "Não" }
                ));
                log_info(&format!(
                    "   📋 Cliente (Info_2): '{}' | Responsável: '{}'",
                    info_2,
                    responsavel_nome
                ));
                
                // Converter AiClassificationResult para OpenAIClassification (compatibilidade)
                use crate::services::OpenAIClassification;
                OpenAIClassification {
                    reason: ai_result.reason,
                    is_activity: ai_result.is_task,
                    category: ai_result.campanha.clone(),
                    campanha: ai_result.campanha.clone(), // Usar campanha
                    description: Some(format!("Classificação IA: {} ({}% confiança)",
                        if ai_result.is_task { "Tarefa válida" } else { "Não é tarefa" },
                        (ai_result.confidence * 100.0) as u8
                    )),
                    space_name: None,
                    folder_name: organizational_context.as_ref().map(|ctx| ctx.folder_name.clone()),
                    list_name: organizational_context.as_ref().map(|ctx| ctx.list_name.clone()),
                    info_1: None,
                    info_2: Some(info_2.clone()),
                    tipo_atividade: ai_result.campanha.clone(),
                    sub_categoria: ai_result.sub_categoria.clone(),
                    subtasks: vec![],
                    status_back_office: None,
                }
            },
            Ok(Err(e)) => {
                log_error(&format!(
                    "❌ FALHA NA CLASSIFICAÇÃO IA APRIMORADA - ChatID: {} | Cliente: '{}' | Error: {}",
                    chat_id.as_deref().unwrap_or("N/A"),
                    info_2,
                    e
                ));
                return Err(AppError::InternalError(format!("IA classification failed: {}", e)));
            },
            Err(_) => {
                log_error(&format!(
                    "❌ TIMEOUT NA CLASSIFICAÇÃO IA APRIMORADA - ChatID: {} | Cliente: '{}' | Exceeded 8s",
                    chat_id.as_deref().unwrap_or("N/A"),
                    info_2
                ));
                return Err(AppError::Timeout("IA classification timeout".to_string()));
            }
        }
    };

    let is_activity = classification.is_activity;

    if is_activity {
        log_info(&format!("✅ Atividade identificada: {}", classification.reason));

        // NOVA LÓGICA SIMPLIFICADA:
        // 1. Extrai Info_2 do payload (já validado anteriormente)
        // 2. Se Info_2 vazio → processo foi encerrado antes da classificação IA
        // 3. Busca hierarquia do workspace unificada
        // 4. Verifica se alguma pasta é compatível com Info_2 (normalização)
        // 5. Se não encontrar pasta compatível → usa fallback se habilitado
        // 6. Se encontrar → verifica/cria lista do mês vigente
        // 7. Cria tarefa com folder_id e list_id determinados

        // Re-extrair Info_2 (já foi validado como não-vazio anteriormente)
        let info_2 = extract_info_2_from_payload(payload).unwrap_or_default();
        log_info(&format!("🔍 Validação simplificada: Info_2='{}'", info_2));

        // Inicializar serviço de hierarquia do workspace
        let secrets_service = services::SecretManagerService::new().await
            .map_err(|e| AppError::ConfigError(format!("Failed to create SecretsService: {}", e)))?;
        
        let api_token = secrets_service.get_clickup_api_token().await
            .map_err(|e| AppError::ConfigError(format!("Failed to get ClickUp token: {}", e)))?;

        let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
            .or_else(|_| std::env::var("CLICKUP_TEAM_ID")) // Fallback para compatibilidade
            .unwrap_or_else(|_| "9013037641".to_string()); // Workspace ID da Nordja

        let clickup_client = clickup::ClickUpClient::new(api_token.clone())
            .map_err(|e| AppError::ClickUpApi(format!("Failed to create ClickUp client: {}", e)))?;

        let mut hierarchy_service = services::WorkspaceHierarchyService::new(clickup_client, workspace_id.clone());

        // Validação simplificada - verifica se Info_2 é compatível com alguma pasta
        let validation_result = hierarchy_service.validate_and_find_target(&info_2).await
            .map_err(|e| AppError::InternalError(format!("Workspace validation failed: {}", e)))?;
if !validation_result.is_valid {
    log_warning(&format!("⚠️ Folder não encontrado para '{}', usando fallback do ClickUpService", info_2));
    
    // NOVA LÓGICA: Aplicar configurações customizadas mesmo no fallback
    let fallback_enabled = std::env::var("ENABLE_FALLBACK_PROCESSING")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase() == "true";
    
    if !fallback_enabled {
        log_info("ℹ️ Anotação de fallback desabilitada — apenas encaminhando para o App Engine");
        log_info("🔄 Encaminhando payload para App Engine...");
        return Ok(json!({
            "status": "forwarded_to_app_engine",
            "reason": "validation_failed_fallback_disabled",
            "validation_reason": validation_result.reason,
            "info_2": info_2
        }));
    }
    
    log_info("💡 Processando com configurações customizadas + fallback para pasta 'Clientes Inativos'");
    
    // Aplicar configurações customizadas com fallback
    return process_with_fallback_configurations(
        state,
        payload,
        &classification,
        &info_2,
        &api_token,
        &prompt_config
    ).await;
        }

        // ⚠️ VERIFICAÇÃO CRÍTICA: Se pasta ou lista não foram encontradas, encerrar como "não-cliente"
        // Implementado conforme checklist MCP para economia de recursos e fail-fast
        if validation_result.folder_id.is_none() || validation_result.list_id.is_none() {
            log_warning(&format!(
                "🚫 MCP CHECKLIST: Pasta ou lista vigente não encontrada para cliente '{}' - encerrando processamento",
                info_2
            ));
            log_info("❌ Motivo: Sistema não conseguiu localizar/criar estrutura organizacional necessária");
            return Ok(json!({
                "status": "skipped",
                "reason": "not_a_client",
                "message": "Não foi encontrada pasta ou lista vigente para este cliente"
            }));
        }

        // 🎯 MCP CHECKLIST: Extração dos IDs validados de pasta e lista vigente
        let folder_id = validation_result.folder_id.clone().unwrap();
        let folder_name = validation_result.folder_name.clone().unwrap();
        let list_id = validation_result.list_id.clone().unwrap();
        let list_name = validation_result.list_name.clone().unwrap();
        
        log_info(&format!(
            "🎯 MCP CHECKLIST: IDs extraídos com sucesso - Folder: '{}' ({}), List: '{}' ({})",
            folder_name, folder_id, list_name, list_id
        ));

        // 📋 ARMAZENAMENTO EXPLÍCITO DE CONTEXTO: IDs de Pasta e Lista
        //
        // OBJETIVO: Garantir rastreabilidade e disponibilidade dos IDs para etapas seguintes
        // CONTEXTO: Após validação bem-sucedida da hierarquia, os IDs devem ser explicitamente
        //          armazenados para uso posterior (criação de tarefa, análise IA, logs, etc.)
        //
        // ESTRUTURA: WorkspaceContext contém os identificadores essenciais do ClickUp
        // USO POSTERIOR: Disponível para todas as etapas seguintes do fluxo de processamento
        #[derive(Debug, Clone)]
        struct WorkspaceContext {
            /// ID da pasta no ClickUp onde a tarefa será criada
            folder_id: String,
            /// Nome da pasta para logs e rastreabilidade
            folder_name: String,
            /// ID da lista no ClickUp onde a tarefa será criada
            list_id: String,
            /// Nome da lista para logs e rastreabilidade
            list_name: String,
            /// Cliente identificado via Info_2 para contexto
            client_info_2: String,
        }

        let workspace_context = WorkspaceContext {
            folder_id: folder_id.clone(),
            folder_name: folder_name.clone(),
            list_id: list_id.clone(),
            list_name: list_name.clone(),
            client_info_2: info_2.clone(),
        };

        log_info(&format!(
            "✅ Validação aprovada: Pasta='{}' ({}), Lista='{}' ({})",
            folder_name, folder_id, list_name, list_id
        ));

        // 📊 MCP CHECKLIST: LOG DE RASTREABILIDADE - Contexto completo armazenado
        log_info(&format!(
            "📋 MCP CHECKLIST: WORKSPACE CONTEXT ARMAZENADO com lista vigente garantida"
        ));
        log_info(&format!(
            "   📁 Cliente: '{}' | Folder: '{}' (ID: {})",
            workspace_context.client_info_2,
            workspace_context.folder_name,
            workspace_context.folder_id
        ));
        log_info(&format!(
            "   📋 Lista vigente: '{}' (ID: {}) - disponível para próximas etapas do fluxo",
            workspace_context.list_name,
            workspace_context.list_id
        ));
        log_info(&format!(
            "   ✅ Rastreabilidade: IDs mantidos no WorkspaceContext para uso posterior"
        ));

        // Buscar assignee (responsável) se disponível
        let assignee_result = if let Some(ref responsavel) = extract_responsavel_nome_from_payload(payload) {
            log_info(&format!("👤 Buscando assignee para responsavel_nome: '{}'", responsavel));

            // Clonar para evitar move
            let assignee_api_token = api_token.clone();
            let assignee_workspace_id = workspace_id.clone();

            let mut assignee_finder = SmartAssigneeFinder::from_token(assignee_api_token, assignee_workspace_id)
                .map_err(|e| AppError::ClickUpApi(format!("Failed to create SmartAssigneeFinder: {}", e)))?;

            match assignee_finder.find_assignee_by_name(responsavel).await {
                Ok(Some(result)) => {
                    log_info(&format!(
                        "✅ Assignee encontrado: {} (user_id: {}, método: {:?}, confiança: {:.2})",
                        result.username,
                        result.user_id,
                        result.search_method,
                        result.confidence
                    ));
                    Some(result)
                }
                Ok(None) => {
                    log_warning(&format!(
                        "⚠️ Assignee não encontrado para '{}', tarefa será criada sem responsável",
                        responsavel
                    ));

                    None
                }
                Err(e) => {
                    log_error(&format!("❌ Erro ao buscar assignee: {}, continuando sem responsável", e));
                    None
                }
            }
        } else {
            log_info("ℹ️ Sem responsavel_nome no payload, tarefa será criada sem assignee");

            None
        };

        // Criar task_data
        let mut task_data = payload.to_clickup_task_data_with_ai(Some(&classification), &prompt_config).await;

        // Adicionar assignee ao task_data se encontrado
        if let Some(assignee_info) = assignee_result {
            if let Some(obj) = task_data.as_object_mut() {
                obj.insert("assignees".to_string(), serde_json::json!(vec![assignee_info.user_id]));
                log_info(&format!("✅ Assignee adicionado ao task_data: {}", assignee_info.username));
            }
        }

        // Processar resultado da validação
        // LÓGICA SIMPLIFICADA - criar tarefa diretamente
        let task_result = {
            // REMOVIDO: Bloco completo de configuração do campo "Cliente Solicitante"
            // Motivo: Eliminação da lógica do campo "Cliente Solicitante"
            // Anteriormente configurava o campo custom baseado no folder_name
            // e sincronizava com o ClickUp via CustomFieldManager
            
            log_info(&format!(
                "🎯 Criando tarefa diretamente na lista: {} (folder: {})",
                workspace_context.list_id, workspace_context.folder_id
            ));

            // Adicionar list_id ao task_data usando workspace_context
            if let Some(obj) = task_data.as_object_mut() {
                obj.insert("list_id".to_string(), serde_json::json!(workspace_context.list_id));
            }

            // Converter Value para Task tipada
            let task: clickup::Task = serde_json::from_value(task_data)?;

            // Deduplicação: checar se já existe tarefa com o mesmo título antes de criar
            let existing = state.clickup.find_existing_task_in_list(
                Some(&workspace_context.list_id),
                &task.name
            ).await;

            match existing {
                Ok(Some(_task_found)) => {
                    log_info(&format!("❗ Tarefa já existe no ClickUp com o mesmo título: '{}'. Não será criada nova task.", &task.name));
                    return Ok(serde_json::json!({
                        "status": "duplicate",
                        "message": "Tarefa já existente, não criada novamente",
                        "task_title": &task.name
                    }));
                }
                Ok(None) => {
                    // Só cria a task se não houver duplicata
                    match state.clickup.create_task(&task).await {
                        Ok(created_task) => {
                            let default_id = "?".to_string();
                            let task_id = created_task.id.as_ref().unwrap_or(&default_id);
                            
                            // 📋 LOG DETALHADO COM WORKSPACE CONTEXT: Rastreabilidade completa da tarefa criada
                            log_info(&format!(
                                "✅ TAREFA CRIADA COM SUCESSO - ID: {} | Cliente: '{}' | Folder: {} ({}) | List: {} ({})",
                                task_id,
                                workspace_context.client_info_2,
                                workspace_context.folder_name,
                                workspace_context.folder_id,
                                workspace_context.list_name,
                                workspace_context.list_id
                            ));
                            
                            serde_json::to_value(&created_task)
                                .unwrap_or_else(|_| serde_json::json!({"id": created_task.id}))
                        }
                        Err(e) => {
                            log_error(&format!("❌ Erro ao criar tarefa: {}", e));
                            return Err(AppError::ClickUpApi(e.to_string()));
                        }
                    }
                }
                Err(e) => {
                    log_error(&format!("❌ Erro ao buscar duplicata no ClickUp: {}", e));
                    return Err(AppError::ClickUpApi(e.to_string()));
                }
            }
        };
        // 📝 MCP CHECKLIST: MONTAGEM DE ANOTAÇÃO E RESPOSTA FINAL COMPLETA
        //
        // OBJETIVO: Criar anotação rica para o ChatGuru e resposta estruturada
        //          contendo todos os dados da tarefa criada e contexto organizacional
        //
        // DADOS INCLUÍDOS:
        // - task_id, task_url: Identificadores da tarefa criada
        // - classification: Resultado da análise IA (categoria, subcategoria)
        // - workspace_context: Estrutura organizacional completa
        // - assignee info: Responsável atribuído (se disponível)
        //
        let task_id = task_result.get("id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
        let task_url = task_result.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let task_name = task_result.get("name").and_then(|v| v.as_str()).unwrap_or("N/A");

        log_info(&format!(
            "📝 MCP CHECKLIST: PREPARANDO ANOTAÇÃO FINAL - Task ID: {} | URL presente: {}",
            task_id,
            !task_url.is_empty()
        ));

        // Extrair prioridade da tarefa criada para exibição
        let priority_stars = task_result.get("priority")
            .and_then(|p| p.get("orderindex"))
            .and_then(|o| o.as_str())
            .map(|s| match s {
                "1" => "4",  // Urgent = 4 estrelas
                "2" => "3",  // High = 3 estrelas
                "3" => "2",  // Normal = 2 estrelas
                _ => "1"     // Low = 1 estrela
            })
            .unwrap_or("N/A");

        // 📋 ANOTAÇÃO ENRIQUECIDA COM CONTEXTO COMPLETO
        let annotation = format!(
            "✅ Tarefa criada no ClickUp\\n\\n📋 Descrição: {}\\n🏷️ Categoria: {}\\n📂 Subcategoria: {}\\n⭐ Prioridade: {} estrela(s)\\n📁 Pasta: {}\\n📋 Lista: {}\\n👤 Cliente: {}\\n🔗 Link: {}",
            classification.reason,
            classification.campanha.as_deref().unwrap_or("N/A"),
            classification.sub_categoria.as_deref().unwrap_or("N/A"),
            priority_stars,
            workspace_context.folder_name,
            workspace_context.list_name,
            workspace_context.client_info_2,
            task_url
        );

        log_info(&format!(
            "📱 MCP CHECKLIST: ENVIANDO ANOTAÇÃO ao ChatGuru - Size: {} chars",
            annotation.len()
        ));

        // 📤 ENVIO DA ANOTAÇÃO AO CHATGURU
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!(
                "⚠️ MCP CHECKLIST: FALHA ao enviar anotação ao ChatGuru - Erro: {} | Processamento continua",
                e
            ));
            // Não falhar o processamento se anotação falhar - a tarefa foi criada com sucesso
        } else {
            log_info("✅ MCP CHECKLIST: ANOTAÇÃO ENVIADA COM SUCESSO ao ChatGuru");
        }

        // 📊 MCP CHECKLIST: RESPOSTA FINAL ESTRUTURADA
        //
        // OBJETIVO: Retornar resposta completa com todos os dados para rastreabilidade
        //          externa e possível uso por outros sistemas/webhooks
        //
        // ESTRUTURA INCLUI:
        // - status: Estado do processamento
        // - task_data: Informações completas da tarefa criada
        // - classification: Resultado da análise IA
        // - workspace_context: Estrutura organizacional utilizada
        // - annotation: Texto enviado ao ChatGuru para referência
        //
        let response = json!({
            "status": "processed",
            "is_activity": true,
            "message": "Tarefa criada com sucesso no ClickUp",
            "task_data": {
                "id": task_id,
                "name": task_name,
                "url": task_url,
                "priority_stars": priority_stars
            },
            "classification": {
                "reason": classification.reason,
                "category": classification.campanha,
                "subcategory": classification.sub_categoria,
                "is_activity": classification.is_activity
            },
            "workspace_context": {
                "folder_id": workspace_context.folder_id,
                "folder_name": workspace_context.folder_name,
                "list_id": workspace_context.list_id,
                "list_name": workspace_context.list_name,
                "client_info_2": workspace_context.client_info_2
            },
            "annotation_sent": annotation,
            "metadata": {
                "processing_timestamp": chrono::Utc::now().to_rfc3339(),
                "worker_version": "mcp_checklist_implementation"
            }
        });

        log_info(&format!(
            "✅ MCP CHECKLIST: PROCESSAMENTO CONCLUÍDO COM SUCESSO"
        ));
        log_info(&format!(
            "   🎯 Tarefa ID: {} | Cliente: '{}' | Pasta: '{}'",
            task_id,
            workspace_context.client_info_2,
            workspace_context.folder_name
        ));
        log_info(&format!(
            "   📋 Classificação: {} | Categoria: {} | Prioridade: {} estrelas",
            if classification.is_activity { "✅ É Tarefa" } else { "❌ Não é Tarefa" },
            classification.campanha.as_deref().unwrap_or("N/A"),
            priority_stars
        ));

        Ok(response)
    } else {
        // ============================================================================
        // MCP CHECKLIST: PROCESSAMENTO DE NÃO TAREFA
        // ============================================================================
        // OBJETIVO: Criar anotação rica para o ChatGuru explicando por que não é tarefa
        // GARANTIA: Rastreabilidade completa e contexto claro para o usuário
        //
        log_info(&format!("❌ NÃO É TAREFA DETECTADO: {}", classification.reason));

        // Extrair contexto adicional para enriquecer a anotação
        let cliente = extract_info_2_from_payload(payload).unwrap_or_else(|| "Cliente não identificado".to_string());
        let atendente = _extract_info_1_from_payload(payload).unwrap_or_else(|| "Atendente não identificado".to_string());
        let chat_id = extract_chat_id_from_payload(payload).unwrap_or_else(|| "N/A".to_string());

        // Criar anotação rica com contexto completo
        let annotation = format!(
            "🚫 NÃO É TAREFA\n\n📋 **Motivo:** {}\n\n🏢 **Cliente:** {}\n👤 **Atendente:** {}\n🆔 **Chat ID:** {}\n\n⏰ **Processado em:** {}",
            classification.reason,
            cliente,
            atendente,
            chat_id,
            chrono::Utc::now().format("%d/%m/%Y %H:%M:%S UTC")
        );

        log_info(&format!(
            "📝 MCP CHECKLIST: PREPARANDO ANOTAÇÃO DE NÃO TAREFA"
        ));
        log_info(&format!(
            "   📋 Motivo: {} | Cliente: '{}' | Atendente: '{}'",
            classification.reason,
            cliente,
            atendente
        ));

        // Enviar anotação enriquecida ao ChatGuru
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!(
                "⚠️ MCP CHECKLIST: FALHA ao enviar anotação de não tarefa ao ChatGuru - Erro: {} | ChatID: {}",
                e,
                chat_id
            ));
        } else {
            log_info(&format!(
                "✅ MCP CHECKLIST: ANOTAÇÃO DE NÃO TAREFA ENVIADA COM SUCESSO ao ChatGuru | ChatID: {}",
                chat_id
            ));
        }

        // Resposta estruturada para rastreabilidade
        let response = json!({
            "status": "processed",
            "is_activity": false,
            "message": "Mensagem analisada e classificada como não tarefa",
            "classification": {
                "reason": classification.reason,
                "category": classification.campanha,
                "subcategory": classification.sub_categoria,
                "is_activity": classification.is_activity
            },
            "context": {
                "client_info_2": cliente,
                "attendant_info_1": atendente,
                "chat_id": chat_id
            },
            "annotation_sent": annotation,
            "metadata": {
                "processing_timestamp": chrono::Utc::now().to_rfc3339(),
                "worker_version": "mcp_checklist_non_task_implementation"
            }
        });

        log_info(&format!(
            "✅ MCP CHECKLIST: PROCESSAMENTO DE NÃO TAREFA CONCLUÍDO | Motivo: '{}' | ChatID: {}",
            classification.reason,
            chat_id
        ));

        Ok(response)
    }
}

/// Envia anotação de volta ao ChatGuru
async fn send_annotation_to_chatguru(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    annotation: &str,
) -> AppResult<()> {
    // ✅ Usa o cliente ChatGuru centralizado do AppState
    let chatguru_service = &state.chatguru_client;
    let default_endpoint = "https://s15.chatguru.app/api/v1".to_string();
    let api_endpoint = state.settings.chatguru.api_endpoint
        .as_ref()
        .unwrap_or(&default_endpoint);

    let chat_id = extract_chat_id_from_payload(payload);
    let phone = extract_phone_from_payload(payload);

    if let Some(chat_id) = chat_id {
        let phone_str = phone.as_deref().unwrap_or("");
        
        // Log detalhado antes de enviar
        log_info(&format!(
            "� ENVIANDO PARA CHATGURU - ChatID: {} | Phone: {} | Endpoint: {} | Size: {} chars",
            chat_id,
            phone_str,
            api_endpoint,
            annotation.len()
        ));
        
        chatguru_service.add_annotation(&chat_id, phone_str, annotation).await?;
        
        log_info(&format!(
            "✅ ANOTAÇÃO CONFIRMADA NO CHATGURU - ChatID: {} | Success",
            chat_id
        ));
    } else {
        log_warning("⚠️ CHAT_ID NÃO ENCONTRADO - Não foi possível enviar anotação ao ChatGuru");
    }

    Ok(())
}

// ============================================================================
// Funções auxiliares de extração de dados
// ============================================================================

fn extract_nome_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            if p.nome.is_empty() {
                "Desconhecido".to_string()
            } else {
                p.nome.clone()
            }
        },
        WebhookPayload::EventType(p) => p.data.lead_name.clone().unwrap_or_else(|| "Desconhecido".to_string()),
        WebhookPayload::Generic(p) => p.nome.clone().unwrap_or_else(|| "Desconhecido".to_string()),
    }
}

fn extract_message_from_payload(payload: &WebhookPayload) -> String {
    match payload {
        WebhookPayload::ChatGuru(p) => p.texto_mensagem.clone(),
        WebhookPayload::EventType(p) => p.data.annotation.clone().unwrap_or_default(),
        WebhookPayload::Generic(p) => p.mensagem.clone().unwrap_or_default(),
    }
}

fn extract_phone_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => Some(p.celular.clone()),
        WebhookPayload::EventType(p) => p.data.phone.clone(),
        WebhookPayload::Generic(_) => None,
    }
}

fn extract_chat_id_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => p.chat_id.clone(),
        WebhookPayload::EventType(_) => None,  // EventType não tem chat_id direto
        WebhookPayload::Generic(_) => None,
    }
}

/// Extrai Info_1 (EMPRESA CLIENTE - apenas para campo personalizado) dos campos personalizados
/// Info_1 = dados.campos_personalizados.Info_1
/// Usado APENAS para preencher o campo personalizado "Conta cliente"
/// NÃO é usado para determinar Space ou Folder
fn _extract_info_1_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            p.campos_personalizados.get("Info_1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        },
        _ => None,
    }
}

/// Extrai Info_2 (NOME DO CLIENTE) dos campos personalizados do ChatGuru
///
/// FLUXO DE BUSCA POR NOME:
/// Info_2 = dados.campos_personalizados.Info_2 (ex: "Nexcode", "Gabriel Benarros")
///
/// IMPORTANTE: Este nome é usado para:
/// 1. Buscar pasta no ClickUp por SIMILARIDADE DE NOME (SmartFolderFinder)
/// 2. Preencher campo personalizado "Solicitante" na tarefa
///
/// NÃO é usado:
/// - Campos customizados das tarefas para determinar estrutura
/// - Mapeamento via banco de dados (Cloud SQL)
/// - Dependências de configuração de campos personalizados
///
/// Exemplo: "Nexcode" → Busca pasta com nome similar a "Nexcode"
fn extract_info_2_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            p.campos_personalizados.get("Info_2")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        },
        _ => None,
    }
}

/// Extrai responsavel_nome (ATENDENTE - determina SPACE) do payload do ChatGuru
/// responsavel_nome = dados.responsavel_nome
/// Usado para determinar qual Space usar (Anne Souza, Gabriel Moreno, William Duarte, etc.)
/// Exemplo: "anne" → Space "Anne Souza"
fn extract_responsavel_nome_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            p.responsavel_nome.clone()
        },
        _ => None,
    }
}

// ============================================================================
// FUNÇÕES OBSOLETAS - MIGRADAS PARA src/models/payload.rs
// ============================================================================
//
// NOVA IMPLEMENTAÇÃO:
// - Subcategorias e Estrelas: src/models/payload.rs:333-362 (função chatguru_to_clickup_with_ai)
// - Usa configuração YAML: config/ai_prompt.yaml
// - Mapeamento via AiPromptConfig::load_default()
// - Log de estrelas: payload.rs:348-353
//
// FLUXO ATUAL:
// 1. OpenAI Service → classifica mensagem (category, sub_categoria)
// 2. ClickUp Service → chama payload.to_clickup_task_data_with_ai()
// 3. Payload.rs → mapeia subcategorias/estrelas via YAML
// 4. ClickUp Service → envia para API via create_task_from_json()
//
// As funções abaixo foram mantidas para referência histórica
// ============================================================================


// ============================================================================
// FALLBACK com configurações customizadas
// ============================================================================

/// Processa a tarefa com configurações customizadas mesmo quando usa fallback
///
/// Esta função é chamada quando a validação da pasta falha, mas queremos aplicar
/// as configurações customizadas (estrelas, categorias) antes de usar o fallback
/// para a pasta "Clientes Inativos".
async fn process_with_fallback_configurations(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    classification: &crate::services::OpenAIClassification,
    info_2: &str,
    api_token: &str,
    prompt_config: &chatguru_clickup_middleware::services::prompts::AiPromptConfig,
) -> AppResult<Value> {
    log_info(&format!("🔧 Iniciando processamento com configurações customizadas + fallback para Info_2: '{}'", info_2));
    
    // Configurar pasta de fallback "Clientes Inativos"
    let fallback_folder_name = "Clientes Inativos";
    let fallback_folder_id = std::env::var("FALLBACK_FOLDER_ID")
        .unwrap_or_else(|_| "90161002969".to_string()); // ID da pasta "Clientes Inativos"
    
    log_info(&format!("📂 Usando pasta de fallback: '{}' (ID: {})", fallback_folder_name, fallback_folder_id));
    
    // Determinar o nome da lista baseado no cliente (Info_2) e mês atual
    let now = chrono::Utc::now();
    let list_name = if info_2.is_empty() {
        format!("Clientes Diversos - {} {}",
            get_month_name_pt(now.month() as u32),
            now.year()
        )
    } else {
        format!("{} - {} {}",
            info_2,
            get_month_name_pt(now.month() as u32),
            now.year()
        )
    };
    
    log_info(&format!("📋 Nome da lista calculado: '{}'", list_name));
    
    // Criar cliente ClickUp para verificar/criar lista
    let clickup_client = clickup::ClickUpClient::new(api_token.to_string())
        .map_err(|e| AppError::ClickUpApi(format!("Failed to create ClickUp client: {}", e)))?;
    
    // Buscar ou criar a lista no folder de fallback
    let list_id = match clickup_client.find_list_by_name(&fallback_folder_id, &list_name).await {
        Ok(Some(existing_list)) => {
            log_info(&format!("✅ Lista encontrada: '{}' (ID: {})", list_name, existing_list.id));
            existing_list.id
        },
        Ok(None) => {
            log_info(&format!("📝 Criando nova lista: '{}'", list_name));
            
            // Criar nova lista na pasta de fallback
            let new_list = clickup::CreateListRequest {
                name: list_name.clone(),
                content: Some(format!("Lista criada automaticamente para cliente: {}", info_2)),
                due_date: None,
                priority: None,
                assignee: None,
                status: None,
            };
            
            match clickup_client.create_list(&fallback_folder_id, &new_list).await {
                Ok(created_list) => {
                    log_info(&format!("✅ Lista criada com sucesso: '{}' (ID: {})", list_name, created_list.id));
                    created_list.id
                },
                Err(e) => {
                    log_error(&format!("❌ Erro ao criar lista: {}", e));
                    return Err(AppError::ClickUpApi(format!("Failed to create fallback list: {}", e)));
                }
            }
        },
        Err(e) => {
            log_error(&format!("❌ Erro ao buscar lista: {}", e));
            return Err(AppError::ClickUpApi(format!("Failed to search for list: {}", e)));
        }
    };
    
    log_info(&format!("🎯 Lista determinada: '{}' (ID: {})", list_name, list_id));
    
    // Buscar assignee (responsável) se disponível
    let assignee_result = if let Some(ref responsavel) = extract_responsavel_nome_from_payload(payload) {
        log_info(&format!("👤 Buscando assignee para responsavel_nome: '{}'", responsavel));
        
        let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
            .or_else(|_| std::env::var("CLICKUP_TEAM_ID"))
            .unwrap_or_else(|_| "9013037641".to_string());
        
        let mut assignee_finder = SmartAssigneeFinder::from_token(api_token.to_string(), workspace_id)
            .map_err(|e| AppError::ClickUpApi(format!("Failed to create SmartAssigneeFinder: {}", e)))?;
        
        match assignee_finder.find_assignee_by_name(responsavel).await {
            Ok(Some(result)) => {
                log_info(&format!(
                    "✅ Assignee encontrado: {} (user_id: {}, método: {:?}, confiança: {:.2})",
                    result.username,
                    result.user_id,
                    result.search_method,
                    result.confidence
                ));
                Some(result)
            }
            Ok(None) => {
                log_warning(&format!(
                    "⚠️ Assignee não encontrado para '{}', tarefa será criada sem responsável",
                    responsavel
                ));
                None
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao buscar assignee: {}, continuando sem responsável", e));
                None
            }
        }
    } else {
        log_info("ℹ️ Sem responsavel_nome no payload, tarefa será criada sem assignee");
        None
    };
    
    // Criar task_data com configurações customizadas (APLICAR ESTRELAS E CATEGORIAS)
    let mut task_data = payload.to_clickup_task_data_with_ai(Some(classification), &prompt_config).await;
    
    log_info(&format!("🌟 Configurações customizadas aplicadas: categoria='{}', subcategoria='{}'",
        classification.campanha.as_deref().unwrap_or("N/A"),
        classification.sub_categoria.as_deref().unwrap_or("N/A")
    ));
    
    // Adicionar assignee ao task_data se encontrado
    if let Some(assignee_info) = assignee_result {
        if let Some(obj) = task_data.as_object_mut() {
            obj.insert("assignees".to_string(), serde_json::json!(vec![assignee_info.user_id]));
            log_info(&format!("✅ Assignee adicionado ao task_data: {}", assignee_info.username));
        }
    }
    
    // REMOVIDO: Bloco completo de configuração do campo "Cliente Solicitante" para fallback
    // Motivo: Eliminação da lógica do campo "Cliente Solicitante"
    // Anteriormente configurava client_display_name baseado em fallback_folder_name + Info_2
    // e sincronizava com o ClickUp via CustomFieldManager
    
    // Adicionar list_id ao task_data
    if let Some(obj) = task_data.as_object_mut() {
        obj.insert("list_id".to_string(), serde_json::json!(list_id));
    }
    
    // Converter Value para Task tipada
    let task: clickup::Task = serde_json::from_value(task_data)?;
    
    // Deduplicação: checar se já existe tarefa com o mesmo título antes de criar
    let existing = state.clickup.find_existing_task_in_list(
        Some(&list_id),
        &task.name
    ).await;
    
    let task_result = match existing {
        Ok(Some(_task_found)) => {
            log_info(&format!("❗ Tarefa já existe no ClickUp com o mesmo título: '{}'. Não será criada nova task.", &task.name));
            return Ok(serde_json::json!({
                "status": "duplicate",
                "message": "Tarefa já existente, não criada novamente",
                "task_title": &task.name,
                "fallback_used": true,
                "folder_name": fallback_folder_name,
                "list_name": list_name
            }));
        }
        Ok(None) => {
            // Só cria a task se não houver duplicata
            match state.clickup.create_task(&task).await {
                Ok(created_task) => {
                    log_info(&format!("✅ Tarefa criada com configurações customizadas: {}", created_task.id.as_ref().unwrap_or(&"?".to_string())));
                    serde_json::to_value(&created_task)
                        .unwrap_or_else(|_| serde_json::json!({"id": created_task.id}))
                }
                Err(e) => {
                    log_error(&format!("❌ Erro ao criar tarefa: {}", e));
                    return Err(AppError::ClickUpApi(e.to_string()));
                }
            }
        }
        Err(e) => {
            log_error(&format!("❌ Erro ao buscar duplicata no ClickUp: {}", e));
            return Err(AppError::ClickUpApi(e.to_string()));
        }
    };

    Ok(task_result)
}


/// 🏗️ FUNÇÃO AUXILIAR: Busca contexto organizacional para enriquecer classificação IA
///
/// OBJETIVO: Fornecer informações de folder_id e list_id à IA para melhorar a classificação
/// BENEFÍCIO: IA pode considerar a estrutura organizacional ao determinar se é uma task
///
/// PARÂMETROS:
/// - info_2: Cliente identificado via campos personalizados
///
/// RETORNO:
/// - Ok(Some(context)): Contexto organizacional encontrado
/// - Ok(None): Cliente não mapeado, mas sem erro
/// - Err: Erro na busca (não deve interromper o processamento principal)
///
/// IMPLEMENTAÇÃO: Utiliza WorkspaceHierarchyService para busca rápida de estrutura organizacional
async fn get_organizational_context_for_ai(info_2: &str) -> Result<Option<OrganizationalContext>, AppError> {
    // 📋 LOG DE INÍCIO DA BUSCA DE CONTEXTO ORGANIZACIONAL
    log_info(&format!(
        "🔍 INICIANDO BUSCA DE CONTEXTO ORGANIZACIONAL - Cliente: '{}'",
        info_2
    ));

    // Validação de entrada
    if info_2.is_empty() {
        log_warning("⚠️ CONTEXTO ORGANIZACIONAL: Info_2 vazio, retornando contexto nulo");
        return Ok(None);
    }

    // 🔑 OBTENÇÃO DE CREDENCIAIS E CONFIGURAÇÃO
    let secrets_service = match services::SecretManagerService::new().await {
        Ok(service) => service,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao inicializar SecretsService: {}",
                e
            ));
            return Ok(None); // Não é erro crítico, retorna contexto nulo
        }
    };
    
    let api_token = match secrets_service.get_clickup_api_token().await {
        Ok(token) => token,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao obter token ClickUp: {}",
                e
            ));
            return Ok(None); // Não é erro crítico, retorna contexto nulo
        }
    };

    let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
        .or_else(|_| std::env::var("CLICKUP_TEAM_ID"))
        .unwrap_or_else(|_| "9013037641".to_string()); // Default workspace da Nordja

    // 🏗️ INICIALIZAÇÃO DO WORKSPACE HIERARCHY SERVICE
    let clickup_client = match clickup::ClickUpClient::new(api_token.clone()) {
        Ok(client) => client,
        Err(e) => {
            log_warning(&format!(
                "⚠️ CONTEXTO ORGANIZACIONAL: Falha ao criar ClickUpClient: {}",
                e
            ));
            return Ok(None); // Não é erro crítico
        }
    };

    let mut hierarchy_service = services::WorkspaceHierarchyService::new(
        clickup_client,
        workspace_id.clone()
    );

    // 🎯 VALIDAÇÃO E BUSCA DE ESTRUTURA ORGANIZACIONAL
    log_info(&format!(
        "🎯 EXECUTANDO VALIDAÇÃO DE ESTRUTURA para cliente '{}'",
        info_2
    ));

    match hierarchy_service.validate_and_find_target(info_2).await {
        Ok(validation_result) => {
            if validation_result.is_valid
                && validation_result.folder_id.is_some()
                && validation_result.list_id.is_some() {
                
                let context = OrganizationalContext {
                    folder_id: validation_result.folder_id.clone().unwrap(),
                    folder_name: validation_result.folder_name.clone().unwrap_or_else(|| "Pasta Desconhecida".to_string()),
                    list_id: validation_result.list_id.clone().unwrap(),
                    list_name: validation_result.list_name.clone().unwrap_or_else(|| "Lista Desconhecida".to_string()),
                };

                log_info(&format!(
                    "✅ CONTEXTO ORGANIZACIONAL ENCONTRADO - Pasta: '{}' ({}), Lista: '{}' ({})",
                    context.folder_name,
                    context.folder_id,
                    context.list_name,
                    context.list_id
                ));

                Ok(Some(context))
            } else {
                log_info(&format!(
                    "ℹ️ CLIENTE NÃO MAPEADO '{}': {} | Validation: folder={}, list={}",
                    info_2,
                    validation_result.reason,
                    validation_result.folder_id.is_some(),
                    validation_result.list_id.is_some()
                ));
                Ok(None) // Cliente não mapeado, mas não é erro
            }
        },
        Err(e) => {
            log_warning(&format!(
                "⚠️ ERRO NA BUSCA DE CONTEXTO ORGANIZACIONAL para '{}': {}",
                info_2,
                e
            ));
            Ok(None) // Não é erro crítico, retorna contexto nulo para não interromper classificação IA
        }
    }
    
}

/// Retorna o nome do mês em português
fn get_month_name_pt(month: u32) -> &'static str {
    match month {
        1 => "JANEIRO",
        2 => "FEVEREIRO",
        3 => "MARÇO",
        4 => "ABRIL",
        5 => "MAIO",
        6 => "JUNHO",
        7 => "JULHO",
        8 => "AGOSTO",
        9 => "SETEMBRO",
        10 => "OUTUBRO",
        11 => "NOVEMBRO",
        12 => "DEZEMBRO",
        _ => "DESCONHECIDO"
    }
}

// ============================================================================
// App Engine Fallback
// ============================================================================

/// Verifica se um erro é elegível para fallback para AppEngine
///
/// # Condições para Fallback:
/// 1. Timeout ou erro de conexão com CloudRun/ClickUp
/// 2. Autorização negada (401/403) nas consultas de spaces, pastas ou listas
/// 3. Indisponibilidade do serviço CloudRun
fn is_fallback_eligible_error(error: &AppError) -> bool {
    match error {
        // Timeouts são sempre elegíveis
        AppError::Timeout(_) => true,
        
        // Erros internos que podem indicar problemas de conexão
        AppError::InternalError(msg) => {
            let msg_lower = msg.to_lowercase();
            msg_lower.contains("timeout") ||
            msg_lower.contains("connection") ||
            msg_lower.contains("network") ||
            msg_lower.contains("dns") ||
            msg_lower.contains("refused") ||
            msg_lower.contains("unreachable") ||
            msg_lower.contains("failed to connect")
        },
        
        // Erros do ClickUp API com códigos de autorização
        AppError::ClickUpApi(msg) => {
            let msg_lower = msg.to_lowercase();
            msg_lower.contains("401") ||
            msg_lower.contains("403") ||
            msg_lower.contains("unauthorized") ||
            msg_lower.contains("forbidden") ||
            msg_lower.contains("authentication") ||
            msg_lower.contains("permission denied") ||
            msg_lower.contains("timeout") ||
            msg_lower.contains("connection")
        },
        
        // Erros de configuração relacionados a autenticação
        AppError::ConfigError(msg) => {
            let msg_lower = msg.to_lowercase();
            msg_lower.contains("token") ||
            msg_lower.contains("auth") ||
            msg_lower.contains("credential")
        },
        
        // Outros tipos não são elegíveis por padrão
        _ => false,
    }
}

/// Encaminha payload original do ChatGuru para o App Engine (fallback inteligente)
///
/// # Objetivo:
/// Garante continuidade operacional quando o CloudRun está indisponível ou com problemas
/// de autenticação. Mantém contexto completo e logs detalhados para rastreabilidade.
///
/// # Condições de Acionamento:
/// - Timeout ou erro de conexão com CloudRun/ClickUp
/// - Autorização negada (401/403) nas consultas de spaces, pastas ou listas
/// - Indisponibilidade do serviço CloudRun
///
/// # Parâmetros:
/// - `payload`: Payload original do ChatGuru
/// - `fallback_reason`: Motivo detalhado que causou o fallback
/// - `original_error`: Erro original que triggou o fallback
///
/// # Retorno:
/// - `Ok(Value)`: Response estruturada indicando sucesso do fallback
/// - `Err(AppError)`: Erro se o próprio AppEngine falhar
async fn forward_to_app_engine_with_context(
    payload: &WebhookPayload,
    fallback_reason: &str,
    original_error: &str
) -> AppResult<Value> {
    const APP_ENGINE_URL: &str = "https://buzzlightear.rj.r.appspot.com/webhook";
    
    // Extrair contexto básico para logs detalhados
    let chat_id = extract_chat_id_from_payload(payload).unwrap_or_else(|| "N/A".to_string());
    let info_2 = extract_info_2_from_payload(payload).unwrap_or_else(|| "N/A".to_string());
    let nome = extract_nome_from_payload(payload);

    log_info(&format!(
        "🔄 INICIANDO FALLBACK PARA APP ENGINE - ChatID: {} | Cliente: '{}' | Sender: {}",
        chat_id, info_2, nome
    ));
    log_info(&format!(
        "   📋 Motivo do fallback: {} | Erro original: {}",
        fallback_reason, original_error
    ));

    // Serializar o payload completo mantendo contexto
    let mut payload_json = serde_json::to_value(payload)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize payload: {}", e)))?;

    // Adicionar metadados de fallback para rastreabilidade no AppEngine
    if let Some(obj) = payload_json.as_object_mut() {
        obj.insert("_fallback_metadata".to_string(), serde_json::json!({
            "triggered_by": "cloud_run_middleware",
            "fallback_reason": fallback_reason,
            "original_error": original_error,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "chat_id": chat_id,
            "client_info_2": info_2
        }));
    }

    // Cliente HTTP com configuração robusta para AppEngine
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45)) // Timeout mais generoso para AppEngine
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    log_info(&format!(
        "📡 ENVIANDO PAYLOAD PARA APP ENGINE - URL: {} | Payload size: {} bytes",
        APP_ENGINE_URL,
        serde_json::to_string(&payload_json).unwrap_or_default().len()
    ));

    // Executar POST para AppEngine com tratamento robusto
    let response_result = client
        .post(APP_ENGINE_URL)
        .header("Content-Type", "application/json")
        .header("X-Forwarded-From", "cloud-run-middleware-fallback")
        .header("X-Fallback-Reason", fallback_reason)
        .header("X-Original-Error", original_error)
        .header("X-Chat-ID", &chat_id)
        .header("X-Client-Info", &info_2)
        .json(&payload_json)
        .send()
        .await;

    match response_result {
        Ok(response) => {
            let status = response.status();
            let response_body = response.text().await.unwrap_or_default();

            if status.is_success() {
                log_info(&format!(
                    "✅ FALLBACK PARA APP ENGINE SUCESSO - Status: {} | ChatID: {} | Cliente: '{}'",
                    status, chat_id, info_2
                ));
                log_info(&format!(
                    "   📋 Response body: {} | Size: {} chars",
                    if response_body.len() > 200 {
                        format!("{}...", &response_body[..200])
                    } else {
                        response_body.clone()
                    },
                    response_body.len()
                ));

                // Retornar resposta estruturada indicando sucesso do fallback
                Ok(serde_json::json!({
                    "status": "processed_via_fallback",
                    "fallback_target": "app_engine",
                    "app_engine_status": status.as_u16(),
                    "app_engine_response": response_body,
                    "fallback_metadata": {
                        "reason": fallback_reason,
                        "original_error": original_error,
                        "chat_id": chat_id,
                        "client_info_2": info_2,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }
                }))
            } else {
                log_error(&format!(
                    "❌ FALLBACK PARA APP ENGINE FALHOU - Status: {} | ChatID: {} | Cliente: '{}'",
                    status, chat_id, info_2
                ));
                log_error(&format!(
                    "   📋 Error body: {} | Fallback reason: {}",
                    response_body, fallback_reason
                ));
                
                Err(AppError::InternalError(format!(
                    "App Engine fallback failed - Status: {}, Body: {}, Original reason: {}",
                    status, response_body, fallback_reason
                )))
            }
        },
        Err(e) => {
            log_error(&format!(
                "❌ ERRO DE REDE NO FALLBACK PARA APP ENGINE - ChatID: {} | Cliente: '{}' | Network Error: {}",
                chat_id, info_2, e
            ));
            log_error(&format!(
                "   📋 Fallback reason: {} | Original error: {}",
                fallback_reason, original_error
            ));

            Err(AppError::InternalError(format!(
                "Failed to connect to App Engine fallback: {} (Original: {})",
                e, original_error
            )))
        }
    }
}

/// Executa operação com fallback automático para AppEngine
///
/// # Funcionalidade:
/// Wrapper inteligente que executa uma operação e, em caso de falha elegível,
/// automaticamente aciona o fallback para AppEngine mantendo contexto completo.
///
/// # Parâmetros:
/// - `operation`: Closure async que executa a operação principal
/// - `operation_name`: Nome da operação para logs
/// - `payload`: Payload original para fallback
/// - `fallback_enabled`: Se fallback está habilitado (padrão: true)
///
/// # Retorno:
/// - `Ok(Value)`: Resultado da operação ou do fallback
/// - `Err(AppError)`: Erro se ambos falharem ou fallback desabilitado
async fn execute_with_fallback<F, Fut>(
    operation: F,
    operation_name: &str,
    payload: &WebhookPayload,
    fallback_enabled: bool,
) -> AppResult<Value>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = AppResult<Value>> + Send,
{
    // Executar operação principal
    match operation().await {
        Ok(result) => {
            log_info(&format!("✅ {} executado com sucesso", operation_name));
            Ok(result)
        },
        Err(error) => {
            log_warning(&format!(
                "⚠️ FALHA EM {} - Error: {} | Verificando elegibilidade para fallback",
                operation_name, error
            ));

            // Verificar se erro é elegível para fallback
            if !is_fallback_eligible_error(&error) {
                log_info(&format!(
                    "❌ Erro não elegível para fallback - Tipo: {:?} | Operation: {}",
                    std::mem::discriminant(&error), operation_name
                ));
                return Err(error);
            }

            if !fallback_enabled {
                log_info(&format!(
                    "❌ Fallback desabilitado - Operation: {} | Error: {}",
                    operation_name, error
                ));
                return Err(error);
            }

            log_info(&format!(
                "🔄 ACIONANDO FALLBACK PARA APP ENGINE - Operation: {} | Error elegível detectado",
                operation_name
            ));

            // Acionar fallback para AppEngine
            forward_to_app_engine_with_context(
                payload,
                &format!("Falha em {}", operation_name),
                &format!("{}", error),
            ).await
        }
    }
}

/// Encaminha payload original do ChatGuru para o App Engine (compatibilidade)
///
/// Mantida para compatibilidade com código existente.
/// Para novos usos, prefira `forward_to_app_engine_with_context` ou `execute_with_fallback`.
async fn _forward_to_app_engine(payload: &WebhookPayload) -> AppResult<()> {
    match forward_to_app_engine_with_context(
        payload,
        "Legacy fallback call",
        "Unspecified error"
    ).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

