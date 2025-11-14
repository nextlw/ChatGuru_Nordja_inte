/// Worker: Processa mensagens do Pub/Sub e cria tasks no ClickUp
/// 
/// Fluxo simplificado seguindo especificação do prompt original:
/// 1. Extrair dados essenciais (mensagem, Info_2)
/// 2. Buscar pasta do cliente usando Info_2 
/// 3. Determinar lista mensal (NOVEMBRO 2025)
/// 4. Classificar com IA Service
/// 5. Criar task se necessário
/// 6. Enviar anotação ao ChatGuru

use std::sync::Arc;
use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use chrono::{Utc, Datelike};
use base64::{Engine as _, engine::general_purpose};

use chatguru_clickup_middleware::{AppState, services::AiPromptConfig};
use chatguru_clickup_middleware::utils::{AppError, logging::*};
use chatguru_clickup_middleware::models::payload::WebhookPayload;

/// Handler principal: processa mensagem do Pub/Sub
#[axum::debug_handler]
pub async fn worker_process_message(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let start_time = tokio::time::Instant::now();
    log_request_received("/worker/process", "POST");

    // 🔒 DEDUPLICAÇÃO: Extrair message_id do Pub/Sub para evitar reprocessamento
    let message_id = body.get("message")
        .and_then(|m| m.get("messageId"))
        .and_then(|id| id.as_str())
        .unwrap_or("unknown");

    // Verificar se já processamos esta mensagem
    {
        let mut cache = state.processed_messages.lock().unwrap();

        // Limpar mensagens antigas (mais de 1 hora)
        let now = std::time::Instant::now();
        cache.retain(|_, timestamp| now.duration_since(*timestamp).as_secs() < 3600);

        // Se já foi processada, retornar sucesso imediatamente (ACK sem reprocessar)
        if cache.contains_key(message_id) {
            log_info(&format!("⚠️ Mensagem duplicada detectada (message_id: {}). Pulando processamento.", message_id));
            return Ok(Json(json!({
                "status": "duplicate_message",
                "message_id": message_id,
                "message": "Mensagem já foi processada anteriormente"
            })));
        }

        // Marcar mensagem como processada
        cache.insert(message_id.to_string(), now);
        log_info(&format!("✅ Mensagem marcada como processada (message_id: {})", message_id));
    }

    // Extrair data do envelope Pub/Sub (formato: message.data em base64)
    let data_base64 = body.get("message")
        .and_then(|m| m.get("data"))
        .and_then(|d| d.as_str())
        .ok_or_else(|| AppError::ValidationError("Missing message.data in Pub/Sub payload".to_string()))?;

    // Decodificar base64
    let data_bytes = general_purpose::STANDARD.decode(data_base64)
        .map_err(|e| AppError::ValidationError(format!("Failed to decode base64: {}", e)))?;

    let envelope_str = String::from_utf8(data_bytes)
        .map_err(|e| AppError::ValidationError(format!("Invalid UTF-8 in payload: {}", e)))?;

    log_info(&format!("🔍 DEBUG - Envelope recebido do Pub/Sub:\n{}",
        &envelope_str[..envelope_str.len().min(500)]));

    // Parsear o envelope primeiro
    let envelope: Value = serde_json::from_str(&envelope_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid envelope JSON: {}", e)))?;

    // Extrair raw_payload do envelope (que é uma STRING serializada)
    let raw_payload_str = envelope.get("raw_payload")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Missing raw_payload in envelope".to_string()))?;

    log_info(&format!("🔍 DEBUG - raw_payload extraído:\n{}",
        &raw_payload_str[..raw_payload_str.len().min(500)]));

    // Parsear o payload do ChatGuru a partir do raw_payload
    let payload: WebhookPayload = serde_json::from_str(raw_payload_str)
        .map_err(|e| AppError::ValidationError(format!("Invalid payload JSON: {}", e)))?;

    // Processar mensagem
    match process_message(state, payload).await {
        Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            log_request_processed("/worker/process", 200, processing_time);
            Ok(Json(result))
        }
        Err(e) => {
            log_error(&format!("❌ Worker processing failed: {}", e));
            Err(e)
        }
    }
}

