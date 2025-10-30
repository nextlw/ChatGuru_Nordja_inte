 /// Worker Handler: Processa mensagens do Pub/Sub
///
/// Arquitetura:
/// 1. Recebe payload RAW do Pub/Sub via HTTP POST
/// 2. Processa com OpenAI para classificação
/// 3. Se for atividade, cria tarefa no ClickUp
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

use chatguru_clickup_middleware::models::payload::WebhookPayload;
use chatguru::ChatGuruClient;
use chatguru_clickup_middleware::utils::{AppResult, AppError};
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;
// Usar services do crate clickup ao invés de duplicar no main project
use clickup::folders::SmartFolderFinder;
use clickup::assignees::SmartAssigneeFinder;
use clickup::fields::CustomFieldManager;

// Configuração de retry
const MAX_RETRY_ATTEMPTS: u32 = 3;

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
                let (final_result, annotation_opt) = if let Some(ref ia_service) = state.ia_service {
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
                                            log_info(&format!("✅ Áudio processado: {} caracteres", result.extracted_content.len()));
                                            (Some(result.extracted_content), result.annotation)
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

                // ENVIAR ANOTAÇÃO IMEDIATAMENTE AO CHATGURU (independente de ser atividade ou não)
                // Enviar DEPOIS de modificar o payload para evitar borrow checker issues
                if let Some(annotation) = annotation_opt {
                    let annotation_preview = if annotation.len() > 100 {
                        format!("{}...", &annotation[..100])
                    } else {
                        annotation.clone()
                    };
                    
                    log_info(&format!(
                        "📤 ENVIANDO ANOTAÇÃO DE MÍDIA - ChatID: {} | Type: {} | Preview: \"{}\"",
                        chat_id.as_deref().unwrap_or("N/A"),
                        processing_type,
                        annotation_preview
                    ));
                    
                    if let Err(e) = send_annotation_to_chatguru(&state, &payload, &annotation).await {
                        log_warning(&format!(
                            "⚠️ FALHA NA ANOTAÇÃO DE MÍDIA - ChatID: {} | Error: {}",
                            chat_id.as_deref().unwrap_or("N/A"),
                            e
                        ));
                    } else {
                        log_info(&format!(
                            "✅ ANOTAÇÃO DE MÍDIA ENVIADA - ChatID: {} | Size: {} chars",
                            chat_id.as_deref().unwrap_or("N/A"),
                            annotation.len()
                        ));
                    }
                }
            }
        }
    }

    // Extrair force_classification se presente
    let force_classification = envelope.get("force_classification");
