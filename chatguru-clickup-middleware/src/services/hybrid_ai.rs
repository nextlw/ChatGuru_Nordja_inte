/// HybridAIService: Sistema Incremental com Fallback OpenAI → Vertex AI
///
/// ARQUITETURA INCREMENTAL:
/// - Sistema OpenAI atual permanece 100% INTACTO
/// - Vertex AI é uma EXTENSÃO OPCIONAL
/// - Fallback automático: Vertex AI falha → OpenAI (garantido)
/// - Feature flags para ativar/desativar componentes
/// - Zero breaking changes no sistema atual
///
/// ESTRATÉGIA DE SEGURANÇA:
/// 1. OpenAI Service é a BASE CONFIÁVEL (sempre funciona)
/// 2. Vertex AI é EXPERIMENTAL (pode ser desabilitado)
/// 3. Configuração via TOML define comportamento
/// 4. Logs detalhados para debugging

use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use crate::services::openai::{OpenAIService, OpenAIClassification};
use crate::services::vertex::VertexAIService;
use crate::config::Settings;
use serde::{Deserialize, Serialize};

/// Configuração do Hybrid AI Service
#[derive(Debug, Clone)]
pub struct HybridAIConfig {
    /// Usar Vertex AI como primeira opção (se disponível)
    pub use_vertex_primary: bool,
    
    /// Vertex AI está habilitado no sistema
    pub vertex_enabled: bool,
    
    /// Timeout para Vertex AI (ms) antes de fallback
    pub vertex_timeout_ms: u64,
    
    /// Sempre usar OpenAI como fallback
    pub openai_fallback_enabled: bool,
    
    /// Log detalhado de decisões de AI
    pub verbose_logging: bool,
}

impl Default for HybridAIConfig {
    fn default() -> Self {
        Self {
            use_vertex_primary: false, // Por padrão, usar OpenAI
            vertex_enabled: false,     // Vertex desabilitado por padrão
            vertex_timeout_ms: 15000,  // 15s timeout
            openai_fallback_enabled: true, // Sempre garantir fallback
            verbose_logging: true,     // Logs detalhados por padrão
        }
    }
}

impl HybridAIConfig {
    /// Cria configuração a partir das settings do sistema
    pub fn from_settings(settings: &Settings) -> Self {
        let vertex_enabled = settings.vertex
            .as_ref()
            .map(|v| v.enabled)
            .unwrap_or(false);
        
        let ai_settings = settings.ai.as_ref();
        
        Self {
            use_vertex_primary: vertex_enabled, // Se Vertex habilitado, usar como primário
            vertex_enabled,
            vertex_timeout_ms: settings.vertex
                .as_ref()
                .map(|v| v.timeout_seconds * 1000)
                .unwrap_or(15000),
            openai_fallback_enabled: true, // SEMPRE garantir fallback OpenAI
            verbose_logging: ai_settings
                .map(|ai| ai.enabled)
                .unwrap_or(true),
        }
    }
}