/// Função principal que implementa o fluxo especificado no prompt original
async fn process_message(state: Arc<AppState>, payload: WebhookPayload) -> Result<Value, AppError> {
    // 1. Extrair dados essenciais do payload ChatGuru
    let mensagem_texto = payload.texto_mensagem();
    let info_2 = payload.get_info_2(); // Nome do atendente

    // NOTA: Processamento de mídia (áudio/imagem/PDF) agora é feito no WEBHOOK
    // antes de enqueue, para evitar expiração de URLs do S3 (5min).
    // Se a mensagem contém mídia, ela já foi processada e o texto_mensagem
    // contém a transcrição/descrição extraída.

    // Se Info_2 está vazio, não é cliente - retornar imediatamente
    if info_2.trim().is_empty() {
        log_info("❌ Info_2 não informado - NÃO é cliente (campos_personalizados vazio)");
        return Ok(json!({
            "status": "not_client",
            "message": "Info_2 não informado - não é cliente"
        }));
    }

    log_info(&format!("📩 Mensagem recebida - Atendente: {}", info_2));

    // 2. Buscar pasta do cliente usando Info_2 nos spaces do workspace
    let search_result = search_folders_by_name(&state.clickup_client, &state.clickup_workspace_id, &info_2).await?;
    
    // Se não encontrar pasta → não é cliente
    if search_result.is_empty() {
        log_info(&format!("❌ Pasta não encontrada para '{}' - NÃO é cliente", info_2));
        return Ok(json!({
            "status": "not_client",
            "message": format!("Nenhuma pasta encontrada para o atendente '{}'", info_2)
        }));
    }
    
    let folder_id = search_result[0].id.clone();
    let folder_name = search_result[0].name.clone();
    log_info(&format!("✅ Cliente encontrado - Pasta: '{}' ({})", folder_name, folder_id));

    // 3. Determinar nome da lista baseado no mês/ano atual
    let now = Utc::now();
    let list_name = format!("{} {}", 
        match now.month() {
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
            _ => "JANEIRO" // fallback
        },
        now.year()
    );
    
    // Buscar lista na pasta
    let lists = get_folder_lists(&state.clickup_client, &folder_id).await?;
    
    let list_id = if let Some(existing_list) = lists.iter().find(|l| l.name == list_name) {
        log_info(&format!("📋 Lista já existe: '{}'", list_name));
        existing_list.id.clone()
    } else {
        // Criar nova lista mensal
        log_info(&format!("📝 Criando nova lista: '{}'", list_name));
        let new_list = create_list(&state.clickup_client, &folder_id, &list_name).await?;

        // Aguardar 2s para API do ClickUp processar a criação da lista
        log_info("⏳ Aguardando 2s para API do ClickUp processar a nova lista...");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        new_list.id
    };

    // 4. Buscar tasks existentes para verificação de duplicatas
    let existing_tasks = get_list_tasks(&state.clickup_client, &list_id).await?;
    
    let task_titles: Vec<String> = existing_tasks.iter()
        .map(|t| t.name.clone())
        .collect();
    
    log_info(&format!("🔍 Lista contém {} tasks existentes", task_titles.len()));

    // 5. Classificar com IA Service
    // Carregar configuração do prompt
    let prompt_config = AiPromptConfig::from_file("config/ai_prompt.yaml")?;

    // Gerar contexto com tasks existentes e mensagem
    let existing_tasks_context = if task_titles.is_empty() {
        "Nenhuma tarefa existe ainda nesta lista.".to_string()
    } else {
        format!(
            "TASKS EXISTENTES NESTA LISTA (para verificação de duplicatas):\n{}\n\n",
            task_titles.iter().enumerate().map(|(i, task)| format!("{}. {}", i + 1, task)).collect::<Vec<_>>().join("\n")
        )
    };

    let context = format!(
        "{}\n\nMENSAGEM DO USUÁRIO:\n{}\n\n\
         🚨 VERIFICAÇÃO DE DUPLICATAS OBRIGATÓRIA:\n\
         1. Compare esta mensagem com TODAS as tasks existentes listadas acima\n\
         2. Se encontrar uma task MUITO SIMILAR (mesmo objetivo/contexto):\n\
            - Marque is_duplicate=true\n\
            - Preencha existing_task_title com o título exato da task similar\n\
            - Explique na reason por que é duplicata\n\
         3. Se NÃO encontrar similar:\n\
            - Marque is_duplicate=false\n\
            - Prossiga com a classificação normal da atividade",
        existing_tasks_context,
        mensagem_texto
    );

    // Gerar prompt completo com toda a configuração
    let full_prompt = prompt_config.generate_prompt(&context);

    let ia_result = state.ia_service
        .as_ref()
        .ok_or(AppError::ServiceUnavailable("IA Service não disponível".to_string()))?
        .classify_activity(&mensagem_texto, &task_titles, &full_prompt)
        .await?;

    // Log de debug para verificação de duplicatas
    log_info(&format!(
        "🔍 Resultado da classificação: is_activity={}, is_duplicate={}, existing_task_title={:?}",
        ia_result.is_activity,
        ia_result.is_duplicate,
        ia_result.existing_task_title
    ));

    // Se NÃO é uma task
    if !ia_result.is_valid_activity() {
        log_info(&format!("ℹ️ Mensagem NÃO é uma tarefa: {}", ia_result.reason));
        
        send_annotation_to_chatguru(
            &state,
            &payload,
            &format!("Mensagem analisada: {}", ia_result.reason)
        ).await?;
        
        return Ok(json!({
            "status": "not_task",
            "reason": ia_result.reason
        }));
    }
    
    // Se task JÁ EXISTE (duplicata detectada pela IA)
    if ia_result.is_duplicate() {
        let existing_task = ia_result.get_existing_task_title().unwrap_or_else(|| "N/A".to_string());
        log_info(&format!("⚠️ Task duplicada detectada: {}", existing_task));
        
        send_annotation_to_chatguru(
            &state,
            &payload,
            &format!("Já existe uma tarefa sobre este assunto: '{}'", existing_task)
        ).await?;
        
        return Ok(json!({
            "status": "duplicate",
            "existing_task": existing_task
        }));
    }

    // 6. Gerar CreateTaskRequest usando IA Service com mapeamentos de custom fields
    let mappings = &state.custom_fields_mappings;

    // Preparar mapeamentos no formato esperado
    let category_mappings = Some(&mappings.categories);

    let subcategory_mappings: std::collections::HashMap<String, (String, u8)> = mappings.subcategories
        .iter()
        .map(|(name, info)| (name.clone(), (info.id.clone(), info.stars)))
        .collect();
    let subcategory_mappings_ref = Some(&subcategory_mappings);

    let task_request = ia_result.to_create_task_request(
        category_mappings,
        subcategory_mappings_ref,
        &mappings.category_field_id,
        &mappings.subcategory_field_id,
        mappings.stars_field_id.as_deref(),
    )?;
    
    log_info(&format!("✅ Criando task: '{}'", task_request.name));
    
    // Criar task no ClickUp
    let created_task = create_task(&state.clickup_client, &list_id, task_request).await?;
    
    log_info(&format!("🎉 Task criada - ID: {}", created_task.id));

    // 7. Enviar anotação formatada ao ChatGuru
    let annotation = format!(
        "✅ Tarefa criada com sucesso!\n\n\
         📝 **Título**: {}\n\
         🔗 **Link**: {}\n\
         📄 **Descrição**: {}\n\
         📂 **Categoria**: {} > {}\n\
         {}",
        created_task.name,
        created_task.url,
        ia_result.description.as_deref().unwrap_or("N/A"),
        ia_result.category.as_deref().unwrap_or("N/A"),
        ia_result.sub_categoria.as_deref().unwrap_or("N/A"),
        if !ia_result.subtasks.is_empty() {
            format!("✓ **Subtarefas**: {}", ia_result.subtasks.join(", "))
        } else {
            String::new()
        }
    );

    send_annotation_to_chatguru(&state, &payload, &annotation).await?;

    Ok(json!({
        "status": "created",
        "task_id": created_task.id,
        "task_url": created_task.url
    }))
}

