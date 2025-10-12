# ChatGuru-ClickUp Middleware

Middleware Rust para integração entre ChatGuru e ClickUp com arquitetura event-driven, classificação IA e estrutura dinâmica.

## 🏗️ Arquitetura

```
ChatGuru → Webhook → Pub/Sub → Worker → OpenAI → ClickUp
            ↓                     ↓                  ↓
         ACK <100ms          Classify          Create Task
                                ↓
                         Cloud SQL Cache
                      (Cliente → Folder/List)
```

## ✨ Features

### Core
- Event-driven com Pub/Sub (ACK < 100ms)
- Classificação de atividades com OpenAI
- Processamento de mídia com Vertex AI
- Estrutura dinâmica (Cliente + Atendente → Pasta/Lista)
- Cache em 3 camadas (memória + DB + API)
- OAuth2 para criar folders no ClickUp

### Estrutura Dinâmica
- Resolução dinâmica por Cliente + Atendente
- Criação automática de listas mensais
- Suporte a clientes inativos com listas individuais
- TTL de 1 hora para cache em memória

## 🚀 Deploy

### Build & Deploy

```bash
# Build da imagem
gcloud builds submit . \
  --tag gcr.io/buzzlightear/chatguru-clickup-middleware:latest

# Deploy no Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --allow-unauthenticated \
  --set-env-vars="RUST_LOG=info"
```

### Configurar OAuth2

```bash
# 1. Acessar endpoint OAuth
curl https://your-service.run.app/auth/clickup

# 2. Autorizar no ClickUp

# 3. Copiar token e salvar
echo "YOUR_TOKEN" | gcloud secrets create clickup-oauth-token --data-file=-
```

## 📊 Endpoints

### Webhook & Worker
- `POST /webhooks/chatguru` - Webhook ChatGuru
- `POST /worker/process` - Worker Pub/Sub

### OAuth2
- `GET /auth/clickup` - Inicia OAuth2
- `GET /auth/clickup/callback` - Callback OAuth2

### Health
- `GET /health` - Liveness
- `GET /ready` - Readiness
- `GET /status` - Status

### ClickUp (Debug)
- `GET /clickup/tasks` - Lista tarefas
- `GET /clickup/list` - Info lista
- `GET /clickup/test` - Testa conexão

## 🗄️ Database (Cloud SQL)

### `folder_mapping`
Mapeia Cliente + Atendente → Pasta ClickUp

```sql
CREATE TABLE folder_mapping (
    id SERIAL PRIMARY KEY,
    client_name VARCHAR(255) NOT NULL,
    attendant_name VARCHAR(255) NOT NULL,
    folder_id VARCHAR(255) NOT NULL,
    folder_path VARCHAR(500) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### `list_cache`
Cache de listas mensais

```sql
CREATE TABLE list_cache (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(255) NOT NULL,
    year_month VARCHAR(7) NOT NULL,
    list_name VARCHAR(255) NOT NULL,
    list_id VARCHAR(255) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_verified TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW()
);
```

## 🔧 Configuração

### Environment Variables

```bash
RUST_LOG=info
RUST_ENV=production
GCP_PROJECT_ID=buzzlightear
DATABASE_URL=postgres://...

# OAuth2
CLICKUP_CLIENT_ID=...
CLICKUP_CLIENT_SECRET=...
CLICKUP_REDIRECT_URI=https://your-service.run.app/auth/clickup/callback
```

### Secrets (Secret Manager)

```bash
# OpenAI
gcloud secrets create openai-api-key --data-file=-

# ClickUp OAuth Token
gcloud secrets create clickup-oauth-token --data-file=-

# ChatGuru
gcloud secrets create chatguru-api-token --data-file=-
```

## 🧪 Testes

### E2E Test

```bash
./test-e2e.js
```

### Manual Webhook Test

```bash
curl -X POST https://your-service.run.app/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "nome": "Cliente Teste",
    "texto_mensagem": "Teste de mensagem",
    "celular": "5511999999999",
    "campos_personalizados": {
      "Info_1": "Nexcode",
      "Info_2": "William"
    }
  }'
```

## 📈 Performance

- **Webhook ACK:** < 100ms
- **Worker:** ~2-5s (OpenAI + ClickUp)
- **Cache Hit:** ~80%
- **Volume:** 1.000-1.200 tarefas/mês

## 🔍 Troubleshooting

### Logs

```bash
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=chatguru-clickup-middleware" \
  --limit=50 --project=buzzlightear
```

### Pub/Sub

```bash
gcloud pubsub subscriptions pull chatguru-webhook-subscription --limit 10 --auto-ack
```

### Database

```bash
gcloud sql connect chatguru-middleware-db --user=postgres

# Ver mapeamentos
SELECT * FROM folder_mapping WHERE is_active = true;

# Ver cache
SELECT * FROM list_cache WHERE is_active = true ORDER BY last_verified DESC;
```

## 🛠️ Desenvolvimento

```bash
# Build
cargo build --release

# Run
cargo run

# Format
cargo fmt

# Lint
cargo clippy
```

## 📁 Estrutura

```
src/
├── main.rs              # Entry point
├── handlers/            # HTTP handlers
│   ├── webhook.rs       # ChatGuru webhook
│   ├── worker.rs        # Pub/Sub worker
│   ├── auth.rs          # OAuth2
│   ├── health.rs        # Health checks
│   └── clickup.rs       # ClickUp debug
├── services/            # Business logic
│   ├── clickup.rs       # ClickUp API
│   ├── clickup_oauth.rs # OAuth2
│   ├── estrutura.rs     # Dynamic structure
│   ├── openai.rs        # OpenAI
│   ├── chatguru.rs      # ChatGuru API
│   ├── secrets.rs       # Secret Manager
│   └── prompts.rs       # AI prompts
├── models/              # Data structures
│   └── payload.rs
├── config/              # Configuration
│   └── mod.rs
└── utils/               # Utilities
    ├── error.rs
    └── logging.rs
```

## 📝 Licença

Proprietary - Nordja Company
