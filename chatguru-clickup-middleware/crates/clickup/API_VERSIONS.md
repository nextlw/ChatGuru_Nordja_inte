# ClickUp API - Estratégia Híbrida v2 + v3

Este documento explica a estratégia de uso das versões v2 e v3 da API do ClickUp implementada neste crate.

## Contexto

A ClickUp está migrando gradualmente da API v2 para v3, mas **nem todos os endpoints foram migrados ainda**. Por isso, implementamos uma **abordagem híbrida** que usa o melhor de cada versão.

## Mapeamento de Endpoints

### ✅ API v3 (Disponível e Utilizada)

| Recurso | Endpoint v3 | Status |
|---------|-------------|--------|
| Workspaces | `GET /workspaces` | ✅ Disponível |
| Groups | `GET /workspaces/{workspace_id}/groups` | ✅ Disponível |
| Docs | `GET /workspaces/{workspace_id}/docs` | ✅ Disponível |
| Webhooks | `POST /workspaces/{workspace_id}/webhooks` | ✅ Disponível |

**Observação**: Atualmente **NÃO** estamos usando v3 porque os endpoints principais (Spaces, Folders, Lists, Tasks) ainda estão em v2.

### ⚙️ API v2 (Utilizada Atualmente)

| Recurso | Endpoint v2 | Usado Por |
|---------|-------------|-----------|
| Spaces | `GET /team/{team_id}/space` | `folders.rs:199` |
| Folders | `GET /space/{space_id}/folder` | `folders.rs:228` |
| Folder Details | `GET /folder/{folder_id}` | `folders.rs:403` |
| Lists | `POST /folder/{folder_id}/list` | `folders.rs:439` |
| Tasks | `GET /team/{team_id}/task` | `folders.rs:318` |
| Custom Fields | `GET /list/{list_id}/field` | *(a implementar)* |

## Nomenclatura Interna vs API

### Nomenclatura Interna (v3-style)
Adotamos a nomenclatura da v3 internamente para facilitar migração futura:

```rust
pub struct SmartFolderFinder {
    client: ClickUpClient,
    workspace_id: String,  // ✅ Usa "workspace" internamente
    cache: HashMap<String, FolderSearchResult>,
}
```

### Mapeamento para API v2
Apesar do nome interno `workspace_id`, os endpoints chamam a API v2 com `team`:

```rust
// Internamente: workspace_id = "9013037641"
// Na API: /team/9013037641/space
let endpoint = format!("/team/{}/space", self.workspace_id);
```

## Variáveis de Ambiente

### Configuração Recomendada (v3-style)
```bash
export CLICKUP_WORKSPACE_ID="9013037641"
export CLICKUP_API_TOKEN="pk_xxxxx"
```

### Compatibilidade com Código Legado (v2-style)
```bash
export CLICKUP_TEAM_ID="9013037641"  # Ainda funciona como fallback
export CLICKUP_API_TOKEN="pk_xxxxx"
```

### Ordem de Precedência
```rust
let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")
    .or_else(|_| std::env::var("CLICKUP_TEAM_ID"))  // Fallback
    .unwrap_or_else(|_| "9013037641".to_string());
```

## Implementação do Cliente Híbrido

O `ClickUpClient` suporta ambas as versões:

```rust
pub struct ClickUpClient {
    http_client: HttpClient,
    api_token: String,
    base_url_v2: String,  // "https://api.clickup.com/api/v2"
    base_url_v3: String,  // "https://api.clickup.com/api/v3"
}
```

### Métodos Disponíveis

**Padrão (v2)**:
- `get(endpoint)` → usa v2
- `get_json(endpoint)` → usa v2
- `post(endpoint, body)` → usa v2
- `post_json(endpoint, body)` → usa v2

**Explícitos v2**:
- `get_v2(endpoint)` → força v2
- `get_json_v2(endpoint)` → força v2
- `post_v2(endpoint, body)` → força v2
- `post_json_v2(endpoint, body)` → força v2