/// Envia uma anotação ao ChatGuru via API
async fn send_annotation_to_chatguru(
    state: &AppState,
    payload: &WebhookPayload,
    annotation: &str,
) -> Result<(), AppError> {
    log_info(&format!("📤 Enviando anotação ao ChatGuru: {}", annotation));

    // Extrair chat_id e phone_number do payload
    let (chat_id, phone_number) = match payload {
        WebhookPayload::ChatGuru(p) => {
            let chat = p.chat_id.as_deref().unwrap_or("unknown");
            let phone = p.celular.as_str();
            (chat, phone)
        },
        WebhookPayload::EventType(p) => {
            let chat = "unknown"; // EventType não tem chat_id estruturado
            let phone = p.data.phone.as_deref().unwrap_or("unknown");
            (chat, phone)
        },
        WebhookPayload::Generic(p) => {
            let chat = "unknown";
            let phone = p.celular.as_deref().unwrap_or("unknown");
            (chat, phone)
        },
    };

    state.chatguru()
        .add_annotation(chat_id, phone_number, annotation)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    log_info("✅ Anotação enviada com sucesso");
    Ok(())
}

// ==================================================
// Usar tipos reais do clickup_v2
// ==================================================

use clickup_v2::client::api::{EntityItem, TaskResponse, EntityType};

