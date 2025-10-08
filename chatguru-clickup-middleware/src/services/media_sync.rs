/// Media Sync Service: Coordena requisições e resultados assíncronos
///
/// Problema: Worker publica requisição e precisa aguardar resultado
/// Solução: Cache em memória com channels oneshot para notificação
///
/// Fluxo:
/// 1. Worker chama wait_for_result() com correlation_id
/// 2. Cria oneshot channel e guarda no cache
/// 3. Background task lê resultados do Pub/Sub
/// 4. Quando chega resultado, envia via channel
/// 5. Worker recebe resultado ou timeout

use crate::services::vertex::MediaProcessingResult;
use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};
use tokio::time::timeout;

type ResultSender = oneshot::Sender<MediaProcessingResult>;
type PendingRequests = Arc<RwLock<HashMap<String, ResultSender>>>;

/// Serviço de sincronização para processamento de mídia
#[derive(Clone)]
pub struct MediaSyncService {
    pending_requests: PendingRequests,
    default_timeout: Duration,
}

impl MediaSyncService {
    /// Cria nova instância do MediaSyncService
    pub fn new(timeout_seconds: u64) -> Self {
        log_info(&format!(
            "Initializing Media Sync Service (timeout: {}s)",
            timeout_seconds
        ));

        Self {
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            default_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Aguarda resultado de processamento com timeout
    /// Retorna resultado ou erro se timeout/falha
    pub async fn wait_for_result(
        &self,
        correlation_id: String,
    ) -> AppResult<MediaProcessingResult> {
        log_info(&format!("⏳ Aguardando resultado: {}", correlation_id));

        // Criar channel para receber resultado
        let (tx, rx) = oneshot::channel();

        // Registrar requisição pendente
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(correlation_id.clone(), tx);
        }

        // Aguardar resultado com timeout
        match timeout(self.default_timeout, rx).await {
            Ok(Ok(result)) => {
                log_info(&format!("✅ Resultado recebido: {}", correlation_id));

                // Verificar se houve erro no processamento
                if let Some(ref error) = result.error {
                    return Err(AppError::InternalError(format!(
                        "Media processing failed: {}",
                        error
                    )));
                }

                Ok(result)
            }
            Ok(Err(_)) => {
                // Channel foi fechado sem enviar resultado
                log_error(&format!("❌ Channel fechado: {}", correlation_id));
                Err(AppError::InternalError(
                    "Result channel closed unexpectedly".to_string(),
                ))
            }
            Err(_) => {
                // Timeout
                log_warning(&format!(
                    "⏱️ Timeout aguardando resultado: {} ({}s)",
                    correlation_id,
                    self.default_timeout.as_secs()
                ));

                // Limpar requisição pendente
                self.cleanup_pending_request(&correlation_id).await;

                Err(AppError::Timeout(format!(
                    "Timeout waiting for media processing result ({}s)",
                    self.default_timeout.as_secs()
                )))
            }
        }
    }

    /// Notifica que um resultado chegou
    /// Chamado pelo background task que lê do Pub/Sub
    pub async fn notify_result(&self, result: MediaProcessingResult) -> bool {
        let correlation_id = result.correlation_id.clone();

        log_info(&format!("📬 Notificando resultado: {}", correlation_id));

        // Buscar channel pendente
        let sender = {
            let mut pending = self.pending_requests.write().await;
            pending.remove(&correlation_id)
        };

        // Enviar resultado se houver alguém aguardando
        if let Some(tx) = sender {
            match tx.send(result) {
                Ok(_) => {
                    log_info(&format!("✉️ Resultado entregue: {}", correlation_id));
                    true
                }
                Err(_) => {
                    log_warning(&format!(
                        "⚠️ Falha ao entregar resultado (receiver dropped): {}",
                        correlation_id
                    ));
                    false
                }
            }
        } else {
            log_warning(&format!(
                "⚠️ Resultado recebido mas não há requisição pendente: {}",
                correlation_id
            ));
            false
        }
    }

    /// Limpa requisição pendente (chamado em caso de timeout)
    async fn cleanup_pending_request(&self, correlation_id: &str) {
        let mut pending = self.pending_requests.write().await;
        pending.remove(correlation_id);
        log_info(&format!("🧹 Requisição pendente removida: {}", correlation_id));
    }

    /// Retorna número de requisições pendentes (para monitoramento)
    pub async fn pending_count(&self) -> usize {
        let pending = self.pending_requests.read().await;
        pending.len()
    }

    /// Limpa todas as requisições pendentes (usado em shutdown)
    pub async fn clear_all(&self) {
        let mut pending = self.pending_requests.write().await;
        let count = pending.len();
        pending.clear();
        log_info(&format!("🧹 Todas as requisições pendentes limpas ({})", count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_for_result_timeout() {
        let sync = MediaSyncService::new(1); // 1 segundo timeout

        let correlation_id = "test-123".to_string();

        let result = sync.wait_for_result(correlation_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Timeout(_)));
    }

    #[tokio::test]
    async fn test_notify_result_success() {
        let sync = MediaSyncService::new(10);

        let correlation_id = "test-456".to_string();

        // Iniciar wait em background
        let sync_clone = sync.clone();
        let cid_clone = correlation_id.clone();
        let handle = tokio::spawn(async move {
            sync_clone.wait_for_result(cid_clone).await
        });

        // Pequeno delay para garantir que wait iniciou
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Enviar resultado
        let result = MediaProcessingResult {
            correlation_id: correlation_id.clone(),
            result: "transcription text".to_string(),
            media_type: "audio".to_string(),
            error: None,
        };

        let notified = sync.notify_result(result).await;
        assert!(notified);

        // Verificar que wait recebeu resultado
        let wait_result = handle.await.unwrap();
        assert!(wait_result.is_ok());
        assert_eq!(wait_result.unwrap().result, "transcription text");
    }

    #[tokio::test]
    async fn test_pending_count() {
        let sync = MediaSyncService::new(10);

        assert_eq!(sync.pending_count().await, 0);

        // Não pode testar facilmente sem iniciar wait real em background
        // devido à natureza assíncrona
    }
}