**Explícitos v3** (para migração futura):
- `get_v3(endpoint)` → força v3
- `get_json_v3(endpoint)` → força v3
- `post_v3(endpoint, body)` → força v3
- `post_json_v3(endpoint, body)` → força v3

## Tabela de Diferenças v2 vs v3

| Aspecto | API v2 | API v3 |
|---------|--------|--------|
| **Termo Principal** | Team | Workspace |
| **Base URL** | `/api/v2` | `/api/v3` |
| **Spaces** | `GET /team/{team_id}/space` | ❌ Não migrado |
| **Folders** | `GET /space/{space_id}/folder` | ❌ Não migrado |
| **Lists** | `GET /folder/{folder_id}/list` | ❌ Não migrado |
| **Tasks** | `GET /team/{team_id}/task` | ❌ Não migrado |
| **Workspaces** | ❌ Não existe | `GET /workspaces` |
| **Groups** | `GET /team/{team_id}/group` | `GET /workspaces/{workspace_id}/groups` |

## Plano de Migração Futura

Quando a ClickUp migrar todos os endpoints para v3:

### Fase 1: Adicionar Suporte v3 (Atual)
- ✅ Cliente híbrido implementado
- ✅ Métodos `_v2` e `_v3` disponíveis
- ✅ Nomenclatura interna usando `workspace_id`

### Fase 2: Migração Gradual (Quando disponível)
1. Substituir `get_json()` por `get_json_v3()` nos endpoints migrados
2. Atualizar endpoints:
   - `/team/{team_id}/space` → `/workspaces/{workspace_id}/spaces`
   - `/team/{team_id}/task` → `/workspaces/{workspace_id}/tasks`
3. Testar compatibilidade

### Fase 3: Deprecar v2 (Futuro distante)
1. Marcar métodos `_v2` como deprecated
2. Atualizar todos os chamadores para v3
3. Remover suporte v2

## Referências

- [ClickUp API v2 vs v3](https://developer.clickup.com/docs/general-v2-v3-api)
- [Getting Started](https://developer.clickup.com/docs/Getting%20Started)
- [Migration Guide](https://developer.clickup.com/docs/migration-guide) *(quando disponível)*

## Decisões de Design

### Por que Híbrido?

✅ **Vantagens**:
- Usa endpoints estáveis (v2) para operações críticas
- Código preparado para migração futura
- Nomenclatura moderna (workspace em vez de team)
- Compatibilidade retroativa com variáveis `CLICKUP_TEAM_ID`

❌ **Alternativas Descartadas**:
- **Só v2**: Código legado que precisaria refatoração massiva futura
- **Só v3**: Muitos endpoints ainda não disponíveis, quebraria funcionalidade existente

### Por que Nomenclatura v3?

A interface do ClickUp já usa "Workspace" há muito tempo. Usar `team_id` no código causa confusão porque:
- ❌ Interface mostra "Workspace"
- ❌ Documentação nova usa "Workspace"
- ❌ API v3 padroniza como "Workspace"

Ao usar `workspace_id` internamente:
- ✅ Código alinhado com interface
- ✅ Preparado para v3
- ✅ Mais fácil de entender para novos desenvolvedores

## Exemplos Práticos

### Buscar Folders (usa v2 internamente)
```rust
let client = ClickUpClient::new("pk_xxxxx")?;
let mut finder = SmartFolderFinder::new(client, "9013037641".to_string());

// Internamente chama: GET /team/9013037641/space (v2)
let result = finder.find_folder_for_client("Nexcode").await?;
```

### Buscar Tasks Históricas (usa v2 internamente)
```rust
// Internamente chama: GET /team/9013037641/task?archived=false (v2)
let tasks = finder.search_historical_tasks("nexcode").await?;
```

### Quando v3 estiver disponível (futuro)
```rust
// Futuro: GET /workspaces/9013037641/tasks?archived=false (v3)
let endpoint = format!("/workspaces/{}/tasks?archived=false", workspace_id);
let tasks: TasksResponse = client.get_json_v3(&endpoint).await?;
```
