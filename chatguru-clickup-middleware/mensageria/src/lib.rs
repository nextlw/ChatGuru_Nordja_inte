//! Message Queue Service: Agrupa mensagens por chat antes de processar
//!
//! Comportamento Inteligente (SmartContextManager):
//! - Cada chat tem sua própria fila
//! - Processa AUTOMATICAMENTE via callback quando uma das 5 regras é ativada:
//!   1. Closing Message Detection (obrigado, tchau, fechado)
//!   2. Silence Detection (>180s / 3min sem mensagens)
//!   3. Topic Change Detection (keyword overlap <30%)
//!   4. Action Completion Pattern (pergunta→resposta→confirmação)
//!   5. Safety Timeout (8 mensagens OU 180s / 3min)
//! - Scheduler roda a cada 10 segundos verificando condições
//! - Callback centraliza todo envio para Pub/Sub
//!
//! Exemplo:
//! ```text
//! Chat A: "preciso criar tarefa" -> "sobre cliente X" -> "obrigado"
//!         -> SmartContextManager detecta closing -> CALLBACK -> Pub/Sub
//! Chat B: "como fazer?" -> "aqui está" -> "ok"
//!         -> SmartContextManager detecta action completion -> CALLBACK -> Pub/Sub
//! ```

pub mod context_manager;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde_json::Value;
use ia_service::IaService;

pub use context_manager::{SmartContextManager, ContextDecision, MessageContext};

/// Configuração da fila
const MAX_MESSAGES_PER_CHAT: usize = 8;  // Safety timeout: 8 mensagens
const MAX_WAIT_SECONDS: u64 = 180;  // Safety timeout: 180s (3 minutos)
const SCHEDULER_INTERVAL_SECONDS: u64 = 10;

/// Mensagem na fila
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub payload: Value,
    pub received_at: Instant,
}

/// Fila de mensagens para um chat específico
#[derive(Debug)]
pub struct ChatQueue {
    pub messages: Vec<QueuedMessage>,
    pub first_message_at: Instant,
}

