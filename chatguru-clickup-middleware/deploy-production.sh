#!/bin/bash
# Script de deploy para Cloud Run com variáveis de ambiente
# Uso: ./deploy-production.sh

set -e

# Cores para output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configurações
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-clickup-middleware"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}:latest"

# ClickUp Credentials (do config/default.toml)
CLICKUP_CLIENT_ID="2Y4X8LGHAKEVKYLTC91YS4TRJABA9SOU"
CLICKUP_CLIENT_SECRET="N8ZKCJMZQ86439FIVHRWZTAQ09VY7ZW86NSV0TB71AIBUISY0RMISXME23XAWBHG"
CLICKUP_ACCESS_TOKEN="106092691_dfa687f9f3f17257583e02da91097804f1e01d15e89617a06db2ba3e3f37e62e"
CLICKUP_REDIRECT_URI="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/auth/clickup/callback"
CLICKUP_API_BASE_URL="https://api.clickup.com/api/v2"

echo -e "${GREEN}🚀 Iniciando deploy para Cloud Run${NC}"
echo -e "${YELLOW}Projeto: ${PROJECT_ID}${NC}"
echo -e "${YELLOW}Região: ${REGION}${NC}"
echo -e "${YELLOW}Serviço: ${SERVICE_NAME}${NC}"
echo ""

# 1. Build da imagem via Cloud Build
echo -e "${GREEN}📦 Step 1: Building Docker image via Cloud Build...${NC}"
gcloud builds submit chatguru-clickup-middleware \
  --tag ${IMAGE_NAME} \
  --project ${PROJECT_ID}

echo -e "${GREEN}✅ Build concluído!${NC}"
echo ""

# 2. Deploy para Cloud Run
echo -e "${GREEN}🚢 Step 2: Deploying to Cloud Run...${NC}"
gcloud run deploy ${SERVICE_NAME} \
  --image ${IMAGE_NAME} \
  --platform managed \
  --region ${REGION} \
  --project ${PROJECT_ID} \
  --allow-unauthenticated \
  --memory 1Gi \
  --cpu 2 \
  --timeout 300 \
  --min-instances 1 \
  --max-instances 10 \
  --concurrency 100 \
  --set-env-vars "RUST_LOG=info,RUST_ENV=production,GCP_PROJECT_ID=${PROJECT_ID},CLICKUP_CLIENT_ID=${CLICKUP_CLIENT_ID},CLICKUP_CLIENT_SECRET=${CLICKUP_CLIENT_SECRET},CLICKUP_ACCESS_TOKEN=${CLICKUP_ACCESS_TOKEN},CLICKUP_REDIRECT_URI=${CLICKUP_REDIRECT_URI},CLICKUP_API_BASE_URL=${CLICKUP_API_BASE_URL}" \
  --set-secrets "OPENAI_API_KEY=openai-api-key:latest,CHATGURU_API_TOKEN=chatguru-api-token:latest"

echo -e "${GREEN}✅ Deploy concluído!${NC}"
echo ""

# 3. Obter URL do serviço
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
  --region=${REGION} \
  --project=${PROJECT_ID} \
  --format='value(status.url)')

echo -e "${GREEN}📍 Service URL: ${SERVICE_URL}${NC}"
echo ""

# 4. Testar health check
echo -e "${GREEN}🏥 Step 3: Testing health check...${NC}"
for i in {1..5}; do
  if curl -f "${SERVICE_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Health check passed!${NC}"
    break
  else
    echo -e "${YELLOW}⏳ Waiting for service to be ready... (attempt $i/5)${NC}"
    sleep 10
  fi
done

echo ""
echo -e "${GREEN}✨ Deploy finalizado com sucesso!${NC}"
echo -e "${GREEN}🌐 Service URL: ${SERVICE_URL}${NC}"
echo -e "${GREEN}📊 Status: ${SERVICE_URL}/status${NC}"
echo -e "${GREEN}🔐 OAuth: ${SERVICE_URL}/auth/clickup${NC}"
echo ""
echo -e "${YELLOW}📝 Variáveis de ambiente configuradas:${NC}"
echo -e "  - CLICKUP_ACCESS_TOKEN: ${CLICKUP_ACCESS_TOKEN:0:20}..."
echo -e "  - CLICKUP_CLIENT_ID: ${CLICKUP_CLIENT_ID}"
echo -e "  - CLICKUP_REDIRECT_URI: ${CLICKUP_REDIRECT_URI}"
echo ""
echo -e "${YELLOW}🔐 Secrets do Secret Manager usados:${NC}"
echo -e "  - OPENAI_API_KEY (openai-api-key:latest)"
echo -e "  - CHATGURU_API_TOKEN (chatguru-api-token:latest)"