// Processar mensagem com tratamento robusto de resposta
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
        // Classificar com IaService (OpenAI)
        log_info(&format!(
            "🤖 INICIANDO CLASSIFICAÇÃO IA - ChatID: {} | Sender: {}",
            chat_id.as_deref().unwrap_or("N/A"),
            nome
        ));
        
        let ia_service = state.ia_service.as_ref()
            .ok_or_else(|| AppError::InternalError("IaService não disponível no AppState".to_string()))?;

        // Carregar configuração de prompt
        use chatguru_clickup_middleware::services::prompts::AiPromptConfig;
        let prompt_config = AiPromptConfig::load_default().await
            .map_err(|e| AppError::InternalError(format!("Failed to load prompt config: {}", e)))?;

        // Montar contexto
        let context = format!(
            "Campanha: WhatsApp\nOrigem: whatsapp\nNome: {}\nMensagem: {}\nTelefone: {}",
            nome, message, phone.as_deref().unwrap_or("N/A")
        );

        // Gerar prompt usando a configuração
        let formatted_prompt = prompt_config.generate_prompt(&context);

        log_info(&format!(
            "📝 PROMPT GERADO - ChatID: {} | Context size: {} chars | Prompt size: {} chars",
            chat_id.as_deref().unwrap_or("N/A"),
            context.len(),
            formatted_prompt.len()
        ));

        // Classificar com IA com timeout (máx 8 segundos)
        match tokio::time::timeout(
            std::time::Duration::from_secs(8),
            ia_service.classify_activity(&formatted_prompt)
        ).await {
            Ok(Ok(c)) => {
                log_info(&format!(
                    "✅ CLASSIFICAÇÃO IA CONCLUÍDA - ChatID: {} | Is_activity: {} | Category: {} | Confidence: {}",
                    chat_id.as_deref().unwrap_or("N/A"),
                    c.is_activity,
                    c.category.as_deref().unwrap_or("N/A"),
                    "N/A" // Confidence não está disponível na struct atual
                ));
                c
            },
            Ok(Err(e)) => {
                log_error(&format!(
                    "❌ FALHA NA CLASSIFICAÇÃO IA - ChatID: {} | Error: {}",
                    chat_id.as_deref().unwrap_or("N/A"),
                    e
                ));
                return Err(AppError::InternalError(format!("IA classification failed: {}", e)));
            },
            Err(_) => {
                log_error(&format!(
                    "❌ TIMEOUT NA CLASSIFICAÇÃO IA - ChatID: {} | Exceeded 8s",
                    chat_id.as_deref().unwrap_or("N/A")
                ));
                return Err(AppError::Timeout("IA classification timeout".to_string()));
            }
        }
    };

    let is_activity = classification.is_activity;

    if is_activity {
        log_info(&format!("✅ Atividade identificada: {}", classification.reason));

        // NOVA LÓGICA COM SMARTFOLDERFINDER:
        // 1. Extrai Info_2 (nome do cliente)
        // 2. Busca folder via API do ClickUp com fuzzy matching
        // 3. Fallback: busca em tarefas anteriores pelo campo "Cliente Solicitante"
        // 4. Retorna folder_id + list_id do mês atual

        let client_name = extract_info_2_from_payload(payload)
            .unwrap_or_else(|| extract_nome_from_payload(payload));

        log_info(&format!("🔍 SmartFolderFinder: Buscando folder para Info_2='{}'", client_name));
        log_info(&format!("📋 Debug campos: Info_2 (cliente)={:?}, Info_1 (empresa)={:?}, responsavel_nome (atendente)={:?}",
            extract_info_2_from_payload(payload),
            extract_info_1_from_payload(payload),
            extract_responsavel_nome_from_payload(payload)
        ));

        // Inicializar SmartFolderFinder
        let api_token = std::env::var("CLICKUP_API_TOKEN")
            .or_else(|_| std::env::var("clickup_api_token"))
            .or_else(|_| std::env::var("CLICKUP_TOKEN"))
            .map_err(|_| AppError::ConfigError("CLICKUP_API_TOKEN não configurado".to_string()))?;

        let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
            .or_else(|_| std::env::var("CLICKUP_TEAM_ID")) // Fallback para compatibilidade
            .unwrap_or_else(|_| "9013037641".to_string()); // Workspace ID da Nordja

        // Clonar para uso posterior no assignee_finder
        let folder_api_token = api_token.clone();
        let folder_workspace_id = workspace_id.clone();

        let mut finder = SmartFolderFinder::from_token(folder_api_token, folder_workspace_id)
            .map_err(|e| AppError::ClickUpApi(format!("Failed to create SmartFolderFinder: {}", e)))?;

        // Buscar folder de forma inteligente
        let folder_result = match finder.find_folder_for_client(&client_name).await {
            Ok(Some(result)) => {
                log_info(&format!(
                    "✅ Folder encontrado: {} (id: {}, método: {:?}, confiança: {:.2})",
                    result.folder_name,
                    result.folder_id,
                    result.search_method,
                    result.confidence
                ));

                if let (Some(list_id), Some(list_name)) = (result.list_id.clone(), result.list_name.clone()) {
                    log_info(&format!("📋 Lista do mês: {} (id: {})", list_name, list_id));
                }

                Some(result)
            }
            Ok(None) => {
                log_warning(&format!(
                    "⚠️ Folder não encontrado para '{}', usando fallback do ClickUpService",
                    client_name
                ));
                None
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao buscar folder: {}, usando fallback", e));
                None
            }
        };

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
        let mut task_data = payload.to_clickup_task_data_with_ai(Some(&classification)).await;

        // Adicionar assignee ao task_data se encontrado
        if let Some(assignee_info) = assignee_result {
            if let Some(obj) = task_data.as_object_mut() {
                obj.insert("assignees".to_string(), serde_json::json!(vec![assignee_info.user_id]));
                log_info(&format!("✅ Assignee adicionado ao task_data: {}", assignee_info.username));
            }
        }

        // Processar resultado do SmartFolderFinder
        let task_result = if let Some(folder_info) = folder_result {
            if let Some(list_id) = folder_info.list_id {
                // Garantir que "Cliente Solicitante" corresponda ao folder encontrado
                log_info(&format!(
                    "📝 Configurando 'Cliente Solicitante' para: '{}'",
                    folder_info.folder_name
                ));

                let custom_field_manager = CustomFieldManager::from_token(api_token.clone())
                    .map_err(|e| AppError::ClickUpApi(format!("Failed to create CustomFieldManager: {}", e)))?;

                match custom_field_manager
                    .ensure_client_solicitante_option(&list_id, &folder_info.folder_name)
                    .await
                {
                    Ok(client_field) => {
                        log_info("✅ Campo 'Cliente Solicitante' configurado");

                        // Adicionar/substituir o campo custom no task_data
                        if let Some(obj) = task_data.as_object_mut() {
                            // Buscar custom_fields existentes ou criar array vazio
                            let custom_fields = obj
                                .entry("custom_fields")
                                .or_insert_with(|| serde_json::json!([]));

                            if let Some(fields_array) = custom_fields.as_array_mut() {
                                // Remover campo "Cliente Solicitante" se já existir
                                fields_array.retain(|f| {
                                    f.get("id")
                                        .and_then(|id| id.as_str())
                                        != Some("0ed63eec-1c50-4190-91c1-59b4b17557f6")
                                });

                                // Adicionar novo valor
                                fields_array.push(client_field);

                                log_info(&format!(
                                    "✅ 'Cliente Solicitante' sincronizado com folder: '{}'",
                                    folder_info.folder_name
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        log_warning(&format!(
                            "⚠️ Erro ao configurar 'Cliente Solicitante': {}, continuando sem o campo",
                            e
                        ));
                    }
                }
                log_info(&format!(
                    "🎯 Criando tarefa diretamente na lista: {} (folder: {})",
                    list_id, folder_info.folder_id
                ));

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
                                log_info(&format!("✅ Tarefa criada via SmartFolderFinder: {}", created_task.id.as_ref().unwrap_or(&"?".to_string())));
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
            } else {
                // Tem folder mas não tem lista do mês - encaminhar para App Engine
                log_warning(&format!(
                    "⚠️ Folder '{}' encontrado mas sem lista do mês, encaminhando para App Engine",
                    folder_info.folder_name
                ));

                // Enviar anotação de warning ao ChatGuru
                let warning_annotation = format!(
                    "⚠️ **LISTA DO MÊS NÃO ENCONTRADA**\n\n\
                    📂 Pasta: '{}'\n\
                    📋 Lista mensal não criada ainda\n\n\
                    ℹ️ A tarefa será processada pelo sistema legado (App Engine).\n\n\
                    ✅ **Ação necessária**: Crie a lista do mês atual na pasta '{}' no ClickUp.",
                    folder_info.folder_name,
                    folder_info.folder_name
                );

                if let Err(e) = send_annotation_to_chatguru(&state, &payload, &warning_annotation).await {
                    log_warning(&format!("⚠️ Não foi possível enviar anotação de warning: {}", e));
                }

                // Enviar payload original para o App Engine
                forward_to_app_engine(&payload).await?;

                log_info("✅ Payload encaminhado para App Engine com sucesso - Processamento encerrado");

                // Retornar resposta de sucesso sem criar task no ClickUp
                return Ok(json!({
                    "status": "forwarded_to_app_engine",
                    "message": "Folder encontrado mas sem lista do mês, payload encaminhado para App Engine",
                    "folder_name": folder_info.folder_name,
                    "app_engine_url": "https://buzzlightear.rj.r.appspot.com/webhook"
                }));
            }
        } else {
            // Não encontrou folder, encaminhar para App Engine
            log_warning(&format!("⚠️ Folder não encontrado para '{}'", client_name));

            log_info("ℹ️ Anotação de fallback desabilitada — apenas encaminhando para o App Engine");

            // Enviar payload original para o App Engine e encerrar processamento
            forward_to_app_engine(&payload).await?;

            log_info("✅ Payload encaminhado para App Engine com sucesso - Processamento encerrado");

            // Retornar resposta de sucesso sem criar task no ClickUp
            // O App Engine será responsável por todo o processamento daqui em diante
            return Ok(json!({
                "status": "forwarded_to_app_engine",
                "message": "Cliente não encontrado no Cloud Run, payload encaminhado para App Engine",
                "client_name": client_name,
                "app_engine_url": "https://buzzlightear.rj.r.appspot.com/webhook"
            }));
        };

        // Montar anotação com informações da task
        let task_id = task_result.get("id").and_then(|v| v.as_str()).unwrap_or("N/A");
        let task_url = task_result.get("url").and_then(|v| v.as_str()).unwrap_or("");

        // CORREÇÃO: Não incluir descrição de mídia na anotação da tarefa se já foi enviada separadamente
        // A descrição do PDF/áudio/imagem já foi enviada como anotação de mídia nas linhas 402-429
        // Incluir aqui seria duplicação desnecessária
        let media_description_section = String::new();

        let annotation = format!(
            "✅ Tarefa criada no ClickUp\n\n📋 Descrição: {}\n🏷️ Categoria: {}\n📂 Subcategoria: {}\n⭐ Prioridade: {} estrela(s)\n🔗 Link: {}{}",
            classification.reason,
            classification.campanha.as_deref().unwrap_or("N/A"),
            classification.sub_categoria.as_deref().unwrap_or("N/A"),
            // Extrair prioridade da task_result se disponível
            task_result.get("priority")
                .and_then(|p| p.get("orderindex"))
                .and_then(|o| o.as_str())
                .map(|s| match s {
                    "1" => "4",
                    "2" => "3",
                    "3" => "2",
                    _ => "1"
                })
                .unwrap_or("N/A"),
            task_url,
            media_description_section
        );

        // Enviar anotação ao ChatGuru
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!("⚠️  Falha ao enviar anotação ao ChatGuru: {}", e));
            // Não falhar o processamento se anotação falhar
        }

        Ok(json!({
            "status": "processed",
            "is_activity": true,
            "task_id": task_id,
            "annotation": annotation
        }))
    } else {
        log_info(&format!("❌ Não é atividade: {}", classification.reason));

        let annotation = format!("Não é uma tarefa: {}", classification.reason);

        // Apenas enviar anotação
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!("⚠️  Falha ao enviar anotação ao ChatGuru: {}", e));
        }

        Ok(json!({
            "status": "processed",
            "is_activity": false,
            "annotation": annotation
        }))
    }
}

