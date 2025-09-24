#!/bin/bash

# =================================================
# Script de Deploy SIMPLIFICADO para Teste LOCAL
# Usa Artifact Registry ao invés de GCR
# =================================================

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configurações
PROJECT_ID="buzzlightear"
PROJECT_NUMBER="707444002434"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-middleware-test"
IMAGE_NAME="chatguru-clickup-middleware-test"
REGISTRY_NAME="chatguru-integrations"

# URL completa do Artifact Registry
AR_REPOSITORY="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REGISTRY_NAME}"

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Deploy Simplificado para Teste${NC}"
echo -e "${GREEN}========================================${NC}"

# 1. Verificar autenticação
echo -e "\n${YELLOW}1. Verificando autenticação...${NC}"
ACTIVE_ACCOUNT=$(gcloud auth list --filter=status:ACTIVE --format="value(account)")
if [ -z "$ACTIVE_ACCOUNT" ]; then
    echo -e "${RED}Erro: Não autenticado. Execute: gcloud auth login${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Autenticado como: $ACTIVE_ACCOUNT${NC}"

# 2. Configurar projeto
echo -e "\n${YELLOW}2. Configurando projeto...${NC}"
gcloud config set project ${PROJECT_ID}
echo -e "${GREEN}✓ Projeto: ${PROJECT_ID}${NC}"

# 3. Configurar Docker para Artifact Registry
echo -e "\n${YELLOW}3. Configurando Docker para Artifact Registry...${NC}"
gcloud auth configure-docker ${REGION}-docker.pkg.dev --quiet
echo -e "${GREEN}✓ Docker configurado para ${REGION}-docker.pkg.dev${NC}"

# 4. Build da imagem
echo -e "\n${YELLOW}4. Construindo imagem Docker...${NC}"
docker build -f Dockerfile.test -t ${IMAGE_NAME}:latest .
echo -e "${GREEN}✓ Imagem construída${NC}"

# 5. Tag para Artifact Registry
echo -e "\n${YELLOW}5. Criando tag para Artifact Registry...${NC}"
docker tag ${IMAGE_NAME}:latest ${AR_REPOSITORY}/${IMAGE_NAME}:latest
docker tag ${IMAGE_NAME}:latest ${AR_REPOSITORY}/${IMAGE_NAME}:$(date +%Y%m%d-%H%M%S)
echo -e "${GREEN}✓ Tags criadas${NC}"

# 6. Push para Artifact Registry
echo -e "\n${YELLOW}6. Enviando imagem para Artifact Registry...${NC}"
docker push ${AR_REPOSITORY}/${IMAGE_NAME}:latest
echo -e "${GREEN}✓ Imagem enviada${NC}"

# 7. Deploy no Cloud Run
echo -e "\n${YELLOW}7. Fazendo deploy no Cloud Run...${NC}"

# Verificar se precisa das credenciais do ClickUp e ChatGuru
if [ -z "$CLICKUP_API_TOKEN" ]; then
    echo -e "${YELLOW}Usando credenciais do arquivo .env.test${NC}"
    source .env.test
fi

gcloud run deploy ${SERVICE_NAME} \
    --image ${AR_REPOSITORY}/${IMAGE_NAME}:latest \
    --region ${REGION} \
    --platform managed \
    --allow-unauthenticated \
    --port 8080 \
    --cpu 1 \
    --memory 512Mi \
    --min-instances 0 \
    --max-instances 5 \
    --timeout 60 \
    --concurrency 80 \
    --service-account ${PROJECT_ID}@appspot.gserviceaccount.com \
    --set-env-vars "RUST_ENV=test,IS_TEST_ENVIRONMENT=true,RUST_LOG=info" \
    --set-env-vars "CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN}" \
    --set-env-vars "CLICKUP_LIST_ID=${CLICKUP_LIST_ID}" \
    --set-env-vars "CHATGURU_API_TOKEN=${CHATGURU_API_TOKEN}" \
    --set-env-vars "CHATGURU_ACCOUNT_ID=${CHATGURU_ACCOUNT_ID}" \
    --set-env-vars "USE_CLOUD_TASKS=false" \
    --quiet

echo -e "${GREEN}✓ Deploy realizado${NC}"

# 8. Obter URL do serviço
echo -e "\n${YELLOW}8. Obtendo informações do serviço...${NC}"
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --format 'value(status.url)')

# 9. Testar health check
echo -e "\n${YELLOW}9. Testando serviço...${NC}"
sleep 5  # Aguardar serviço inicializar

if curl -sf ${SERVICE_URL}/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Serviço respondendo corretamente!${NC}"
    HEALTH_RESPONSE=$(curl -s ${SERVICE_URL}/health)
    echo -e "Health: $HEALTH_RESPONSE"
else
    echo -e "${YELLOW}⚠ Serviço pode estar iniciando, tente novamente em alguns segundos${NC}"
fi

# 10. Salvar informações
cat > deploy-test-info.txt <<EOF
Deploy de Teste - $(date)
========================
Projeto: ${PROJECT_ID} (${PROJECT_NUMBER})
Serviço: ${SERVICE_NAME}
Região: ${REGION}
URL: ${SERVICE_URL}
Imagem: ${AR_REPOSITORY}/${IMAGE_NAME}:latest
Service Account: ${PROJECT_ID}@appspot.gserviceaccount.com

Endpoints:
- Health: ${SERVICE_URL}/health
- Ready: ${SERVICE_URL}/ready
- Status: ${SERVICE_URL}/status
- Webhook: ${SERVICE_URL}/webhooks/chatguru

Comandos úteis:
- Ver logs: gcloud run logs tail --service ${SERVICE_NAME} --region ${REGION}
- Deletar: gcloud run services delete ${SERVICE_NAME} --region ${REGION}
EOF

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}       Deploy Concluído!${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "URL: ${YELLOW}${SERVICE_URL}${NC}"
echo -e "Logs: ${YELLOW}gcloud run logs tail --service ${SERVICE_NAME} --region ${REGION}${NC}"
echo -e "\nInformações salvas em: ${YELLOW}deploy-test-info.txt${NC}"