# ClickUp API Client

Cliente completo e tipo-seguro para a API do ClickUp, com funcionalidades avançadas de busca inteligente (fuzzy matching), suporte híbrido para API v2 e v3, e tipos estruturados.

## 🎯 Features

- ✅ **Type-Safe API**: Structs tipadas para Task, Priority, Status, CustomField, User
- ✅ **Smart Folder Finder**: Busca inteligente de folders com fuzzy matching (Jaro-Winkler)
- ✅ **Smart Assignee Finder**: Busca de usuários por nome com cache
- ✅ **Custom Field Manager**: Gerenciamento automático de campos personalizados
- ✅ **Task Manager**: CRUD completo com assignees, status, subtasks, due dates, dependencies
- ✅ **Webhook Manager**: Gerenciamento completo de webhooks (create, list, update, delete)
- ✅ **Webhook Signature Verification**: Validação HMAC-SHA256 para segurança
- ✅ **API Híbrida v2+v3**: Suporte para ambas as versões da API
- ✅ **Error Handling**: Tipos de erro específicos com `thiserror`
- ✅ **Async/Await**: Totalmente assíncrono com Tokio
- ✅ **Cache**: Sistema de cache in-memory para otimização
- ✅ **Logging**: Integração com `tracing`
- ✅ **Testes**: Testes unitários incluídos

## 📦 Instalação

Adicione ao `Cargo.toml`:

```toml
[dependencies]
clickup = { path = "crates/clickup" }
```

## 🚀 Uso Rápido

### ⚠️ IMPORTANTE: Configuração Segura

**NUNCA hardcode tokens ou IDs no código!** Use variáveis de ambiente:

```bash
# Configure as variáveis de ambiente
export CLICKUP_API_TOKEN="pk_your_token_here"
export CLICKUP_WORKSPACE_ID="your_workspace_id"
```

### 1. Criar Cliente

```rust
use clickup::ClickUpClient;

#[tokio::main]
async fn main() -> clickup::Result<()> {
    // Ler token de variável de ambiente (OBRIGATÓRIO)
    let api_token = std::env::var("CLICKUP_API_TOKEN")
        .expect("CLICKUP_API_TOKEN não configurado");

    let client = ClickUpClient::new(api_token)?;
    Ok(())
}
```

### 2. Criar Task com Tipos

```rust
use clickup::{Task, Priority};
use clickup::tasks::TaskManager;

let client = ClickUpClient::new(api_token)?;
let task_manager = TaskManager::new(client, Some("list_123".to_string()));

// Criar task usando builder pattern
let task = Task::new("Nova tarefa")
    .with_description("Descrição detalhada")
    .with_priority(Priority::High)
    .with_list_id("list_123");

// Criar no ClickUp
let created_task = task_manager.create_task(&task).await?;
println!("Task criada: {}", created_task.id.unwrap());
```

### 3. Assignees, Status, Subtasks

```rust
// Atribuir usuário
let task = task_manager.assign_task("task_id", vec![12345]).await?;

// Atualizar status
let task = task_manager.update_task_status("task_id", "em progresso").await?;

// Criar subtask
let subtask = Task::new("Subtarefa")
    .with_parent("parent_task_id");
let created_subtask = task_manager.create_subtask("parent_task_id", &subtask).await?;

// Definir due date
let due_date_ms = 1730073600000i64; // Unix timestamp em milissegundos
let task = task_manager.set_due_date("task_id", due_date_ms).await?;

// Adicionar dependência
task_manager.add_dependency("task_id", "depends_on_task_id", "waiting_on").await?;
```

### 4. Buscar Folder Inteligente

```rust
use clickup::folders::SmartFolderFinder;

let api_token = std::env::var("CLICKUP_API_TOKEN")
    .expect("CLICKUP_API_TOKEN não configurado");
let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
    .expect("CLICKUP_WORKSPACE_ID não configurado");

let mut finder = SmartFolderFinder::from_token(api_token, workspace_id)?;
let result = finder.find_folder_for_client("Nexcode").await?;

if let Some(folder) = result {
    println!("Folder: {} (id: {})", folder.folder_name, folder.folder_id);
    println!("Confidence: {:.2}", folder.confidence);
}
```

