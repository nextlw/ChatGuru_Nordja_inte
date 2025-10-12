# ChatGuru-ClickUp Integration Middleware

## Descrição
Middleware de integração entre ChatGuru (chatbot WhatsApp) e ClickUp (gerenciamento de tarefas) para automatizar a criação e atualização de tarefas baseadas em conversas do WhatsApp para a Nordja Company.

## 🏗️ Arquitetura Atual

```
ChatGuru → Webhook → Pub/Sub (RAW) → Worker → OpenAI → ClickUp
            ↓                         ↓
         ACK <100ms            Processa assíncrono
                                      ↓
                            Cloud SQL (PostgreSQL)
                          - Mapeamento Cliente/Atendente
                          - Cache de Estrutura ClickUp
```

## ✨ Funcionalidades Implementadas

### Core Features
- ✅ Recebimento de webhooks do ChatGuru com ACK imediato (<100ms)
- ✅ Processamento assíncrono via Google Pub/Sub
- ✅ Classificação inteligente de atividades com OpenAI
- ✅ Integração com Vertex AI para processamento de mídia
- ✅ Criação automática de tarefas no ClickUp
- ✅ Prevenção de duplicatas (detecta tarefas existentes)
- ✅ Atualização de tarefas existentes com histórico em comentários
- ✅ Envio de anotações de volta ao ChatGuru

### Estrutura Dinâmica (Cloud SQL)
- ✅ Resolução dinâmica de pastas/listas por Cliente + Atendente
- ✅ Mapeamento flexível via banco de dados PostgreSQL
- ✅ Cache em três camadas (memória + DB + ClickUp API)
- ✅ Criação automática de listas mensais por cliente
- ✅ Suporte a clientes inativos com listas individuais
- ✅ TTL de 1 hora para cache em memória

### OAuth2 & Segurança
- ✅ Autenticação OAuth2 para ClickUp (criar folders/spaces)
- ✅ Gerenciamento de secrets via Google Secret Manager
- ✅ Endpoints de OAuth2 para autorização manual

### Observabilidade
- ✅ Logging estruturado com tracing
- ✅ Health checks e endpoints de status
- ✅ Métricas de tempo de processamento
- ✅ Cloud Monitoring integration

## 📁 Estrutura do Projeto

```
ChatGuru_Nordja_inte/
├── chatguru-clickup-middleware/     # Middleware Rust (core)
│   ├── src/                         # Código fonte
│   ├── migrations/                  # PostgreSQL migrations
│   ├── cloud_functions/             # Vertex AI function
│   ├── config/                      # Configuração TOML
│   └── README.md                    # Documentação técnica
├── legacy-reference/                # Código de referência (Node.js)
├── CLAUDE.md                        # Instruções para Claude Code
└── README.md                        # Este arquivo
```

## 🚀 Deploy Rápido

### Deploy do Middleware

```bash
cd chatguru-clickup-middleware

# Build e deploy
gcloud builds submit . \
  --tag gcr.io/buzzlightear/chatguru-clickup-middleware:latest

gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --allow-unauthenticated
```

### Configurar OAuth2

1. Acesse: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/auth/clickup
2. Autorize o app no ClickUp
3. Copie o access token e salve no Secret Manager

## 📊 API Endpoints

### Webhook & Worker
- `POST /webhooks/chatguru` - Recebe payload do ChatGuru
- `POST /worker/process` - Processa mensagens do Pub/Sub

### OAuth2
- `GET /auth/clickup` - Inicia fluxo OAuth2
- `GET /auth/clickup/callback` - Callback OAuth2

### Health Checks
- `GET /health` - Liveness probe
- `GET /ready` - Readiness probe
- `GET /status` - Status detalhado

## 🗄️ Estrutura do Banco de Dados

### `folder_mapping`
Mapeia Cliente + Atendente → Pasta ClickUp

### `list_cache`
Cache de listas ClickUp para performance

Ver [migrations/README.md](chatguru-clickup-middleware/migrations/README.md) para detalhes.

## 📈 Métricas

- **Webhook ACK:** < 100ms
- **Worker Processing:** ~2-5s
- **Cache Hit Rate:** ~80%
- **Volume:** 1.000-1.200 tarefas/mês

## 💰 Custos GCP (Estimado)

- Cloud Run: ~$5/mês
- Cloud SQL: ~$7/mês
- Pub/Sub: $0 (free tier)
- **Total: ~$15/mês**

## 🔍 Troubleshooting

### Logs do Cloud Run

```bash
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=chatguru-clickup-middleware" \
  --limit=50 --project=buzzlightear
```

### Verificar Pub/Sub

```bash
gcloud pubsub subscriptions pull chatguru-webhook-subscription --limit 10 --auto-ack
```

## 📚 Documentação

- [Documentação Técnica do Middleware](chatguru-clickup-middleware/README.md)
- [Instruções Claude Code](CLAUDE.md)
- [Database Migrations](chatguru-clickup-middleware/migrations/README.md)

## 📝 Licença

Proprietary - Nordja Company / Buzzlightear Project

---

**Última atualização:** Outubro 2025 | **Versão:** 2.0.0
