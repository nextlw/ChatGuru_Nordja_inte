#[allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::utils::logging::*;

/// Rastreador simplificado de conversas para detectar mudanças de ideia
#[derive(Clone)]
pub struct ConversationTracker {
    /// Mapa de conversas (phone -> histórico)
    conversations: Arc<RwLock<HashMap<String, ConversationHistory>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConversationHistory {
    pub phone: String,
    pub last_task_id: Option<String>,
    pub last_task_title: Option<String>,
    pub messages: Vec<MessageRecord>,
    pub last_activity: DateTime<Utc>,
    pub revision_count: u32,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MessageRecord {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub was_activity: bool,
    pub embedding: Option<Vec<f32>>,  // Embedding para análise semântica
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskAction {
    CreateNew,
    UpdateExisting { task_id: String, changes: Vec<String> },
    NoAction,
}

impl ConversationTracker {
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calcula similaridade coseno entre dois embeddings
    #[allow(dead_code)]
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return 0.0;
        }

        dot_product / (magnitude_a * magnitude_b)
    }

    /// Verifica se uma mensagem é muito curta para ter contexto suficiente
    pub fn is_short_message(message: &str) -> bool {
        let words: Vec<&str> = message.split_whitespace().collect();
        words.len() <= 3  // 3 palavras ou menos = mensagem curta
    }

    /// Agrega mensagens curtas recentes para formar contexto
    pub async fn aggregate_recent_context(&self, phone: &str, current_message: &str, max_messages: usize) -> String {
        let conversations = self.conversations.read().await;

        if let Some(history) = conversations.get(phone) {
            // Pegar últimas N mensagens
            let recent_messages: Vec<String> = history.messages
                .iter()
                .rev()
                .take(max_messages)
                .rev()
                .map(|m| m.content.clone())
                .collect();

            if !recent_messages.is_empty() {
                // Combinar mensagens recentes + mensagem atual
                let mut full_context = recent_messages.join(" ");
                full_context.push_str(" ");
                full_context.push_str(current_message);
                return full_context;
            }
        }

        current_message.to_string()
    }
    
    /// Analisa se deve criar nova tarefa ou atualizar existente
    /// Usa embeddings semânticos quando disponíveis, fallback para keywords
    pub async fn analyze_context(
        &self,
        phone: &str,
        message: &str,
        is_activity: bool,
    ) -> TaskAction {
        if !is_activity {
            return TaskAction::NoAction;
        }

        let mut conversations = self.conversations.write().await;
        let history = conversations.entry(phone.to_string()).or_insert_with(|| {
            ConversationHistory {
                phone: phone.to_string(),
                last_task_id: None,
                last_task_title: None,
                messages: Vec::new(),
                last_activity: Utc::now(),
                revision_count: 0,
            }
        });

        // 🔧 FIX: Capturar timestamp ANTES de atualizar para calcular tempo corretamente
        let last_activity_timestamp = history.last_activity;
        let time_since_last = Utc::now() - last_activity_timestamp;

        // Verificar se é modificação de pedido anterior
        // Prioridade: 1) Embeddings (mais preciso), 2) Keywords (fallback)
        let is_modification = self.is_modification_smart(message, &history);

        // Adicionar mensagem ao histórico (embedding será adicionado depois se disponível)
        history.messages.push(MessageRecord {
            timestamp: Utc::now(),
            content: message.to_string(),
            was_activity: is_activity,
            embedding: None,  // Será preenchido externamente se necessário
        });

        // Manter apenas últimas 20 mensagens
        if history.messages.len() > 20 {
            history.messages.drain(0..history.messages.len() - 20);
        }

        // Decidir ação baseado na modificação e tempo
        if is_modification && history.last_task_id.is_some() {
            // Se passou menos de 10 minutos desde a última atividade, considerar como atualização
            if time_since_last < Duration::minutes(10) {
                history.revision_count += 1;

                let changes = self.detect_changes(message, &history);

                log_info(&format!(
                    "🔄 Detected modification for task {} (revision #{}, {} minutes ago): {:?}",
                    history.last_task_id.as_ref().unwrap(),
                    history.revision_count,
                    time_since_last.num_minutes(),
                    changes
                ));

                // Atualizar timestamp DEPOIS de decidir
                history.last_activity = Utc::now();

                return TaskAction::UpdateExisting {
                    task_id: history.last_task_id.clone().unwrap(),
                    changes,
                };
            } else {
                log_info(&format!(
                    "⏰ Time window expired ({} minutes > 10), creating new task instead of updating",
                    time_since_last.num_minutes()
                ));
            }
        }

        // Criar nova tarefa
        log_info(&format!(
            "✨ Creating new task (is_modification: {}, has_previous_task: {}, time_since_last: {} min)",
            is_modification,
            history.last_task_id.is_some(),
            time_since_last.num_minutes()
        ));

        history.revision_count = 0;
        history.last_activity = Utc::now();
        TaskAction::CreateNew
    }

    /// Análise inteligente de modificação usando embeddings quando disponíveis
    fn is_modification_smart(&self, message: &str, history: &ConversationHistory) -> bool {
        // 1. Procurar última atividade com embedding
        let last_activity_with_embedding = history.messages.iter().rev()
            .find(|m| m.was_activity && m.embedding.is_some());

        if let Some(prev_msg) = last_activity_with_embedding {
            if let Some(ref _prev_embedding) = prev_msg.embedding {
                // TODO: Gerar embedding da mensagem atual (será feito pelo VertexAI)
                // Por enquanto, usar método baseado em keywords
                log_info("📊 Previous activity has embedding, but current message embedding not yet generated");
            }
        }

        // 2. Fallback: Análise baseada em padrões linguísticos (melhorado)
        self.is_modification_by_patterns(message, history)
    }

    /// Análise com embeddings (método público para uso externo com embeddings pré-calculados)
    pub async fn analyze_with_embeddings_sync(
        &self,
        phone: &str,
        current_embedding: &[f32],
        message: &str,
    ) -> (bool, f32) {
        let conversations = self.conversations.read().await;

        if let Some(history) = conversations.get(phone) {
            // Encontrar a última atividade com embedding
            for msg in history.messages.iter().rev() {
                if msg.was_activity {
                    if let Some(ref prev_embedding) = msg.embedding {
                        let similarity = Self::cosine_similarity(current_embedding, prev_embedding);

                        log_info(&format!(
                            "📊 Semantic similarity analysis: {:.3} (high: >0.75, medium: 0.5-0.75, low: <0.5)",
                            similarity
                        ));

                        // Alta similaridade (>0.75) = provavelmente continuação/modificação
                        if similarity > 0.75 {
                            // Confirmar com padrões linguísticos
                            let has_modification_patterns = self.is_modification_by_patterns(message, history);

                            if has_modification_patterns {
                                log_info(&format!("🎯 HIGH confidence modification: similarity={:.3} + modification patterns", similarity));
                                return (true, similarity);
                            } else {
                                // Alta similaridade mas sem padrões de modificação
                                // Pode ser continuação do mesmo assunto
                                log_info(&format!("📝 Same topic but no modification patterns (similarity={:.3})", similarity));
                                return (false, similarity);
                            }
                        }

                        // Baixa similaridade (<0.5) = provavelmente nova atividade
                        if similarity < 0.5 {
                            log_info(&format!("✨ LOW similarity ({:.3}) - likely new activity", similarity));
                            return (false, similarity);
                        }

                        // Similaridade média (0.5-0.75) - usar padrões linguísticos como critério final
                        let is_modification = self.is_modification_by_patterns(message, history);
                        log_info(&format!(
                            "🤔 MEDIUM similarity ({:.3}) - using linguistic patterns: {}",
                            similarity,
                            if is_modification { "MODIFICATION" } else { "NEW TASK" }
                        ));
                        return (is_modification, similarity);
                    }
                }
            }
        }

        // Sem embeddings disponíveis - usar método baseado em padrões
        log_info("📝 No previous embeddings found - using linguistic pattern analysis");
        (false, 0.0)
    }

    /// Análise de modificação usando padrões linguísticos inteligentes
    fn is_modification_by_patterns(&self, message: &str, history: &ConversationHistory) -> bool {
        let message_lower = message.to_lowercase();

        // Padrão 1: Negação explícita + referência ao anterior
        let negation_patterns = [
            r"\bnão\b.*\b(quero|preciso|é|era)\b",  // "não quero", "não preciso"
            r"\b(melhor|prefiro)\b.*\b(fazer|ser|pedir)\b",  // "melhor fazer", "prefiro ser"
            r"\b(muda|troca|altera|substitui)\b.*\bpor\b",  // "muda por", "troca por"
        ];

        for pattern in &negation_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(&message_lower) {
                log_info(&format!("🎯 Modification detected by negation pattern: {}", pattern));
                return true;
            }
        }

        // Padrão 2: Frases de mudança de ideia
        let change_of_mind_phrases = [
            "na verdade",
            "mudei de ideia",
            "pensando melhor",
            "ao invés dis",
            "esquece",
            "cancela",
            "espera",
        ];

        for phrase in &change_of_mind_phrases {
            if message_lower.contains(phrase) {
                log_info(&format!("🎯 Modification detected by change-of-mind phrase: {}", phrase));
                return true;
            }
        }

        // Padrão 3: Referência ao pedido anterior com palavras de modificação
        if let Some(last_title) = &history.last_task_title {
            let title_words: Vec<&str> = last_title.split_whitespace()
                .filter(|w| w.len() > 3)  // Palavras significativas
                .collect();

            // Contar quantas palavras do pedido anterior aparecem na mensagem
            let matching_words = title_words.iter()
                .filter(|w| message_lower.contains(&w.to_lowercase()))
                .count();

            // Se menciona palavras do pedido anterior + tem palavra de modificação
            if matching_words >= 2 {
                let modification_words = ["não", "muda", "troca", "altera", "corrige", "remove", "tira"];
                if modification_words.iter().any(|w| message_lower.contains(w)) {
                    log_info(&format!(
                        "🎯 Modification detected: references previous task ({} words) + modification word",
                        matching_words
                    ));
                    return true;
                }
            }
        }

        // Padrão 4: Adição explícita NÃO é modificação
        let addition_indicators = ["também", "mais", "adiciona", "inclui", "junto", "além"];
        if addition_indicators.iter().any(|w| message_lower.contains(w)) {
            log_info("➕ Detected addition keywords - NOT a modification, will create new task");
            return false;
        }

        false
    }
    