/// Busca folders por nome usando clickup_v2 - retorna EntityItem diretamente
async fn search_folders_by_name(
    client: &clickup_v2::client::ClickUpClient,
    workspace_id: &str,
    folder_name: &str,
) -> Result<Vec<EntityItem>, AppError> {
    log_info(&format!("🔍 Buscando folder '{}' no workspace {}", folder_name, workspace_id));

    let search_result = client
        .search_entity(
            folder_name,
            EntityType::Folder,
            Some(workspace_id.to_string())
        )
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Erro ao buscar folders: {}", e)))?;

    if !search_result.found {
        log_warning(&format!("⚠️ Nenhum folder encontrado com nome '{}'", folder_name));
        return Ok(vec![]);
    }

    log_info(&format!("✅ Encontrados {} folder(s)", search_result.items.len()));

    Ok(search_result.items)
}

/// Struct simples para listas (enquanto clickup_v2 não tem um tipo específico)
#[derive(Clone, Debug)]
struct ListInfo {
    pub id: String,
    pub name: String,
}

/// Busca listas de um folder usando clickup_v2
async fn get_folder_lists(
    client: &clickup_v2::client::ClickUpClient,
    folder_id: &str,
) -> Result<Vec<ListInfo>, AppError> {
    log_info(&format!("📋 Buscando listas do folder {}", folder_id));

    let response = client
        .get_lists(None, Some(folder_id))
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Erro ao buscar listas: {}", e)))?;

    let lists = response
        .get("lists")
        .and_then(|l| l.as_array())
        .ok_or_else(|| AppError::ClickUpApi("Campo 'lists' não encontrado na resposta".to_string()))?;

    log_info(&format!("✅ Encontradas {} lista(s)", lists.len()));

    Ok(lists
        .iter()
        .filter_map(|list| {
            let id = list.get("id")?.as_str()?.to_string();
            let name = list.get("name")?.as_str()?.to_string();
            Some(ListInfo { id, name })
        })
        .collect())
}

