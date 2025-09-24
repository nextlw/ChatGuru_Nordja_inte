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
    
    /// Analisa se deve criar nova tarefa ou atualizar existente
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
        
        // Adicionar mensagem ao histórico
        history.messages.push(MessageRecord {
            timestamp: Utc::now(),
            content: message.to_string(),
            was_activity: is_activity,
        });
        
        // Manter apenas últimas 20 mensagens
        if history.messages.len() > 20 {
            history.messages.drain(0..history.messages.len() - 20);
        }
        
        // Verificar se é modificação de pedido anterior
        let is_modification = self.is_modification(message, &history);
        
        // Atualizar timestamp
        history.last_activity = Utc::now();
        
        // Decidir ação
        if is_modification && history.last_task_id.is_some() {
            // É uma modificação e temos tarefa anterior
            let time_since_last = Utc::now() - history.last_activity;
            
            // Se passou menos de 10 minutos, considerar como atualização
            if time_since_last < Duration::minutes(10) {
                history.revision_count += 1;
                
                let changes = self.detect_changes(message, &history);
                
                log_info(&format!(
                    "Detected modification for task {} (revision #{}): {:?}",
                    history.last_task_id.as_ref().unwrap(),
                    history.revision_count,
                    changes
                ));
                
                return TaskAction::UpdateExisting {
                    task_id: history.last_task_id.clone().unwrap(),
                    changes,
                };
            }
        }
        
        // Criar nova tarefa
        history.revision_count = 0;
        TaskAction::CreateNew
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
    
    /// Detecta se é uma modificação
    fn is_modification(&self, message: &str, history: &ConversationHistory) -> bool {
        let message_lower = message.to_lowercase();
        
        // Palavras que indicam modificação
        let modification_indicators = vec![
            "não", "espera", "muda", "troca", "ao invés", "melhor", 
            "prefiro", "corrige", "altera", "substitui", "esquece",
            "na verdade", "mudei de ideia", "pensando melhor", "afinal",
            "cancela", "remove", "tira",
        ];
        
        // Palavras que indicam adição (não é modificação)
        let addition_indicators = vec![
            "também", "mais", "adiciona", "inclui", "junto",
            "além", "e também", "complementando",
        ];
        
        // Verificar indicadores
        let has_modification = modification_indicators.iter()
            .any(|word| message_lower.contains(word));
        
        let has_addition = addition_indicators.iter()
            .any(|word| message_lower.contains(word));
        
        // Se tem indicador de modificação e não é adição
        if has_modification && !has_addition {
            return true;
        }
        
        // Verificar se menciona produtos do pedido anterior
        if let Some(last_title) = &history.last_task_title {
            let title_words: Vec<&str> = last_title.split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            
            let matching_words = title_words.iter()
                .filter(|w| message_lower.contains(&w.to_lowercase()))
                .count();
            
            // Se menciona muitas palavras do pedido anterior, pode ser modificação
            if matching_words >= 2 && has_modification {
                return true;
            }
        }
        
        false
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