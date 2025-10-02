# SmartClassifier - Armazenamento e Arquitetura

## 📦 Onde o modelo fica armazenado?

### **Opção 1: Configuração em memória (Atual - Implementado)**

```
config/
├── activity_terms.yaml      # Termos que indicam atividade + pesos
├── non_activity_terms.yaml  # Termos que indicam não-atividade + pesos
```

**Como funciona:**
- Os arquivos YAML são carregados na **primeira inicialização**
- Ficam em memória usando `Lazy<HashMap>` (once_cell)
- Sem banco de dados, sem overhead
- **Aprendizado dinâmico** é armazenado em memória

**Vantagens:**
- ✅ Zero latência
- ✅ Zero custo de infraestrutura
- ✅ Fácil de editar manualmente
- ✅ Funciona offline

**Desvantagens:**
- ❌ Aprendizado é perdido ao reiniciar
- ❌ Não compartilha aprendizado entre instâncias

---

### **Opção 2: Persistência com arquivo JSON (Recomendado para produção)**

```rust
// Salvar aprendizado periodicamente
impl SmartClassifier {
    pub async fn save_learned_patterns(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.learned_patterns)?;
        fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load_learned_patterns(&mut self, path: &str) -> Result<()> {
        let json = fs::read_to_string(path).await?;
        self.learned_patterns = serde_json::from_str(&json)?;
        Ok(())
    }
}
```

**Estrutura:**
```
config/
├── activity_terms.yaml         # Base estática
├── non_activity_terms.yaml     # Base estática
└── learned_patterns.json       # Aprendizado dinâmico (auto-gerado)
```

**Auto-save a cada X classificações:**
```rust
// Em context_cache.rs
if self.classification_count % 100 == 0 {
    classifier.save_learned_patterns("config/learned_patterns.json").await?;
}
```

---

### **Opção 3: Banco de dados SQLite (Máxima persistência)**

```toml
[dependencies]
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
```

**Schema:**
```sql
CREATE TABLE learned_patterns (
    pattern TEXT PRIMARY KEY,
    is_activity BOOLEAN,
    confidence REAL,
    times_seen INTEGER,
    last_updated TIMESTAMP
);

CREATE TABLE term_weights (
    term TEXT PRIMARY KEY,
    weight REAL,
    category TEXT  -- 'activity' ou 'non_activity'
);
```

**Vantagens:**
- ✅ Persistência completa
- ✅ Consultas rápidas
- ✅ Histórico de evolução
- ✅ Compartilhamento entre instâncias

---

### **Opção 4: Redis (Distribuído)**

Para múltiplas instâncias do middleware:

```toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "json"] }
```

**Como funciona:**
```rust
// Salvar padrão aprendido
redis.hset("learned_patterns", pattern_hash, json_data).await?;

// Carregar ao inicializar
let patterns: HashMap<String, Pattern> = redis.hgetall("learned_patterns").await?;
```

---

## 🎯 Arquitetura Recomendada (Híbrida)

```
┌─────────────────────────────────────────┐
│ Inicialização                           │
├─────────────────────────────────────────┤
│ 1. Carregar activity_terms.yaml        │
│ 2. Carregar non_activity_terms.yaml    │
│ 3. Carregar learned_patterns.json      │
│    (se existir)                         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ Runtime (em memória)                    │
├─────────────────────────────────────────┤
│ - HashMap de termos + pesos            │
│ - Padrões aprendidos                    │
│ - Stemmer (Portuguese)                  │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│ Auto-save (a cada 100 classificações)   │
├─────────────────────────────────────────┤
│ learned_patterns.json ← Disco           │
└─────────────────────────────────────────┘
```

---

## 📊 Comparação de Opções

| Aspecto | Memória (atual) | JSON + Auto-save | SQLite | Redis |
|---------|----------------|------------------|--------|-------|
| **Velocidade** | ⚡⚡⚡ | ⚡⚡⚡ | ⚡⚡ | ⚡⚡ |
| **Persistência** | ❌ | ✅ | ✅✅ | ✅✅ |
| **Distribuído** | ❌ | ❌ | ❌ | ✅ |
| **Complexidade** | Simples | Simples | Média | Alta |
| **Custo** | Zero | Zero | Zero | $$ |
| **Infraestrutura** | Nenhuma | Nenhuma | Arquivo | Servidor |

---

## 💡 Implementação Sugerida

### **Fase 1 (Atual):** Memória + YAML
- Funcional, zero overhead
- Perfeito para testes e desenvolvimento

### **Fase 2 (Próximo passo):** + Auto-save JSON
```bash
config/
├── activity_terms.yaml
├── non_activity_terms.yaml
└── learned_patterns.json  # ← Novo
```

### **Fase 3 (Se necessário):** SQLite
- Quando precisar de queries complexas
- Quando precisar de histórico

### **Fase 4 (Escala):** Redis
- Quando tiver múltiplas instâncias
- Quando precisar de sincronização em tempo real

---

## 🔧 Código de Auto-save (Fase 2)

Adicionar ao `smart_classifier.rs`:

```rust
use std::fs;
use std::path::Path;

impl SmartClassifier {
    /// Salva padrões aprendidos em JSON
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.learned_patterns)?;
        fs::write(path, json)?;
        tracing::info!("Saved {} learned patterns to {}", self.learned_patterns.len(), path);
        Ok(())
    }

    /// Carrega padrões aprendidos de JSON
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !Path::new(path).exists() {
            tracing::info!("No learned patterns file found at {}", path);
            return Ok(());
        }

        let json = fs::read_to_string(path)?;
        self.learned_patterns = serde_json::from_str(&json)?;
        tracing::info!("Loaded {} learned patterns from {}", self.learned_patterns.len(), path);
        Ok(())
    }
}
```

Adicionar ao `context_cache.rs`:

```rust
impl ContextCache {
    /// Salva aprendizado periodicamente
    pub async fn auto_save_if_needed(&self) {
        let stats = self.stats.read().await;

        // Auto-save a cada 100 classificações
        if stats.total_requests % 100 == 0 && stats.total_requests > 0 {
            let classifier = self.smart_classifier.read().await;
            if let Err(e) = classifier.save_to_file("config/learned_patterns.json") {
                tracing::error!("Failed to auto-save learned patterns: {}", e);
            } else {
                tracing::info!("✅ Auto-saved learned patterns (after {} requests)", stats.total_requests);
            }
        }
    }
}
```

---

## 📝 Resumo

**Atualmente:** Modelo está em **memória** (YAML carregado na inicialização)

**Recomendação:** Adicionar **auto-save JSON** para persistir aprendizado

**Futuro (se necessário):** Migrar para SQLite ou Redis dependendo da escala