/// Resultado unificado de classificação AI
/// 
/// Mantém compatibilidade 100% com OpenAIClassification existente
/// para não quebrar nenhum código atual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridAIResult {
    /// Classificação no formato OpenAI (compatibilidade garantida)
    pub classification: OpenAIClassification,
    
    /// Metadados sobre qual serviço foi usado
    pub metadata: AIProcessingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProcessingMetadata {
    /// Serviço que foi usado para classificação
    pub service_used: AIServiceType,
    
    /// Se houve fallback
    pub fallback_occurred: bool,
    
    /// Tempo de processamento em ms
    pub processing_time_ms: u64,
    
    /// Mensagem de debug
    pub debug_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIServiceType {
    OpenAI,
    VertexAI,
}

/// HybridAIService: Orquestrador inteligente entre OpenAI e Vertex AI
#[derive(Clone)]
pub struct HybridAIService {
    /// OpenAI Service (BASE CONFIÁVEL - sempre inicializado)
    openai_service: OpenAIService,
    
    /// Vertex AI Service (OPCIONAL - pode ser None)
    vertex_service: Option<VertexAIService>,
    
    /// Configuração do hybrid service
    config: HybridAIConfig,
}

impl HybridAIService {
    /// Cria novo HybridAIService (incrementa sistema atual)
    ///
    /// GARANTIAS DE SEGURANÇA:
    /// - OpenAI Service sempre inicializado (sistema atual)
    /// - Vertex AI opcional (pode falhar sem afetar funcionamento)
    /// - Se Vertex AI falha na inicialização, sistema continua só com OpenAI
    pub async fn new(settings: &Settings) -> AppResult<Self> {
        let config = HybridAIConfig::from_settings(settings);
        
        if config.verbose_logging {
            log_info("🔄 Inicializando HybridAIService (sistema incremental)");
            log_info(&format!("📋 Configuração: vertex_enabled={}, use_vertex_primary={}, fallback_enabled={}", 
                config.vertex_enabled, config.use_vertex_primary, config.openai_fallback_enabled));
        }
        
        // 1. SEMPRE inicializar OpenAI Service (BASE CONFIÁVEL)
        let openai_service = match OpenAIService::new(None).await {
            Some(service) => {
                if config.verbose_logging {
                    log_info("✅ OpenAI Service inicializado com sucesso (sistema base)");
                }
                service
            }
            None => {
                log_error("❌ FALHA CRÍTICA: Não foi possível inicializar OpenAI Service");
                return Err(AppError::InternalError(
                    "OpenAI Service é obrigatório para funcionamento do sistema".to_string()
                ));
            }
        };
        
        // 2. OPCIONALMENTE inicializar Vertex AI Service (EXPERIMENTAL)
        let vertex_service = if config.vertex_enabled {
            if config.verbose_logging {
                log_info("🚀 Tentando inicializar Vertex AI Service (experimental)...");
            }
            
            let vertex_settings = settings.vertex.as_ref().unwrap(); // Safe unwrap, já verificamos enabled
            
            match VertexAIService::new(
                vertex_settings.project_id.clone(),
                vertex_settings.location.clone(),
                vertex_settings.model.clone(),
            ).await {
                Ok(service) => {
                    if config.verbose_logging {
                        log_info("✅ Vertex AI Service inicializado com sucesso");
                    }
                    Some(service)
                }
                Err(e) => {
                    log_warning(&format!("⚠️ Falha ao inicializar Vertex AI Service: {}. Continuando apenas com OpenAI.", e));
                    None
                }
            }
        } else {
            if config.verbose_logging {
                log_info("⏭️ Vertex AI desabilitado na configuração, usando apenas OpenAI");
            }
            None
        };
        
        if config.verbose_logging {
            let status = match (&vertex_service, config.vertex_enabled) {
                (Some(_), true) => "OpenAI + Vertex AI disponíveis",
                (None, true) => "Apenas OpenAI (Vertex AI falhou)",
                (None, false) => "Apenas OpenAI (Vertex AI desabilitado)",
                (Some(_), false) => "Estado inconsistente", // Não deveria acontecer
            };
            log_info(&format!("🎯 HybridAIService inicializado: {}", status));
        }
        
        Ok(Self {
            openai_service,
            vertex_service,
            config,
        })
    }
    
    /// Cria HybridAIService a partir do AppState (para integração com worker)
    pub async fn from_app_state(state: &std::sync::Arc<crate::AppState>) -> AppResult<Self> {
        log_info("🏗️ Criando HybridAIService a partir do AppState");
        Self::new(&state.settings).await
    }
    
    /// Classifica atividade com fallback automático
    ///
    /// FLUXO INCREMENTAL:
    /// 1. Se Vertex AI disponível e habilitado → tentar Vertex AI primeiro
    /// 2. Se Vertex AI falha OU não disponível → usar OpenAI (GARANTIDO)
    /// 3. Sistema atual OpenAI nunca é afetado
    pub async fn classify_activity(&self, context: &str) -> AppResult<HybridAIResult> {
        let start_time = std::time::Instant::now();
        
        if self.config.verbose_logging {
            log_info(&format!("🔍 Iniciando classificação híbrida para contexto ({} chars)", context.len()));
        }
        
        // ESTRATÉGIA 1: Tentar Vertex AI se disponível e habilitado
        if self.config.use_vertex_primary && self.vertex_service.is_some() {
            if self.config.verbose_logging {
                log_info("🚀 Tentando classificação com Vertex AI (primário)");
            }
            
            match self.try_vertex_classification(context).await {
                Ok(classification) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    
                    if self.config.verbose_logging {
                        log_info(&format!("✅ Vertex AI classificação bem-sucedida em {}ms", processing_time));
                    }
                    
                    return Ok(HybridAIResult {
                        classification,
                        metadata: AIProcessingMetadata {
                            service_used: AIServiceType::VertexAI,
                            fallback_occurred: false,
                            processing_time_ms: processing_time,
                            debug_message: "Vertex AI successful".to_string(),
                        },
                    });
                }
                Err(e) => {
                    if self.config.verbose_logging {
                        log_warning(&format!("⚠️ Vertex AI falhou: {}. Fazendo fallback para OpenAI.", e));
                    }
                    // Continua para fallback OpenAI
                }
            }
        }
        
        // ESTRATÉGIA 2: OpenAI (SISTEMA BASE - sempre funciona)
        if self.config.verbose_logging {
            let reason = if self.config.use_vertex_primary && self.vertex_service.is_some() {
                "fallback após falha Vertex AI"
            } else if !self.config.vertex_enabled {
                "Vertex AI desabilitado"
            } else {
                "Vertex AI não disponível"
            };
            log_info(&format!("🔄 Usando OpenAI ({})", reason));
        }
        
        match self.openai_service.classify_activity_fallback(context).await {
            Ok(classification) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                
                if self.config.verbose_logging {
                    log_info(&format!("✅ OpenAI classificação bem-sucedida em {}ms", processing_time));
                }
                
                Ok(HybridAIResult {
                    classification,
                    metadata: AIProcessingMetadata {
                        service_used: AIServiceType::OpenAI,
                        fallback_occurred: self.config.use_vertex_primary && self.vertex_service.is_some(),
                        processing_time_ms: processing_time,
                        debug_message: "OpenAI successful".to_string(),
                    },
                })
            }
            Err(e) => {
                log_error(&format!("❌ FALHA CRÍTICA: OpenAI também falhou: {}", e));
                Err(AppError::InternalError(format!(
                    "Todos os serviços AI falharam. OpenAI: {}", e
                )))
            }
        }
    }
    
    /// Tenta classificação com Vertex AI (com timeout)
    async fn try_vertex_classification(&self, context: &str) -> AppResult<OpenAIClassification> {
        let _vertex_service = self.vertex_service.as_ref()
            .ok_or_else(|| AppError::InternalError("Vertex AI service not available".to_string()))?;
        
        // TODO: Implementar conversão Vertex AI → OpenAIClassification
        // Por enquanto, returna erro para forçar fallback para OpenAI
        // Isso será implementado na próxima fase
        
        let _vertex_prompt = format!(
            "Classifique se a seguinte mensagem representa uma atividade de trabalho válida.\n\nContexto: {}\n\nResponda em JSON com: is_activity, reason, category, sub_categoria",
            context
        );
        
        // Placeholder: Na implementação real, chamaria vertex_service.process_text()
        // e converteria o resultado para OpenAIClassification
        Err(AppError::VertexError("Vertex AI classification not implemented yet".to_string()))
    }
    
    /// Processa mídia com fallback automático
    ///
    /// COMPATIBILIDADE: Mantém mesma interface que OpenAI Service atual
    pub async fn process_media(&self, media_url: &str, media_type: &str) -> AppResult<String> {
        if self.config.verbose_logging {
            log_info(&format!("📎 Processando mídia: {} ({})", media_url, media_type));
        }
        
        // ESTRATÉGIA 1: Tentar Vertex AI se disponível (multimodal nativo)
        if self.config.use_vertex_primary && self.vertex_service.is_some() {
            if self.config.verbose_logging {
                log_info("🚀 Tentando processamento de mídia com Vertex AI");
            }
            
            if let Ok(result) = self.try_vertex_media_processing(media_url, media_type).await {
                if self.config.verbose_logging {
                    log_info("✅ Vertex AI processamento de mídia bem-sucedido");
                }
                return Ok(result);
            } else if self.config.verbose_logging {
                log_warning("⚠️ Vertex AI mídia falhou, fazendo fallback para OpenAI");
            }
        }
        
        // ESTRATÉGIA 2: OpenAI (SISTEMA ATUAL - sempre funciona)
        if self.config.verbose_logging {
            log_info("🔄 Usando OpenAI para processamento de mídia");
        }
        
        if media_type.contains("audio") {
            // Áudio: usar OpenAI Whisper
            match self.openai_service.download_audio(media_url).await {
                Ok(audio_bytes) => {
                    let extension = media_url
                        .split('.')
                        .last()
                        .and_then(|ext| ext.split('?').next())
                        .unwrap_or("ogg");
                    
                    self.openai_service.transcribe_audio(&audio_bytes, extension).await
                }
                Err(e) => Err(e),
            }
        } else if media_type.contains("image") {
            // Imagem: usar OpenAI Vision
            match self.openai_service.download_image(media_url).await {
                Ok(image_bytes) => {
                    self.openai_service.describe_image(&image_bytes).await
                }
                Err(e) => Err(e),
            }
        } else {
            Err(AppError::InternalError(format!("Tipo de mídia não suportado: {}", media_type)))
        }
    }
    
    /// Tenta processamento de mídia com Vertex AI
    async fn try_vertex_media_processing(&self, _media_url: &str, _media_type: &str) -> AppResult<String> {
        // TODO: Implementar processamento multimodal real
        // Por enquanto, força fallback para OpenAI
        Err(AppError::VertexError("Vertex AI media processing not implemented yet".to_string()))
    }
    
    /// Retorna status dos serviços disponíveis
    pub fn get_service_status(&self) -> ServiceStatus {
        ServiceStatus {
            openai_available: true, // Sempre true se inicializou
            vertex_available: self.vertex_service.is_some(),
            vertex_enabled: self.config.vertex_enabled,
            current_strategy: if self.config.use_vertex_primary && self.vertex_service.is_some() {
                "Vertex AI primary with OpenAI fallback".to_string()
            } else {
                "OpenAI only".to_string()
            },
        }
    }
    
    /// Testa conectividade com todos os serviços
    pub async fn test_connectivity(&self) -> ConnectivityTestResult {
        let mut result = ConnectivityTestResult {
            openai_status: "unknown".to_string(),
            vertex_status: "unknown".to_string(),
            overall_health: "unknown".to_string(),
        };
        
        // Teste OpenAI
        match self.openai_service.classify_activity_fallback("teste de conectividade").await {
            Ok(_) => {
                result.openai_status = "healthy".to_string();
                if self.config.verbose_logging {
                    log_info("✅ OpenAI conectividade OK");
                }
            }
            Err(e) => {
                result.openai_status = format!("error: {}", e);
                log_error(&format!("❌ OpenAI conectividade falhou: {}", e));
            }
        }
        
        // Teste Vertex AI (se disponível)
        if let Some(vertex_service) = &self.vertex_service {
            match vertex_service.test_connection().await {
                Ok(_) => {
                    result.vertex_status = "healthy".to_string();
                    if self.config.verbose_logging {
                        log_info("✅ Vertex AI conectividade OK");
                    }
                }
                Err(e) => {
                    result.vertex_status = format!("error: {}", e);
                    log_warning(&format!("⚠️ Vertex AI conectividade falhou: {}", e));
                }
            }
        } else {
            result.vertex_status = "not_available".to_string();
        }
        
        // Status geral
        result.overall_health = if result.openai_status == "healthy" {
            "healthy".to_string() // OpenAI é suficiente para saúde geral
        } else {
            "degraded".to_string()
        };
        
        result
    }

    /// Métodos de compatibilidade para integração com worker existente
    ///
    /// Estes métodos mantêm a mesma interface que o worker espera,
    /// mas usam o sistema híbrido internamente
    
    /// Classifica atividade retornando apenas OpenAIClassification (compatibilidade)
    pub async fn classify_activity_with_fallback(&self, context: &str) -> AppResult<OpenAIClassification> {
        match self.classify_activity(context).await {
            Ok(hybrid_result) => Ok(hybrid_result.classification),
            Err(e) => Err(e),
        }
    }
    
    /// Processa áudio com fallback automático
    pub async fn process_audio_with_fallback(&self, media_url: &str) -> AppResult<String> {
        self.process_media(media_url, "audio").await
    }
    
    /// Processa imagem com fallback automático
    pub async fn process_image_with_fallback(&self, media_url: &str) -> AppResult<String> {
        self.process_media(media_url, "image").await
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub openai_available: bool,
    pub vertex_available: bool,
    pub vertex_enabled: bool,
    pub current_strategy: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectivityTestResult {
    pub openai_status: String,
    pub vertex_status: String,
    pub overall_health: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::{
        Settings, ServerSettings, ClickUpSettings, GcpSettings,
        ChatGuruSettings, AISettings, VertexSettings
    };

    /// Cria configuração de teste com Vertex AI desabilitado (comportamento padrão)
    fn create_test_settings_vertex_disabled() -> Settings {
        Settings {
            server: ServerSettings { host: "0.0.0.0".to_string(), port: 8080 },
            clickup: ClickUpSettings {
                token: "test_token".to_string(),
                list_id: "test_list".to_string(),
                base_url: "https://api.clickup.com".to_string(),
            },
            gcp: GcpSettings {
                project_id: "test-project".to_string(),
                topic_name: "test-topic".to_string(),
                subscription_name: "test-sub".to_string(),
                pubsub_topic: None,
                media_processing_topic: None,
                media_results_topic: None,
                media_results_subscription: None,
            },
            chatguru: ChatGuruSettings {
                webhook_secret: None,
                validate_signature: false,
                legacy_response_mode: None,
                api_token: None,
                api_endpoint: None,
                account_id: None,
                phone_ids: None,
            },
            vertex: None, // Vertex AI desabilitado
            ai: Some(AISettings {
                enabled: true,
                use_hybrid: Some(true),
                prefer_vertex: Some(false),
            }),
            hybrid_ai: None,
        }
    }

    /// Cria configuração de teste com Vertex AI habilitado
    fn create_test_settings_vertex_enabled() -> Settings {
        Settings {
            server: ServerSettings { host: "0.0.0.0".to_string(), port: 8080 },
            clickup: ClickUpSettings {
                token: "test_token".to_string(),
                list_id: "test_list".to_string(),
                base_url: "https://api.clickup.com".to_string(),
            },
            gcp: GcpSettings {
                project_id: "test-project".to_string(),
                topic_name: "test-topic".to_string(),
                subscription_name: "test-sub".to_string(),
                pubsub_topic: None,
                media_processing_topic: None,
                media_results_topic: None,
                media_results_subscription: None,
            },
            chatguru: ChatGuruSettings {
                webhook_secret: None,
                validate_signature: false,
                legacy_response_mode: None,
                api_token: None,
                api_endpoint: None,
                account_id: None,
                phone_ids: None,
            },
            vertex: Some(VertexSettings {
                enabled: true,
                project_id: "test-project".to_string(),
                location: "us-central1".to_string(),
                model: Some("gemini-1.5-pro".to_string()),
                timeout_seconds: 30,
                max_media_size_mb: None,
                supported_mime_types: None,
                generation: None,
            }),
            ai: Some(AISettings {
                enabled: true,
                use_hybrid: Some(true),
                prefer_vertex: Some(true),
            }),
            hybrid_ai: None,
        }
    }

    #[test]
    fn test_hybrid_ai_config_default() {
        let config = HybridAIConfig::default();
        
        assert!(!config.use_vertex_primary, "Por padrão, deve usar OpenAI como primário");
        assert!(!config.vertex_enabled, "Por padrão, Vertex AI deve estar desabilitado");
        assert!(config.openai_fallback_enabled, "Fallback OpenAI sempre habilitado");
        assert!(config.verbose_logging, "Logs detalhados por padrão");
        assert_eq!(config.vertex_timeout_ms, 15000, "Timeout padrão 15s");
        
        println!("✅ HybridAIConfig::default() funcionando corretamente");
    }

    #[test]
    fn test_hybrid_ai_config_from_settings_vertex_disabled() {
        let settings = create_test_settings_vertex_disabled();
        let config = HybridAIConfig::from_settings(&settings);
        
        assert!(!config.vertex_enabled, "Vertex AI deve estar desabilitado");
        assert!(!config.use_vertex_primary, "OpenAI deve ser primário");
        assert!(config.openai_fallback_enabled, "Fallback sempre habilitado");
        
        println!("✅ Configuração com Vertex AI desabilitado funcionando");
    }

    #[test]
    fn test_hybrid_ai_config_from_settings_vertex_enabled() {
        let settings = create_test_settings_vertex_enabled();
        let config = HybridAIConfig::from_settings(&settings);
        
        assert!(config.vertex_enabled, "Vertex AI deve estar habilitado");
        assert!(config.use_vertex_primary, "Vertex AI deve ser primário quando habilitado");
        assert!(config.openai_fallback_enabled, "Fallback sempre habilitado");
        assert_eq!(config.vertex_timeout_ms, 30000, "Timeout configurado: 30s");
        
        println!("✅ Configuração com Vertex AI habilitado funcionando");
    }

    #[test]
    fn test_ai_service_type_serialization() {
        use serde_json;
        
        let openai_type = AIServiceType::OpenAI;
        let vertex_type = AIServiceType::VertexAI;
        
        let openai_json = serde_json::to_string(&openai_type).unwrap();
        let vertex_json = serde_json::to_string(&vertex_type).unwrap();
        
        assert_eq!(openai_json, "\"OpenAI\"");
        assert_eq!(vertex_json, "\"VertexAI\"");
        
        println!("✅ Serialização de AIServiceType funcionando");
    }

    #[test]
    fn test_hybrid_ai_result_structure() {
        use crate::services::openai::OpenAIClassification;
        
        let classification = OpenAIClassification {
            reason: "Teste".to_string(),
            is_activity: true,
            category: Some("Logística".to_string()),
            campanha: Some("WhatsApp".to_string()),
            description: Some("Teste".to_string()),
            space_name: None,
            folder_name: None,
            list_name: None,
            info_1: None,
            info_2: None,
            tipo_atividade: None,
            sub_categoria: Some("Corrida de motoboy".to_string()),
            subtasks: vec![],
            status_back_office: None,
        };
        
        let metadata = AIProcessingMetadata {
            service_used: AIServiceType::OpenAI,
            fallback_occurred: false,
            processing_time_ms: 150,
            debug_message: "OpenAI successful".to_string(),
        };
        
        let result = HybridAIResult {
            classification: classification.clone(),
            metadata,
        };
        
        // Verificar que a estrutura está correta
        assert_eq!(result.classification.reason, "Teste");
        assert!(result.classification.is_activity);
        assert_eq!(result.metadata.processing_time_ms, 150);
        assert!(!result.metadata.fallback_occurred);
        
        println!("✅ Estrutura HybridAIResult correta");
    }

    #[test]
    fn test_service_status_structure() {
        let status = ServiceStatus {
            openai_available: true,
            vertex_available: false,
            vertex_enabled: false,
            current_strategy: "OpenAI only".to_string(),
        };
        
        assert!(status.openai_available);
        assert!(!status.vertex_available);
        assert!(!status.vertex_enabled);
        assert_eq!(status.current_strategy, "OpenAI only");
        
        println!("✅ Estrutura ServiceStatus correta");
    }

    #[test]
    fn test_connectivity_test_result_structure() {
        let result = ConnectivityTestResult {
            openai_status: "healthy".to_string(),
            vertex_status: "not_available".to_string(),
            overall_health: "healthy".to_string(),
        };
        
        assert_eq!(result.openai_status, "healthy");
        assert_eq!(result.vertex_status, "not_available");
        assert_eq!(result.overall_health, "healthy");
        
        println!("✅ Estrutura ConnectivityTestResult correta");
    }

    /// Teste de feature flags
    #[test]
    fn test_feature_flag_vertex_disabled_by_default() {
        let settings = create_test_settings_vertex_disabled(); // Vertex = None
        
        let config = HybridAIConfig::from_settings(&settings);
        
        assert!(!config.vertex_enabled, "Vertex AI deve estar desabilitado por padrão");
        assert!(!config.use_vertex_primary, "OpenAI deve ser primário por padrão");
        
        println!("✅ Feature flag: Vertex AI desabilitado por padrão");
    }

    #[test]
    fn test_feature_flag_vertex_explicitly_disabled() {
        let mut settings = create_test_settings_vertex_disabled();
        settings.vertex = Some(VertexSettings {
            enabled: false, // Explicitamente desabilitado
            project_id: "test".to_string(),
            location: "us-central1".to_string(),
            model: None,
            timeout_seconds: 30,
            max_media_size_mb: None,
            supported_mime_types: None,
            generation: None,
        });
        
        let config = HybridAIConfig::from_settings(&settings);
        
        assert!(!config.vertex_enabled, "Vertex AI deve estar desabilitado quando enabled=false");
        assert!(!config.use_vertex_primary, "OpenAI deve ser primário quando Vertex desabilitado");
        
        println!("✅ Feature flag: Vertex AI explicitamente desabilitado");
    }

    #[test]
    fn test_feature_flag_vertex_enabled() {
        let settings = create_test_settings_vertex_enabled();
        
        let config = HybridAIConfig::from_settings(&settings);
        
        assert!(config.vertex_enabled, "Vertex AI deve estar habilitado quando enabled=true");
        assert!(config.use_vertex_primary, "Vertex AI deve ser primário quando habilitado");
        assert!(config.openai_fallback_enabled, "Fallback OpenAI sempre habilitado");
        
        println!("✅ Feature flag: Vertex AI habilitado");
    }
}