### 5. Buscar Assignee (Responsável)

```rust
use clickup::assignees::SmartAssigneeFinder;

let mut finder = SmartAssigneeFinder::from_token(api_token, workspace_id)?;
let result = finder.find_assignee_by_name("William").await?;

if let Some(assignee) = result {
    println!("User: {} (id: {})", assignee.username, assignee.user_id);
}
```

### 6. Custom Fields

```rust
use clickup::fields::CustomFieldManager;
use clickup::{CustomField, CustomFieldValue};

let manager = CustomFieldManager::from_token(api_token)?;

// Garantir que opção existe no dropdown
let custom_field = manager
    .ensure_client_solicitante_option("list_123", "Nexcode")
    .await?;

// Criar custom fields tipados
let checkbox = CustomField::checkbox("field_id", true);
let date = CustomField::date("field_id", 1730073600000i64); // milissegundos
let text = CustomField::text("field_id", "Valor texto");
```

### 7. Webhooks (Tempo Real)

```rust
use clickup::webhooks::{WebhookManager, WebhookConfig, WebhookEvent};

let manager = WebhookManager::from_token(api_token, workspace_id)?;

// Criar webhook para receber eventos
let config = WebhookConfig {
    endpoint: "https://myapp.com/webhooks/clickup".to_string(),
    events: vec![
        WebhookEvent::TaskCreated,
        WebhookEvent::TaskUpdated,
        WebhookEvent::TaskStatusUpdated,
    ],
    status: Some("active".to_string()),
};

let webhook = manager.create_webhook(&config).await?;
println!("Webhook criado: {}", webhook.id);

// Listar webhooks
let webhooks = manager.list_webhooks().await?;

// Criar ou atualizar (idempotente)
let webhook = manager.ensure_webhook(&config).await?;

// Validar assinatura (segurança)
use clickup::webhooks::WebhookPayload;

let is_valid = WebhookPayload::verify_signature(
    &signature_header,
    &webhook_secret,
    &request_body_bytes
);
```

#### Arquitetura Recomendada: Webhooks + Pub/Sub

Combine webhooks ClickUp com Google Cloud Pub/Sub para escalabilidade:

1. **Webhook recebe evento** do ClickUp (tempo real)
2. **Valida assinatura** (segurança)
3. **Publica no Pub/Sub** (desacoplamento)
4. **Subscribers processam** (escalabilidade)
5. **Retry automático** (confiabilidade)

```
ClickUp → Webhook Handler → Pub/Sub Topic → Workers
                ↓ valida assinatura
                ↓ ACK < 100ms
                ✓ publicado
```

**Eventos Disponíveis** (30+ eventos):
- Task: `Created`, `Updated`, `Deleted`, `Moved`, `StatusUpdated`, `PriorityUpdated`
- List: `Created`, `Updated`, `Deleted`
- Folder: `Created`, `Updated`, `Deleted`
- Space: `Created`, `Updated`, `Deleted`
- Goal: `Created`, `Updated`, `Deleted`

Ver `WebhookEvent` enum para lista completa.

## 🏗️ Arquitetura

### Estrutura de Módulos

```
crates/clickup/src/
├── client.rs         # Cliente HTTP híbrido v2+v3
├── error.rs          # Tipos de erro (ClickUpError)
├── matching.rs       # Fuzzy matching utilities (Jaro-Winkler)
├── folders.rs        # SmartFolderFinder (588 linhas)
├── assignees.rs      # SmartAssigneeFinder (340 linhas)
├── fields.rs         # CustomFieldManager (302 linhas)
├── tasks.rs          # TaskManager - CRUD completo (800+ linhas)
├── webhooks.rs       # WebhookManager - create, list, update, delete (400+ linhas)
├── types/            # Tipos estruturados (1,400 linhas)
│   ├── mod.rs        # Re-exports
│   ├── priority.rs   # Priority enum (1-4)
│   ├── status.rs     # Status struct
│   ├── user.rs       # User struct
│   ├── custom_field.rs # 18 tipos de custom fields
│   └── task.rs       # Task struct + builder
└── lib.rs            # Re-exports públicos
```

