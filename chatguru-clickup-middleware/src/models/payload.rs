use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::services::prompts::AiPromptConfig;

// ClickUp v2 API types
use clickup_v2::client::api::{
    CreateTaskRequest,
    CustomField,
    CustomFieldValue,
    TaskPriority,
};

/// Estrutura flexível que aceita múltiplos formatos de webhook
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum WebhookPayload {
    /// Formato ChatGuru (campanha_id, nome, etc)
    ChatGuru(ChatGuruPayload),
    /// Formato com event_type (antigo)
    EventType(EventTypePayload),
    /// Formato genérico/mínimo
    Generic(GenericPayload),
}

/// Payload do ChatGuru atual
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGuruPayload {
    #[serde(default)]
    pub campanha_id: String,
    #[serde(default)]
    pub campanha_nome: String,
    #[serde(default)]
    pub origem: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub nome: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, alias = "mensagem", alias = "message", alias = "text")]
    pub texto_mensagem: String,

    // Campos de mídia - formato antigo (media_url, media_type)
    #[serde(default)]
    pub media_url: Option<String>,  // URL do áudio ou mídia anexada
    #[serde(default)]
    pub media_type: Option<String>, // Tipo da mídia (audio, image, video)

    // Campos de mídia - formato novo ChatGuru (tipo_mensagem, url_arquivo)
    #[serde(default)]
    pub tipo_mensagem: Option<String>, // "image", "ptt" (áudio), "video", etc
    #[serde(default, alias = "url_midia")]
    pub url_arquivo: Option<String>, // URL do arquivo de mídia

    #[serde(default)]
    pub campos_personalizados: HashMap<String, Value>,
    #[serde(default)]
    pub bot_context: Option<BotContext>,
    #[serde(default)]
    pub responsavel_nome: Option<String>,
    #[serde(default)]
    pub responsavel_email: Option<String>,
    #[serde(default)]
    pub link_chat: String,
    #[serde(default)]
    pub celular: String,
    #[serde(default)]
    pub phone_id: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
    #[serde(default)]
    pub chat_created: Option<String>,

    // Campos adicionados pela agregação de mensagens (batch processing)
    #[serde(default)]
    pub _batch_chat_id: Option<String>,
    #[serde(default)]
    pub _batch_size: Option<u32>,

    // Campos para mensagens sintéticas (processamento imediato de mídia)
    /// Indica se esta mensagem é sintética (gerada pelo processamento de mídia no webhook)
    #[serde(default)]
    pub _is_synthetic: Option<bool>,
    /// Tipo original da mídia processada (ex: "audio/ogg", "image/jpeg")
    #[serde(default)]
    pub _original_media_type: Option<String>,
}

impl ChatGuruPayload {
    /// Normaliza os campos de mídia do ChatGuru
    /// Converte tipo_mensagem + url_arquivo → media_type + media_url
    pub fn normalize_media_fields(&mut self) {
        // Se já tem media_url e media_type, não faz nada
        if self.media_url.is_some() && self.media_type.is_some() {
            return;
        }

        // Mapear url_arquivo → media_url
        if self.url_arquivo.is_some() && self.media_url.is_none() {
            self.media_url = self.url_arquivo.clone();
        }

        // Mapear tipo_mensagem → media_type (se tipo_mensagem for útil)
        if let Some(ref tipo) = self.tipo_mensagem {
            if self.media_type.is_none() {
                self.media_type = Some(match tipo.as_str() {
                    "image" => "image/jpeg".to_string(),
                    "ptt" => "audio/ogg".to_string(), // ptt = push-to-talk (áudio)
                    "audio" => "audio/ogg".to_string(),
                    "voice" => "audio/ogg".to_string(),
                    "video" => "video/mp4".to_string(),
                    "document" => "application/pdf".to_string(),
                    // Se tipo_mensagem não for útil (ex: "chat"), tentar detectar pela URL
                    _ => {
                        if let Some(ref url) = self.media_url {
                            Self::detect_media_type_from_url(url)
                        } else if let Some(ref url) = self.url_arquivo {
                            Self::detect_media_type_from_url(url)
                        } else {
                            "application/octet-stream".to_string()
                        }
                    }
                });
            }
        } else {
            // Se não tem tipo_mensagem, tentar detectar pela URL
            if self.media_type.is_none() {
                if let Some(ref url) = self.media_url {
                    self.media_type = Some(Self::detect_media_type_from_url(url));
                } else if let Some(ref url) = self.url_arquivo {
                    self.media_type = Some(Self::detect_media_type_from_url(url));
                }
            }
        }
    }

