#!/bin/bash

# Script para verificar configurações do serviço deployado

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configurações
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-clickup-middleware"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Verificando Configurações do Serviço  ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# 1. Verificar se o serviço existe
echo -e "${BLUE}[1/5]${NC} Verificando serviço Cloud Run..."
if gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --project ${PROJECT_ID} &> /dev/null; then
    echo -e "${GREEN}✓${NC} Serviço encontrado"
    
    # Obter URL do serviço
    SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
        --region ${REGION} \
        --project ${PROJECT_ID} \
        --format 'value(status.url)')
    echo -e "   URL: ${SERVICE_URL}"
else
    echo -e "${RED}✗${NC} Serviço não encontrado!"
    exit 1
fi

# 2. Verificar variáveis de ambiente críticas
echo ""
echo -e "${BLUE}[2/5]${NC} Verificando variáveis de ambiente..."

REQUIRED_VARS=(
    "CLICKUP_API_TOKEN"
    "CLICKUP_LIST_ID"
    "AI__ENABLED"
    "CHATGURU__API_TOKEN"
    "GCP__PROJECT_ID"
    "VERTEX_AI_REGION"
)

ENV_VARS=$(gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --project ${PROJECT_ID} \
    --format 'export(spec.template.spec.containers[0].env[].name)')

for var in "${REQUIRED_VARS[@]}"; do
    if echo "$ENV_VARS" | grep -q "$var"; then
        echo -e "${GREEN}✓${NC} $var está configurada"
    else
        echo -e "${RED}✗${NC} $var NÃO está configurada"
    fi
done

# 3. Verificar APIs do Google Cloud
echo ""
echo -e "${BLUE}[3/5]${NC} Verificando APIs habilitadas..."

APIS_NEEDED=(
    "run.googleapis.com"
    "aiplatform.googleapis.com"
    "cloudbuild.googleapis.com"
)

for api in "${APIS_NEEDED[@]}"; do
    if gcloud services list --enabled --filter="name:${api}" --format="value(name)" | grep -q "${api}"; then
        echo -e "${GREEN}✓${NC} API $api habilitada"
    else
        echo -e "${YELLOW}!${NC} API $api NÃO está habilitada"
    fi
done

# 4. Verificar permissões da Service Account para Vertex AI
echo ""
echo -e "${BLUE}[4/5]${NC} Verificando permissões da Service Account..."

SERVICE_ACCOUNT="707444002434-compute@developer.gserviceaccount.com"
REQUIRED_ROLE="roles/aiplatform.user"

if gcloud projects get-iam-policy ${PROJECT_ID} \
    --flatten="bindings[].members" \
    --format="table(bindings.role)" \
    --filter="bindings.members:${SERVICE_ACCOUNT}" | grep -q "${REQUIRED_ROLE}"; then
    echo -e "${GREEN}✓${NC} Service Account tem permissão ${REQUIRED_ROLE}"
else
    echo -e "${RED}✗${NC} Service Account NÃO tem permissão ${REQUIRED_ROLE}"
fi

# 5. Testar endpoints do serviço
echo ""
echo -e "${BLUE}[5/5]${NC} Testando endpoints do serviço..."

# Health check
echo -n "   Health Check: "
if curl -s "${SERVICE_URL}/health" 2>/dev/null | grep -q "healthy"; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ Falhou${NC}"
fi

# Status endpoint
echo -n "   Status: "
STATUS_RESPONSE=$(curl -s "${SERVICE_URL}/status" 2>/dev/null)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ OK${NC}"
    
    # Parse do JSON para verificar configurações
    if command -v jq &> /dev/null; then
        echo ""
        echo -e "${CYAN}   Configurações detectadas:${NC}"
        echo "   - ClickUp conectado: $(echo $STATUS_RESPONSE | jq -r '.clickup_connected // "N/A"')"
        echo "   - AI habilitada: $(echo $STATUS_RESPONSE | jq -r '.ai_enabled // "N/A"')"
        echo "   - Versão: $(echo $STATUS_RESPONSE | jq -r '.version // "N/A"')"
    fi
else
    echo -e "${RED}✗ Falhou${NC}"
fi

# Resumo
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}             RESUMO                     ${NC}"
echo -e "${CYAN}========================================${NC}"

echo ""
echo -e "${YELLOW}Comandos úteis:${NC}"
echo ""
echo "Ver logs em tempo real:"
echo -e "${BLUE}gcloud run logs tail ${SERVICE_NAME} --region ${REGION}${NC}"
echo ""
echo "Ver variáveis de ambiente completas:"
echo -e "${BLUE}gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format='export(spec.template.spec.containers[0].env)'${NC}"
echo ""
echo "Testar webhook manualmente:"
echo -e "${BLUE}curl -X POST ${SERVICE_URL}/webhooks/chatguru \\
  -H 'Content-Type: application/json' \\
  -d '{\"nome\":\"Teste\",\"texto_mensagem\":\"Preciso de 10 parafusos\"}'${NC}"
echo ""
echo "Executar teste completo:"
echo -e "${BLUE}node test-complete-flow.js${NC}"
echo ""