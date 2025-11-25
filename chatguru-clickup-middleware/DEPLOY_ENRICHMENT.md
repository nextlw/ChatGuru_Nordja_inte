# Deploy do Job de Enriquecimento de Tarefas

Este documento descreve como fazer o deploy do job de enriquecimento de tarefas no Google Cloud Platform.

## Arquitetura

```
Cloud Logging (App Engine/Cloud Run)
    ↓ (logs com "Task criada")
Cloud Logging Sink
    ↓
Pub/Sub Topic: task-enrichment-trigger
    ↓
Push Subscription: task-enrichment-sub
    ↓
Cloud Run Service: /enrich
    ↓
1. Extrai task_id do log
2. Busca tarefa no ClickUp
3. Verifica campos vazios
4. IA Service classifica
5. Valida valores no YAML
6. Atualiza tarefa
```

## Pré-requisitos

1. **Google Cloud SDK** instalado e autenticado
2. **Projeto GCP** configurado
3. **Secrets no Secret Manager**:
   - `openai-api-key` - Chave da API OpenAI
   - `clickup-api-token` - Token da API do ClickUp
   - `chatguru-api-token` - Token da API ChatGuru (opcional)

## Passo 1: Configurar Secrets

```bash
# Definir projeto
export PROJECT_ID=buzzlightear
gcloud config set project $PROJECT_ID

# Criar/atualizar secrets
echo -n "sua-openai-api-key" | gcloud secrets create openai-api-key \
  --data-file=- --replication-policy="automatic" || \
  echo -n "sua-openai-api-key" | gcloud secrets versions add openai-api-key \
  --data-file=-

echo -n "sua-clickup-api-token" | gcloud secrets create clickup-api-token \
  --data-file=- --replication-policy="automatic" || \
  echo -n "sua-clickup-api-token" | gcloud secrets versions add clickup-api-token \
  --data-file=-
```

## Passo 2: Deploy do Serviço

O deploy é feito automaticamente via Cloud Build quando você faz push para o repositório.

### Deploy Manual (se necessário)

```bash
cd chatguru-clickup-middleware

# Build da imagem
gcloud builds submit --config ../cloudbuild.yaml

# Ou deploy direto via Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --source . \
  --region southamerica-east1 \
  --platform managed \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 2 \
  --timeout 300 \
  --min-instances 1 \
  --max-instances 10 \
  --set-secrets="OPENAI_API_KEY=openai-api-key:latest,CLICKUP_API_TOKEN=clickup-api-token:latest"
```

## Passo 3: Configurar Cloud Logging Sink e Pub/Sub

Execute o script de setup:

```bash
cd chatguru-clickup-middleware
chmod +x setup-task-enrichment.sh
./setup-task-enrichment.sh
```

Este script:
1. Cria o Pub/Sub topic `task-enrichment-trigger`
2. Cria o Cloud Logging Sink que filtra logs de criação de tarefa
3. Configura permissões do Sink no topic
4. Cria a Push Subscription que envia mensagens para `/enrich`

### Configuração Manual (alternativa)

```bash
# 1. Criar topic
gcloud pubsub topics create task-enrichment-trigger \
  --project=buzzlightear

# 2. Criar Cloud Logging Sink
gcloud logging sinks create task-created-sink \
  pubsub.googleapis.com/projects/buzzlightear/topics/task-enrichment-trigger \
  --log-filter='(resource.type="gae_app" OR resource.type="cloud_run_revision") AND (textPayload=~"Task criada" OR textPayload=~"Tarefa criada" OR textPayload=~"🎉 Task criada")' \
  --project=buzzlightear

# 3. Obter service account do sink
SINK_SA=$(gcloud logging sinks describe task-created-sink \
  --project=buzzlightear \
  --format='value(writerIdentity)')

# 4. Dar permissão ao sink no topic
gcloud pubsub topics add-iam-policy-binding task-enrichment-trigger \
  --project=buzzlightear \
  --member="$SINK_SA" \
  --role="roles/pubsub.publisher"

# 5. Obter URL do serviço
SERVICE_URL=$(gcloud run services describe chatguru-clickup-middleware \
  --region=southamerica-east1 \
  --format='value(status.url)')

# 6. Criar Push Subscription
gcloud pubsub subscriptions create task-enrichment-sub \
  --topic=task-enrichment-trigger \
  --push-endpoint="$SERVICE_URL/enrich" \
  --ack-deadline=60 \
  --message-retention-duration=1d
```

