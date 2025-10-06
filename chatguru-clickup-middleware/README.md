# ChatGuru-ClickUp Middleware (Event-Driven)

Middleware para integração entre ChatGuru e ClickUp usando arquitetura event-driven com Google Cloud Pub/Sub.

## 🏗️ Arquitetura

```
ChatGuru → Webhook → Pub/Sub (RAW) → Worker → OpenAI → ClickUp
            ↓                         ↓
         ACK <100ms            Processa assíncrono
```

### Componentes

**Handlers:**
- `webhook.rs` - Recebe payload e publica no Pub/Sub (ACK imediato)
- `worker.rs` - Consome Pub/Sub e processa com OpenAI
- `health.rs` - Health checks
- `clickup.rs` - Endpoints ClickUp (debug)

**Services:**
- `openai.rs` - Classificação de atividades com OpenAI
- `clickup.rs` - Integração com ClickUp API
- `chatguru.rs` - Envio de anotações
- `secrets.rs` - Gerenciamento de API keys (Secret Manager)
- `prompts.rs` - Configuração de prompts da IA

**Models:**
- `payload.rs` - Estruturas de dados (ChatGuru, ClickUp)

## 🚀 Deploy

### Build da imagem Docker

```bash
gcloud builds submit \
  --region=southamerica-east1 \
  --tag gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --timeout=30m
```

### Deploy no Cloud Run

```bash
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --platform managed \
  --allow-unauthenticated \
  --set-env-vars "RUST_LOG=info" \
  --min-instances 1 \
  --max-instances 10
```

### Criar tópico Pub/Sub

```bash
gcloud pubsub topics create chatguru-webhook-raw \
  --project buzzlightear

gcloud pubsub subscriptions create chatguru-webhook-subscription \
  --topic chatguru-webhook-raw \
  --ack-deadline 60
```

### Configurar Cloud Tasks

```bash
gcloud tasks queues create chatguru-worker-queue \
  --location southamerica-east1
```

## 🧪 Testes

### Teste End-to-End (E2E)

O teste E2E valida todo o fluxo com observabilidade completa:

```bash
# Local (desenvolvimento)
./test-e2e.js

# Produção (Cloud Run)
WEBHOOK_URL=https://your-service.run.app \
WORKER_URL=https://your-service.run.app \
./test-e2e.js
```

**Output do teste E2E:**

```
━━━ ETAPA 1: Enviar payload para Webhook ━━━
ℹ Enviando payload ChatGuru para /webhooks/chatguru
Payload de entrada:
{
  "nome": "Cliente Teste E2E",
  "texto_mensagem": "Preciso implementar funcionalidade...",
  ...
}
⏱  Tempo de resposta do webhook: 45ms
✓ Webhook ACK recebido: {"message":"Success"}
✓ ✨ ACK em 45ms - EXCELENTE (<100ms target)

━━━ ETAPA 2: Pub/Sub recebe payload RAW ━━━
...

━━━ ETAPA 3: Worker processa mensagem do Pub/Sub ━━━
⏱  Tempo de processamento do worker: 2500ms
✓ Worker processou mensagem com sucesso

━━━ ETAPA 4: OpenAI classifica a atividade ━━━
🤖 Prompt OpenAI:
System: Você é um assistente especializado...
User: Campanha: WhatsApp...

━━━ ETAPA 5: ClickUp Task criada ━━━
...

━━━ RESUMO DO TESTE E2E ━━━
Etapas completadas: 6/6
✓ Etapa 1: Webhook (45ms)
✓ Etapa 2: Pub/Sub
✓ Etapa 3: Worker (2500ms)
✓ Etapa 4: OpenAI
✓ Etapa 5: ClickUp
✓ Etapa 6: ChatGuru

⏱  Tempo total do teste: 3200ms

✨ TESTE E2E PASSOU COM SUCESSO! ✨
```

### Teste de Health Check

```bash
curl https://your-service.run.app/health
curl https://your-service.run.app/status
```

### Teste manual do Webhook

```bash
curl -X POST https://your-service.run.app/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "nome": "Teste Manual",
    "texto_mensagem": "Testar integração",
    "celular": "5511999999999",
    "chat_id": "test-123"
  }'
```

## 📊 Observabilidade

### Logs estruturados

Todos os logs são estruturados para fácil análise:

```
[INFO] 📥 Webhook payload recebido (1234 bytes)
[INFO] ✅ Payload enviado para Pub/Sub com sucesso
[INFO] 💬 Processando mensagem de Cliente X: mensagem...
[INFO] 🤖 Classificando com OpenAI diretamente...
[INFO] ✅ Atividade identificada: Nova funcionalidade...
[INFO] ✅ Tarefa criada no ClickUp: TASK-123
[INFO] 📝 Anotação enviada ao ChatGuru
```

### Métricas importantes

- **Webhook latency:** Target < 100ms
- **Worker processing:** Variável (OpenAI + ClickUp)
- **OpenAI latency:** ~2-5s
- **ClickUp API:** ~500ms

### Cloud Monitoring

Queries úteis:

```
# Webhook latency
resource.type="cloud_run_revision"
resource.labels.service_name="chatguru-clickup-middleware"
httpRequest.requestUrl=~"webhooks/chatguru"

# Worker errors
severity>=ERROR
resource.type="cloud_run_revision"
```