### API Híbrida v2 + v3

Este crate implementa uma **estratégia híbrida**:

- **API v2**: Usado para spaces, folders, lists, tasks, custom fields (endpoints estáveis)
- **API v3**: Preparado para workspaces, groups, docs (quando disponível)
- **Nomenclatura v3**: Usa `workspace_id` internamente para clareza

#### Cliente HTTP

```rust
pub struct ClickUpClient {
    http_client: HttpClient,
    api_token: String,
    base_url_v2: String,  // "https://api.clickup.com/api/v2"
    base_url_v3: String,  // "https://api.clickup.com/api/v3"
}
```

**Métodos disponíveis**:
- `get_json(endpoint)` - Padrão usa v2
- `post_json(endpoint, body)` - Padrão usa v2
- `put_json(endpoint, body)` - Padrão usa v2
- `delete_json(endpoint)` - Padrão usa v2
- `get_json_v3(endpoint)` - Força v3 (para migração futura)
- `post_json_v3(endpoint, body)` - Força v3

#### Mapeamento de Endpoints

| Recurso | API v2 (atual) | API v3 (futuro) |
|---------|----------------|-----------------|
| Spaces | `/team/{team_id}/space` | `/workspaces/{workspace_id}/spaces` |
| Folders | `/space/{space_id}/folder` | ❌ Não migrado |
| Lists | `/folder/{folder_id}/list` | ❌ Não migrado |
| Tasks | `/list/{list_id}/task` | ❌ Não migrado |
| Workspaces | ❌ Não existe | `/workspaces` ✅ |
| Groups | `/team/{team_id}/group` | `/workspaces/{workspace_id}/groups` ✅ |

#### Nomenclatura

**Interno (código)**:
```rust
let workspace_id = "9013037641"; // ✅ Nomenclatura v3
```

**API calls (atual)**:
```rust
// Internamente: workspace_id = "9013037641"
// Na API v2: /team/9013037641/space
let endpoint = format!("/team/{}/space", workspace_id);
```

## 📊 Tipos Estruturados

### Priority

```rust
pub enum Priority {
    Urgent = 1,  // ⚠️ Urgente
    High = 2,    // 🔴 Alta
    Normal = 3,  // 🟡 Normal (default)
    Low = 4,     // 🟢 Baixa
}
```

### Custom Fields (18 tipos)

```rust
pub enum CustomFieldValue {
    Text(String),
    Number(f64),
    Checkbox(String),      // ⚠️ CRÍTICO: "true"/"false", NÃO bool!
    Dropdown(String),
    Labels(Vec<String>),
    Date(i64),             // ⚠️ CRÍTICO: milissegundos, NÃO segundos!
    Users(Vec<u32>),
    Phone(String),
    Email(String),
    Url(String),
    Currency(f64),
    Rating(u8),
    Location(String),
    Attachment(String),
    // ... mais 4 tipos
}
```

**Atenção:**
- **Checkbox**: Usa strings `"true"`/`"false"`, não boolean
- **Date/Timestamp**: Usa i64 em **milissegundos**, não segundos

### Task Builder

```rust
let task = Task::new("Título da tarefa")
    .with_description("Descrição")
    .with_list_id("list_123")
    .with_priority(Priority::High)
    .with_assignees(vec![User { id: 12345, username: "william".to_string() }])
    .with_due_date(1730073600000i64)
    .with_parent("parent_task_id")  // Para subtasks
    .with_custom_fields(vec![
        CustomField::checkbox("field_id", true),
        CustomField::text("field_id2", "Valor"),
    ]);
```

## 🧪 Testes

