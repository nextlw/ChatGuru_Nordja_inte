//! GCP Optimizations - Fase 3: Rate limiting, circuit breaker, batch processing e monitoramento
//! 
//! Este módulo implementa as otimizações nativas do GCP para melhorar
//! resiliência, performance e observabilidade do middleware.

use anyhow::Result;
use governor::{Quota, RateLimiter};
use prometheus::{Counter, Histogram, IntGauge, Registry};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::settings::{
    BatchProcessingConfig, CircuitBreakerConfig, MonitoringConfig, RateLimitingConfig,
};

/// Gerenciador de otimizações GCP
#[derive(Clone)]
pub struct GCPOptimizations {
    pub rate_limiter: Arc<RateLimiter<governor::DefaultKeyedRateLimiter<String>>>,
    pub circuit_breaker: Arc<SimpleCircuitBreaker>,
    pub batch_processor: Arc<BatchProcessor>,
    pub metrics_collector: Arc<MetricsCollector>,
}

/// Circuit breaker simples implementado internamente
#[derive(Debug)]
pub struct SimpleCircuitBreaker {
    config: CircuitBreakerConfig,
    failure_count: AtomicU32,
    last_failure_time: AtomicU64,
    state: AtomicU32, // 0 = Closed, 1 = Open, 2 = HalfOpen
    success_count: AtomicU32,
}

#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

/// Processador de lotes para otimizar throughput
pub struct BatchProcessor {
    config: BatchProcessingConfig,
    pending_tasks: Arc<RwLock<Vec<BatchTask>>>,
}

/// Item de tarefa em lote
#[derive(Debug, Clone)]
pub struct BatchTask {
    pub task_id: String,
    pub payload: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Coletor de métricas customizadas
pub struct MetricsCollector {
    registry: Registry,
    pub task_duration: Histogram,
    pub api_calls: Counter,
    pub batch_size: Histogram,
    pub circuit_breaker_state: IntGauge,
    pub active_tasks: IntGauge,
}

impl GCPOptimizations {
    /// Cria uma nova instância das otimizações GCP
    pub fn new(
        rate_config: Option<RateLimitingConfig>,
        circuit_config: Option<CircuitBreakerConfig>,
        batch_config: Option<BatchProcessingConfig>,
        monitoring_config: Option<MonitoringConfig>,
    ) -> Result<Self> {
        info!("🚀 Inicializando otimizações GCP - Fase 3");

        // Rate Limiter
        let rate_limiter = Self::create_rate_limiter(rate_config)?;

        // Circuit Breaker
        let circuit_breaker = Self::create_circuit_breaker(circuit_config)?;

        // Batch Processor
        let batch_processor = Arc::new(BatchProcessor::new(
            batch_config.unwrap_or_default(),
        )?);

        // Metrics Collector
        let metrics_collector = Arc::new(MetricsCollector::new(
            monitoring_config.unwrap_or_default(),
        )?);

        Ok(Self {
            rate_limiter,
            circuit_breaker,
            batch_processor,
            metrics_collector,
        })
    }

    /// Cria rate limiter baseado na configuração
    fn create_rate_limiter(
        config: Option<RateLimitingConfig>,
    ) -> Result<Arc<RateLimiter<governor::DefaultKeyedRateLimiter<String>>>> {
        let config = config.unwrap_or_default();
        
        let quota = Quota::per_second(
            std::num::NonZeroU32::new(config.max_dispatches_per_second as u32)
                .unwrap_or(std::num::NonZeroU32::new(10).unwrap()),
        )
        .allow_burst(
            std::num::NonZeroU32::new(config.burst_capacity)
                .unwrap_or(std::num::NonZeroU32::new(20).unwrap()),
        );

        let limiter = RateLimiter::keyed(quota);
        
        info!(
            "✅ Rate limiter configurado: {:.1} req/s, burst: {}",
            config.max_dispatches_per_second, config.burst_capacity
        );

        Ok(Arc::new(limiter))
    }

    /// Cria circuit breaker baseado na configuração
    fn create_circuit_breaker(
        config: Option<CircuitBreakerConfig>,
    ) -> Result<Arc<SimpleCircuitBreaker>> {
        let config = config.unwrap_or_default();

        let circuit_breaker = SimpleCircuitBreaker::new(config.clone());
        
        info!(
            "✅ Circuit breaker configurado: threshold: {}, timeout: {}s",
            config.failure_threshold, config.timeout_duration
        );

        Ok(Arc::new(circuit_breaker))
    }

