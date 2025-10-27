# Próximos Passos - Crate ClickUp

Este documento detalha exatamente o que precisa ser feito para completar a migração do crate `clickup`.

## 📊 Status Atual (27 de Outubro de 2025)

### ✅ Concluído (1.729 linhas migradas)

| Módulo | Arquivo Original | Destino | Linhas | Status |
|--------|------------------|---------|--------|--------|
| ✅ Client | (novo) | `client.rs` | 293 | Completo + Híbrido v2/v3 |
| ✅ Error | (novo) | `error.rs` | 42 | Completo |
| ✅ Matching | (extraído) | `matching.rs` | 164 | Completo |
| ✅ Folders | `smart_folder_finder.rs` | `folders.rs` | 588 | Refatorado |
| ✅ Assignees | `smart_assignee_finder.rs` | `assignees.rs` | 340 | Refatorado |
| ✅ Fields | `custom_field_manager.rs` | `fields.rs` | 302 | Refatorado |

**Mudanças implementadas:**
- ✅ Cliente HTTP híbrido (v2 + v3)
- ✅ Nomenclatura v3 (`workspace_id` em vez de `team_id`)
- ✅ Refatoração para usar `ClickUpClient` abstrato
- ✅ Remoção de chamadas HTTP diretas (reqwest)
- ✅ Tipos de erro específicos (`ClickUpError`)
- ✅ Documentação completa (README.md, API_VERSIONS.md)
- ✅ Testes unitários incluídos

### 🔄 Pendente (937 linhas)

| Módulo | Arquivo Original | Destino | Linhas | Prioridade |
|--------|------------------|---------|--------|------------|
| 🔄 Tasks | `clickup.rs` | `tasks.rs` | 937 | **ALTA** |

## 🎯 Passo 1: Migrar tasks.rs (Estimativa: 40-60 min)

### 1.1. Análise do Arquivo Original

**Localização**: `src/services/clickup.rs` (937 linhas)

**Estrutura Atual**:
```rust
pub struct ClickUpService {
    client: Client,
    token: String,
    list_id: String,
    base_url: String,
}
```

**Funções Públicas (14 total)**:

| Função | Linhas | Complexidade | Depende de |
|--------|--------|--------------|------------|
| `new()` | ~50 | Baixa | Settings |
| `new_with_secrets()` | ~50 | Média | SecretManager |
| `create_task_from_json()` | ~80 | Alta | - |
| `update_task()` | ~30 | Baixa | - |
| `find_existing_task_in_list()` | ~100 | Alta | - |
| `add_comment_to_task()` | ~75 | Média | - |
| `test_connection()` | ~60 | Baixa | - |
| `get_list_info()` | ~60 | Baixa | - |
| `process_payload()` | ~10 | Baixa | process_payload_with_ai |
| `process_payload_with_ai()` | ~110 | **Muito Alta** | AI Service |
| `create_task_by_client()` | ~95 | Alta | SmartFolderFinder |
| `create_task_dynamic()` | ~100 | Alta | SmartFolderFinder |
| E mais 2 funções... | - | - | - |

### 1.2. Estratégia de Migração

#### Opção A: Migração Completa (RECOMENDADA)
Migrar todo `clickup.rs` → `crates/clickup/src/tasks.rs`

**Passos**:
1. Copiar `clickup.rs` → `crates/clickup/src/tasks.rs`
2. Refatorar struct:
   ```rust
   // ANTES
   pub struct ClickUpService {
       client: Client,
       token: String,
       list_id: String,
       base_url: String,
   }

   // DEPOIS
   pub struct TaskManager {
       client: ClickUpClient,  // Usa abstração
       list_id: String,
   }
   ```
3. Atualizar todos os métodos para usar `self.client.get_json()` em vez de HTTP direto
4. Substituir `AppError` → `ClickUpError`
5. Remover dependências de `Settings` (passar valores diretamente)
6. Exportar em `lib.rs`
7. Build e teste

**Desafios**:
- ⚠️ `process_payload_with_ai()` depende de `IaService` (fora do crate)
  - **Solução**: Mover para `src/handlers/worker.rs` ou aceitar `IaService` como parâmetro
- ⚠️ `new_with_secrets()` depende de `SecretManagerService` (GCP específico)
  - **Solução**: Manter essa função no projeto principal, não no crate

#### Opção B: Migração Parcial
Migrar apenas funções CRUD puras, deixar lógica de negócio no main

**Vantagens**: Mais rápido, menos refatoração
**Desvantagens**: Crate incompleto, duplicação de código

