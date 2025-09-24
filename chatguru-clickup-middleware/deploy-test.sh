#!/bin/bash

# =================================================
# Script de Deploy para Ambiente de TESTE
# Google Cloud Run - Região: southamerica-east1
# =================================================

set -e  # Para em caso de erro

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configurações
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-middleware-test"
IMAGE_NAME="chatguru-clickup-middleware-test"
REGISTRY="gcr.io"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Deploy para Ambiente de TESTE${NC}"
echo -e "${GREEN}========================================${NC}"

# 1. Verificar se está logado no gcloud
echo -e "\n${YELLOW}1. Verificando autenticação gcloud...${NC}"
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    echo -e "${RED}Erro: Não está autenticado no gcloud${NC}"
    echo "Execute: gcloud auth login"
    exit 1
fi

# 2. Configurar projeto
echo -e "\n${YELLOW}2. Configurando projeto GCP...${NC}"
gcloud config set project ${PROJECT_ID}
echo -e "${GREEN}Projeto configurado: ${PROJECT_ID}${NC}"

# 3. Habilitar APIs necessárias
echo -e "\n${YELLOW}3. Habilitando APIs necessárias...${NC}"
gcloud services enable \
    run.googleapis.com \
    containerregistry.googleapis.com \
    cloudbuild.googleapis.com \
    artifactregistry.googleapis.com \
    --quiet

# 4. Build da imagem Docker
echo -e "\n${YELLOW}4. Construindo imagem Docker...${NC}"
docker build -f Dockerfile.test -t ${IMAGE_NAME}:latest .

# 5. Tag da imagem para GCR
echo -e "\n${YELLOW}5. Criando tag para Google Container Registry...${NC}"
docker tag ${IMAGE_NAME}:latest ${REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}:latest
docker tag ${IMAGE_NAME}:latest ${REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}:$(date +%Y%m%d-%H%M%S)

# 6. Push para GCR
echo -e "\n${YELLOW}6. Enviando imagem para GCR...${NC}"
docker push ${REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}:latest

# 7. Deploy no Cloud Run
echo -e "\n${YELLOW}7. Fazendo deploy no Cloud Run...${NC}"
gcloud run deploy ${SERVICE_NAME} \
    --image ${REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}:latest \
    --region ${REGION} \
    --platform managed \
    --allow-unauthenticated \
    --port 8080 \
    --cpu 1 \
    --memory 512Mi \
    --min-instances 0 \
    --max-instances 10 \
    --timeout 300 \
    --concurrency 80 \
    --service-account ${PROJECT_ID}@appspot.gserviceaccount.com \
    --set-env-vars "RUST_ENV=test" \
    --set-env-vars "IS_TEST_ENVIRONMENT=true" \
    --set-env-vars "RUST_LOG=debug" \
    --set-env-vars "PORT=8080" \
    --set-env-vars "USE_CLOUD_TASKS=false" \
    --set-env-vars "CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN}" \
    --set-env-vars "CLICKUP_LIST_ID=901300373349" \
    --set-env-vars "CHATGURU_API_TOKEN=${CHATGURU_API_TOKEN}" \
    --set-env-vars "CHATGURU_ACCOUNT_ID=${CHATGURU_ACCOUNT_ID}" \
    --quiet

# 8. Obter URL do serviço
echo -e "\n${YELLOW}8. Obtendo URL do serviço...${NC}"
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --platform managed \
    --format 'value(status.url)')

# 9. Testar health check
echo -e "\n${YELLOW}9. Testando health check...${NC}"
if curl -s ${SERVICE_URL}/health | grep -q "healthy"; then
    echo -e "${GREEN}✓ Health check passou com sucesso!${NC}"
else
    echo -e "${RED}✗ Health check falhou${NC}"
fi

# 10. Resumo do deploy
echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}  Deploy Concluído com Sucesso!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "Serviço: ${SERVICE_NAME}"
echo -e "Região: ${REGION}"
echo -e "URL: ${SERVICE_URL}"
echo -e "\n${YELLOW}Endpoints disponíveis:${NC}"
echo -e "  - Health: ${SERVICE_URL}/health"
echo -e "  - Ready: ${SERVICE_URL}/ready"
echo -e "  - Status: ${SERVICE_URL}/status"
echo -e "  - Webhook: ${SERVICE_URL}/webhooks/chatguru"
echo -e "\n${YELLOW}Comandos úteis:${NC}"
echo -e "  Ver logs: gcloud run logs read --service ${SERVICE_NAME} --region ${REGION}"
echo -e "  Ver métricas: gcloud run services describe ${SERVICE_NAME} --region ${REGION}"
echo -e "  Deletar serviço: gcloud run services delete ${SERVICE_NAME} --region ${REGION}"

# 11. Criar arquivo com informações do deploy
echo -e "\n${YELLOW}Salvando informações do deploy...${NC}"
cat > deploy-test-info.txt <<EOF
Deploy de Teste - $(date)
========================
Projeto: ${PROJECT_ID}
Serviço: ${SERVICE_NAME}
Região: ${REGION}
URL: ${SERVICE_URL}
Imagem: ${REGISTRY}/${PROJECT_ID}/${IMAGE_NAME}:latest

Comandos úteis:
- Logs: gcloud run logs read --service ${SERVICE_NAME} --region ${REGION} --tail 50
- Logs contínuos: gcloud run logs tail --service ${SERVICE_NAME} --region ${REGION}
- Deletar: gcloud run services delete ${SERVICE_NAME} --region ${REGION}
- Atualizar env vars: gcloud run services update ${SERVICE_NAME} --update-env-vars KEY=VALUE --region ${REGION}
EOF

echo -e "${GREEN}Informações salvas em deploy-test-info.txt${NC}"