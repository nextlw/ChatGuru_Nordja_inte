/// Media Sync Service: Coordena requisições e resultados assíncronos
///
/// FIXME: Este arquivo foi TEMPORARIAMENTE COMENTADO durante a refatoração das estruturas Vertex AI
/// 
/// MOTIVO: A implementação atual usa MediaProcessingResult que foi removido na Fase 1
/// PRÓXIMOS PASSOS: Refatorar para nova arquitetura de chamadas diretas (Fases 2-4)
///
/// TODO Vertex AI Implementation Plan:
/// - Fase 2: Autenticação (Google ADC + OAuth2)  
/// - Fase 3: Processamento de mídia (download + base64)
/// - Fase 4: Cliente HTTP (chamadas diretas à API)
/// - Fase 5: Service principal (integração completa)

use crate::utils::{AppError, AppResult};
use crate::utils::logging::*;
use std::time::Duration;

/// FIXME: Implementação temporária para manter compatibilidade
/// Será completamente refatorado nas próximas fases
#[derive(Clone)]
pub struct MediaSyncService {
    default_timeout: Duration,
}

impl MediaSyncService {
    /// Construtor temporário
    pub fn new(timeout_seconds: u64) -> Self {
        log_info(&format!(
            "Media Sync Service temporarily disabled (timeout: {}s)",
            timeout_seconds
        ));
        
        Self {
            default_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Método temporário - será substituído por chamadas diretas Vertex AI
    pub async fn wait_for_result(&self, correlation_id: String) -> AppResult<String> {
        log_warning(&format!(
            "Media Sync Service is disabled. Correlation ID: {}",
            correlation_id
        ));
        
        Err(AppError::InternalError(
            "Media Sync Service disabled during Vertex AI refactoring".to_string()
        ))
    }

    /// Método temporário - será substituído por chamadas diretas Vertex AI  
    pub async fn notify_result(&self, _result: String) -> bool {
        log_warning("Media Sync Service notify_result is disabled");
        false
    }

    /// Método de monitoramento temporário
    pub async fn pending_count(&self) -> usize {
        0
    }

    /// Método de limpeza temporário
    pub async fn clear_all(&self) {
        log_info("Media Sync Service clear_all called (no-op)");
    }
}

/*
IMPLEMENTAÇÃO ORIGINAL COMENTADA - SERÁ RESTAURADA/REFATORADA NAS PRÓXIMAS FASES:

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};
use tokio::time::timeout;

type ResultSender = oneshot::Sender<MediaProcessingResult>;
type PendingRequests = Arc<RwLock<HashMap<String, ResultSender>>>;

#[derive(Clone)]
pub struct MediaSyncService {
    pending_requests: PendingRequests,
    default_timeout: Duration,
}

impl MediaSyncService {
    pub fn new(timeout_seconds: u64) -> Self {
        // ... implementação original
    }

    pub async fn wait_for_result(&self, correlation_id: String) -> AppResult<MediaProcessingResult> {
        // ... implementação original
    }

    pub async fn notify_result(&self, result: MediaProcessingResult) -> bool {
        // ... implementação original  
    }

    // ... outros métodos
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_media_sync_disabled() {
        let sync = MediaSyncService::new(1);
        
        let result = sync.wait_for_result("test-123".to_string()).await;
        assert!(result.is_err());
        
        let notified = sync.notify_result("test".to_string()).await;
        assert!(!notified);
        
        assert_eq!(sync.pending_count().await, 0);
    }
}
