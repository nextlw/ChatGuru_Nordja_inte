# Configuração de Webhooks ClickUp

Guia completo para configurar webhooks do ClickUp integrados com Pub/Sub.

## 🎯 Arquitetura

```
ClickUp → Webhook Handler → Pub/Sub → Worker(s)
           ↓ Valida assinatura
           ↓ ACK < 100ms
           ✓ Publicado no topic
```

**Benefícios:**
- ✅ Processamento em tempo real de eventos ClickUp
- ✅ Retry automático via Pub/Sub
- ✅ Escalabilidade horizontal (múltiplos workers)
- ✅ Auditoria completa de eventos
- ✅ Desacoplamento total

## 📋 Pré-requisitos

1. **Aplicação deployada** com endpoint público HTTPS
2. **Workspace ID** do ClickUp (team_id)
3. **API Token** do ClickUp com permissões de webhook
4. **Webhook Secret** gerado pelo ClickUp
5. **Tópico Pub/Sub** criado no GCP

## 🔧 Passo 1: Configurar Tópico Pub/Sub

```bash
# Criar tópico para eventos do ClickUp
gcloud pubsub topics create clickup-webhook-events --project=buzzlightear

# Criar subscription para worker processar
gcloud pubsub subscriptions create clickup-events-worker \
  --topic=clickup-webhook-events \
  --ack-deadline=60 \
  --message-retention-duration=7d \
  --project=buzzlightear
```

## 🔧 Passo 2: Configurar Variáveis de Ambiente

Adicione ao seu Cloud Run / Cloud Functions:

```bash
# ClickUp
export CLICKUP_API_TOKEN="pk_your_token_here"
export CLICKUP_WORKSPACE_ID="9013037641"
export CLICKUP_WEBHOOK_SECRET="generated_by_clickup"

# GCP Pub/Sub
export GCP_PROJECT_ID="buzzlightear"
export GCP_CLICKUP_WEBHOOK_TOPIC="clickup-webhook-events"
```

Ou use Secret Manager:

```bash
# Armazenar secret do webhook
echo -n "webhook_secret_here" | gcloud secrets create clickup-webhook-secret \
  --data-file=- \
  --project=buzzlightear

# Configurar Cloud Run para usar secret
gcloud run deploy chatguru-clickup-middleware \
  --set-secrets=CLICKUP_WEBHOOK_SECRET=clickup-webhook-secret:latest \
  --region=southamerica-east1 \
  --project=buzzlightear
```

## 🔧 Passo 3: Criar Webhook no ClickUp

### Opção A: Via API (Recomendado)

```rust
use clickup::webhooks::{WebhookManager, WebhookConfig, WebhookEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_token = std::env::var("CLICKUP_API_TOKEN")?;
    let workspace_id = std::env::var("CLICKUP_WORKSPACE_ID")?;

    let manager = WebhookManager::from_token(api_token, workspace_id)?;

    let config = WebhookConfig {
        endpoint: "https://your-app.run.app/webhooks/clickup".to_string(),
        events: vec![
            WebhookEvent::TaskCreated,
            WebhookEvent::TaskUpdated,
            WebhookEvent::TaskStatusUpdated,
            WebhookEvent::TaskDeleted,
        ],
        status: Some("active".to_string()),
    };

    // Criar webhook (idempotente)
    let webhook = manager.ensure_webhook(&config).await?;
    println!("✅ Webhook configurado: {}", webhook.id);

    Ok(())
}
```

### Opção B: Via Endpoint HTTP

```bash
# Usando a API do próprio middleware
curl -X POST https://your-app.run.app/clickup/webhooks \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint": "https://your-app.run.app/webhooks/clickup",
    "events": ["taskCreated", "taskUpdated", "taskStatusUpdated"]
  }'
```

### Opção C: Via Interface ClickUp

1. Vá para Settings → Integrations → Webhooks
2. Clique em "Add Webhook"
3. Configure:
   - **Endpoint URL**: `https://your-app.run.app/webhooks/clickup`
   - **Events**: Selecione os eventos desejados
4. Copie o **Webhook Secret** gerado

## 🔒 Passo 4: Validar Assinatura (Segurança)

O handler já implementa validação automática:

```rust
// src/handlers/clickup_webhook.rs
pub async fn handle_clickup_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Json<Value>, AppError> {
    // 1. Extrair assinatura do header
    let signature = headers
        .get("X-Signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::ValidationError("Missing X-Signature".to_string()))?;

    // 2. Obter body raw (necessário para validar)
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await?;

    // 3. Obter secret
    let webhook_secret = std::env::var("CLICKUP_WEBHOOK_SECRET")?;

    // 4. Validar assinatura HMAC-SHA256
    if !WebhookPayload::verify_signature(signature, &webhook_secret, &body_bytes) {
        return Err(AppError::ValidationError("Invalid signature".to_string()));
    }

    // ✅ Assinatura válida, processar evento...
}
```

**IMPORTANTE**: Sempre valide assinaturas em produção!

## 📊 Passo 5: Monitorar Webhooks

### Listar Webhooks Registrados

```bash
curl https://your-app.run.app/clickup/webhooks
```

Resposta:
```json
{
  "count": 1,
  "webhooks": [
    {
      "id": "abc123",
      "endpoint": "https://your-app.run.app/webhooks/clickup",
      "status": "active",
      "events": ["taskCreated", "taskUpdated"],
      "health": {
        "status": "active",
        "fail_count": 0
      }
    }
  ]
}
```

