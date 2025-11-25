#!/bin/bash
# Setup do Job de Enriquecimento de Tarefas via Cloud Logging
#
# Este script configura:
# 1. Pub/Sub Topic para receber logs de criação de tarefas
# 2. Cloud Logging Sink para filtrar e enviar logs ao Pub/Sub
# 3. Subscription para acionar o job de enriquecimento
#
# Pré-requisitos:
# - gcloud CLI instalado e autenticado
# - Projeto GCP configurado

set -e

PROJECT_ID="${PROJECT_ID:-buzzlightear}"
REGION="${REGION:-southamerica-east1}"
SERVICE_NAME="${SERVICE_NAME:-chatguru-clickup-middleware}"
TOPIC_NAME="task-enrichment-trigger"
SINK_NAME="task-created-sink"
SUBSCRIPTION_NAME="task-enrichment-sub"

# Obter URL do serviço Cloud Run
echo "🔍 Obtendo URL do serviço Cloud Run..."
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME \
  --project=$PROJECT_ID \
  --region=$REGION \
  --format='value(status.url)' 2>/dev/null)

if [ -z "$SERVICE_URL" ]; then
  echo "⚠️ Serviço Cloud Run não encontrado. Usando URL padrão."
  ENRICH_ENDPOINT="${ENRICH_ENDPOINT:-https://${SERVICE_NAME}-$(gcloud config get-value project 2>/dev/null | cut -d: -f2).${REGION}.run.app/enrich}"
else
  ENRICH_ENDPOINT="${SERVICE_URL}/enrich"
  echo "✅ URL do serviço: $SERVICE_URL"
fi

echo "🚀 Configurando Job de Enriquecimento de Tarefas"
echo "   Projeto: $PROJECT_ID"
echo "   Região: $REGION"
echo "   Serviço: $SERVICE_NAME"
echo "   Topic: $TOPIC_NAME"
echo "   Sink: $SINK_NAME"
echo "   Endpoint: $ENRICH_ENDPOINT"
echo ""

# 1. Criar Pub/Sub Topic
echo "📬 Criando Pub/Sub Topic..."
gcloud pubsub topics create $TOPIC_NAME \
    --project=$PROJECT_ID \
    2>/dev/null || echo "   Topic já existe"

# 2. Criar Cloud Logging Sink
echo "🔍 Criando Cloud Logging Sink..."
SINK_DESTINATION="pubsub.googleapis.com/projects/$PROJECT_ID/topics/$TOPIC_NAME"
# Filtro para capturar logs de criação de tarefa do App Engine ou Cloud Run
SINK_FILTER='(resource.type="gae_app" OR resource.type="cloud_run_revision") AND (textPayload=~"Task criada" OR textPayload=~"Tarefa criada" OR textPayload=~"🎉 Task criada")'

gcloud logging sinks create $SINK_NAME $SINK_DESTINATION \
    --project=$PROJECT_ID \
    --log-filter="$SINK_FILTER" \
    2>/dev/null || {
    echo "   Sink já existe, atualizando..."
    gcloud logging sinks update $SINK_NAME \
        --project=$PROJECT_ID \
        --log-filter="$SINK_FILTER" \
        2>/dev/null || echo "   Erro ao atualizar sink"
}

# 3. Obter service account do sink e dar permissão no topic
echo "🔐 Configurando permissões..."
SINK_SERVICE_ACCOUNT=$(gcloud logging sinks describe $SINK_NAME \
    --project=$PROJECT_ID \
    --format='value(writerIdentity)')

echo "   Service Account do Sink: $SINK_SERVICE_ACCOUNT"

gcloud pubsub topics add-iam-policy-binding $TOPIC_NAME \
    --project=$PROJECT_ID \
    --member="$SINK_SERVICE_ACCOUNT" \
    --role="roles/pubsub.publisher" \
    2>/dev/null || echo "   Permissão já configurada"

# 4. Criar Subscription (Push)
echo "📩 Criando Push Subscription..."
gcloud pubsub subscriptions create $SUBSCRIPTION_NAME \
    --project=$PROJECT_ID \
    --topic=$TOPIC_NAME \
    --push-endpoint=$ENRICH_ENDPOINT \
    --ack-deadline=60 \
    --message-retention-duration=1d \
    2>/dev/null || echo "   Subscription já existe"

echo ""
echo "✅ Configuração completa!"
echo ""
echo "📋 Resumo:"
echo "   - Logs com 'Task criada' ou 'Tarefa criada' serão capturados"
echo "   - Enviados para Pub/Sub topic: $TOPIC_NAME"
echo "   - Push subscription: $SUBSCRIPTION_NAME"
echo "   - Push para: $ENRICH_ENDPOINT"
echo ""
echo "🔍 Para verificar os logs capturados:"
echo "   gcloud logging read 'resource.type=\"cloud_run_revision\" AND textPayload=~\"Task criada\"' --project=$PROJECT_ID --limit=5"
echo ""
echo "🧪 Para testar manualmente:"
echo "   ./test-enrich-local.sh <task_id>"
echo ""
echo "📊 Para monitorar mensagens do Pub/Sub:"
echo "   gcloud pubsub subscriptions pull $SUBSCRIPTION_NAME --project=$PROJECT_ID --limit=5"

