use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

/// Cache de contexto para reduzir custos com Vertex AI
/// Armazena classificações recentes e padrões identificados
#[derive(Debug, Clone)]
pub struct ContextCache {
    /// Cache de mensagens já classificadas (hash da mensagem -> classificação)
    message_cache: Arc<RwLock<HashMap<String, CachedClassification>>>,
    
    /// Padrões aprendidos (palavras-chave -> é atividade)
    patterns: Arc<RwLock<PatternMatcher>>,
    
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

#[derive(Debug, Clone)]
struct PatternMatcher {
    /// Palavras que indicam atividade
    activity_keywords: Vec<String>,
    
    /// Palavras que indicam NÃO atividade
    non_activity_keywords: Vec<String>,
    
    /// Frases exatas já classificadas
    exact_matches: HashMap<String, bool>,
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
            patterns: Arc::new(RwLock::new(PatternMatcher::default())),
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
    
    /// Tenta classificar usando padrões aprendidos (sem chamar AI)
    pub async fn classify_by_pattern(&self, message: &str) -> Option<(bool, String)> {
        let patterns = self.patterns.read().await;
        let message_lower = message.to_lowercase();
        
        // 1. Verificar correspondência exata
        if let Some(&is_activity) = patterns.exact_matches.get(&message_lower) {
            let mut stats = self.stats.write().await;
            stats.pattern_hits += 1;
            stats.total_saved += 0.0000075;
            
            let reason = if is_activity {
                "Mensagem idêntica já classificada como atividade"
            } else {
                "Mensagem idêntica já classificada como não-atividade"
            };
            return Some((is_activity, reason.to_string()));
        }
        
        // 2. Verificar palavras-chave com alta confiança
        let activity_score: i32 = patterns.activity_keywords
            .iter()
            .filter(|kw| message_lower.contains(kw.as_str()))
            .count() as i32;
            
        let non_activity_score: i32 = patterns.non_activity_keywords
            .iter()
            .filter(|kw| message_lower.contains(kw.as_str()))
            .count() as i32;
        
        // Se há forte indicação baseada em padrões
        if activity_score > 0 && non_activity_score == 0 {
            let mut stats = self.stats.write().await;
            stats.pattern_hits += 1;
            stats.total_saved += 0.0000075;
            
            return Some((true, format!("Contém palavras-chave de atividade: {}", activity_score)));
        }
        
        if non_activity_score > 0 && activity_score == 0 {
            let mut stats = self.stats.write().await;
            stats.pattern_hits += 1;
            stats.total_saved += 0.0000075;
            
            return Some((false, format!("Contém palavras-chave de não-atividade: {}", non_activity_score)));
        }
        
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
        
        // 2. Aprender padrões
        let mut patterns = self.patterns.write().await;
        let message_lower = message.to_lowercase();
        
        // Se a mensagem é curta, adicionar como correspondência exata
        if message.len() < 100 {
            patterns.exact_matches.insert(message_lower.clone(), is_activity);
        }
        
        // Extrair e aprender palavras-chave
        self.learn_keywords(&mut patterns, &message_lower, is_activity).await;
        
        // 3. Limpar cache antigo (manter últimas 1000 entradas)
        if cache.len() > 1000 {
            self.cleanup_old_cache(&mut cache).await;
        }
    }
    
    /// Aprende palavras-chave das mensagens classificadas
    async fn learn_keywords(&self, patterns: &mut PatternMatcher, message: &str, is_activity: bool) {
        // Palavras-chave comuns para atividades
        let activity_indicators = vec![
            "preciso", "quero", "necessito", "comprar", "orçamento",
            "pedido", "encomenda", "urgente", "favor", "solicito",
            "peças", "material", "produto", "serviço", "reparo",
            "caixas", "unidades", "metros", "quilos", "litros",
        ];
        
        // Palavras-chave comuns para NÃO atividades
        let non_activity_indicators = vec![
            "oi", "olá", "bom dia", "boa tarde", "boa noite",
            "tudo bem", "como está", "obrigado", "abraço", "tchau",
            "ok", "certo", "entendi", "sim", "não", "talvez",
        ];
        
        if is_activity {
            for word in &activity_indicators {
                if message.contains(word) && !patterns.activity_keywords.contains(&word.to_string()) {
                    patterns.activity_keywords.push(word.to_string());
                }
            }
        } else {
            for word in &non_activity_indicators {
                if message.contains(word) && !patterns.non_activity_keywords.contains(&word.to_string()) {
                    patterns.non_activity_keywords.push(word.to_string());
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
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self {
            activity_keywords: vec![
                "preciso".to_string(),
                "quero".to_string(),
                "pedido".to_string(),
                "orçamento".to_string(),
                "comprar".to_string(),
            ],
            non_activity_keywords: vec![
                "oi".to_string(),
                "olá".to_string(),
                "bom dia".to_string(),
                "boa tarde".to_string(),
                "tudo bem".to_string(),
            ],
            exact_matches: HashMap::new(),
        }
    }
}