```bash
# Testes do crate
cargo test -p clickup

# Com output detalhado
cargo test -p clickup -- --nocapture

# Teste específico
cargo test -p clickup test_normalize_name

# Testes de integração
cargo test --test test_assignee_finder
```

## 📊 Status de Implementação

| Módulo | Status | Linhas | Descrição |
|--------|--------|--------|-----------|
| ✅ client | Completo | 293 | Cliente HTTP v2+v3 |
| ✅ error | Completo | 42 | Tipos de erro |
| ✅ matching | Completo | 164 | Fuzzy matching |
| ✅ folders | Completo | 588 | Smart folder finder |
| ✅ assignees | Completo | 340 | Smart assignee finder |
| ✅ fields | Completo | 302 | Custom field manager |
| ✅ tasks | Completo | 800+ | Task CRUD + features |
| ✅ webhooks | Completo | 400+ | Webhook management (create, list, update, delete) |
| ✅ types | Completo | 1,400 | Task, Priority, Status, CustomField |

**Total**: ~4,300 linhas de código Rust

## ✅ Migração Completa (Fases 1-5)

### Fase 1: Types e Features
- ✅ Types module (1,400 linhas)
- ✅ OAuth2 fix (Bearer prefix)
- ✅ Assignees, status, subtasks, due dates, dependencies

### Fase 2: Payload Migration
- ✅ payload.rs migrado para usar Task
- ✅ Handlers usam API tipada

### Fase 3: Imports e Cleanup
- ✅ Services deletados (duplicatas)
- ✅ Imports atualizados para crates

### Fase 4: Service Migration
- ✅ SmartFolderFinder migrado
- ✅ SmartAssigneeFinder migrado
- ✅ CustomFieldManager migrado

### Fase 5: Validação
- ✅ 14 testes passando
- ✅ Build release OK (0 warnings)
- ✅ APIs idênticas validadas

## 🔧 Variáveis de Ambiente

### Configuração Obrigatória

```bash
# Token de autenticação ClickUp (OBRIGATÓRIO)
export CLICKUP_API_TOKEN="seu_token_aqui"

# ID do Workspace (OBRIGATÓRIO)
export CLICKUP_WORKSPACE_ID="seu_workspace_id_aqui"
```

### Compatibilidade v2

```bash
# Fallback para código legado
export CLICKUP_TEAM_ID="seu_workspace_id"  # Mesmo valor, nome antigo
```

### Como Obter os Valores

1. **CLICKUP_API_TOKEN**:
   - Vá para: ClickUp → Settings → Apps → API Token
   - Gere um Personal Token
   - **NUNCA** commite este valor no git!

2. **CLICKUP_WORKSPACE_ID**:
   - Na URL do ClickUp: `https://app.clickup.com/<WORKSPACE_ID>/...`
   - Ou via API: `GET https://api.clickup.com/api/v2/team`

### Produção com Google Secret Manager

```bash
# Armazenar secrets no GCP
gcloud secrets create clickup-api-token --data-file=- <<< "seu_token_aqui"
gcloud secrets create clickup-workspace-id --data-file=- <<< "seu_workspace_id"

# Usar no Cloud Run/Functions
gcloud run deploy ... --set-secrets=CLICKUP_API_TOKEN=clickup-api-token:latest
```

## 🤝 Contribuindo

Este é um crate interno para o projeto ChatGuru-ClickUp middleware.

### Convenções

1. **Nomenclatura**: Use `workspace_id` (não `team_id`)
2. **API Version**: Endpoints v2 por padrão, v3 onde disponível
3. **Testes**: Adicionar testes para novas funcionalidades
4. **Docs**: Documentar funções públicas
5. **Errors**: Usar `Result<T>` com tipos específicos

## 📝 Licença

Propriedade da eLai Integration Team / Nordja.

## 🔗 Links Úteis

- [ClickUp API v2 Docs](https://clickup.com/api)
- [API v2 vs v3 Guide](https://developer.clickup.com/docs/general-v2-v3-api)
- [Projeto Principal](../../README.md)