## Passo 4: Verificar Deploy

```bash
# Verificar serviço
gcloud run services describe chatguru-clickup-middleware \
  --region=southamerica-east1

# Testar health check
SERVICE_URL=$(gcloud run services describe chatguru-clickup-middleware \
  --region=southamerica-east1 \
  --format='value(status.url)')

curl "$SERVICE_URL/health"
curl "$SERVICE_URL/ready"

# Verificar logs recentes
gcloud logging read 'resource.type="cloud_run_revision" AND resource.labels.service_name="chatguru-clickup-middleware"' \
  --limit=10 \
  --format=json
```

## Testando o Job

### Teste Manual

```bash
# Usar o script de teste local (simula mensagem do Pub/Sub)
./test-enrich-local.sh <task_id>

# Exemplo:
./test-enrich-local.sh 901322079100
```

### Teste Real (criar tarefa no ClickUp)

1. Crie uma tarefa no ClickUp via webhook existente
2. Verifique os logs do Cloud Logging:
   ```bash
   gcloud logging read 'textPayload=~"Task criada"' --limit=5
   ```
3. Verifique mensagens no Pub/Sub:
   ```bash
   gcloud pubsub subscriptions pull task-enrichment-sub --limit=5
   ```
4. Verifique logs do serviço de enriquecimento:
   ```bash
   gcloud logging read 'resource.type="cloud_run_revision" AND textPayload=~"enriquecendo"' --limit=10
   ```

## Monitoramento

### Logs Importantes

```bash
# Logs de criação de tarefa (origem)
gcloud logging read 'textPayload=~"Task criada"' --limit=10

# Logs do job de enriquecimento
gcloud logging read 'resource.type="cloud_run_revision" AND (textPayload=~"enriquecendo" OR textPayload=~"enriquecida")' --limit=10

# Erros do job
gcloud logging read 'resource.type="cloud_run_revision" AND severity>=ERROR' --limit=10
```

### Métricas do Pub/Sub

```bash
# Ver estatísticas da subscription
gcloud pubsub subscriptions describe task-enrichment-sub

# Ver mensagens não processadas
gcloud pubsub subscriptions pull task-enrichment-sub --limit=10
```

## Troubleshooting

### Job não está sendo acionado

1. **Verificar se o Sink está funcionando**:
   ```bash
   gcloud logging sinks describe task-created-sink
   ```

2. **Verificar se há mensagens no topic**:
   ```bash
   gcloud pubsub topics describe task-enrichment-trigger
   gcloud pubsub subscriptions pull task-enrichment-sub --limit=5
   ```

3. **Verificar se o endpoint está acessível**:
   ```bash
   curl -X POST "$SERVICE_URL/enrich" \
     -H "Content-Type: application/json" \
     -d '{"message":{"data":"dGVzdA=="}}'
   ```

### Erros de autenticação

- Verificar se os secrets estão configurados:
  ```bash
  gcloud secrets list
  gcloud secrets versions access latest --secret=clickup-api-token
  gcloud secrets versions access latest --secret=openai-api-key
  ```

### Campos não estão sendo atualizados

1. Verificar se a tarefa realmente tem os campos vazios
2. Verificar logs de validação:
   ```bash
   gcloud logging read 'textPayload=~"validação" OR textPayload=~"Validation"' --limit=10
   ```
3. Verificar se os IDs dos campos no `ai_prompt.yaml` estão corretos

## Variáveis de Ambiente

O serviço usa as seguintes variáveis de ambiente (configuradas no Cloud Run):

- `RUST_LOG=info` - Nível de log
- `RUST_ENV=production` - Ambiente
- `GCP_PROJECT_ID` - ID do projeto GCP
- `CONFIG_BUCKET` - Bucket com configurações
- `CLICKUP_API_TOKEN` - Secret do Secret Manager
- `OPENAI_API_KEY` - Secret do Secret Manager
- `AI_PROMPT_SOURCE=file` - Usar arquivo local (config/ai_prompt.yaml)

## Arquivos de Configuração

- `cloudbuild.yaml` - Pipeline de CI/CD
- `Dockerfile` - Imagem Docker
- `config/ai_prompt.yaml` - Configuração de categorias/subcategorias
- `setup-task-enrichment.sh` - Script de setup do Pub/Sub

## Atualizações

Para atualizar o serviço:

```bash
# Push para o repositório (dispara Cloud Build automaticamente)
git push origin main

# Ou deploy manual
gcloud builds submit --config cloudbuild.yaml
```

