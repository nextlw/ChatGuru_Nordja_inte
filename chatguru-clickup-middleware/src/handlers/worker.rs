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

use chatguru_clickup_middleware::models::WebhookPayload;
use chatguru_clickup_middleware::utils::{AppResult, AppError};
use chatguru_clickup_middleware::utils::logging::*;
use chatguru_clickup_middleware::AppState;
use chatguru_clickup_middleware::services::openai::OpenAIService;
use chatguru_clickup_middleware::services::chatguru::ChatGuruApiService;

/// Handler do worker
/// Retorna 200 OK se processado com sucesso
/// Retorna 4xx se erro não recuperável (não faz retry)
/// Retorna 5xx se erro recuperável (Pub/Sub faz retry)
pub async fn handle_worker(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let start_time = Instant::now();
    log_request_received("/worker/process", "POST");

    // Extrair body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            log_error(&format!("Failed to read request body: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid request body"}))
            ));
        }
    };

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

    // Parsear envelope do Pub/Sub
    let envelope: Value = match serde_json::from_str(&body_str) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("Invalid JSON: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid JSON"}))
            ));
        }
    };

    // Extrair e decodificar payload do Pub/Sub
    // Formato esperado: { "message": { "data": "base64_encoded_data" } }
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

    // Parsear o envelope interno que contém o raw_payload
    let inner_envelope: Value = match serde_json::from_str(&raw_payload_str) {
        Ok(v) => v,
        Err(e) => {
            log_error(&format!("Failed to parse inner envelope: {}", e));
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid inner envelope"}))
            ));
        }
    };

    // Extrair o raw_payload do envelope interno
    let raw_payload_str = match inner_envelope.get("raw_payload").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            log_error("Missing raw_payload in inner envelope");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing raw_payload"}))
            ));
        }
    };

    // Parsear payload do ChatGuru
    let payload: WebhookPayload = match serde_json::from_str(raw_payload_str) {
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

    // Extrair force_classification se presente
    let force_classification = envelope.get("force_classification");

    // Processar mensagem
    match process_message(&state, &payload, force_classification).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            log_request_processed("/worker/process", 200, processing_time);
            Ok(Json(result))
        }
        Err(e) => {
            log_error(&format!("Worker processing error: {}", e));
            // Erro recuperável - Pub/Sub vai fazer retry
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()}))
            ))
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
    let _chat_id = extract_chat_id_from_payload(payload);

    log_info(&format!(
        "💬 Processando mensagem de {}: {}",
        if !nome.is_empty() { nome.clone() } else { "Desconhecido".to_string() },
        message
    ));

    // Verificar se há classificação forçada (bypass OpenAI)
    let classification = if let Some(forced) = force_classification {
        log_info("🔧 Usando classificação forçada (bypass OpenAI)");

        use crate::services::openai::OpenAIClassification;
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
                .unwrap_or("Atendimento")
                .to_string(),
            description: forced.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("Classificação manual")
                .to_string(),
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
        // Classificar com OpenAI
        let openai_service = match OpenAIService::new(None).await {
            Some(service) => service,
            None => {
                return Err(AppError::InternalError("Failed to initialize OpenAI service".to_string()));
            }
        };

        let context = format!(
            "Campanha: WhatsApp\nOrigem: whatsapp\nNome: {}\nMensagem: {}\nTelefone: {}",
            nome, message, phone.as_deref().unwrap_or("N/A")
        );

        match openai_service.classify_activity_fallback(&context).await {
            Ok(c) => c,
            Err(e) => {
                log_error(&format!("❌ Erro na classificação OpenAI: {}", e));
                return Err(AppError::InternalError(format!("OpenAI classification failed: {}", e)));
            }
        }
    };

    let annotation = format!("Tarefa: {}", classification.reason);
    let is_activity = classification.is_activity;

    if is_activity {
        log_info(&format!("✅ Atividade identificada: {}", classification.reason));

        // Criar tarefa no ClickUp usando process_payload do serviço ClickUp
        let task_result = state.clickup.process_payload_with_ai(payload, Some(&classification)).await?;

        // Enviar anotação ao ChatGuru
        if let Err(e) = send_annotation_to_chatguru(state, payload, &annotation).await {
            log_warning(&format!("⚠️  Falha ao enviar anotação ao ChatGuru: {}", e));
            // Não falhar o processamento se anotação falhar
        }

        Ok(json!({
            "status": "processed",
            "is_activity": true,
            "task_id": task_result.get("id"),
            "annotation": annotation
        }))
    } else {
        log_info(&format!("❌ Não é atividade: {}", classification.reason));

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

/// FUNÇÃO OBSOLETA - NÃO MAIS UTILIZADA
///
/// Esta função foi substituída por chamada direta a:
/// `state.clickup.process_payload_with_ai()` na linha 173
///
/// NOVA IMPLEMENTAÇÃO: src/services/clickup.rs:215-262
/// A lógica de criação de tarefas agora está centralizada no ClickUpService
#[allow(dead_code)]
async fn create_clickup_task(
    state: &Arc<AppState>,
    payload: &WebhookPayload,
    classification: &chatguru_clickup_middleware::services::openai::OpenAIClassification,
    _nome: &str,
    _message: &str,
) -> AppResult<Value> {
    // Usar o método público process_payload_with_ai do serviço ClickUp
    state.clickup.process_payload_with_ai(payload, Some(classification)).await
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

    let chatguru_service = ChatGuruApiService::new(api_token, api_endpoint, account_id);

    let chat_id = extract_chat_id_from_payload(payload);
    let phone = extract_phone_from_payload(payload);

    if let Some(chat_id) = chat_id {
        let phone_str = phone.as_deref().unwrap_or("");
        chatguru_service.add_annotation(&chat_id, phone_str, annotation).await?;
        log_info("📝 Anotação enviada ao ChatGuru");
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
fn prepare_custom_fields(
    payload: &WebhookPayload,
    classification: &chatguru_clickup_middleware::services::openai::OpenAIClassification,
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
    let stars = determine_estrelas(classification, payload);
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
fn determine_subcategoria(classification: &chatguru_clickup_middleware::services::openai::OpenAIClassification) -> Option<String> {
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
fn determine_estrelas(
    classification: &chatguru_clickup_middleware::services::openai::OpenAIClassification,
    _payload: &WebhookPayload,
) -> i32 {
    // Usar a subcategoria determinada para mapear as estrelas
    if let Some(subcategory) = determine_subcategoria(classification) {
        // MAPEAMENTO EXATO do categorize_tasks.js - SUBCATEGORIA_ESTRELAS
        match subcategory.as_str() {
            // 1 estrela - Agendamentos
            "Consultas" | "Exames" | "Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)" |
            "Vacinas" | "Manicure" | "Cabeleleiro" => 1,
            
            // Compras - Variado
            "Mercados" | "Presentes" | "Petshop" | "Papelaria" => 1,
            "Shopper" | "Farmácia" | "Ingressos" | "Móveis e Eletros" | "Itens pessoais e da casa" => 2,
            
            // Documentos - Variado
            "CIN" | "Documento de Vacinação (BR/Iternacional)" | "Assinatura Digital" |
            "Contratos/Procurações" | "Passaporte" | "CNH" | "Averbações" | "Certidões" => 1,
            "Certificado Digital" | "Seguros Carro/Casa/Viagem (anual)" |
            "Vistos e Vistos Eletrônicos" => 2,
            "Cidadanias" => 4,
            
            // Lazer - Variado
            "Reserva de restaurantes/bares" => 1,
            "Fornecedores no exterior (passeios, fotógrafos)" => 2,
            "Pesquisa de passeios/eventos (BR)" => 3,
            "Planejamento de festas" => 4,
            
            // Logística - Todas 1 estrela
            "Corrida de motoboy" | "Motoboy + Correios e envios internacionais" |
            "Lalamove" | "Corridas com Taxistas" | "Transporte Urbano (Uber/99)" => 1,
            
            // Viagens - Variado
            "Compra de Assentos e Bagagens" | "Passagens de Ônibus" | "Checkins (Early/Late)" |
            "Seguro Viagem (Temporário)" | "Programa de Milhagem" | "Gestão de Contas (CIAs Aereas)" => 1,
            "Passagens Aéreas" | "Hospedagens" | "Passagens de Trem" | "Extravio de Bagagens" |
            "Transfer" | "Aluguel de Carro/Ônibus e Vans" => 2,
            "Roteiro de Viagens" => 3,
            
            // Plano de Saúde - Variado
            "Extrato para IR" | "Prévia de Reembolso" | "Contestações" | "Autorizações" => 1,
            "Reembolso Médico" | "Contratações/Cancelamentos" => 2,
            
            // Agenda - Todas 1 estrela
            "Gestão de Agenda" | "Criação e envio de invites" => 1,
            
            // Financeiro - Variado
            "Emissão de NF" | "Rotina de Pagamentos" | "Emissão de boletos" |
            "Imposto de Renda" | "Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)" => 1,
            "Conciliação Bancária" | "Encerramento e Abertura de CNPJ" => 2,
            "Planilha de Gastos/Pagamentos" => 4,
            
            // Assuntos Pessoais - Variado
            "Troca de titularidade" | "Assuntos do Carro/Moto" | "Internet e TV por Assinatura" |
            "Contas de Consumo" | "Assuntos Escolares e Professores Particulares" |
            "Academia e Cursos Livres" | "Telefone" | "Assistência Técnica" | "Consertos na Casa" => 1,
            "Mudanças" | "Anúncio de Vendas Online (Itens, eletros. móveis)" => 3,
            
            // Atividades Corporativas - Variado
            "Financeiro/Contábil" | "Atendimento ao Cliente" | "Documentos/Contratos e Assinaturas" |
            "Gestão de Agenda (Corporativa)" | "Recursos Humanos" | "Gestão de Estoque" | "Compras/vendas" => 1,
            "Fornecedores" => 2,
            "Gestão de Planilhas e Emails" => 4,
            
            // Gestão de Funcionário - Todas 1 estrela
            "eSocial" | "Contratações e Desligamentos" | "DIRF" | "Férias" => 1,
            
            // Padrão para subcategorias não mapeadas
            _ => 1
        }
    } else {
        // Fallback: usar categoria se não conseguir determinar subcategoria
        if let Some(category) = &classification.category {
            match category.as_str() {
                "Logística" | "Agendamentos" => 1,
                "Compras" | "Plano de Saúde" | "Financeiro" | "Viagens" => 2,
                "Lazer" | "Documentos" | "Assuntos Pessoais" => 2,
                _ => 1
            }
        } else {
            1
        }
    }
}