/// Cria uma nova lista usando API direta do ClickUp
async fn create_list(
    _client: &clickup_v2::client::ClickUpClient,
    folder_id: &str,
    list_name: &str,
) -> Result<ListInfo, AppError> {
    log_info(&format!("📝 Criando lista '{}' no folder {}", list_name, folder_id));

    // ClickUp API: POST /folder/{folder_id}/list
    let url = format!("https://api.clickup.com/api/v2/folder/{}/list", folder_id);

    let token = std::env::var("CLICKUP_ACCESS_TOKEN")
        .map_err(|_| AppError::ConfigError("CLICKUP_ACCESS_TOKEN não configurado".to_string()))?;

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", token)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "name": list_name,
            "content": format!("Lista criada automaticamente: {}", list_name),
        }))
        .send()
        .await
        .map_err(|e| AppError::HttpError(e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "unknown error".to_string());
        return Err(AppError::ClickUpApi(format!(
            "Erro ao criar lista: {}",
            error_text
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::HttpError(e))?;

    let list_id = body
        .get("id")
        .and_then(|id| id.as_str())
        .ok_or_else(|| AppError::ClickUpApi("ID da lista não retornado".to_string()))?
        .to_string();

    log_info(&format!("✅ Lista criada com sucesso: {} ({})", list_name, list_id));

    Ok(ListInfo {
        id: list_id,
        name: list_name.to_string(),
    })
}

/// Struct simples para tasks existentes (quando obtidas por GET, não por create)
#[derive(Clone, Debug)]
#[allow(dead_code)]
struct TaskInfo {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

/// Busca tasks de uma lista usando API direta do ClickUp
async fn get_list_tasks(
    _client: &clickup_v2::client::ClickUpClient,
    list_id: &str,
) -> Result<Vec<TaskInfo>, AppError> {
    log_info(&format!("🔍 Buscando tasks da lista {}", list_id));

    // ClickUp API: GET /list/{list_id}/task
    let url = format!("https://api.clickup.com/api/v2/list/{}/task", list_id);

    let token = std::env::var("CLICKUP_ACCESS_TOKEN")
        .map_err(|_| AppError::ConfigError("CLICKUP_ACCESS_TOKEN não configurado".to_string()))?;

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", token)
        .send()
        .await
        .map_err(|e| AppError::HttpError(e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "unknown error".to_string());
        return Err(AppError::ClickUpApi(format!(
            "Erro ao buscar tasks: {}",
            error_text
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::HttpError(e))?;

    let tasks = body
        .get("tasks")
        .and_then(|t| t.as_array())
        .ok_or_else(|| AppError::ClickUpApi("Campo 'tasks' não encontrado".to_string()))?;

    log_info(&format!("✅ Encontradas {} task(s)", tasks.len()));

    Ok(tasks
        .iter()
        .filter_map(|task| {
            let id = task.get("id")?.as_str()?.to_string();
            let name = task.get("name")?.as_str()?.to_string();
            let url = task.get("url").and_then(|u| u.as_str()).map(|s| s.to_string());
            Some(TaskInfo { id, name, url })
        })
        .collect())
}

/// Cria uma nova task usando clickup_v2 - retorna TaskResponse diretamente
async fn create_task(
    client: &clickup_v2::client::ClickUpClient,
    list_id: &str,
    task_request: clickup_v2::client::api::CreateTaskRequest,
) -> Result<TaskResponse, AppError> {
    log_info(&format!("✅ Criando task '{}' na lista {}", task_request.name, list_id));

    let response = client
        .create_task(list_id, task_request)
        .await
        .map_err(|e| AppError::ClickUpApi(format!("Erro ao criar task: {}", e)))?;

    log_info(&format!("✅ Task criada com sucesso: {} ({})", response.name, response.id));

    Ok(response)
}