    /// Adiciona embedding à última mensagem de um usuário
    pub async fn add_embedding_to_last_message(&self, phone: &str, embedding: Vec<f32>) {
        let mut conversations = self.conversations.write().await;
        if let Some(history) = conversations.get_mut(phone) {
            if let Some(last_msg) = history.messages.last_mut() {
                last_msg.embedding = Some(embedding);
                log_info(&format!("Embedding added to last message for {}", phone));
            }
        }
    }

    /// Registra uma tarefa criada
    pub async fn register_task(&self, phone: &str, task_id: String, title: String) {
        let mut conversations = self.conversations.write().await;
        if let Some(history) = conversations.get_mut(phone) {
            history.last_task_id = Some(task_id);
            history.last_task_title = Some(title);
            history.last_activity = Utc::now();
            
            log_info(&format!(
                "Registered task {} for phone {}",
                history.last_task_id.as_ref().unwrap(),
                phone
            ));
        }
    }
    

    
    /// Detecta mudanças específicas
    fn detect_changes(&self, message: &str, _history: &ConversationHistory) -> Vec<String> {
        let mut changes = Vec::new();
        let message_lower = message.to_lowercase();
        
        // Detectar tipo de mudança
        if message_lower.contains("cancela") || message_lower.contains("remove") {
            changes.push("Remoção de item".to_string());
        }
        
        if message_lower.contains("troca") || message_lower.contains("substitui") {
            changes.push("Substituição de item".to_string());
        }
        
        if message_lower.contains("quantidade") || 
           regex::Regex::new(r"\b\d+\s*(unidade|caixa|metro|kg|litro)").unwrap().is_match(&message_lower) {
            changes.push("Alteração de quantidade".to_string());
        }
        
        if message_lower.contains("urgente") || message_lower.contains("prioridade") {
            changes.push("Alteração de urgência".to_string());
        }
        
        // Adicionar a mensagem como contexto
        changes.push(format!("Nova instrução: {}", message));
        
        changes
    }
    
    /// Limpa conversas antigas
    #[allow(dead_code)]
    pub async fn cleanup_old_conversations(&self) {
        let mut conversations = self.conversations.write().await;
        let now = Utc::now();
        
        conversations.retain(|_phone, history| {
            let age = now - history.last_activity;
            age < Duration::hours(2) // Manter por 2 horas
        });
    }
}