### Verificar Logs

```bash
# Logs do webhook handler
gcloud logging read 'resource.type=cloud_run_revision
  AND resource.labels.service_name=chatguru-clickup-middleware
  AND textPayload=~"clickup-webhook"' \
  --limit=50 \
  --project=buzzlightear

# Logs do Pub/Sub
gcloud logging read 'resource.type=pubsub_topic
  AND resource.labels.topic_id=clickup-webhook-events' \
  --limit=50 \
  --project=buzzlightear
```

### Health Check do Webhook

O ClickUp monitora a saúde do webhook:
- Envia eventos de teste periodicamente
- Marca como "failing" se houver muitas falhas consecutivas
- Desativa automaticamente após threshold de falhas

Verifique no response de `GET /clickup/webhooks`:
```json
"health": {
  "status": "active",  // ✅ Saudável
  "fail_count": 0
}
```

## 🔧 Passo 6: Processar Eventos (Worker)

O worker já está configurado para processar eventos do Pub/Sub:

```rust
// Pub/Sub → Cloud Run via /worker/process
gcloud pubsub subscriptions create clickup-events-push \
  --topic=clickup-webhook-events \
  --push-endpoint=https://your-app.run.app/worker/process \
  --push-auth-service-account=worker@buzzlightear.iam.gserviceaccount.com
```

Ou use Pull subscription com Cloud Functions/Cloud Run Jobs.

## 🧪 Passo 7: Testar o Webhook

### Teste Manual (Simular Evento)

```bash
# Gerar assinatura válida
echo -n '{"webhook_id":"test","event":"taskCreated","task_id":"123"}' | \
  openssl dgst -sha256 -hmac "your_webhook_secret" | \
  awk '{print $2}'
# Output: abc123def456... (assinatura HMAC-SHA256)

# Enviar evento
curl -X POST https://your-app.run.app/webhooks/clickup \
  -H "Content-Type: application/json" \
  -H "X-Signature: abc123def456..." \
  -d '{"webhook_id":"test","event":"taskCreated","task_id":"123"}'
```

### Teste com Evento Real

Crie uma task no ClickUp e verifique os logs:

```bash
# Verificar se evento foi recebido
gcloud logging read 'textPayload=~"Evento ClickUp recebido"' \
  --limit=10 \
  --project=buzzlightear

# Verificar se foi publicado no Pub/Sub
gcloud logging read 'textPayload=~"publicado no Pub/Sub"' \
  --limit=10 \
  --project=buzzlightear
```

## 🚨 Troubleshooting

### Webhook não recebe eventos

1. **Verificar se webhook está ativo:**
   ```bash
   curl https://your-app.run.app/clickup/webhooks
   ```

2. **Verificar logs do ClickUp:**
   - Interface ClickUp → Settings → Webhooks → Ver logs

3. **Verificar endpoint público:**
   ```bash
   curl https://your-app.run.app/health
   ```

### Assinatura inválida

1. **Verificar secret configurado:**
   ```bash
   gcloud secrets versions access latest --secret=clickup-webhook-secret
   ```

2. **Comparar com secret do ClickUp:**
   - Interface ClickUp → Settings → Webhooks → Ver secret

### Eventos não chegam no worker

1. **Verificar tópico Pub/Sub:**
   ```bash
   gcloud pubsub topics describe clickup-webhook-events
   ```

2. **Verificar subscription:**
   ```bash
   gcloud pubsub subscriptions describe clickup-events-worker
   ```

3. **Verificar mensagens pendentes:**
   ```bash
   gcloud pubsub subscriptions pull clickup-events-worker --limit=1
   ```

## 📈 Escalabilidade

### Horizontal Scaling

O Cloud Run escala automaticamente baseado em:
- Número de requisições simultâneas
- CPU/memory usage

Configure limites:
```bash
gcloud run services update chatguru-clickup-middleware \
  --min-instances=1 \
  --max-instances=10 \
  --concurrency=80 \
  --region=southamerica-east1
```

### Rate Limiting

O ClickUp tem rate limits por workspace:
- **100 requests/min** por endpoint
- Webhooks não contam para rate limit de API

### Retry Policy

Pub/Sub garante entrega:
- Retry exponencial automático
- Dead letter queue após 5 tentativas
- Retention de 7 dias

## 🔐 Segurança

### Checklist

- ✅ HTTPS obrigatório (Cloud Run garante)
- ✅ Validar assinatura HMAC-SHA256
- ✅ Usar Secret Manager para secrets
- ✅ Autenticação IAM para Pub/Sub
- ✅ Logging de todas as requisições
- ✅ Rate limiting no Cloud Run

### Rotação de Secret

```bash
# 1. Gerar novo secret no ClickUp
# 2. Atualizar Secret Manager
echo -n "new_secret" | gcloud secrets versions add clickup-webhook-secret --data-file=-

# 3. Cloud Run usa automaticamente nova versão (latest)
# 4. Deletar secret antigo após validação
```

## 📚 Referências

- [ClickUp Webhooks API](https://developer.clickup.com/reference/createwebhook)
- [ClickUp Webhook Events](https://developer.clickup.com/docs/webhookevents)
- [ClickUp Webhook Signature](https://developer.clickup.com/docs/webhooksignature)
- [Google Cloud Pub/Sub](https://cloud.google.com/pubsub/docs)
- [Cloud Run Push Subscriptions](https://cloud.google.com/run/docs/tutorials/pubsub)
