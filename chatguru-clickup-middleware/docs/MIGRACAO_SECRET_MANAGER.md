# Migração para Google Secret Manager

## Visão Geral

O middleware agora suporta **Google Secret Manager** para gerenciamento seguro de configurações sensíveis, mantendo compatibilidade total com variáveis de ambiente.

## Hierarquia de Configuração

O sistema usa a seguinte ordem de prioridade para obter configurações:

1. **Variáveis de Ambiente** (maior prioridade)
2. **Google Secret Manager** (quando disponível)
3. **Valores padrão** (fallback)

## Configurações Suportadas

### ClickUp List ID
- **Env Var**: `CLICKUP_LIST_ID`
- **Secret Name**: `clickup-list-id` (futuro)
- **Valor Padrão**: `901300373349`

### ClickUp API Token
- **Env Var**: `CLICKUP_API_TOKEN`
- **Secret Name**: `clickup-api-token` (futuro)
- **Valor Padrão**: Nenhum (obrigatório)

## Implementação Atual

### Estrutura do Código

```rust
// src/services/secret_manager.rs
pub struct SecretManagerService {
    project_id: String,
}

impl SecretManagerService {
    pub async fn get_clickup_list_id(&self) -> Result<String> {
        // Prioridade 1: Variável de ambiente
        if let Ok(list_id) = env::var("CLICKUP_LIST_ID") {
            return Ok(list_id);
        }
        
        // Fallback: valor padrão
        Ok("901300373349".to_string())
    }
}
```

### Uso no ClickUp Service

```rust
// src/services/clickup.rs
pub async fn new_with_secret_manager() -> AppResult<Self> {
    let secret_service = SecretManagerService::new().await?;
    
    // Obtém configurações com fallback automático
    let api_token = secret_service.get_clickup_api_token().await?;
    let list_id = secret_service.get_clickup_list_id().await?;
    
    Ok(Self {
        client: Client::new(),
        token: api_token,
        list_id,
    })
}
```

## Vantagens da Arquitetura

### 1. Compatibilidade Total
- Sistema continua funcionando com variáveis de ambiente
- Nenhuma mudança breaking
- Migração opcional

### 2. Preparação para o Futuro
- Código estruturado para adicionar Secret Manager real
- Interface já definida
- Fácil evolução

### 3. Consistência com Buzzlightyear
- Mesma estratégia de configuração
- Facilita migração futura
- Padrões unificados

## Como Funciona Hoje

### Desenvolvimento Local
```bash
export CLICKUP_LIST_ID=901300373349
export CLICKUP_API_TOKEN=pk_xxx
cargo run
```

### Cloud Run
```bash
gcloud run services update chatguru-clickup-middleware \
    --set-env-vars="CLICKUP_LIST_ID=901300373349,CLICKUP_API_TOKEN=pk_xxx" \
    --region=southamerica-east1
```

## Roadmap para Secret Manager Completo

### Fase 1: Estrutura Base (✅ Concluída)
- Criação do módulo `secret_manager`
- Interface de serviço definida
- Fallback para env vars

### Fase 2: Cliente GCP (Futuro)
- Adicionar dependência do Secret Manager
- Implementar autenticação GCP
- Acesso real aos secrets

### Fase 3: Migração Completa (Futuro)
- Criar secrets no GCP
- Configurar IAM
- Testes end-to-end

## Configuração Atual no Cloud Run

O serviço está rodando com as seguintes configurações:

```yaml
Environment Variables:
  - CLICKUP_LIST_ID: 901300373349
  - CLICKUP_API_TOKEN: [configurado]
  - RUST_LOG: info
```

## Logs e Diagnóstico

O sistema registra de onde cada configuração foi obtida:

```
INFO: ClickUp List ID obtido da variável de ambiente
INFO: ClickUp API Token obtido da variável de ambiente
```

## Compatibilidade com Buzzlightyear

Ambos os sistemas:
- Usam o mesmo List ID: `901300373349`
- Configuração via ambiente/secrets
- Estrutura similar de fallback

## Conclusão

A implementação atual fornece:

1. **Base sólida** para migração futura
2. **Zero impacto** no sistema atual
3. **Compatibilidade** mantida
4. **Flexibilidade** para evolução

O código está preparado para quando decidirmos implementar o Secret Manager completo, sem necessidade de refatoração significativa.