/// Envia anotação de volta ao ChatGuru
async fn send_annotation_to_chatguru(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    annotation: &str,
) -> AppResult<()> {
    let api_token = state.settings.chatguru.api_token.clone()
        .unwrap_or_else(|| "default_token".to_string());
    let api_endpoint = state.settings.chatguru.api_endpoint.clone()
        .unwrap_or_else(|| "https://s15.chatguru.app/api/v1".to_string());
    let account_id = state.settings.chatguru.account_id.clone()
        .unwrap_or_else(|| "default_account".to_string());

    let chatguru_service = ChatGuruClient::new(api_token, api_endpoint.clone(), account_id);

    let chat_id = extract_chat_id_from_payload(payload);
    let phone = extract_phone_from_payload(payload);

    if let Some(chat_id) = chat_id {
        let phone_str = phone.as_deref().unwrap_or("");
        
        // Log detalhado antes de enviar
        log_info(&format!(
            "📡 ENVIANDO PARA CHATGURU - ChatID: {} | Phone: {} | Endpoint: {} | Size: {} chars",
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
fn extract_info_1_from_payload(payload: &WebhookPayload) -> Option<String> {
    match payload {
        WebhookPayload::ChatGuru(p) => {
            p.campos_personalizados.get("Info_1")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        },
        _ => None,
    }
}

/// Extrai Info_2 (NOME DO SOLICITANTE - campo personalizado) dos campos personalizados
/// Info_2 = dados.campos_personalizados.Info_2
/// Usado para preencher o campo personalizado "Solicitante" (não determina estrutura)
/// Exemplo: "João Silva" → Campo personalizado "Solicitante"
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

/// FUNÇÃO OBSOLETA - NÃO MAIS UTILIZADA
///
/// NOVA IMPLEMENTAÇÃO: src/models/payload.rs:240-441 (custom_fields)
/// A preparação de campos personalizados agora usa configuração YAML
/// e está integrada diretamente na conversão do payload
#[allow(dead_code)]
async fn prepare_custom_fields(
    payload: &WebhookPayload,
    classification: &chatguru_clickup_middleware::services::OpenAIClassification,
    _nome: &str,
) -> Vec<Value> {
    let mut custom_fields = Vec::new();

    // IDs reais dos campos personalizados (do script categorize_tasks.js)

    // 1. Campo: Categoria* (dropdown) - ID real do ClickUp
    if let Some(category) = &classification.category {
        custom_fields.push(json!({
            "id": "eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a", // ID real do campo Categoria*
            "value": category // Categoria determinada pela classificação IA
        }));
    }

    // 2. Campo: SubCategoria (dropdown) - ID real do ClickUp
    if let Some(subcategory) = determine_subcategoria(classification) {
        custom_fields.push(json!({
            "id": "5333c095-eb40-4a5a-b0c2-76bfba4b1094", // ID real do campo SubCategoria
            "value": subcategory
        }));
    }

    // 3. Campo: Estrelas (rating) - ID real do ClickUp
    let stars = determine_estrelas(classification, payload).await;
    custom_fields.push(json!({
        "id": "83afcb8c-2866-498f-9c62-8ea9666b104b", // ID real do campo Estrelas
        "value": stars // Valor numérico de 1 a 4
    }));

    custom_fields
}

/// FUNÇÃO OBSOLETA - NÃO MAIS UTILIZADA
///
/// NOVA IMPLEMENTAÇÃO:
/// - OpenAI Service já retorna `sub_categoria` classificada
/// - Mapeamento de IDs via config/ai_prompt.yaml
/// - Processamento em src/models/payload.rs:333-362
///
/// A determinação de subcategorias agora é feita pela IA e mapeada via YAML,
/// não mais por palavra-chave hardcoded
#[allow(dead_code)]
fn determine_subcategoria(classification: &chatguru_clickup_middleware::services::OpenAIClassification) -> Option<String> {
    // Análise de palavras-chave da mensagem/descrição para determinar subcategoria
    let message_text = classification.reason.to_lowercase();
    
    // MAPEAMENTO EXATO do categorize_tasks.js - KEYWORD_MAPPING
    // Logística
    if message_text.contains("motoboy") || message_text.contains("entrega") || message_text.contains("retirada") {
        Some("Corrida de motoboy".to_string())
    } else if message_text.contains("sedex") || message_text.contains("correio") {
        Some("Motoboy + Correios e envios internacionais".to_string())
    } else if message_text.contains("lalamove") {
        Some("Lalamove".to_string())
    } else if message_text.contains("uber") || message_text.contains("99") {
        Some("Transporte Urbano (Uber/99)".to_string())
    } else if message_text.contains("taxista") {
        Some("Corridas com Taxistas".to_string())
    }
    // Plano de Saúde
    else if message_text.contains("reembolso") || message_text.contains("bradesco saúde") || message_text.contains("plano de saúde") {
        Some("Reembolso Médico".to_string())
    }
    // Compras
    else if message_text.contains("mercado") {
        Some("Mercados".to_string())
    } else if message_text.contains("farmácia") {
        Some("Farmácia".to_string())
    } else if message_text.contains("presente") {
        Some("Presentes".to_string())
    } else if message_text.contains("shopper") {
        Some("Shopper".to_string())
    } else if message_text.contains("papelaria") {
        Some("Papelaria".to_string())
    } else if message_text.contains("petshop") {
        Some("Petshop".to_string())
    } else if message_text.contains("ingresso") {
        Some("Ingressos".to_string())
    }
    // Assuntos Pessoais
    else if message_text.contains("troca") {
        Some("Troca de titularidade".to_string())
    } else if message_text.contains("internet") {
        Some("Internet e TV por Assinatura".to_string())
    } else if message_text.contains("telefone") {
        Some("Telefone".to_string())
    } else if message_text.contains("conserto") {
        Some("Consertos na Casa".to_string())
    } else if message_text.contains("assistência") {
        Some("Assistência Técnica".to_string())
    }
    // Financeiro
    else if message_text.contains("pagamento") {
        Some("Rotina de Pagamentos".to_string())
    } else if message_text.contains("boleto") {
        Some("Emissão de boletos".to_string())
    } else if message_text.contains("nota fiscal") {
        Some("Emissão de NF".to_string())
    }
    // Viagens
    else if message_text.contains("passagem") {
        Some("Passagens Aéreas".to_string())
    } else if message_text.contains("hospedagem") || message_text.contains("hotel") {
        Some("Hospedagens".to_string())
    } else if message_text.contains("check in") {
        Some("Checkins (Early/Late)".to_string())
    } else if message_text.contains("bagagem") {
        Some("Extravio de Bagagens".to_string())
    }
    // Agendamentos
    else if message_text.contains("consulta") {
        Some("Consultas".to_string())
    } else if message_text.contains("exame") {
        Some("Exames".to_string())
    } else if message_text.contains("vacina") {
        Some("Vacinas".to_string())
    } else if message_text.contains("manicure") {
        Some("Manicure".to_string())
    } else if message_text.contains("cabeleireiro") {
        Some("Cabeleleiro".to_string())
    }
    // Lazer
    else if message_text.contains("restaurante") || message_text.contains("reserva") {
        Some("Reserva de restaurantes/bares".to_string())
    } else if message_text.contains("festa") {
        Some("Planejamento de festas".to_string())
    }
    // Documentos
    else if message_text.contains("passaporte") {
        Some("Passaporte".to_string())
    } else if message_text.contains("cnh") {
        Some("CNH".to_string())
    } else if message_text.contains("cidadania") {
        Some("Cidadanias".to_string())
    } else if message_text.contains("visto") {
        Some("Vistos e Vistos Eletrônicos".to_string())
    } else if message_text.contains("certidão") {
        Some("Certidões".to_string())
    } else if message_text.contains("contrato") {
        Some("Contratos/Procurações".to_string())
    }
    // Fallback: usar categoria padrão
    else if let Some(category) = &classification.category {
        match category.as_str() {
            "Logística" => Some("Corrida de motoboy".to_string()),
            "Plano de Saúde" => Some("Reembolso Médico".to_string()),
            "Compras" => Some("Mercados".to_string()),
            "Agendamentos" => Some("Consultas".to_string()),
            "Lazer" => Some("Reserva de restaurantes/bares".to_string()),
            "Viagens" => Some("Passagens Aéreas".to_string()),
            "Financeiro" => Some("Rotina de Pagamentos".to_string()),
            "Documentos" => Some("Passaporte".to_string()),
            "Assuntos Pessoais" => Some("Telefone".to_string()),
            _ => Some("Consultas".to_string()) // Padrão geral
        }
    } else {
        None
    }
}

/// FUNÇÃO OBSOLETA - NÃO MAIS UTILIZADA
///
/// NOVA IMPLEMENTAÇÃO:
/// - Mapeamento de estrelas via config/ai_prompt.yaml
/// - Processamento em src/models/payload.rs:348-353
/// - Log automático: "✨ Tarefa classificada: 'categoria' > 'subcategoria' (X estrela(s))"
///
/// As estrelas agora são determinadas pela configuração YAML baseada na
/// subcategoria retornada pela classificação IA
#[allow(dead_code)]
async fn determine_estrelas(
    classification: &chatguru_clickup_middleware::services::OpenAIClassification,
    _payload: &WebhookPayload,
) -> i32 {
    use chatguru_clickup_middleware::services::prompts::AiPromptConfig;

    // Carregar configuração do YAML
    let config = match AiPromptConfig::load_default().await {
        Ok(cfg) => cfg,
        Err(e) => {
            log_warning(&format!("Failed to load AI prompt config for stars: {}, using fallback", e));
            return 1; // Fallback direto
        }
    };

    // Usar categoria e subcategoria retornadas pelo OpenAI para buscar as estrelas
    if let (Some(category), Some(sub_categoria)) = (&classification.category, &classification.sub_categoria) {
        if let Some(stars) = config.get_subcategory_stars(category, sub_categoria) {
            log_info(&format!("⭐ Estrelas determinadas via YAML: {} ({}→{})",
                stars, category, sub_categoria));
            return stars as i32;
        } else {
            log_warning(&format!("Subcategoria '{}' não encontrada no YAML para categoria '{}', usando fallback",
                sub_categoria, category));
        }
    }

    // Fallback: 1 estrela padrão
    log_info("Using fallback: 1 star");
    1
}

// ============================================================================
// App Engine Fallback
// ============================================================================

/// Encaminha payload original do ChatGuru para o App Engine (fallback)
///
/// Usado quando o SmartFolderFinder não consegue encontrar o folder do cliente.
/// O App Engine processa o payload com sua própria lógica e pode ter outros
/// folders/listas cadastrados.
async fn forward_to_app_engine(payload: &WebhookPayload) -> AppResult<()> {
    const APP_ENGINE_URL: &str = "https://buzzlightear.rj.r.appspot.com/webhook";

    log_info("🔄 Encaminhando payload para App Engine...");

    // Serializar o payload completo
    let payload_json = serde_json::to_value(payload)
        .map_err(|e| AppError::InternalError(format!("Failed to serialize payload: {}", e)))?;

    // Fazer POST para o App Engine
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let response = client
        .post(APP_ENGINE_URL)
        .header("Content-Type", "application/json")
        .header("X-Forwarded-From", "cloud-run-middleware")
        .json(&payload_json)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to forward to App Engine: {}", e)))?;

    let status = response.status();

    if status.is_success() {
        let response_body = response.text().await.unwrap_or_default();
        log_info(&format!("✅ App Engine response ({}): {}", status, response_body));
        Ok(())
    } else {
        let error_body = response.text().await.unwrap_or_default();
        log_error(&format!("❌ App Engine returned error ({}): {}", status, error_body));
        Err(AppError::InternalError(format!(
            "App Engine returned status {}: {}",
            status, error_body
        )))
    }
}