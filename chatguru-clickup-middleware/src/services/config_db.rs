/// Serviço para ler configurações dinâmicas do banco de dados
///
/// Este módulo centraliza o acesso às configurações armazenadas na tabela `prompt_config`,
/// permitindo que a aplicação seja configurada dinamicamente sem necessidade de rebuild/redeploy.
///
/// Funcionalidades:
/// - Leitura de configurações com cache em memória (1 hora TTL)
/// - Fallback para valores padrão se banco indisponível
/// - Suporte a tipos: text, boolean, integer

use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::utils::logging::*;

#[derive(Clone)]
pub struct ConfigService {
    db: PgPool,
    cache: Arc<RwLock<ConfigCache>>,
}

#[derive(Debug)]
struct ConfigCache {
    values: HashMap<String, (String, Instant)>, // key → (value, timestamp)
    ttl: Duration,
}

impl ConfigCache {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            ttl: Duration::from_secs(3600), // 1 hora
        }
    }

    fn is_expired(&self, timestamp: Instant) -> bool {
        timestamp.elapsed() > self.ttl
    }
}

impl ConfigService {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(ConfigCache::new())),
        }
    }

    /// Obter valor string de configuração
    pub async fn get_string(&self, key: &str) -> Option<String> {
        self.get_value(key).await
    }

    /// Obter valor boolean de configuração
    pub async fn get_bool(&self, key: &str) -> bool {
        self.get_value(key)
            .await
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Obter valor integer de configuração
    pub async fn get_int(&self, key: &str) -> Option<i64> {
        self.get_value(key)
            .await
            .and_then(|v| v.parse::<i64>().ok())
    }

    /// Obter valor com fallback
    pub async fn get_string_or(&self, key: &str, default: &str) -> String {
        self.get_string(key).await.unwrap_or_else(|| default.to_string())
    }

    /// Obter múltiplas configurações de uma vez
    pub async fn get_multiple(&self, keys: &[&str]) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for &key in keys {
            if let Some(value) = self.get_value(key).await {
                result.insert(key.to_string(), value);
            }
        }

        result
    }

    /// Método interno para obter valor (com cache)
    async fn get_value(&self, key: &str) -> Option<String> {
        // Verificar cache
        {
            let cache = self.cache.read().await;
            if let Some((value, timestamp)) = cache.values.get(key) {
                if !cache.is_expired(*timestamp) {
                    log_info(&format!("💾 Config cache hit: {} = {}", key, value));
                    return Some(value.clone());
                }
            }
        }

        // Buscar no banco
        if self.db.is_closed() {
            log_warning(&format!("⚠️ Banco fechado, não foi possível ler config: {}", key));
            return None;
        }

        match sqlx::query_scalar::<_, String>(
            "SELECT value FROM prompt_config WHERE key = $1 AND is_active = true LIMIT 1"
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await
        {
            Ok(Some(value)) => {
                log_info(&format!("📊 Config do banco: {} = {}", key, value));

                // Atualizar cache
                let mut cache = self.cache.write().await;
                cache.values.insert(key.to_string(), (value.clone(), Instant::now()));

                Some(value)
            }
            Ok(None) => {
                log_warning(&format!("⚠️ Config não encontrada: {}", key));
                None
            }
            Err(e) => {
                log_error(&format!("❌ Erro ao ler config '{}': {}", key, e));
                None
            }
        }
    }

    /// Limpar cache (útil para testes ou forçar recarga)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.values.clear();
        log_info("🗑️ Cache de configurações limpo");
    }

    /// Recarregar configurações do banco (bypass cache)
    pub async fn reload(&self, key: &str) -> Option<String> {
        // Remover do cache primeiro
        {
            let mut cache = self.cache.write().await;
            cache.values.remove(key);
        }

        // Buscar do banco novamente
        self.get_value(key).await
    }
}

// Configurações específicas do sistema (helpers)
impl ConfigService {
    /// Verificar se sistema dinâmico está habilitado
    pub async fn is_dynamic_structure_enabled(&self) -> bool {
        self.get_bool("dynamic_structure_enabled").await
    }

    /// Obter space ID padrão para clientes inativos
    pub async fn get_default_inactive_space_id(&self) -> Option<String> {
        self.get_string("default_inactive_space_id").await
    }

    /// Obter folder ID fallback (para retrocompatibilidade)
    pub async fn get_fallback_folder_id(&self) -> Option<String> {
        self.get_string("fallback_folder_id").await
    }

    /// Obter folder path fallback
    pub async fn get_fallback_folder_path(&self) -> Option<String> {
        self.get_string("fallback_folder_path").await
    }

    /// Obter system role para OpenAI
    pub async fn get_system_role(&self) -> String {
        self.get_string_or("system_role", "Você é um assistente especializado em classificar solicitações.").await
    }

    /// Obter task description para OpenAI
    pub async fn get_task_description(&self) -> String {
        self.get_string_or("task_description", "Classifique se é uma atividade de trabalho válida.").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bool_parsing() {
        // Este teste só demonstra a lógica de parsing
        assert_eq!("true".to_lowercase() == "true", true);
        assert_eq!("false".to_lowercase() == "true", false);
        assert_eq!("1" == "1", true);
        assert_eq!("0" == "1", false);
    }
}