    /// Detecta o tipo de mídia pela extensão da URL
    fn detect_media_type_from_url(url: &str) -> String {
        // Remover query params e pegar apenas o path
        let path = url.split('?').next().unwrap_or(url);
        let extension = path.split('.').last().unwrap_or("").to_lowercase();

        match extension.as_str() {
            // Imagens
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "gif" => "image/gif".to_string(),
            "webp" => "image/webp".to_string(),

            // Áudios
            "ogg" | "oga" => "audio/ogg".to_string(),
            "mp3" => "audio/mpeg".to_string(),
            "wav" => "audio/wav".to_string(),
            "m4a" => "audio/mp4".to_string(),
            "opus" => "audio/opus".to_string(),

            // Vídeos
            "mp4" => "video/mp4".to_string(),
            "webm" => "video/webm".to_string(),
            "avi" => "video/x-msvideo".to_string(),

            // Documentos
            "pdf" => "application/pdf".to_string(),
            "doc" | "docx" => "application/msword".to_string(),
            "xls" | "xlsx" => "application/vnd.ms-excel".to_string(),

            // Fallback
            _ => "application/octet-stream".to_string(),
        }
    }
}

/// Payload com event_type (formato antigo)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventTypePayload {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub data: EventData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventData {
    pub lead_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub project_name: Option<String>,
    pub task_title: Option<String>,
    pub annotation: Option<String>,
    pub amount: Option<f64>,
    pub status: Option<String>,
    #[serde(default)]
    pub custom_data: HashMap<String, Value>,

    // Campos adicionais que podem vir
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Payload genérico/mínimo
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericPayload {
    pub nome: Option<String>,
    pub celular: Option<String>,
    pub email: Option<String>,
    pub mensagem: Option<String>,

    // Captura campos extras
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotContext {
    #[serde(rename = "ChatGuru")]
    pub chat_guru: Option<bool>,
}

/// Função helper para extrair título profissional da reason
fn extract_professional_title(reason: &str) -> String {
    // Remover prefixos comuns e deixar apenas a essência da atividade
    let clean_reason = reason
        .replace("A mensagem contém", "")
        .replace("O usuário solicitou", "")
        .replace("A solicitação é sobre", "")
        .replace("Trata-se de", "")
        .replace("É uma solicitação de", "")
        .replace("um pedido específico de", "")
        .replace("um pedido de", "")
        .replace("uma solicitação de", "")
        .replace("uma solicitação para", "")
        .replace("A ação envolve", "")
        .replace("O pedido é para", "")
        .replace("uma série de", "")
        .trim()
        .to_string();

    // Capitalizar primeira letra e limitar tamanho
    let mut title = if !clean_reason.is_empty() {
        let mut chars = clean_reason.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        clean_reason
    };

    // Remover quebras de linha e espaços extras
    title = title.replace('\n', " ").replace('\r', " ").trim().to_string();

    // Remover caracteres especiais indesejados
    title = title.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .collect::<String>()
        .trim()
        .to_string();

    // Limitar tamanho para 50 caracteres para evitar títulos muito longos
    if title.len() > 50 {
        use chatguru_clickup_middleware::utils::truncate_with_suffix;
        title = truncate_with_suffix(&title, 47, "...");
    }

    // Se título ficar vazio ou muito curto, usar fallback genérico
    if title.is_empty() || title.len() < 3 {
        title = "Atividade Profissional".to_string();
    }

    title
}

impl WebhookPayload {
    /// Extrai o texto da mensagem do payload
    pub fn texto_mensagem(&self) -> String {
        match self {
            WebhookPayload::ChatGuru(p) => p.texto_mensagem.clone(),
            WebhookPayload::EventType(p) => {
                p.data.task_title.clone()
                    .or(p.data.annotation.clone())
                    .unwrap_or_default()
            },
            WebhookPayload::Generic(p) => p.mensagem.clone().unwrap_or_default(),
        }
    }

    /// Extrai Info_2 do payload (nome do atendente)
    pub fn get_info_2(&self) -> String {
        match self {
            WebhookPayload::ChatGuru(p) => {
                p.campos_personalizados.get("Info_2")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string()
            },
            WebhookPayload::EventType(p) => {
                p.data.lead_name.clone().unwrap_or_default()
            },
            WebhookPayload::Generic(p) => {
                p.nome.clone().unwrap_or_default()
            }
        }
    }
    /// Converte qualquer formato para dados do ClickUp
    pub fn to_clickup_task_data(&self) -> CreateTaskRequest {
        match self {
            WebhookPayload::ChatGuru(payload) => {
                self.chatguru_to_clickup(payload)
            },
            WebhookPayload::EventType(payload) => {
                self.eventtype_to_clickup(payload)
            },
            WebhookPayload::Generic(payload) => {
                self.generic_to_clickup(payload)
            }
        }
    }

    /// Converte payload para dados do ClickUp com classificação AI
    pub async fn to_clickup_task_data_with_ai(
        &self,
        ai_classification: Option<&crate::services::OpenAIClassification>,
        prompt_config: &AiPromptConfig,
    ) -> CreateTaskRequest {
        match self {
            WebhookPayload::ChatGuru(payload) => {
                self.chatguru_to_clickup_with_ai(payload, ai_classification, prompt_config).await
            },
            WebhookPayload::EventType(payload) => {
                self.eventtype_to_clickup(payload)
            },
            WebhookPayload::Generic(payload) => {
                self.generic_to_clickup(payload)
            }
        }
    }

    /// Converte payload ChatGuru para ClickUp com classificação AI
    async fn chatguru_to_clickup_with_ai(
        &self,
        payload: &ChatGuruPayload,
        ai_classification: Option<&crate::services::OpenAIClassification>,
        prompt_config: &AiPromptConfig,
    ) -> CreateTaskRequest {
        // NOVO FORMATO: descrição focada na mensagem
        let mut description = String::new();

        // Mensagem principal (mais importante)
        if !payload.texto_mensagem.is_empty() {
            description.push_str(&payload.texto_mensagem);
            description.push_str("\n\n");
        }

        // Adicionar contexto adicional se houver
        if let Some(ai) = ai_classification {
            // Se há description da AI, usar ela como descrição principal
            if let Some(ref ai_description) = ai.description {
                if !ai_description.is_empty() {
                    description = ai_description.clone();
                    description.push_str("\n\n");
                }
            }

            // Se há campanha da AI, incluir junto com a descrição
            if let Some(ref campanha) = ai.campanha {
                if !campanha.is_empty() {
                    description.push_str(&format!("**Campanha**: {}\n", campanha));
                }
            }

            if let Some(ref category) = ai.category {
                description.push_str(&format!("**Categoria**: {}\n", category));
            }
            if let Some(ref tipo) = ai.tipo_atividade {
                description.push_str(&format!("**Tipo**: {}\n", tipo));
            }
        }

        // Dados de contato (menos ênfase que antes)
        description.push_str(&format!("\n**Contato**: {} ({})\n", payload.nome, payload.celular));

        // Link para o chat
        if !payload.link_chat.is_empty() {
            description.push_str(&format!("\n[Ver conversa completa]({})", payload.link_chat));
        }

        // Adicionar mídia anexada se houver
        if let Some(ref media_url) = payload.media_url {
            if let Some(ref media_type) = payload.media_type {
                description.push_str("\n\n**Mídia Anexada**\n");

                if media_type.contains("image") || media_type.contains("photo") {
                    // Para imagens, incluir link direto e preview no Markdown
                    description.push_str(&format!("🖼️ Imagem anexada: [Visualizar]({})\n", media_url));
                    // Tentar incluir a imagem inline no Markdown do ClickUp
                    description.push_str(&format!("\n![Imagem anexada]({})\n", media_url));
                } else if media_type.contains("audio") || media_type.contains("voice") {
                    // Para áudios, apenas incluir o link
                    description.push_str(&format!("🎵 Áudio anexado: [Ouvir]({})\n", media_url));
                } else {
                    // Para outros tipos de mídia
                    description.push_str(&format!("📎 Arquivo anexado: [Baixar]({})\n", media_url));
                }
            }
        }

        // Preparar campos personalizados do ClickUp
        let mut custom_fields = Vec::<CustomField>::new();

        // ===== CATEGORIZAÇÃO AUTOMÁTICA BASEADA EM PALAVRAS-CHAVE =====
        // Tentar categorizar automaticamente baseado no nome da tarefa e mensagem
        // task_categorizer foi deprecado - agora usamos apenas a classificação da AI
        // que é mais precisa e dinâmica

        // Detectar se é um teste (email começa com "test_" ou campanha_id é "test_campaign")
        let is_test = payload.email.starts_with("test_") || payload.campanha_id == "test_campaign";

        // Nome da tarefa - usar título profissional se temos AI classification
        let mut task_name = if let Some(ai) = ai_classification {
            if ai.is_activity {
                // Se o reason vem do pattern matching (começa com "Contém palavras-chave")
                // usar a mensagem original como título ao invés do reason genérico
                if ai.reason.starts_with("Contém palavras-chave") {
                    // Usar a mensagem original como título (limitando a 80 caracteres)
                    use chatguru_clickup_middleware::utils::truncate_with_suffix;
                    let titulo = if payload.texto_mensagem.len() > 80 {
                        truncate_with_suffix(&payload.texto_mensagem, 77, "...")
                    } else {
                        payload.texto_mensagem.clone()
                    };
                    titulo
                } else {
                    // Para classificação via IA, extrair título profissional da reason
                    let titulo = extract_professional_title(&ai.reason);
                    if !titulo.is_empty() && titulo.len() > 5 {
                        titulo
                    } else if let Some(ref tipo) = ai.tipo_atividade {
                        // Fallback: usar tipo de atividade + contexto
                        format!("{} - {}", tipo, payload.nome)
                    } else {
                        // Fallback final: usar mensagem original
                        use chatguru_clickup_middleware::utils::truncate_with_suffix;
                        let titulo = if payload.texto_mensagem.len() > 80 {
                            truncate_with_suffix(&payload.texto_mensagem, 77, "...")
                        } else {
                            payload.texto_mensagem.clone()
                        };
                        titulo
                    }
                }
            } else {
                payload.nome.clone()
            }
        } else {
            format!("[ChatGuru] {}", payload.nome)
        };

        // Adicionar [TESTE] no início se for um teste
        if is_test {
            task_name = format!("[TESTE] {}", task_name);
            tracing::info!("🧪 Tarefa de teste detectada, adicionado prefixo [TESTE]");
        }

        // Mapear campos baseado na classificação AI
        if let Some(ai) = ai_classification {
            // Log dos novos campos opcionais para debug
            if let Some(ref space_name) = ai.space_name {
                tracing::info!("🏢 Space sugerido pela AI: {}", space_name);
            }
            if let Some(ref folder_name) = ai.folder_name {
                tracing::info!("📁 Folder sugerido pela AI: {}", folder_name);
            }
            if let Some(ref list_name) = ai.list_name {
                tracing::info!("📋 List sugerida pela AI: {}", list_name);
            }

            // Configuração carregada uma vez no worker e reutilizada aqui (melhoria de performance)

            // Tipo de Atividade (dropdown) - OTIMIZADO: usar YAML como única fonte
            if let Some(ref tipo) = ai.tipo_atividade {
                // Buscar ID do tipo de atividade
                if let Some(activity_type) = prompt_config.activity_types.iter().find(|at| at.name == *tipo) {
                    let field_id = prompt_config.get_field_ids()
                        .map(|ids| ids.activity_type_field_id.clone())
                        .unwrap_or_else(|| "f1259ffb-7be8-49ff-92f8-5ff9882888d0".to_string());

                    custom_fields.push(CustomField {
                        id: field_id,
                        value: CustomFieldValue::DropdownOption(activity_type.id.clone()),
                    });
                } else {
                    tracing::warn!("Tipo de atividade '{}' não encontrado no YAML config", tipo);
                }
            }

            // Categoria (dropdown) - OTIMIZADO: usar YAML como única fonte
            if let Some(ref category) = ai.category {
                // Obter ID da categoria do YAML
                if let Some(cat_id) = prompt_config.get_category_id(category) {
                    // Obter ID do campo categoria do YAML
                    let field_id = prompt_config.get_field_ids()
                        .map(|ids| ids.category_field_id.clone())
                        .unwrap_or_else(|| "c19b4f95-1ff7-4966-b201-02905d33cec6".to_string());

                    custom_fields.push(CustomField {
                        id: field_id,
                        value: CustomFieldValue::DropdownOption(cat_id.clone()),
                    });
                } else {
                    tracing::warn!("Categoria '{}' não encontrada no YAML config", category);
                }
            }

            // Subcategoria (dropdown) - OTIMIZADO: usar mapeamento com IDs e estrelas
            if let Some(ref sub_categoria) = ai.sub_categoria {
                if let Some(ref category) = ai.category {
                    if let Some(subcat_id) = prompt_config.get_subcategory_id(category, sub_categoria) {
                        let field_id = prompt_config.get_field_ids()
                            .map(|ids| ids.subcategory_field_id.clone())
                            .unwrap_or_else(|| "5333c095-eb40-4a5a-b0c2-76bfba4b1094".to_string());

                        custom_fields.push(CustomField {
                            id: field_id,
                            value: CustomFieldValue::DropdownOption(subcat_id.clone()),
                        });

                        // Adicionar campo de Estrelas (emoji rating)
                        if let Some(stars) = prompt_config.get_subcategory_stars(category, sub_categoria) {
                            let stars_field_id = prompt_config.get_field_ids()
                                .map(|ids| ids.stars_field_id.clone())
                                .unwrap_or_else(|| "83afcb8c-2866-498f-9c62-8ea9666b104b".to_string());

                            custom_fields.push(CustomField {
                                id: stars_field_id,
                                value: CustomFieldValue::Rating(stars as i32),  // Número inteiro de 1 a 4
                            });

                            tracing::info!(
                                "✨ Tarefa classificada: '{}' > '{}' ({} estrela{})",
                                category, sub_categoria, stars, if stars > 1 { "s" } else { "" }
                            );
                        }
                    } else {
                        tracing::warn!(
                            "⚠️ Subcategoria '{}' não encontrada para categoria '{}'",
                            sub_categoria, category
                        );
                    }
                }
            }

            // GARANTIR PRESENÇA DOS CAMPOS CATEGORIA_NOVA E SUBCATEGORIA_NOVA SEMPRE
            // Estes campos usam o ID da opção, não o nome
            {
                let config = prompt_config; // Usar config passado como parâmetro
                if let Some(field_ids) = config.get_field_ids() {
                    // Verificar se Categoria_nova já foi adicionada
                    if !custom_fields.iter().any(|f| f.id == field_ids.category_field_id) {
                        // Se AI retornou categoria, buscar o ID da opção
                        if let Some(ref category) = ai.category {
                            if let Some(cat_id) = config.get_category_id(category) {
                                custom_fields.push(CustomField {
                                    id: field_ids.category_field_id.clone(),
                                    value: CustomFieldValue::DropdownOption(cat_id.clone()),
                                });
                                tracing::debug!("Campo Categoria_nova adicionado: {} -> {}", category, &cat_id);
                            } else {
                                tracing::warn!("Categoria '{}' não tem ID mapeado no YAML", category);
                            }
                        }
                    }

                    // Verificar se SubCategoria_nova já foi adicionada
                    if !custom_fields.iter().any(|f| f.id == field_ids.subcategory_field_id) {
                        // Se AI retornou subcategoria, buscar o ID da opção
                        if let Some(ref sub_categoria) = ai.sub_categoria {
                            if let Some(ref category) = ai.category {
                                if let Some(subcat_id) = config.get_subcategory_id(category, sub_categoria) {
                                    custom_fields.push(CustomField {
                                        id: field_ids.subcategory_field_id.clone(),
                                        value: CustomFieldValue::DropdownOption(subcat_id.clone()),
                                    });
                                    tracing::debug!("Campo SubCategoria_nova adicionado: {} -> {}", sub_categoria, &subcat_id);
                                } else {
                                    tracing::warn!("Subcategoria '{}/{}' não tem ID mapeado no YAML", category, sub_categoria);
                                }
                            }
                        }
                    }
                }
            }

            // Status Back Office (dropdown) - OTIMIZADO: usar YAML como única fonte
            if let Some(ref status) = ai.status_back_office {
                if let Some(status_id) = prompt_config.get_status_id(status) {
                    let field_id = prompt_config.get_field_ids()
                        .map(|ids| ids.status_field_id.clone())
                        .unwrap_or_else(|| "6abbfe79-f80b-4b55-9b4b-9bd7f65b6458".to_string());

                    custom_fields.push(CustomField {
                        id: field_id,
                        value: CustomFieldValue::DropdownOption(status_id),
                    });
                } else {
                    tracing::warn!("Status '{}' não encontrado no YAML config", status);
                }
            }
        }

        // REMOVED: Campo "Cliente Solicitante" (Info_2) e mapeamento YAML
        // Data: 2025-11-07
        // Razão: Simplificação do sistema - removendo mapeamento YAML e busca por Info_2/nome
        // Este bloco de código foi removido porque não é mais necessário adicionar
        // o campo personalizado "Cliente Solicitante" ao payload/task_data.
        //
        // Funcionalidades removidas:
        // - Busca de cliente solicitante via Info_2 e payload.nome
        // - Mapeamento YAML através de AiPromptConfig::load_with_gcs_fallback()
        // - Adição do campo ClickUp ID "0ed63eec-1c50-4190-91c1-59b4b17557f6"
        // - Logs de debug e warning relacionados ao mapeamento de clientes

        // Adicionar campo "Conta cliente" (Info_1) - campo de texto livre
        if let Some(info_1) = payload.campos_personalizados.get("Info_1") {
            if let Some(info_1_str) = info_1.as_str() {
                custom_fields.push(CustomField {
                    id: "0cd1d510-1906-4484-ba66-06ccdd659768".to_string(),  // Campo "Conta cliente"
                    value: CustomFieldValue::Text(info_1_str.to_string()),
                });
                tracing::debug!("Campo Conta cliente (Info_1) adicionado: {}", info_1_str);
            }
        }

        // Adicionar nome, Cliente (Info_1) e celular na descrição, nesta ordem e sem linhas extras
        let mut extra_info = String::new();

        if !payload.nome.is_empty() {
            extra_info.push_str(&format!("\nNome: {}", payload.nome));
        }

        if let Some(info_1) = payload.campos_personalizados.get("Info_1") {
            if let Some(info_1_str) = info_1.as_str() {
                extra_info.push_str(&format!("\nCliente: {}", info_1_str));
            }
        }

        if !payload.celular.is_empty() {
            extra_info.push_str(&format!("\nCelular: {}", payload.celular));
        }

        let final_description = format!("{}{}", description.trim(), extra_info);

        CreateTaskRequest::builder(task_name)
            .description(final_description)
            .tags(Vec::<String>::new())
            .priority(TaskPriority::Normal)
            .custom_fields(custom_fields)
            .build()
    }

    /// Converte payload ChatGuru para ClickUp (FORMATO IDÊNTICO AO LEGADO)
    fn chatguru_to_clickup(&self, payload: &ChatGuruPayload) -> CreateTaskRequest {
        // FORMATO EXATO DO SISTEMA LEGADO
        let mut description = String::new();

        // Dados do Contato - FORMATO LEGADO
        description.push_str("**Dados do Contato**\n\n");
        description.push_str(&format!("- Nome: {}\n", payload.nome));
        description.push_str(&format!("- Email: {}\n",
            if !payload.email.is_empty() { &payload.email } else { &payload.celular }
        ));
        description.push_str(&format!("- Celular: {}\n", payload.celular));
        description.push_str(&format!("- Campanha: {}\n", payload.campanha_nome));
        description.push_str(&format!("- Origem: {}\n",
            if !payload.origem.is_empty() { &payload.origem } else { "scheduler" }
        ));

        // Mensagem - FORMATO LEGADO
        if !payload.texto_mensagem.is_empty() {
            description.push_str("\n**Mensagem**\n");
            description.push_str(&payload.texto_mensagem);
        }

        // Adicionar mídia anexada se houver
        if let Some(ref media_url) = payload.media_url {
            if let Some(ref media_type) = payload.media_type {
                description.push_str("\n\n**Mídia Anexada**\n");

                if media_type.contains("image") || media_type.contains("photo") {
                    // Para imagens, incluir link direto e preview no Markdown
                    description.push_str(&format!("🖼️ Imagem anexada: [Visualizar]({})\n", media_url));
                    // Tentar incluir a imagem inline no Markdown do ClickUp
                    description.push_str(&format!("\n![Imagem anexada]({})\n", media_url));
                } else if media_type.contains("audio") || media_type.contains("voice") {
                    // Para áudios, apenas incluir o link
                    description.push_str(&format!("🎵 Áudio anexado: [Ouvir]({})\n", media_url));
                } else {
                    // Para outros tipos de mídia
                    description.push_str(&format!("📎 Arquivo anexado: [Baixar]({})\n", media_url));
                }
            }
        }

        // Nome da tarefa
        let task_name = payload.nome.clone();

        // Preparar campos personalizados do ClickUp
        let mut custom_fields = Vec::<CustomField>::new();

        // Mapeamento correto: Info_2 → Solicitante, Info_1 → Conta cliente
        if let Some(info_2) = payload.campos_personalizados.get("Info_2") {
            if let Some(info_2_str) = info_2.as_str() {
                // Info_2 vai para o campo "Solicitante (Info_1)"
                custom_fields.push(CustomField {
                    id: "bf24f5b1-e909-473e-b864-75bf22edf67e".to_string(),  // Campo Solicitante
                    value: CustomFieldValue::Text(info_2_str.to_string()),
                });
            }
        }

        if let Some(info_1) = payload.campos_personalizados.get("Info_1") {
            if let Some(info_1_str) = info_1.as_str() {
                // Info_1 vai para o campo "Conta cliente"
                custom_fields.push(CustomField {
                    id: "0cd1d510-1906-4484-ba66-06ccdd659768".to_string(),  // Campo Conta cliente
                    value: CustomFieldValue::Text(info_1_str.to_string()),
                });
            }
        }

        CreateTaskRequest::builder(task_name)
            .description(description.trim().to_string())
            .tags(Vec::<String>::new())  // Legado não usa tags
            .priority(TaskPriority::Normal)  // Prioridade normal (3) como no legado
            .custom_fields(custom_fields)  // Adicionar campos personalizados
            .build()
    }

    /// Converte payload EventType para ClickUp
    fn eventtype_to_clickup(&self, payload: &EventTypePayload) -> CreateTaskRequest {
        let data = &payload.data;

        // Determina o título da tarefa
        let title = if let Some(ref task_title) = data.task_title {
            task_title.clone()
        } else if let Some(ref annotation) = data.annotation {
            annotation.clone()
        } else if let Some(ref lead_name) = data.lead_name {
            format!("[{}] {}", payload.event_type, lead_name)
        } else {
            format!("Evento: {}", payload.event_type)
        };

        // Constrói descrição
        let mut description = format!(
            "**Tipo de Evento**: {}\n\
             **ID**: {}\n\
             **Timestamp**: {}\n\n",
            payload.event_type, payload.id, payload.timestamp
        );

        if let Some(ref lead_name) = data.lead_name {
            description.push_str(&format!("**Nome**: {}\n", lead_name));
        }
        if let Some(ref phone) = data.phone {
            description.push_str(&format!("**Telefone**: {}\n", phone));
        }
        if let Some(ref email) = data.email {
            description.push_str(&format!("**Email**: {}\n", email));
        }
        if let Some(ref project) = data.project_name {
            description.push_str(&format!("**Projeto**: {}\n", project));
        }
        if let Some(ref amount) = data.amount {
            description.push_str(&format!("**Valor**: R$ {:.2}\n", amount));
        }
        if let Some(ref status) = data.status {
            description.push_str(&format!("**Status**: {}\n", status));
        }

        if !data.custom_data.is_empty() {
            description.push_str("\n**Dados Customizados**\n");
            for (key, value) in &data.custom_data {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        // Define tags baseadas no tipo de evento
        let tags = match payload.event_type.as_str() {
            "new_lead" => vec!["lead".to_string(), "novo".to_string()],
            "payment_created" => vec!["pagamento".to_string(), "criado".to_string()],
            "payment_completed" => vec!["pagamento".to_string(), "concluído".to_string()],
            _ => vec![payload.event_type.clone()],
        };

        CreateTaskRequest::builder(title)
            .description(description)
            .tags(tags)
            .priority(TaskPriority::Normal)
            .build()
    }

    /// Converte payload genérico para ClickUp
    fn generic_to_clickup(&self, payload: &GenericPayload) -> CreateTaskRequest {
        let nome = payload.nome.as_ref().map(|s| s.as_str()).unwrap_or("Contato");
        let celular = payload.celular.as_ref().map(|s| s.as_str()).unwrap_or("Não informado");
        let email = payload.email.as_ref().map(|s| s.as_str()).unwrap_or("Não informado");

        let mut description = format!(
            "**Dados do Contato**\n\n\
             - Nome: {}\n\
             - Celular: {}\n\
             - Email: {}\n",
            nome, celular, email
        );

        if let Some(ref mensagem) = payload.mensagem {
            description.push_str(&format!("\n**Mensagem**\n{}\n", mensagem));
        }

        if !payload.extra.is_empty() {
            description.push_str("\n**Dados Adicionais**\n");
            for (key, value) in &payload.extra {
                description.push_str(&format!("- {}: {}\n", key, value));
            }
        }

        CreateTaskRequest::builder(format!("[Webhook] {}", nome))
            .description(description)
            .tags(vec!["webhook".to_string(), "genérico".to_string()])
            .priority(TaskPriority::Normal)
            .build()
    }

    /// Extrai um identificador único para busca de duplicatas
    pub fn get_task_title(&self) -> String {
        match self {
            WebhookPayload::ChatGuru(p) => format!("[{}] {}", p.campanha_nome, p.nome),
            WebhookPayload::EventType(p) => {
                if let Some(ref title) = p.data.task_title {
                    title.clone()
                } else if let Some(ref name) = p.data.lead_name {
                    format!("[{}] {}", p.event_type, name)
                } else {
                    format!("Evento: {}", p.event_type)
                }
            },
            WebhookPayload::Generic(p) => {
                format!("[Webhook] {}", p.nome.as_ref().unwrap_or(&"Contato".to_string()))
            }
        }
    }
}