### 1.3. Checklist de Execução

```markdown
- [ ] Copiar `src/services/clickup.rs` → `crates/clickup/src/tasks.rs`
- [ ] Renomear `ClickUpService` → `TaskManager`
- [ ] Refatorar struct para usar `ClickUpClient`
- [ ] Atualizar imports:
  - [ ] `AppResult` → `Result`
  - [ ] `AppError` → `ClickUpError`
  - [ ] Remover `use crate::config::Settings`
  - [ ] Remover `use crate::services::secrets`
- [ ] Refatorar métodos HTTP (20+ chamadas):
  - [ ] `create_task_from_json()` - POST /list/{list_id}/task
  - [ ] `update_task()` - PUT /task/{task_id}
  - [ ] `find_existing_task_in_list()` - GET /list/{list_id}/task
  - [ ] `add_comment_to_task()` - POST /task/{task_id}/comment
  - [ ] `test_connection()` - GET /list/{list_id}
  - [ ] `get_list_info()` - GET /list/{list_id}
- [ ] Mover lógica de negócio (process_payload_with_ai) para main
- [ ] Adicionar em `lib.rs`: `pub mod tasks;`
- [ ] Build: `cargo build -p clickup`
- [ ] Testes: `cargo test -p clickup`
```

### 1.4. Exemplo de Refatoração

**ANTES** (clickup.rs):
```rust
pub async fn create_task_from_json(&self, task_data: &Value) -> AppResult<Value> {
    let url = format!("{}/list/{}/task", self.base_url, self.list_id);

    let response = self.client
        .post(&url)
        .header("Authorization", &self.token)
        .header("Content-Type", "application/json")
        .json(task_data)
        .send()
        .await
        .map_err(|e| AppError::InternalError(format!("HTTP error: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::ClickUpApi(format!("POST failed: {} - {}", status, body)));
    }

    let task: Value = response.json().await
        .map_err(|e| AppError::InternalError(format!("JSON parse: {}", e)))?;

    Ok(task)
}
```

**DEPOIS** (tasks.rs):
```rust
pub async fn create_task(&self, list_id: &str, task_data: &Value) -> Result<Value> {
    let endpoint = format!("/list/{}/task", list_id);
    let task: Value = self.client.post_json(&endpoint, task_data).await?;
    Ok(task)
}
```

**Redução**: 25 linhas → 3 linhas! 🎉

## 🎯 Passo 2: Atualizar Projeto Principal (15-20 min)

### 2.1. Atualizar `src/handlers/worker.rs`

**Mudanças necessárias**:

```rust
// ANTES
use chatguru_clickup_middleware::services::smart_folder_finder::SmartFolderFinder;
use chatguru_clickup_middleware::services::smart_assignee_finder::SmartAssigneeFinder;
use chatguru_clickup_middleware::services::custom_field_manager::CustomFieldManager;

let mut finder = SmartFolderFinder::new(folder_api_token, folder_workspace_id);
let mut assignee_finder = SmartAssigneeFinder::new(assignee_api_token, assignee_workspace_id);
let field_manager = CustomFieldManager::new(api_token);

// DEPOIS
use clickup::folders::SmartFolderFinder;
use clickup::assignees::SmartAssigneeFinder;
use clickup::fields::CustomFieldManager;
use clickup::ClickUpClient;

let client = ClickUpClient::new(api_token)?;
let mut finder = SmartFolderFinder::new(client.clone(), workspace_id);
let mut assignee_finder = SmartAssigneeFinder::new(client.clone(), workspace_id);
let field_manager = CustomFieldManager::new(client.clone());
```

### 2.2. Checklist

```markdown
- [ ] Atualizar imports em `worker.rs`
- [ ] Criar `ClickUpClient` único
- [ ] Passar `client.clone()` para finders/managers
- [ ] Atualizar tipos de retorno (`AppResult` → `Result`)
- [ ] Build: `cargo build`
- [ ] Testar compilação
```

## 🎯 Passo 3: Limpeza (10 min)

### 3.1. Remover Arquivos Antigos

```markdown
- [ ] Deletar `src/services/smart_folder_finder.rs`
- [ ] Deletar `src/services/smart_assignee_finder.rs`
- [ ] Deletar `src/services/custom_field_manager.rs`
- [ ] (Opcional) Deletar `src/services/clickup.rs` após migrar tasks
- [ ] Atualizar `src/services/mod.rs` para remover re-exports
- [ ] Verificar que nada mais importa os arquivos antigos
```

### 3.2. Verificação