## 🔧 Configuração

### Variáveis de ambiente

```bash
RUST_LOG=info                    # Log level
CLICKUP_API_TOKEN=pk_xxx         # ClickUp API token
CLICKUP_LIST_ID=901300373349     # ClickUp list ID
```

### Arquivo config/default.toml

```toml
[server]
host = "0.0.0.0"
port = 8080

[clickup]
base_url = "https://api.clickup.com/api/v2"
list_id = "901300373349"

[gcp]
project_id = "buzzlightear"
pubsub_topic = "chatguru-webhook-raw"

[chatguru]
api_token = "YOUR_TOKEN"
api_endpoint = "https://s15.chatguru.app/api/v1"
account_id = "625584ce6fdcb7bda7d94aa8"
```

## 🔐 Secrets

As API keys são gerenciadas pelo Google Secret Manager:

```bash
# OpenAI API Key
gcloud secrets create openai-api-key \
  --data-file=- <<< "sk-proj-xxx"

# ClickUp API Token
gcloud secrets create clickup-api-token \
  --data-file=- <<< "pk_xxx"
```

## 📈 Custos

Para volume atual (~1.200 mensagens/mês):

- **Cloud Run:** ~$5/mês (min-instances=0, média de requests)
- **Pub/Sub:** $0 (dentro do free tier de 10GB/mês)
- **Cloud Tasks:** $0 (dentro do free tier de 1M operações/mês)
- **Secret Manager:** ~$0.06/mês (6 secrets × $0.01)

**Total estimado: ~$5-10/mês**

## 🛠️ Desenvolvimento

### Compilar localmente

```bash
cargo build --release
```

### Executar localmente

```bash
cargo run
```

### Verificar código

```bash
cargo fmt
cargo clippy
```

## 📚 Endpoints

### Webhook
- `POST /webhooks/chatguru` - Recebe payload do ChatGuru

### Worker
- `POST /worker/process` - Processa mensagem do Pub/Sub

### Health Checks
- `GET /health` - Liveness probe
- `GET /ready` - Readiness probe
- `GET /status` - Status detalhado

### ClickUp (Debug)
- `GET /clickup/tasks` - Lista tarefas
- `GET /clickup/list` - Info da lista
- `GET /clickup/test` - Testa conexão

## 🐛 Troubleshooting

### Webhook não responde

```bash
# Verificar logs
gcloud run services logs read chatguru-clickup-middleware \
  --region southamerica-east1 \
  --limit 50

# Verificar se está rodando
curl https://your-service.run.app/health
```

### Worker não processa mensagens

```bash
# Verificar subscription
gcloud pubsub subscriptions describe chatguru-webhook-subscription

# Ver mensagens na fila
gcloud pubsub subscriptions pull chatguru-webhook-subscription \
  --limit 10 \
  --auto-ack
```

### OpenAI timeout

- Verificar se a API key está configurada
- Verificar quota da OpenAI
- Ver logs: `RUST_LOG=debug cargo run`

### ClickUp API error

- Verificar token no Secret Manager
- Verificar list_id correto
- Testar endpoint: `curl https://your-service.run.app/clickup/test`

## 📋 Campos Personalizados (Custom Fields)

O middleware suporta campos personalizados na criação de tarefas. Para configurar:

### 1. Descobrir IDs dos Campos
```bash
# Via endpoint do middleware (mais fácil)
curl http://localhost:8080/clickup/fields

# Via API REST direta
curl -H "Authorization: $CLICKUP_API_TOKEN" \
  https://api.clickup.com/api/v2/list/901300373349/field

# Via script automatizado
cd scripts && node discover_custom_fields.js
```

### 2. Configurar Campos no Código
Edite `src/handlers/worker.rs` na função `prepare_custom_fields()`:
- Descomente os campos necessários
- Substitua os IDs pelos valores reais descobertos
- Configure valores dinâmicos baseados nos dados recebidos

### 3. Tipos de Campo Suportados
- **text**: Campos de texto simples
- **number**: Campos numéricos
- **dropdown**: Seleção (valores devem existir nas opções)
- **date**: Data/hora (timestamp em milliseconds)
- **email**: Campos de email
- **phone**: Campos de telefone

### 4. Exemplo de Implementação
```rust
// Campo: Nome do Cliente (text)
custom_fields.push(json!({
    "id": "12345678-1234-1234-1234-123456789012",
    "value": nome
}));

// Campo: Categoria (dropdown)
if let Some(category) = &classification.category {
    custom_fields.push(json!({
        "id": "87654321-4321-4321-4321-210987654321",
        "value": category // Deve existir nas opções
    }));
}
```

### 5. Campos Disponíveis para Configuração
- **Origem da campanha** (WhatsApp)
- **Nome do cliente** (extraído do payload)
- **Telefone** (extraído do payload)
- **Categoria** (classificação IA)
- **Score de confiança** (classificação IA)
- **Data de criação** (timestamp automático)
- **Prioridade** (configurável)
- **Status da campanha** (configurável)

**Importante**: Campos dropdown devem usar valores exatos que existem nas opções configuradas no ClickUp.

## � Licença

Proprietary - Nordja/Buzzlightear
