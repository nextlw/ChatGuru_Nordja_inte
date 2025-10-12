# Módulo de Autenticação OAuth2

Módulo **completamente isolado** para gerenciar autenticação OAuth2 com ClickUp API.

## 📋 Estrutura

```
src/auth/
├── mod.rs            # Módulo principal
├── config.rs         # Configurações OAuth2
├── client.rs         # Cliente HTTP OAuth2
├── token_manager.rs  # Gerenciamento de tokens
├── handlers.rs       # Handlers HTTP (rotas)
└── README.md         # Este arquivo
```

## 🎯 Responsabilidades

### config.rs
- Carregar configurações OAuth2 de variáveis de ambiente
- Gerar URL de autorização do ClickUp
- Centralizar CLIENT_ID, CLIENT_SECRET, REDIRECT_URI

### client.rs
- Trocar authorization code por access token
- Verificar workspaces autorizados
- Validar tokens OAuth2
- Comunicação com ClickUp OAuth API

### token_manager.rs
- **Cache em memória** (TTL 1 hora)
- **Leitura do Secret Manager** (se cache expirado)
- **Validação automática** de tokens
- **Salvamento no Secret Manager** (create_or_update)
- Fornecimento de tokens válidos para API calls

### handlers.rs
- **GET /auth/clickup**: Inicia fluxo OAuth2
- **GET /auth/clickup/callback**: Recebe code e obtém token
- Renderização de páginas HTML (sucesso/erro)
- Exibição de workspaces autorizados

## 🔑 Fluxo OAuth2

```
1. Usuário acessa → GET /auth/clickup
2. Redireciona para → https://app.clickup.com/api?client_id=XXX
3. Usuário autoriza workspaces no ClickUp
4. ClickUp redireciona → GET /auth/clickup/callback?code=YYY
5. Backend troca code por access_token
6. Valida token e verifica workspaces autorizados
7. Salva token no Secret Manager (clickup-oauth-token)
8. Atualiza cache em memória
9. Exibe página de sucesso com lista de workspaces
```

## 🚀 Como Usar

### 1. Configurar Variáveis de Ambiente

```bash
export CLICKUP_CLIENT_ID="seu_client_id"
export CLICKUP_CLIENT_SECRET="seu_client_secret"
export CLICKUP_REDIRECT_URI="https://your-app.com/auth/clickup/callback"
export GCP_PROJECT_ID="seu-projeto-gcp"
```

### 2. Integrar no main.rs

```rust
use chatguru_clickup_middleware::auth::{
    OAuth2Config, TokenManager, OAuth2State,
    start_oauth_flow, handle_oauth_callback
};

// Inicializar OAuth2
let oauth_config = OAuth2Config::from_env()
    .expect("Failed to load OAuth2 config");

let token_manager = Arc::new(TokenManager::new(
    oauth_config.clone(),
    project_id.clone(),
));

let oauth_state = Arc::new(OAuth2State {
    config: oauth_config,
    token_manager,
});

// Adicionar rotas
let app = Router::new()
    .route("/auth/clickup", get(start_oauth_flow))
    .route("/auth/clickup/callback", get(handle_oauth_callback))
    .with_state(oauth_state.clone());
```

### 3. Usar Token em API Calls

```rust
// Obter token válido
let token = token_manager.get_valid_token().await?;

// Usar em chamadas ClickUp API
let response = client
    .post("https://api.clickup.com/api/v2/space/123/folder")
    .header("Authorization", &token)
    .json(&body)
    .send()
    .await?;
```

## 🔒 Segurança

- ✅ **Token nunca exposto em logs** (apenas primeiros 20 caracteres)
- ✅ **Client Secret nunca salvo em memória** (apenas durante troca)
- ✅ **Validação automática** antes de usar token
- ✅ **Cache TTL** para minimizar validações
- ✅ **Secret Manager** para armazenamento seguro

## 🧪 Testes

```bash
# Testar configuração
cargo test auth::config::tests

# Testar cliente OAuth2
cargo test auth::client::tests

# Testar token manager
cargo test auth::token_manager::tests
```

## 📊 Logs

O módulo usa logging estruturado com prefixo `[OAuth2]` e `[TokenManager]`:

```
✅ [OAuth2] Access token obtido: 5a823a64061246a2fb94...
✅ [OAuth2] 1 workspaces autorizados
  ├─ Nordja (ID: 9013037641)
✅ [TokenManager] Token validado e atualizado no cache
```

## ⚠️  Diferenças: Personal Token vs OAuth2 Token

| Aspecto | Personal Token | OAuth2 Token |
|---------|----------------|--------------|
| Formato | `{user_id}_{hash}` | String aleatória longa |
| Workspaces | Todos que você tem acesso | Apenas autorizados |
| Criar Folders | ❌ Não (OAUTH_019) | ✅ Sim |
| Criar Spaces | ❌ Não (OAUTH_019) | ✅ Sim |
| Criar Tasks | ✅ Sim | ✅ Sim |
| Expiração | Nunca | Pode expirar |
| Revogação | Manual (UI ClickUp) | Automática se token inválido |

## 🔧 Troubleshooting

### Erro: "OAuth2 token não configurado"
- Execute `/auth/clickup` para iniciar fluxo de autorização
- Verifique se `CLICKUP_CLIENT_ID` e `CLICKUP_CLIENT_SECRET` estão definidos

### Erro: "Token inválido ou expirado"
- Execute `/auth/clickup` novamente para re-autorizar
- Verifique se o token no Secret Manager não foi corrompido

### Erro: "Team not authorized" (OAUTH_027)
- O workspace não foi autorizado durante o OAuth
- Re-execute `/auth/clickup` e autorize TODOS os workspaces necessários

### Erro: "Oauth token not found" (OAUTH_019)
- Está usando Personal Token ao invés de OAuth2 token
- Execute `/auth/clickup` para obter token OAuth2 verdadeiro

## 📚 Referências

- [ClickUp OAuth Documentation](https://developer.clickup.com/docs/authentication)
- [ClickUp Common Errors](https://developer.clickup.com/docs/common_errors)
- [Google Secret Manager](https://cloud.google.com/secret-manager/docs)