impl ChatQueue {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            first_message_at: Instant::now(),
        }
    }

    /// Verifica se a fila está pronta para ser processada usando SmartContextManager
    async fn is_ready_to_process(&self, ia_service: Option<&IaService>) -> (bool, Option<String>) {
        // Extrair payloads e timestamps
        let payloads: Vec<Value> = self.messages.iter().map(|m| m.payload.clone()).collect();
        let timestamps: Vec<Instant> = self.messages.iter().map(|m| m.received_at).collect();

        // Calcular similaridade semântica se IaService disponível e houver 3+ mensagens
        let semantic_similarity = if let Some(ia) = ia_service {
            if self.messages.len() >= 3 {
                // Converter para MessageContext
                let contexts: Vec<MessageContext> = payloads
                    .iter()
                    .zip(timestamps.iter())
                    .filter_map(|(payload, timestamp)| MessageContext::from_payload(payload, *timestamp))
                    .collect();

                if contexts.len() >= 3 {
                    SmartContextManager::calculate_semantic_similarity(ia, &contexts).await
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Usar SmartContextManager para decidir
        match SmartContextManager::should_process_now(&payloads, &timestamps, semantic_similarity) {
            ContextDecision::ProcessNow { reason } => (true, Some(reason)),
            ContextDecision::Wait => (false, None),
        }
    }

    /// Verifica se a fila está pronta (versão legada - mantida para compatibilidade de testes)
    #[allow(dead_code)]
    fn is_ready_to_process_legacy(&self) -> bool {
        // Pronta se atingiu o limite de mensagens
        if self.messages.len() >= MAX_MESSAGES_PER_CHAT {
            return true;
        }

        // Pronta se passou o tempo máximo
        let elapsed = self.first_message_at.elapsed();
        elapsed >= Duration::from_secs(MAX_WAIT_SECONDS)
    }

    /// Adiciona uma mensagem à fila
    fn push(&mut self, payload: Value) {
        self.messages.push(QueuedMessage {
            payload,
            received_at: Instant::now(),
        });
    }

    /// Extrai todas as mensagens (consome a fila)
    fn drain(&mut self) -> Vec<QueuedMessage> {
        std::mem::take(&mut self.messages)
    }
}

/// Serviço de fila de mensagens
pub struct MessageQueueService {
    queues: Arc<RwLock<HashMap<String, ChatQueue>>>,
    on_batch_ready: Option<Arc<dyn Fn(String, Vec<QueuedMessage>) + Send + Sync>>,
    ia_service: Option<Arc<IaService>>, // Serviço de IA para análise semântica
}

impl MessageQueueService {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            on_batch_ready: None,
            ia_service: None,
        }
    }

    /// Define callback para quando um batch estiver pronto
    pub fn with_batch_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(String, Vec<QueuedMessage>) + Send + Sync + 'static,
    {
        self.on_batch_ready = Some(Arc::new(callback));
        self
    }

    /// Define serviço de IA para análise semântica (opcional, mas recomendado)
    pub fn with_ia_service(mut self, ia_service: IaService) -> Self {
        self.ia_service = Some(Arc::new(ia_service));
        self
    }

    /// Adiciona uma mensagem à fila do chat
    /// Processa automaticamente quando atingir 8 mensagens ou 180 segundos
    pub async fn enqueue(&self, chat_id: String, payload: Value) {
        let mut queues = self.queues.write().await;

        // Log do payload recebido (debug)
        tracing::debug!(
            "📥 Payload recebido para chat '{}': {}",
            chat_id,
            serde_json::to_string(&payload).unwrap_or_else(|_| "invalid".to_string())
        );

        // Criar fila se não existir
        let queue = queues.entry(chat_id.clone()).or_insert_with(ChatQueue::new);

        // Adicionar mensagem
        queue.push(payload);

        tracing::info!(
            "📬 Chat '{}': {} mensagens na fila (aguardando análise SmartContextManager)",
            chat_id,
            queue.messages.len()
        );

        // Verificar se está pronto para processar usando SmartContextManager
        // Passar IaService se disponível para análise semântica com embeddings
        let ia_service_ref = self.ia_service.as_ref().map(|arc| arc.as_ref());
        let (is_ready, reason_opt) = queue.is_ready_to_process(ia_service_ref).await;

        if is_ready {
            let reason = reason_opt.unwrap_or_else(|| "Condição desconhecida".to_string());
            let message_count = queue.messages.len();

            tracing::info!(
                "🚀 Chat '{}': Fila pronta para processar ({}) - {} mensagens acumuladas",
                chat_id,
                reason,
                message_count
            );

            // Remover fila do HashMap e processar com callback
            if let Some(mut queue) = queues.remove(&chat_id) {
                let messages = queue.drain();

                // Se há callback registrado, chamar
                if let Some(ref callback) = self.on_batch_ready {
                    tracing::info!(
                        "📤 Chat '{}': Enviando {} mensagens para callback (processamento via {})",
                        chat_id,
                        message_count,
                        reason
                    );

                    let cb = Arc::clone(callback);
                    let chat_id_clone = chat_id.clone();
                    tokio::spawn(async move {
                        tracing::debug!(
                            "🔄 Chat '{}': Callback iniciado para processar {} mensagens",
                            chat_id_clone,
                            messages.len()
                        );
                        cb(chat_id_clone, messages);
                    });
                } else {
                    // Fallback: apenas agregar e logar
                    tracing::warn!(
                        "⚠️ Chat '{}': Nenhum callback configurado - usando fallback",
                        chat_id
                    );

                    let chat_id_clone = chat_id.clone();
                    tokio::spawn(async move {
                        match aggregate_messages(chat_id_clone.clone(), messages) {
                            Ok(payload) => {
                                tracing::info!(
                                    "✅ Batch agregado para chat '{}' (sem callback configurado)",
                                    chat_id_clone
                                );
                                tracing::debug!(
                                    "📋 Payload agregado (não enviado): {}",
                                    serde_json::to_string(&payload).unwrap_or_else(|_| "invalid".to_string())
                                );
                            }
                            Err(e) => {
                                tracing::error!("❌ Erro ao agregar batch do chat '{}': {}", chat_id_clone, e);
                            }
                        }
                    });
                }
            }
        } else {
            tracing::debug!(
                "⏳ Chat '{}': Mensagem adicionada, aguardando mais ({}/{})",
                chat_id,
                queue.messages.len(),
                MAX_MESSAGES_PER_CHAT
            );
        }
    }

    /// Inicia o scheduler que processa filas por timeout
    pub fn start_scheduler(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(SCHEDULER_INTERVAL_SECONDS));

            loop {
                interval.tick().await;

                if let Err(e) = self.check_timeouts().await {
                    tracing::error!("❌ Erro ao verificar timeouts: {}", e);
                }
            }
        });

        tracing::info!(
            "🕐 Scheduler iniciado: verifica filas a cada {}s",
            SCHEDULER_INTERVAL_SECONDS
        );
    }

    /// Verifica filas que atingiram o timeout e as processa
    async fn check_timeouts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let queues = self.queues.read().await;

        tracing::trace!("🔍 Verificando timeouts em {} filas ativas", queues.len());

        // Coletar chat_ids para verificar (não podemos iterar e await ao mesmo tempo)
        let chat_ids: Vec<String> = queues.keys().cloned().collect();
        drop(queues); // Liberar read lock

        let mut ready_chats = Vec::new();

        // Verificar cada chat de forma assíncrona
        for chat_id in chat_ids {
            let queues = self.queues.read().await;
            if let Some(queue) = queues.get(&chat_id) {
                let ia_service_ref = self.ia_service.as_ref().map(|arc| arc.as_ref());
                let (is_ready, reason_opt) = queue.is_ready_to_process(ia_service_ref).await;

                if is_ready {
                    let elapsed = queue.first_message_at.elapsed().as_secs();
                    let message_count = queue.messages.len();
                    let reason = reason_opt.unwrap_or_else(|| "Condição desconhecida".to_string());

                    tracing::info!(
                        "⏰ Chat '{}': SmartContextManager ativado ({}s, {} mensagens) - Razão: {}",
                        chat_id,
                        elapsed,
                        message_count,
                        reason
                    );

                    ready_chats.push((chat_id.clone(), message_count, elapsed));
                }
            }
        }

        // Processar chats prontos (agora com write lock)
        let mut queues = self.queues.write().await;

        // Processar chats prontos
        for (chat_id, message_count, elapsed_secs) in ready_chats {
            if let Some(mut queue) = queues.remove(&chat_id) {
                let messages = queue.drain();

                // Se há callback registrado, chamar
                if let Some(ref callback) = self.on_batch_ready {
                    tracing::info!(
                        "📤 Chat '{}': Enviando {} mensagens para callback (timeout após {}s)",
                        chat_id,
                        message_count,
                        elapsed_secs
                    );

                    let cb = Arc::clone(callback);
                    let chat_id_clone = chat_id.clone();
                    tokio::spawn(async move {
                        tracing::debug!(
                            "🔄 Chat '{}': Callback iniciado para processar {} mensagens (timeout)",
                            chat_id_clone,
                            messages.len()
                        );
                        cb(chat_id_clone, messages);
                    });
                } else {
                    // Fallback: apenas agregar e logar
                    tracing::warn!(
                        "⚠️ Chat '{}': Nenhum callback configurado para timeout - usando fallback",
                        chat_id
                    );

                    tokio::spawn(async move {
                        match aggregate_messages(chat_id.clone(), messages) {
                            Ok(payload) => {
                                tracing::info!(
                                    "✅ Batch agregado para chat '{}' (timeout sem callback configurado)",
                                    chat_id
                                );
                                tracing::debug!(
                                    "📋 Payload agregado por timeout (não enviado): {}",
                                    serde_json::to_string(&payload).unwrap_or_else(|_| "invalid".to_string())
                                );
                            }
                            Err(e) => {
                                tracing::error!("❌ Erro ao agregar batch do chat '{}': {}", chat_id, e);
                            }
                        }
                    });
                }
            }
        }

        Ok(())
    }

    /// Processa um batch de mensagens e retorna o payload agregado
    pub fn process_batch_sync(
        chat_id: String,
        messages: Vec<QueuedMessage>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        aggregate_messages(chat_id, messages)
    }

    /// Obtém estatísticas das filas (para debug/monitoring)
    pub async fn get_stats(&self) -> HashMap<String, usize> {
        let queues = self.queues.read().await;
        queues
            .iter()
            .map(|(chat_id, queue)| (chat_id.clone(), queue.messages.len()))
            .collect()
    }
}

