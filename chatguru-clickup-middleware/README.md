# ChatGuru-ClickUp Middleware

Middleware Rust para integração entre ChatGuru e ClickUp com arquitetura event-driven, classificação IA e estrutura dinâmica.

## 📑 Índice

- [Arquitetura](#-arquitetura)
- [Features](#-features)
- [Deploy](#-deploy)
- [API Endpoints](#-api-endpoints)
- [Database](#️-database)
- [Configuração](#-configuração)
- [Desenvolvimento](#️-desenvolvimento)
- [Integração e Fluxos](#-integração-e-fluxos)
- [Testes](#-testes)
- [Troubleshooting](#-troubleshooting)

---

## 🏗️ Arquitetura

```
ChatGuru → Webhook → Pub/Sub → Worker → OpenAI → ClickUp
            ↓                     ↓                  ↓
         ACK <100ms          Classify          Create Task
                                ↓
                         Cloud SQL Cache
                      (Cliente → Folder/List)
```

### Componentes

- **Webhook Handler**: Recebe payload do ChatGuru, ACK <100ms, publica no Pub/Sub
- **Worker Handler**: Consome Pub/Sub, classifica com OpenAI, cria task no ClickUp
- **Estrutura Service**: Resolve Cliente + Atendente → Folder/List dinamicamente do Cloud SQL
- **Cache Strategy**: 3 camadas (memória 1h TTL → DB cache → ClickUp API)
- **OAuth2**: Autenticação para criar folders/spaces (Personal Token não pode)

---

## ✨ Features

### Core
- ✅ Event-driven com Pub/Sub (ACK < 100ms)
- ✅ Classificação de atividades com OpenAI
- ✅ Processamento de mídia com Vertex AI
- ✅ Estrutura dinâmica (Cliente → Pasta/Lista via YAML)
- ✅ Fuzzy matching para nomes de clientes (85% threshold)
- ✅ Cache em 3 camadas (memória + DB + API)
- ✅ OAuth2 para criar folders no ClickUp
- ✅ Proteção contra loop infinito do Pub/Sub (máx 3 retries)

### Estrutura Dinâmica
- Resolução via `Info_2` (Cliente Solicitante) → Folder ID
- Criação automática de listas mensais (formato "OUTUBRO 2025")
- Fallback para lista padrão se cliente não mapeado
- TTL de 1 hora para cache em memória

---

## 🚀 Deploy

### Build & Deploy

```bash
# Build da imagem via Cloud Build
gcloud builds submit . \
  --tag gcr.io/buzzlightear/chatguru-clickup-middleware:latest

# Deploy no Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --allow-unauthenticated \
  --set-env-vars="RUST_LOG=info,RUST_ENV=production"
```

### Configurar OAuth2

```bash
# 1. Acessar endpoint OAuth
curl https://your-service.run.app/auth/clickup

# 2. Autorizar no ClickUp (autorizar TODOS os workspaces)

# 3. Token é salvo automaticamente no Secret Manager
```

---

## 📊 API Endpoints

### Webhook & Worker
- `POST /webhooks/chatguru` - Webhook ChatGuru (ACK <100ms)
- `POST /worker/process` - Worker Pub/Sub

### OAuth2
- `GET /auth/clickup` - Inicia fluxo OAuth2
- `GET /auth/clickup/callback` - Callback OAuth2

### Health Checks
- `GET /health` - Liveness probe
- `GET /ready` - Readiness probe
- `GET /status` - Status detalhado

### ClickUp Debug (Admin)
- `GET /clickup/tasks` - Lista tarefas
- `GET /clickup/list` - Info da lista
- `GET /clickup/test` - Testa conexão

---

## 🗄️ Database

### Cloud SQL PostgreSQL

#### Estrutura Hierárquica

```
teams (Nordja)
  └── spaces (Atendentes)
      └── folders (Clientes)
          └── lists (Mensais: "OUTUBRO 2025")
```

#### Tabelas Principais

**folder_mapping** - Mapeia Cliente + Atendente → Folder
```sql
CREATE TABLE folder_mapping (
    id SERIAL PRIMARY KEY,
    client_name VARCHAR(255) NOT NULL,
    attendant_name VARCHAR(255) NOT NULL,
    folder_id VARCHAR(255) NOT NULL,
    folder_path VARCHAR(500) NOT NULL,
    is_active BOOLEAN DEFAULT true
);
```

**list_cache** - Cache de listas mensais
```sql
CREATE TABLE list_cache (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(255) NOT NULL,
    year_month VARCHAR(7) NOT NULL,
    list_name VARCHAR(255) NOT NULL,
    list_id VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_verified TIMESTAMP DEFAULT NOW()
);
```

#### Custom Fields

- `categories` - 12 categorias (Agendamentos, Compras, etc.)
- `subcategories` - 88 subcategorias com stars (1-4)
- `activity_types` - Rotineira, Específica, Dedicada
- `status_options` - Executar, Aguardando, Concluído
- `client_requesters` - 95 clientes mapeados

#### Ver Estrutura

```bash
# Conectar ao banco
gcloud sql connect chatguru-middleware-db --user=postgres

# Ver mapeamentos ativos
SELECT * FROM folder_mapping WHERE is_active = true;

# Ver cache de listas
SELECT * FROM list_cache WHERE is_active = true ORDER BY last_verified DESC;

# Ver estatísticas
SELECT
    'teams' AS table, COUNT(*) FROM teams
UNION ALL SELECT 'spaces', COUNT(*) FROM spaces
UNION ALL SELECT 'folders', COUNT(*) FROM folders
UNION ALL SELECT 'lists', COUNT(*) FROM lists;
```

---

## 🔧 Configuração

### Environment Variables

```bash
RUST_LOG=info
RUST_ENV=production
GCP_PROJECT_ID=buzzlightear
DATABASE_URL=postgres://user:pass@host/db

# OAuth2
CLICKUP_CLIENT_ID=...
CLICKUP_CLIENT_SECRET=...
CLICKUP_REDIRECT_URI=https://your-service.run.app/auth/clickup/callback
```

### Secrets (Secret Manager)

```bash
# OpenAI API Key
echo "sk-..." | gcloud secrets create openai-api-key --data-file=-

# ClickUp OAuth Token (criado automaticamente via /auth/clickup)
gcloud secrets create clickup-oauth-token --data-file=-

# ChatGuru API Token
echo "token..." | gcloud secrets create chatguru-api-token --data-file=-
```

### Mapeamento de Clientes

Arquivo: `config/cliente_solicitante_mappings.yaml`

```yaml
client_to_folder_mapping:
  - client_name: "Gabriel Benarros"
    folder_id: "600227bb-3d52-405d-b5a8-68f0de3aa94a"
  - client_name: "Raphaela Spielberg"
    folder_id: "abeb7e51-2ca7-4322-9a91-4a3b7f4ebd85"
  # ... 95 clientes mapeados

fallback_list_id: "901321080769"  # Lista "Sem Identificação" → OUTUBRO 2025
```

**Fuzzy Matching**: Sistema tolera erros de digitação (ex: "Spilberg" → "Spielberg", 98% score)

---

## 🛠️ Desenvolvimento

### Comandos

```bash
# Build
cargo build --release

# Run local
cargo run

# Format
cargo fmt

# Lint
cargo clippy

# Tests
cargo test
```

### Estrutura do Código

```
src/
├── main.rs              # Entry point + rotas
├── handlers/            # HTTP handlers
│   ├── webhook.rs       # ChatGuru webhook (ACK <100ms)
│   ├── worker.rs        # Pub/Sub worker (classificação + criação)
│   ├── auth.rs          # OAuth2 flow
│   ├── health.rs        # Health checks
│   └── clickup.rs       # ClickUp debug endpoints
├── services/            # Business logic
│   ├── clickup.rs       # ClickUp API client
│   ├── openai.rs        # OpenAI classification
│   ├── vertex.rs        # Vertex AI media processing
│   ├── folder_resolver.rs  # Fuzzy matching + fallback
│   ├── secrets.rs       # Secret Manager
│   └── prompts.rs       # AI prompts
├── models/              # Data structures
│   └── payload.rs       # ChatGuru → ClickUp transformation
└── config/              # Configuration loaders
    └── mod.rs
```

---

## 🔄 Integração e Fluxos

### Fluxo Event-Driven (Atual)

1. **WhatsApp → ChatGuru**: Usuário envia mensagem
2. **ChatGuru → Webhook**: `POST /webhooks/chatguru` com payload
3. **Webhook → Pub/Sub**: Publica RAW payload, retorna ACK <100ms ✅
4. **Pub/Sub → Worker**: Trigga `/worker/process`
5. **Worker Processing**:
   - Extrai Cliente (`Info_2`)
   - Resolve Folder via fuzzy matching (YAML)
   - Resolve List ID (cache 3-tier: memory → DB → API)
   - Classifica atividade com OpenAI
   - Processa mídia com Vertex AI (se aplicável)
   - Cria/atualiza task no ClickUp
   - Envia annotation de volta ao ChatGuru
6. **Resultado**: Task criada na estrutura correta ✅

### Resolução Dinâmica de Folder

```
1. Payload: campos_personalizados.Info_2 = "Raphaela Spilberg"
2. Normalize: "raphaela spilberg"
3. Fuzzy Match: Score 98% com "Raphaela Spielberg"
4. Resolve: folder_id = "abeb7e51-2ca7-4322-9a91-4a3b7f4ebd85"
5. Cache Check: Existe lista "OUTUBRO 2025"?
   - L1: In-memory (< 1ms)
   - L2: Database (~ 10ms)
   - L3: ClickUp API (~ 500ms, cria se não existe)
6. Create Task: POST /list/{list_id}/task
```

### Transformação de Payload

**ChatGuru Input:**
```json
{
  "nome": "João Silva",
  "texto_mensagem": "Preciso agendar consulta",
  "celular": "5511999999999",
  "campos_personalizados": {
    "Info_2": "Gabriel Benarros"
  }
}
```

**ClickUp Output:**
```json
{
  "name": "📋 Agendar consulta médica",
  "description": "**Mensagem:**\nPreciso agendar consulta\n\n**Contato:**\n📱 WhatsApp: +55 11 99999-9999\n👤 Nome: João Silva\n🏢 Cliente: Gabriel Benarros",
  "priority": 3,
  "custom_fields": [
    {"id": "eac5bbd3-...", "value": "4b6cd768-..."},  // Categoria: Agendamentos
    {"id": "5333c095-...", "value": "bff3cb1c-..."},  // Subcategoria: Consultas
    {"id": "83afcb8c-...", "value": 1}               // Estrelas: 1
  ]
}
```

### Proteção contra Loop Infinito (Pub/Sub)

- **Max Retries**: 3 tentativas
- **Erros Recuperáveis**: ClickUp API timeout, OpenAI rate limit (permite retry)
- **Erros Não Recuperáveis**: Config error, validation error (descarta imediatamente)
- **Dead Letter Topic**: Mensagens descartadas vão para DLT para análise

```rust
const MAX_RETRY_ATTEMPTS: u32 = 3;

match error {
    AppError::ClickUpApi(_) => retry(),  // Recuperável
    AppError::ConfigError(_) => discard(),  // Não recuperável
    _ if attempts > 3 => discard(),  // Limite excedido
}
```

---

## 🧪 Testes

### E2E Test (Recomendado)

```bash
cd tests
./test-e2e.js
```

### Manual Webhook Test

```bash
curl -X POST https://your-service.run.app/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "nome": "Cliente Teste",
    "texto_mensagem": "Teste de integração",
    "celular": "5511999999999",
    "campos_personalizados": {
      "Info_2": "Gabriel Benarros"
    }
  }'
```

### Testes Validados

| Caso | Input | Output | Status |
|------|-------|--------|--------|
| Nome exato | "Alex Ikonomopoulos" | Match exato (100%) | ✅ |
| Erro de digitação | "Raphaela Spilberg" | Fuzzy 98% → "Spielberg" | ✅ |
| Sem acento | "Dada Ribeiro" | Normalização → "Dadá" | ✅ |
| Cliente inexistente | "João Silva" | Fallback → Lista padrão | ✅ |

---

## 📈 Performance

| Métrica | Valor | Meta |
|---------|-------|------|
| **Webhook ACK** | < 100ms | ✅ |
| **Worker Processing** | 2-5s | ✅ |
| **Cache Hit Rate** | ~80% | ✅ |
| **Volume Mensal** | 1.000-1.200 tasks | ✅ |
| **Uptime** | 99.9% | ✅ |
| **Fuzzy Match Score** | 98% (teste) | ✅ |

### Cache Breakdown
- L1 (memory): < 1ms
- L2 (database): ~10ms
- L3 (API): ~500ms

---

## 🔍 Troubleshooting

### Logs

```bash
# Ver logs recentes
gcloud logging read "resource.type=cloud_run_revision AND \
  resource.labels.service_name=chatguru-clickup-middleware" \
  --limit=50 --project=buzzlightear

# Ver erros específicos
gcloud logging read 'textPayload=~"ERROR"' --limit=20

# Ver fallbacks (clientes não mapeados)
gcloud logging read 'textPayload=~"não encontrado, usando fallback"' --limit=50
```

### Pub/Sub

```bash
# Pull mensagens pendentes
gcloud pubsub subscriptions pull chatguru-worker-sub --limit 10 --auto-ack

# Ver mensagens no Dead Letter Topic
gcloud pubsub subscriptions pull chatguru-worker-dead-letter-sub --limit 10
```

### Database

```bash
# Conectar
gcloud sql connect chatguru-middleware-db --user=postgres

# Queries úteis
SELECT * FROM folder_mapping WHERE is_active = true;
SELECT * FROM list_cache WHERE is_active = true ORDER BY last_verified DESC;

# Ver cache expirado (> 1 hora)
SELECT * FROM list_cache
WHERE last_verified < NOW() - INTERVAL '1 hour';

# Invalidar cache manualmente
UPDATE list_cache SET is_active = FALSE
WHERE last_verified < NOW() - INTERVAL '1 hour';
```

### Problemas Comuns

#### 1. "Cliente não encontrado, usando fallback"

**Causa**: Cliente não está no YAML `cliente_solicitante_mappings.yaml`

**Solução**:
```yaml
# Adicionar ao YAML
- client_name: "Nome do Cliente"
  folder_id: "folder-id-do-clickup"
```

Ou verificar se é erro de digitação que fuzzy matching deveria pegar.

#### 2. "OAuth2 token não configurado"

**Causa**: Fluxo OAuth2 não foi completado

**Solução**:
```bash
# Executar fluxo OAuth2
curl https://your-service.run.app/auth/clickup
# Seguir redirecionamento e autorizar
```

#### 3. "Team not authorized (OAUTH_027)"

**Causa**: Workspace não foi autorizado durante OAuth2

**Solução**: Re-executar `/auth/clickup` e autorizar TODOS os workspaces

#### 4. Loop infinito de retries

**Causa**: Erro não classificado corretamente

**Solução**: Verificar logs e classificar erro apropriadamente em `worker.rs`

---

## 🔐 Segurança

- ✅ Tokens nunca expostos em logs (apenas primeiros 20 chars)
- ✅ Client Secret nunca salvo em memória
- ✅ Validação automática de tokens OAuth2
- ✅ Secret Manager para armazenamento seguro
- ✅ Cache TTL para minimizar validações
- ✅ Rate limiting via Pub/Sub

---

## 📚 Referências

### ClickUp API
- [Authentication](https://developer.clickup.com/docs/authentication)
- [Common Errors](https://developer.clickup.com/docs/common_errors)
- [API Reference](https://clickup.com/api)

### Google Cloud
- [Cloud Run](https://cloud.google.com/run/docs)
- [Pub/Sub](https://cloud.google.com/pubsub/docs)
- [Cloud SQL](https://cloud.google.com/sql/docs/postgres)
- [Secret Manager](https://cloud.google.com/secret-manager/docs)

### IA & ML
- [OpenAI API](https://platform.openai.com/docs)
- [Vertex AI](https://cloud.google.com/vertex-ai/docs)

---

## 📝 Licença

Proprietary - Nordja Company

**Última atualização**: 2025-10-15