    /// Verifica rate limit antes de processar
    pub async fn check_rate_limit(&self, key: &str) -> bool {
        match self.rate_limiter.check_key(key) {
            Ok(_) => {
                debug!("✅ Rate limit OK para chave: {}", key);
                true
            }
            Err(_) => {
                warn!("⚠️ Rate limit excedido para chave: {}", key);
                self.metrics_collector.increment_rate_limited();
                false
            }
        }
    }

    /// Executa operação através do circuit breaker
    pub async fn execute_with_circuit_breaker<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        match self.circuit_breaker.call(operation).await {
            Ok(result) => {
                debug!("✅ Operação executada com sucesso através do circuit breaker");
                self.metrics_collector.record_circuit_breaker_success();
                Ok(result)
            }
            Err(e) => {
                error!("❌ Operação falhou no circuit breaker: {:?}", e);
                self.metrics_collector.record_circuit_breaker_failure();
                Err(anyhow::anyhow!("Circuit breaker failure: {:?}", e))
            }
        }
    }

    /// Adiciona tarefa ao processador de lotes
    pub async fn add_to_batch(&self, task: BatchTask) -> Result<()> {
        self.batch_processor.add_task(task).await
    }

    /// Processa lote se disponível
    pub async fn process_batch_if_ready(&self) -> Result<Option<Vec<BatchTask>>> {
        self.batch_processor.get_batch_if_ready().await
    }

    /// Registra métricas de processamento
    pub fn record_task_duration(&self, duration: Duration) {
        self.metrics_collector.task_duration.observe(duration.as_secs_f64());
    }

    /// Registra chamada de API
    pub fn record_api_call(&self) {
        self.metrics_collector.api_calls.inc();
    }
}

impl SimpleCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            failure_count: AtomicU32::new(0),
            last_failure_time: AtomicU64::new(0),
            state: AtomicU32::new(CircuitBreakerState::Closed as u32),
            success_count: AtomicU32::new(0),
        }
    }

    pub async fn call<F, T>(&self, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Verifica se pode executar
        if !self.can_execute() {
            return Err(anyhow::anyhow!("Circuit breaker is open"));
        }

        match operation.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(e)
            }
        }
    }

    fn can_execute(&self) -> bool {
        let state = self.state.load(Ordering::Relaxed);
        
        match state {
            0 => true, // Closed
            1 => {     // Open
                let now = chrono::Utc::now().timestamp() as u64;
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);
                
                if now - last_failure > self.config.timeout_duration {
                    // Transição para HalfOpen
                    self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            2 => {     // HalfOpen
                self.success_count.load(Ordering::Relaxed) < self.config.half_open_max_calls
            }
            _ => false,
        }
    }

    fn on_success(&self) {
        let state = self.state.load(Ordering::Relaxed);
        
        if state == CircuitBreakerState::HalfOpen as u32 {
            let success_count = self.success_count.fetch_add(1, Ordering::Relaxed);
            if success_count + 1 >= self.config.success_threshold {
                // Fecha o circuit
                self.state.store(CircuitBreakerState::Closed as u32, Ordering::Relaxed);
                self.failure_count.store(0, Ordering::Relaxed);
            }
        } else if state == CircuitBreakerState::Closed as u32 {
            // Reset failure count em caso de sucesso
            self.failure_count.store(0, Ordering::Relaxed);
        }
    }

    fn on_failure(&self) {
        let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.last_failure_time.store(
            chrono::Utc::now().timestamp() as u64,
            Ordering::Relaxed,
        );

        if failure_count + 1 >= self.config.failure_threshold {
            self.state.store(CircuitBreakerState::Open as u32, Ordering::Relaxed);
        }
    }

    pub fn get_state(&self) -> u32 {
        self.state.load(Ordering::Relaxed)
    }
}