/// Agrega mensagens em um único payload
///
/// Lógica:
/// 1. Usa a PRIMEIRA mensagem como base (mantém campos_personalizados, chat_id, etc)
/// 2. Agrupa todos os texto_mensagem concatenando-os
/// 3. Mantém consistência nos outros campos
fn aggregate_messages(
    chat_id: String,
    messages: Vec<QueuedMessage>,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "📦 Agregando batch do chat '{}': {} mensagens para processar",
        chat_id,
        messages.len()
    );

    if messages.is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }

    // Usar PRIMEIRA mensagem como base
    let mut aggregated_payload = messages[0].payload.clone();

    // Agregar todos os texto_mensagem
    let mut aggregated_text = String::new();

    for (idx, msg) in messages.iter().enumerate() {
        // Calcular tempo decorrido
        let elapsed = msg.received_at.elapsed().as_secs();

        // Extrair texto_mensagem (pode vir como "texto_mensagem", "mensagem", "message", ou "text")
        let text = msg.payload
            .get("texto_mensagem")
            .or_else(|| msg.payload.get("mensagem"))
            .or_else(|| msg.payload.get("message"))
            .or_else(|| msg.payload.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if !text.is_empty() {
            if !aggregated_text.is_empty() {
                aggregated_text.push_str("\n\n");
            }

            // Adicionar timestamp relativo para contexto
            aggregated_text.push_str(&format!(
                "[Mensagem {} - há {}s]\n{}",
                idx + 1,
                elapsed,
                text
            ));
        }

        tracing::debug!(
            "  Mensagem {}/{}: '{}' (recebida há {}s)",
            idx + 1,
            messages.len(),
            text.chars().take(50).collect::<String>(),
            elapsed
        );
    }

    // Atualizar texto_mensagem no payload agregado
    if let Some(obj) = aggregated_payload.as_object_mut() {
        obj.insert("texto_mensagem".to_string(), Value::String(aggregated_text.clone()));

        // Adicionar metadados do batch
        obj.insert("_batch_size".to_string(), Value::Number(messages.len().into()));
        obj.insert("_batch_chat_id".to_string(), Value::String(chat_id.clone()));
    }

    tracing::info!(
        "✅ Batch agregado: {} mensagens → {} caracteres de texto",
        messages.len(),
        aggregated_text.len()
    );

    // Log detalhado do payload final (debug)
    tracing::debug!(
        "📋 Payload final agregado para chat '{}' (batch_size={}):\n{}",
        chat_id,
        messages.len(),
        serde_json::to_string_pretty(&aggregated_payload).unwrap_or_default()
    );

    Ok(aggregated_payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn test_queue_fills_up() {
        // Capturar batches processados pelo callback
        let processed_batches = Arc::new(Mutex::new(Vec::new()));
        let processed_clone = Arc::clone(&processed_batches);

        let service = MessageQueueService::new()
            .with_batch_callback(move |chat_id, messages| {
                processed_clone.lock().unwrap().push((chat_id, messages.len()));
            });

        let chat_id = "test_chat".to_string();

        // Adicionar 7 mensagens com texto_mensagem (formato esperado pelo SmartContextManager)
        for i in 1..=7 {
            service.enqueue(
                chat_id.clone(),
                serde_json::json!({
                    "texto_mensagem": format!("Mensagem de teste {}", i),
                    "chat_id": chat_id
                })
            ).await;
        }

        // Aguardar um pouco para garantir que callback não foi chamado
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(
            processed_batches.lock().unwrap().len(),
            0,
            "Não deve processar com 7 mensagens"
        );

        // Adicionar 8ª mensagem - deve processar pelo Safety Timeout (8 mensagens)
        service.enqueue(
            chat_id.clone(),
            serde_json::json!({
                "texto_mensagem": "Mensagem de teste 8",
                "chat_id": chat_id
            })
        ).await;

        // Aguardar callback ser executado
        tokio::time::sleep(Duration::from_millis(100)).await;

        let batches = processed_batches.lock().unwrap();
        assert_eq!(batches.len(), 1, "Deve processar com 8 mensagens");
        assert_eq!(batches[0].0, chat_id, "Chat ID deve corresponder");
        assert_eq!(batches[0].1, 8, "Deve ter 8 mensagens no batch");
    }

    #[tokio::test]
    async fn test_multiple_chats() {
        let service = MessageQueueService::new();

        // Chat A: 3 mensagens
        for i in 1..=3 {
            service.enqueue(
                "chat_a".to_string(),
                serde_json::json!({"msg": i})
            ).await;
        }

        // Chat B: 2 mensagens
        for i in 1..=2 {
            service.enqueue(
                "chat_b".to_string(),
                serde_json::json!({"msg": i})
            ).await;
        }

        let stats = service.get_stats().await;
        assert_eq!(stats.get("chat_a"), Some(&3));
        assert_eq!(stats.get("chat_b"), Some(&2));
    }

    #[tokio::test]
    async fn test_batch_aggregation() {
        let service = MessageQueueService::new();
        let chat_id = "test_aggregation".to_string();

        // Adicionar mensagens com texto
        for i in 1..=3 {
            service.enqueue(
                chat_id.clone(),
                serde_json::json!({
                    "texto_mensagem": format!("Mensagem {}", i),
                    "chat_id": chat_id
                })
            ).await;
        }

        let stats = service.get_stats().await;
        assert_eq!(stats.get(&chat_id), Some(&3), "Deve ter 3 mensagens na fila");
    }
}
