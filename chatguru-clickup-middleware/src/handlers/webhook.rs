/// Webhook Handler: Recebe payload do ChatGuru e adiciona à fila
///
/// Arquitetura Unificada Event-Driven:
/// 1. Webhook ACK imediato (<100ms)
/// 2. Adiciona mensagem à fila (MessageQueueService)
/// 3. Callback processa automaticamente quando:
///    - 8 mensagens acumuladas OU
///    - 180 segundos transcorridos
/// 4. Callback envia batch para Pub/Sub
/// 5. Worker processa de forma assíncrona
///
/// Benefícios:
/// - Arquitetura consistente e centralizada
/// - Rate limiting automático via batching + Pub/Sub
/// - Retry e dead-letter queues gerenciados pelo GCP
/// - Nenhuma lógica de negócio no webhook
/// - Uma única rota de processamento via callback

use axum::{
    extract::{Request, State},
    response::Json,
    body::Body,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::time::Instant;
use uuid;

use crate::utils::AppError;
use crate::utils::logging::*;
use crate::utils::{truncate_safe, truncate_with_suffix};
use chatguru_clickup_middleware::AppState;
use chatguru_clickup_middleware::models::payload::ChatGuruPayload;

// Importar função de publicação direta (definida em main.rs)
use crate::publish_single_message_to_pubsub;

/// Processa mídia imediatamente (antes de expirar URLs do S3)
///
/// # Argumentos
/// * `state` - AppState com IA Service e ChatGuru client
/// * `payload` - Payload original do ChatGuru
///
/// # Retorna
/// - `Some((synthetic_payload, is_audio))` - Se mídia foi processada com sucesso
///   - `synthetic_payload`: Payload sintético com conteúdo extraído
///   - `is_audio`: `true` se é áudio, `false` caso contrário
/// - `None` - Se não há mídia ou processamento falhou (payload original deve ser usado)
async fn process_media_immediately(
    state: &Arc<AppState>,
    payload: &mut Value,
) -> Option<(Value, bool)> {
    log_info("🔍 Verificando presença de mídia no payload...");

    // Tentar parsear como ChatGuruPayload para acessar métodos de normalização
    let mut chatguru_payload: ChatGuruPayload = match serde_json::from_value(payload.clone()) {
        Ok(p) => p,
        Err(e) => {
            log_warning(&format!("⚠️ Não foi possível parsear como ChatGuruPayload: {}", e));
            return None;
        }
    };

    // Normalizar campos de mídia
    chatguru_payload.normalize_media_fields();

    // Verificar se há mídia
    let media_url = chatguru_payload.media_url.as_ref()?;
    let media_type = chatguru_payload.media_type.as_ref()?;

    if media_url.is_empty() {
        return None;
    }

    log_info(&format!("📎 Mídia detectada: {} ({})", media_url, media_type));

    // Verificar se IA Service está disponível
    let ia_service = match state.ia_service.as_ref() {
        Some(service) => service,
        None => {
            log_error("❌ IA Service não disponível - skipping processamento de mídia");
            return None;
        }
    };

    // Determinar tipo de mídia e processar
    let processed_result = if media_type.contains("audio") || media_type.contains("ptt") || media_type.contains("voice") {
        // ÁUDIO: Baixar e transcrever
        log_info("🎤 Processando áudio...");

        match ia_service.download_audio(media_url).await {
            Ok(audio_bytes) => {
                let extension = media_url
                    .split('.')
                    .last()
                    .and_then(|ext| ext.split('?').next())
                    .unwrap_or("ogg");
                let filename = format!("audio.{}", extension);

                match ia_service.transcribe_audio(&audio_bytes, &filename).await {
                    Ok(transcription) => {
                        log_info(&format!("✅ Áudio transcrito: {} caracteres", transcription.len()));

                        // Enviar anotação ao ChatGuru
                        let annotation = format!(
                            "🎵 **Áudio Transcrito**\n\n\"{}\"\n\nℹ️ A transcrição foi processada automaticamente.",
                            transcription
                        );

                        let phone_number = chatguru_payload.celular.as_str();
                        if let Err(e) = state.chatguru().send_confirmation_message(phone_number, None, &annotation).await {
                            log_warning(&format!("⚠️ Falha ao enviar anotação ao ChatGuru: {}", e));
                        } else {
                            log_info("✅ Anotação enviada ao ChatGuru com sucesso");
                        }

                        // Preparar payload sintético
                        let mut synthetic_payload = chatguru_payload.clone();
                        synthetic_payload.texto_mensagem = if synthetic_payload.texto_mensagem.is_empty() {
                            transcription.clone()
                        } else {
                            format!("{}\n\n[Mídia processada]: {}", synthetic_payload.texto_mensagem, transcription)
                        };
                        synthetic_payload._is_synthetic = Some(true);
                        synthetic_payload._original_media_type = Some(media_type.clone());
                        synthetic_payload.media_url = None;
                        synthetic_payload.media_type = None;
                        synthetic_payload.url_arquivo = None;
                        synthetic_payload.tipo_mensagem = None;

                        // Converter para Value
                        match serde_json::to_value(&synthetic_payload) {
                            Ok(payload_value) => Some((payload_value, true)), // true = é áudio
                            Err(e) => {
                                log_error(&format!("❌ Erro ao serializar payload sintético: {}", e));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log_error(&format!("❌ Erro ao transcrever áudio: {}", e));
                        None
                    }
                }
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao baixar áudio: {}", e));
                None
            }
        }
    } else if media_type.contains("image") {
        // IMAGEM: Baixar e descrever
        log_info("🖼️ Processando imagem...");

        match ia_service.download_image(media_url).await {
            Ok(image_bytes) => {
                    match ia_service.describe_image(&image_bytes).await {
                        Ok(description) => {
                            log_info(&format!("✅ Imagem descrita: {} caracteres", description.len()));

                            // Preparar payload sintético para imagem
                            let mut synthetic_payload = chatguru_payload.clone();
                            synthetic_payload.texto_mensagem = if synthetic_payload.texto_mensagem.is_empty() {
                                description.clone()
                            } else {
                                format!("{}\n\n[Mídia processada]: {}", synthetic_payload.texto_mensagem, description)
                            };
                            synthetic_payload._is_synthetic = Some(true);
                            synthetic_payload._original_media_type = Some(media_type.clone());
                            synthetic_payload.media_url = None;
                            synthetic_payload.media_type = None;
                            synthetic_payload.url_arquivo = None;
                            synthetic_payload.tipo_mensagem = None;

                            match serde_json::to_value(&synthetic_payload) {
                                Ok(payload_value) => Some((payload_value, false)), // false = não é áudio
                                Err(e) => {
                                    log_error(&format!("❌ Erro ao serializar payload sintético: {}", e));
                                    None
                                }
                            }
                        }
                    Err(e) => {
                        log_error(&format!("❌ Erro ao descrever imagem: {}", e));
                        None
                    }
                }
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao baixar imagem: {}", e));
                None
            }
        }
    } else if media_type.contains("pdf") || media_type.contains("application/pdf") {
        // PDF: Baixar e extrair texto
        log_info("📄 Processando PDF...");

        match ia_service.download_file(media_url, "PDF").await {
            Ok(pdf_bytes) => {
                match ia_service.process_pdf(&pdf_bytes).await {
                    Ok(text) => {
                        log_info(&format!("✅ PDF processado: {} caracteres extraídos", text.len()));

                        // Preparar payload sintético para PDF
                        let mut synthetic_payload = chatguru_payload.clone();
                        synthetic_payload.texto_mensagem = if synthetic_payload.texto_mensagem.is_empty() {
                            text.clone()
                        } else {
                            format!("{}\n\n[Mídia processada]: {}", synthetic_payload.texto_mensagem, text)
                        };
                        synthetic_payload._is_synthetic = Some(true);
                        synthetic_payload._original_media_type = Some(media_type.clone());
                        synthetic_payload.media_url = None;
                        synthetic_payload.media_type = None;
                        synthetic_payload.url_arquivo = None;
                        synthetic_payload.tipo_mensagem = None;

                        match serde_json::to_value(&synthetic_payload) {
                            Ok(payload_value) => Some((payload_value, false)), // false = não é áudio
                            Err(e) => {
                                log_error(&format!("❌ Erro ao serializar payload sintético: {}", e));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log_error(&format!("❌ Erro ao processar PDF: {}", e));
                        None
                    }
                }
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao baixar PDF: {}", e));
                None
            }
        }
    } else {
        log_warning(&format!("⚠️ Tipo de mídia não suportado: {}", media_type));
        None
    };

    // processed_result já retorna (Value, bool) ou None
    // Cada tipo de mídia já cria seu próprio payload sintético
    match processed_result {
        Some(result) => {
            log_info("✅ Payload sintético criado com sucesso");
            Some(result)
        }
        None => None,
    }
}

/// Handler principal do webhook
/// Retorna Success imediatamente após enviar para Pub/Sub
pub async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<Value>, AppError> {
    let start_time = Instant::now();
    let request_id = truncate_safe(&uuid::Uuid::new_v4().to_string(), 8).to_string(); // ID único para tracking

    log_info(&format!(
        "🔍 WEBHOOK INICIADO - RequestID: {} | Endpoint: {} | Method: {}",
        request_id, "/webhooks/chatguru", "POST"
    ));

    // Extrair body como bytes
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to read request body: {}", e)))?;

    log_info(&format!(
        "📦 BODY EXTRAÍDO - RequestID: {} | Size: {} bytes",
        request_id, body_bytes.len()
    ));

    // Validar UTF-8
    let body_str = String::from_utf8(body_bytes.to_vec())
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in request body: {}", e)))?;

    // Parsear JSON para extrair chat_id
    let payload: Value = serde_json::from_str(&body_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid JSON payload: {}", e)))?;

    log_info(&format!(
        "✅ JSON PARSEADO - RequestID: {} | Success",
        request_id
    ));

    // LOG DO PAYLOAD COMPLETO (RAW) para debug no GCloud
    log_info(&format!(
        "📋 PAYLOAD RAW COMPLETO - RequestID: {} | JSON: {}",
        request_id,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "Error serializing payload".to_string())
    ));

    // Extrair chat_id do payload
    let chat_id = payload
        .get("chat_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Extrair informações adicionais para logging
    let sender_name = payload
        .get("sender_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let message_type = payload
        .get("message_type")
        .and_then(|v| v.as_str())
        .unwrap_or("text");

    // Verificar AMBOS os formatos de mídia: media_url (antigo) e url_arquivo (novo ChatGuru)
    let has_media = payload
        .get("media_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
        || payload
            .get("url_arquivo")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .is_some();

    // Extrair texto da mensagem (truncado para logs)
    let message_text = payload
        .get("texto_mensagem")
        .and_then(|v| v.as_str())
        .map(|text| {
            if text.len() > 100 {
                truncate_with_suffix(text, 100, "...")
            } else {
                text.to_string()
            }
        })
        .unwrap_or_default();

    // Verificar se é PDF duplicado (pode ter descrição vazia)
    let is_pdf = payload
        .get("media_url")
        .and_then(|v| v.as_str())
        .map(|url| url.to_lowercase().contains(".pdf"))
        .unwrap_or(false);

    let pdf_info = if is_pdf {
        " | ⚠️ PDF_DETECTED"
    } else {
        ""
    };

    // Log detalhado do webhook recebido
    log_info(&format!(
        "📥 WEBHOOK RECEBIDO - RequestID: {} | ChatID: {} | Sender: {} | Type: {} | Media: {} | Size: {} bytes{} | Text: \"{}\"",
        request_id, chat_id, sender_name, message_type,
        if has_media { "Sim" } else { "Não" },
        body_str.len(),
        pdf_info,
        message_text
    ));

    // PROCESSAMENTO IMEDIATO DE MÍDIA (antes de expirar URLs do S3)
    let mut final_payload = payload.clone();

    if has_media {
        log_info(&format!(
            "🎬 INICIANDO PROCESSAMENTO DE MÍDIA - RequestID: {} | ChatID: {}",
            request_id, chat_id
        ));

        // Adicionar timeout de 60 segundos para processamento de mídia
        // para evitar que o handler exceda o timeout do Cloud Run (300s)
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(60),
            process_media_immediately(&state, &mut final_payload)
        ).await {
            Ok(Some((synthetic_payload, is_audio))) => {
                log_info(&format!(
                    "✅ MÍDIA PROCESSADA - RequestID: {} | ChatID: {} | Is Audio: {} | Payload sintético criado",
                    request_id, chat_id, is_audio
                ));
                final_payload = synthetic_payload;

                // NOVO: Se é áudio, publicar imediatamente (bypass da fila)
                if is_audio {
                    log_info(&format!(
                        "🎤 ÁUDIO DETECTADO - Publicando imediatamente no PubSub | RequestID: {} | ChatID: {}",
                        request_id, chat_id
                    ));

                    match publish_single_message_to_pubsub(&state.settings, &final_payload).await {
                        Ok(_) => {
                            let processing_time = start_time.elapsed().as_millis() as u64;

                            log_info(&format!(
                                "✅ ÁUDIO PUBLICADO DIRETAMENTE - RequestID: {} | ChatID: {} | Time: {}ms",
                                request_id, chat_id, processing_time
                            ));

                            // Retornar sucesso SEM adicionar à fila
                            return Ok(Json(json!({
                                "message": "Audio processed and published immediately",
                                "request_id": request_id,
                                "chat_id": chat_id,
                                "processing_time_ms": processing_time,
                                "audio_fast_track": true
                            })));
                        }
                        Err(e) => {
                            log_error(&format!(
                                "❌ Erro ao publicar áudio no PubSub: {} | Continuando com fila normal | RequestID: {} | ChatID: {}",
                                e, request_id, chat_id
                            ));
                            // Se falhar, continua com o fluxo normal (fila)
                        }
                    }
                }
            }
            Ok(None) => {
                log_warning(&format!(
                    "⚠️ FALHA AO PROCESSAR MÍDIA - RequestID: {} | ChatID: {} | Usando payload original",
                    request_id, chat_id
                ));
                // final_payload já é o payload original
            }
            Err(_) => {
                // Timeout no processamento de mídia
                log_warning(&format!(
                    "⏱️ TIMEOUT NO PROCESSAMENTO DE MÍDIA (60s) - RequestID: {} | ChatID: {} | Usando payload original",
                    request_id, chat_id
                ));
                // final_payload já é o payload original - continuar com processamento normal
            }
        }
    }

    log_info(&format!(
        "📬 ADICIONANDO À FILA - RequestID: {} | ChatID: {} | Queue size: estimating...",
        request_id, chat_id
    ));

    // Adicionar à fila (processa automaticamente quando atingir 5 msgs ou 100s via callback)
    state.message_queue.enqueue(chat_id.clone(), final_payload).await;

    let processing_time = start_time.elapsed().as_millis() as u64;

    log_info(&format!(
        "✅ WEBHOOK CONCLUÍDO - RequestID: {} | ChatID: {} | Processing time: {}ms | Status: 200",
        request_id, chat_id, processing_time
    ));

    // ACK imediato (compatível com legado)
    Ok(Json(json!({
        "message": "Success",
        "request_id": request_id,
        "chat_id": chat_id,
        "processing_time_ms": processing_time
    })))
}
