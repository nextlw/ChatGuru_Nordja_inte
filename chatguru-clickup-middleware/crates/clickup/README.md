# ClickUp API Client

Cliente completo e tipo-seguro para a API do ClickUp, com funcionalidades avançadas de busca inteligente (fuzzy matching) e suporte híbrido para API v2 e v3.

## 🎯 Features

- ✅ **Smart Folder Finder**: Busca inteligente de folders com fuzzy matching (Jaro-Winkler)
- ✅ **Smart Assignee Finder**: Busca de usuários por nome com cache
- ✅ **Custom Field Manager**: Gerenciamento automático de campos personalizados
- ✅ **API Híbrida v2+v3**: Suporte para ambas as versões da API
- ✅ **Error Handling**: Tipos de erro específicos com `thiserror`
- ✅ **Async/Await**: Totalmente assíncrono com Tokio
- ✅ **Type-Safe**: Structs tipadas para todas as entidades
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

### 1. Criar Cliente

```rust
use clickup::ClickUpClient;

#[tokio::main]
async fn main() -> clickup::Result<()> {
    let client = ClickUpClient::new("pk_your_api_token")?;
    Ok(())
}
```

### 2. Buscar Folder Inteligente

```rust
use clickup::folders::SmartFolderFinder;

let mut finder = SmartFolderFinder::from_token(
    "pk_your_token".to_string(),
    "9013037641".to_string() // workspace_id
)?;

let result = finder.find_folder_for_client("Nexcode").await?;

if let Some(folder) = result {
    println!("Folder: {} (id: {})", folder.folder_name, folder.folder_id);
    println!("Confidence: {:.2}", folder.confidence);
}
```

### 3. Buscar Assignee (Responsável)

```rust
use clickup::assignees::SmartAssigneeFinder;

let mut finder = SmartAssigneeFinder::from_token(
    "pk_your_token".to_string(),
    "9013037641".to_string()
)?;

let result = finder.find_assignee_by_name("William").await?;

if let Some(assignee) = result {
    println!("User: {} (id: {})", assignee.username, assignee.user_id);
}
```

### 4. Gerenciar Custom Fields

```rust
use clickup::fields::CustomFieldManager;

let manager = CustomFieldManager::from_token("pk_your_token".to_string())?;

// Garante que a opção existe no dropdown e retorna o value
let custom_field = manager
    .ensure_client_solicitante_option("list_123", "Nexcode")
    .await?;

println!("{}", custom_field); // {"id": "...", "value": "Nexcode"}
```

## 🏗️ Arquitetura

### Módulos

```
crates/clickup/src/
├── client.rs         # Cliente HTTP híbrido v2+v3
├── error.rs          # Tipos de erro customizados
├── matching.rs       # Fuzzy matching utilities
├── folders.rs        # SmartFolderFinder
├── assignees.rs      # SmartAssigneeFinder
├── fields.rs         # CustomFieldManager
└── lib.rs            # Re-exports e documentação
```

### API Híbrida v2 + v3

Este crate implementa uma **estratégia híbrida**:

- **API v2**: Usado para spaces, folders, lists, tasks, custom fields (endpoints estáveis)
- **API v3**: Preparado para workspaces, groups, docs (quando disponível)
- **Nomenclatura v3**: Usa `workspace_id` internamente para clareza

Veja [API_VERSIONS.md](./API_VERSIONS.md) para detalhes completos.

## 🧪 Testes

```bash
# Rodar testes do crate
cargo test -p clickup

# Com output detalhado
cargo test -p clickup -- --nocapture

# Teste específico
cargo test -p clickup test_normalize_name
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
| 🔄 tasks | Pendente | - | Task CRUD (próximo) |
| 🔄 types | Pendente | - | Tipos da API |
| 🔄 lists | Pendente | - | List operations |

**Total migrado**: 1.729 linhas
**Pendente**: ~937 linhas (tasks.rs)

## 🔧 Variáveis de Ambiente

```bash
# Recomendado (v3-style)
export CLICKUP_WORKSPACE_ID="9013037641"
export CLICKUP_API_TOKEN="pk_xxxxx"

# Compatibilidade (v2-style, funciona como fallback)
export CLICKUP_TEAM_ID="9013037641"
```

## 📚 Documentação

- **[API_VERSIONS.md](./API_VERSIONS.md)**: Estratégia híbrida v2+v3
- **Inline docs**: Use `cargo doc --open -p clickup`
- **Exemplos**: Ver testes em cada módulo

## 🎯 Próximos Passos

Ver [NEXT_STEPS.md](./NEXT_STEPS.md) para roadmap detalhado.

### Migração Pendente (tasks.rs)

**Arquivo**: `src/services/clickup.rs` (937 linhas)
**Destino**: `crates/clickup/src/tasks.rs`

**Funções a migrar**:
- `create_task_from_json()` - Criar task
- `update_task()` - Atualizar task
- `find_existing_task_in_list()` - Buscar duplicatas
- `add_comment_to_task()` - Adicionar comentários
- `test_connection()` - Testar conexão
- `get_list_info()` - Info da lista
- E mais 8 funções...

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