impl BatchProcessor {
    pub fn new(config: BatchProcessingConfig) -> Result<Self> {
        info!(
            "📦 Inicializando batch processor: size={}, timeout={}s",
            config.batch_size, config.batch_timeout
        );

        Ok(Self {
            config,
            pending_tasks: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn add_task(&self, task: BatchTask) -> Result<()> {
        let mut tasks = self.pending_tasks.write().await;
        tasks.push(task);
        
        debug!("📦 Task adicionada ao lote. Total: {}", tasks.len());
        Ok(())
    }

    pub async fn get_batch_if_ready(&self) -> Result<Option<Vec<BatchTask>>> {
        let mut tasks = self.pending_tasks.write().await;
        
        if tasks.is_empty() {
            return Ok(None);
        }

        let should_process = tasks.len() >= self.config.batch_size
            || tasks
                .first()
                .map(|task| {
                    chrono::Utc::now()
                        .signed_duration_since(task.created_at)
                        .num_seconds()
                        >= self.config.batch_timeout as i64
                })
                .unwrap_or(false);

        if should_process {
            let batch = tasks.drain(..).collect();
            info!("📦 Processando lote com {} tasks", batch.len());
            Ok(Some(batch))
        } else {
            Ok(None)
        }
    }
}

impl MetricsCollector {
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        let registry = Registry::new();

        let task_duration = Histogram::new(
            "task_processing_duration_seconds",
            "Duração do processamento de tasks",
        )?;

        let api_calls = Counter::new(
            "clickup_api_calls_total",
            "Total de chamadas à API do ClickUp",
        )?;

        let batch_size = Histogram::new(
            "batch_processing_size",
            "Tamanho dos lotes processados",
        )?;

        let circuit_breaker_state = IntGauge::new(
            "circuit_breaker_state",
            "Estado do circuit breaker (0=closed, 1=open, 2=half-open)",
        )?;

        let active_tasks = IntGauge::new(
            "active_tasks_count",
            "Número de tasks ativas sendo processadas",
        )?;

        registry.register(Box::new(task_duration.clone()))?;
        registry.register(Box::new(api_calls.clone()))?;
        registry.register(Box::new(batch_size.clone()))?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;
        registry.register(Box::new(active_tasks.clone()))?;

        info!("📊 Metrics collector inicializado com {} métricas customizadas", 5);

        Ok(Self {
            registry,
            task_duration,
            api_calls,
            batch_size,
            circuit_breaker_state,
            active_tasks,
        })
    }

    pub fn increment_rate_limited(&self) {
        // Implementar counter específico para rate limiting se necessário
    }

    pub fn record_circuit_breaker_success(&self) {
        self.circuit_breaker_state.set(0); // closed
    }

    pub fn record_circuit_breaker_failure(&self) {
        self.circuit_breaker_state.set(1); // open
    }

    pub fn update_circuit_breaker_state(&self, state: u32) {
        self.circuit_breaker_state.set(state as i64);
    }

    pub fn get_registry(&self) -> &Registry {
        &self.registry
    }
}

// Implementações Default para configurações
impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            max_dispatches_per_second: 10.0,
            max_concurrent_dispatches: 100,
            batch_size: 5,
            burst_capacity: 20,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_duration: 30,
            half_open_max_calls: 3,
            success_threshold: 3,
        }
    }
}

impl Default for BatchProcessingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 5,
            batch_timeout: 30,
            max_batch_wait: 10,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_port: 9090,
            custom_metrics: vec![
                "task_processing_duration".to_string(),
                "clickup_api_calls".to_string(),
                "batch_processing_size".to_string(),
                "circuit_breaker_state".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gcp_optimizations_creation() {
        let optimizations = GCPOptimizations::new(None, None, None, None).unwrap();
        
        // Test rate limiting
        assert!(optimizations.check_rate_limit("test-key").await);
        
        // Test batch processing
        let task = BatchTask {
            task_id: "test-123".to_string(),
            payload: "{}".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        optimizations.add_to_batch(task).await.unwrap();
        
        // Deve não ter lote pronto ainda (size = 1 < batch_size = 5)
        let batch = optimizations.process_batch_if_ready().await.unwrap();
        assert!(batch.is_none());
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let config = BatchProcessingConfig {
            enabled: true,
            batch_size: 2,
            batch_timeout: 1,
            max_batch_wait: 1,
        };
        
        let processor = BatchProcessor::new(config).unwrap();
        
        // Adiciona tasks
        processor.add_task(BatchTask {
            task_id: "1".to_string(),
            payload: "{}".to_string(),
            created_at: chrono::Utc::now(),
        }).await.unwrap();
        
        processor.add_task(BatchTask {
            task_id: "2".to_string(),
            payload: "{}".to_string(),
            created_at: chrono::Utc::now(),
        }).await.unwrap();
        
        // Agora deve ter um lote pronto
        let batch = processor.get_batch_if_ready().await.unwrap();
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 2);
    }
}