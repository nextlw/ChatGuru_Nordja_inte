#!/bin/bash

# Script para deploy do ChatGuru-ClickUp Middleware no Cloud Run

echo "🚀 Iniciando deploy do ChatGuru-ClickUp Middleware no Cloud Run..."

# Configurações
PROJECT_ID="sigma-access-249521"
SERVICE_NAME="chatguru-clickup-middleware"
REGION="us-central1"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

# Verificar se já existe um serviço
echo "📋 Verificando se o serviço já existe..."
if gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format="value(status.url)" 2>/dev/null; then
    echo "✅ Serviço já existe! URL acima."
    echo "🔄 Atualizando o serviço..."
else
    echo "📦 Serviço não existe. Criando novo..."
fi

# Build da imagem Docker
echo "🔨 Construindo imagem Docker..."
docker build -t ${IMAGE_NAME} .

if [ $? -ne 0 ]; then
    echo "❌ Erro ao construir imagem Docker"
    exit 1
fi

# Push da imagem para o GCR
echo "📤 Enviando imagem para Google Container Registry..."
docker push ${IMAGE_NAME}

if [ $? -ne 0 ]; then
    echo "❌ Erro ao enviar imagem para GCR"
    exit 1
fi

# Deploy no Cloud Run
echo "🚀 Fazendo deploy no Cloud Run..."
gcloud run deploy ${SERVICE_NAME} \
    --image ${IMAGE_NAME} \
    --platform managed \
    --region ${REGION} \
    --allow-unauthenticated \
    --port 8080 \
    --memory 512Mi \
    --cpu 1 \
    --min-instances 0 \
    --max-instances 10 \
    --timeout 60 \
    --set-env-vars "RUST_LOG=info" \
    --set-env-vars "CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN}" \
    --set-env-vars "CLICKUP_LIST_ID=${CLICKUP_LIST_ID}"

if [ $? -eq 0 ]; then
    echo "✅ Deploy concluído com sucesso!"
    
    # Obter a URL do serviço
    SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format="value(status.url)")
    echo "🌐 URL do serviço: ${SERVICE_URL}"
    echo ""
    echo "📝 Próximos passos:"
    echo "1. Teste o serviço: curl ${SERVICE_URL}/health"
    echo "2. Configure o webhook no ChatGuru para: ${SERVICE_URL}/webhooks/chatguru"
    echo "3. Use o script: node update-dialog-webhook.js"
else
    echo "❌ Erro no deploy"
    exit 1
fi