```bash
# Buscar referências aos arquivos antigos
cd /Users/williamduarte/NCMproduto/integrações/ChatGuru_Nordja_inte/chatguru-clickup-middleware
grep -r "smart_folder_finder" src/
grep -r "smart_assignee_finder" src/
grep -r "custom_field_manager" src/

# Se não retornar nada, pode deletar com segurança
```

## 🎯 Passo 4: Testes End-to-End (15 min)

### 4.1. Checklist de Testes

```markdown
- [ ] Compilação: `cargo build`
- [ ] Testes unitários crate: `cargo test -p clickup`
- [ ] Testes projeto: `cargo test`
- [ ] Deploy local: `cargo run`
- [ ] Testar endpoint webhook manualmente
- [ ] Verificar logs
- [ ] Confirmar que folders/assignees/fields funcionam
```

### 4.2. Testes Manuais

**Endpoint de teste**:
```bash
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d @test_payload.json
```

**Verificar logs**:
```bash
# Buscar por:
# - "SmartFolderFinder: Buscando folder"
# - "SmartAssigneeFinder: Buscando assignee"
# - "CustomFieldManager: Garantindo opção"
```

## 🎯 Passo 5: Commit & Documentação (10 min)

### 5.1. Commit Message

```
feat(clickup): migra modules para crate independente

Refatoração completa do código ClickUp em crate reutilizável:

MÓDULOS MIGRADOS:
- ✅ folders.rs (SmartFolderFinder) - 588 linhas
- ✅ assignees.rs (SmartAssigneeFinder) - 340 linhas
- ✅ fields.rs (CustomFieldManager) - 302 linhas
- ✅ client.rs (ClickUpClient híbrido v2+v3) - 293 linhas
- ✅ error.rs (ClickUpError) - 42 linhas
- ✅ matching.rs (fuzzy matching utils) - 164 linhas

MELHORIAS:
- Cliente HTTP abstraído (remove reqwest direto)
- Suporte híbrido API v2+v3
- Nomenclatura v3 (workspace_id)
- Tipos de erro específicos
- Testes unitários incluídos
- Documentação completa (README + API_VERSIONS)

TOTAL: 1.729 linhas refatoradas
PENDENTE: tasks.rs (937 linhas)

Refs: #migration #refactor #clickup-api
```

### 5.2. Arquivos a Commitar

```markdown
- [ ] crates/clickup/Cargo.toml
- [ ] crates/clickup/src/lib.rs
- [ ] crates/clickup/src/client.rs
- [ ] crates/clickup/src/error.rs
- [ ] crates/clickup/src/matching.rs
- [ ] crates/clickup/src/folders.rs
- [ ] crates/clickup/src/assignees.rs
- [ ] crates/clickup/src/fields.rs
- [ ] crates/clickup/README.md
- [ ] crates/clickup/API_VERSIONS.md
- [ ] crates/clickup/NEXT_STEPS.md
- [ ] Cargo.toml (workspace)
- [ ] src/handlers/worker.rs (se já atualizado)
```

## 📋 Resumo de Tempo Estimado

| Fase | Tarefa | Tempo Estimado |
|------|--------|----------------|
| 1 | Migrar tasks.rs | 40-60 min |
| 2 | Atualizar worker.rs | 15-20 min |
| 3 | Limpeza | 10 min |
| 4 | Testes | 15 min |
| 5 | Commit | 10 min |
| **TOTAL** | **Completar migração** | **90-115 min** |

## 🚀 Como Retomar

1. Abrir este arquivo: `crates/clickup/NEXT_STEPS.md`
2. Seguir **Passo 1.3. Checklist de Execução**
3. Marcar itens conforme completa
4. Prosseguir para próximos passos

## 📞 Comandos Úteis

```bash
# Build apenas o crate
cargo build -p clickup

# Testes do crate
cargo test -p clickup

# Build projeto completo
cargo build

# Verificar compilação rápida
cargo check

# Documentação
cargo doc --open -p clickup

# Buscar TODOs
grep -r "TODO\|FIXME" crates/clickup/src/
```

## 🎯 Meta Final

Criar crate `clickup` 100% independente e reutilizável com:

- [x] Client abstrato (v2+v3)
- [x] Folders module
- [x] Assignees module
- [x] Fields module
- [ ] **Tasks module** ← PRÓXIMO
- [ ] Types module (opcional)
- [ ] Lists module (opcional)

**Quando completo**: Publicar internamente para reuso em outros projetos!

---

**Última atualização**: 27 de Outubro de 2025
**Próxima sessão**: Migrar tasks.rs (Passo 1)
