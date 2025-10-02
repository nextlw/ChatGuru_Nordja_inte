use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::services::smart_classifier::SmartClassifier;

/// Cache de contexto para reduzir custos com Vertex AI
/// Armazena classificações recentes e padrões identificados
#[derive(Debug, Clone)]
pub struct ContextCache {
    /// Cache de mensagens já classificadas (hash da mensagem -> classificação)
    message_cache: Arc<RwLock<HashMap<String, CachedClassification>>>,

    /// Classificador inteligente (TF-IDF + Stemming)
    smart_classifier: Arc<RwLock<SmartClassifier>>,

    /// Estatísticas para otimização
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedClassification {
    is_activity: bool,
    reason: String,
    confidence: f32,
    timestamp: DateTime<Utc>,
    ttl_minutes: i64,
}


#[derive(Debug, Clone, Default)]
struct CacheStats {
    total_requests: u64,
    cache_hits: u64,
    pattern_hits: u64,
    ai_calls: u64,
    total_saved: f64, // Em dólares
}

impl ContextCache {
    pub fn new() -> Self {
        Self {
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            smart_classifier: Arc::new(RwLock::new(SmartClassifier::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    /// Verifica se a mensagem já foi classificada recentemente
    pub async fn get_cached_classification(&self, message: &str) -> Option<(bool, String)> {
        let hash = self.hash_message(message);
        let cache = self.message_cache.read().await;
        
        if let Some(cached) = cache.get(&hash) {
            // Verificar se ainda está válido (TTL)
            let age = Utc::now() - cached.timestamp;
            if age < Duration::minutes(cached.ttl_minutes) {
                // Atualizar estatísticas
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                stats.total_saved += 0.0000075; // Custo economizado
                
                return Some((cached.is_activity, cached.reason.clone()));
            }
        }
        
        None
    }
    
    /// Tenta classificar usando SmartClassifier (TF-IDF + Stemming)
    pub async fn classify_by_pattern(&self, message: &str) -> Option<(bool, String)> {
        let classifier = self.smart_classifier.read().await;

        let (is_activity, confidence) = classifier.classify(message);

        // Só retornar se confiança >= 0.7
        if confidence >= 0.7 {
            let mut stats = self.stats.write().await;
            stats.pattern_hits += 1;
            stats.total_saved += 0.0000075;

            let reason = if is_activity {
                format!("Classificação TF-IDF: atividade (confiança: {:.2})", confidence)
            } else {
                format!("Classificação TF-IDF: não-atividade (confiança: {:.2})", confidence)
            };

            return Some((is_activity, reason));
        }

        // Se confiança < 0.7, deixar para AI decidir
        None
    }
    
    /// Armazena uma nova classificação e aprende padrões
    pub async fn store_classification(
        &self,
        message: &str,
        is_activity: bool,
        reason: &str,
        confidence: f32,
    ) {
        // 1. Adicionar ao cache
        let hash = self.hash_message(message);
        let mut cache = self.message_cache.write().await;

        // TTL baseado na confiança (maior confiança = TTL mais longo)
        let ttl_minutes = if confidence > 0.9 {
            60  // 1 hora para alta confiança
        } else if confidence > 0.7 {
            30  // 30 minutos para confiança média
        } else {
            15  // 15 minutos para baixa confiança
        };

        cache.insert(hash, CachedClassification {
            is_activity,
            reason: reason.to_string(),
            confidence,
            timestamp: Utc::now(),
            ttl_minutes,
        });

        // 2. Fazer SmartClassifier aprender com esta classificação
        let mut classifier = self.smart_classifier.write().await;
        classifier.learn(message, is_activity, confidence);

        // 3. Auto-save padrões aprendidos a cada 50 classificações
        drop(classifier);  // Liberar lock antes de chamar auto_save
        self.auto_save_if_needed().await;

        // 4. Limpar cache antigo (manter últimas 1000 entradas)
        if cache.len() > 1000 {
            self.cleanup_old_cache(&mut cache).await;
        }
    }

    /// Auto-save de padrões aprendidos periodicamente
    async fn auto_save_if_needed(&self) {
        let stats = self.stats.read().await;

        // Auto-save a cada 50 requisições (ajustável)
        let should_save = stats.total_requests % 50 == 0 && stats.total_requests > 0;
        let total_requests = stats.total_requests;

        if should_save {
            drop(stats);  // Liberar lock antes de salvar

            let classifier = self.smart_classifier.read().await;

            if classifier.learned_patterns_count() > 0 {
                if let Err(e) = classifier.save_to_file("config/learned_patterns.json") {
                    use crate::utils::logging::log_error;
                    log_error(&format!("Failed to auto-save learned patterns: {}", e));
                } else {
                    use crate::utils::logging::log_info;
                    log_info(&format!(
                        "💾 Auto-saved {} learned patterns (after {} requests)",
                        classifier.learned_patterns_count(),
                        total_requests
                    ));
                }
            }
        }
    }
    
    /// Limpa entradas antigas do cache
    async fn cleanup_old_cache(&self, cache: &mut HashMap<String, CachedClassification>) {
        let now = Utc::now();
        cache.retain(|_, v| {
            let age = now - v.timestamp;
            age < Duration::hours(2) // Manter no máximo 2 horas
        });
    }
    
    /// Gera hash da mensagem para uso como chave
    fn hash_message(&self, message: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Retorna estatísticas do cache
    pub async fn get_stats(&self) -> String {
        let stats = self.stats.read().await;
        let hit_rate = if stats.total_requests > 0 {
            ((stats.cache_hits + stats.pattern_hits) as f64 / stats.total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        format!(
            "Cache Stats: Total: {}, Hits: {}, Patterns: {}, AI Calls: {}, Hit Rate: {:.1}%, Saved: ${:.4}",
            stats.total_requests,
            stats.cache_hits,
            stats.pattern_hits,
            stats.ai_calls,
            hit_rate,
            stats.total_saved
        )
    }
    
    /// Incrementa o contador de requisições
    pub async fn increment_request_count(&self) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
    }
    
    /// Incrementa o contador de chamadas AI
    pub async fn increment_ai_calls(&self) {
        let mut stats = self.stats.write().await;
        stats.ai_calls += 1;
    }

    /// Força o save de padrões aprendidos (útil para shutdown graceful)
    pub async fn force_save_patterns(&self) -> Result<(), Box<dyn std::error::Error>> {
        let classifier = self.smart_classifier.read().await;

        if classifier.learned_patterns_count() > 0 {
            classifier.save_to_file("config/learned_patterns.json")?;

            use crate::utils::logging::log_info;
            log_info(&format!(
                "💾 Force-saved {} learned patterns on shutdown",
                classifier.learned_patterns_count()
            ));
        }

        Ok(())
    }

    /// Executa limpeza de padrões fracos
    pub async fn cleanup_weak_patterns(&self) {
        let mut classifier = self.smart_classifier.write().await;
        classifier.cleanup_weak_patterns();
    }
}

