#!/bin/bash

# Deploy rápido com todas as configurações corretas
# Este script assume que você já está autenticado no gcloud

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configurações
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-clickup-middleware"

# Configurações das APIs
CLICKUP_API_TOKEN="pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657"
CLICKUP_LIST_ID="901300373349"
CHATGURU_API_TOKEN="TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK"
CHATGURU_API_ENDPOINT="https://s15.chatguru.app/api/v1"
CHATGURU_ACCOUNT_ID="625584ce6fdcb7bda7d94aa8"
AI_ENABLED="true"

# OpenAI fallback (recuperada do sistema legado)
OPENAI_API_KEY="${OPENAI_API_KEY:-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}"

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}    Deploy Rápido - ChatGuru-ClickUp    ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# 1. Verificar se está no diretório correto
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}✗${NC} Execute este script do diretório chatguru-clickup-middleware/"
    exit 1
fi

echo -e "${BLUE}[INFO]${NC} Iniciando deploy rápido..."
echo -e "${BLUE}[INFO]${NC} Projeto: ${PROJECT_ID}"
echo -e "${BLUE}[INFO]${NC} Região: ${REGION}"
echo -e "${BLUE}[INFO]${NC} Serviço: ${SERVICE_NAME}"
echo ""

# 2. Configurar projeto
gcloud config set project ${PROJECT_ID} --quiet

# 3. Compilar localmente primeiro (opcional, mas ajuda a detectar erros)
echo -e "${YELLOW}[1/3]${NC} Compilando aplicação localmente..."
if cargo check --release 2>&1 | grep -E "error|Error"; then
    echo -e "${RED}✗${NC} Erro na compilação! Corrija antes de fazer deploy."
    exit 1
else
    echo -e "${GREEN}✓${NC} Compilação OK"
fi

# 4. Remover ENV PORT do Dockerfile se existir
if grep -q "ENV PORT=" Dockerfile; then
    echo -e "${YELLOW}[INFO]${NC} Removendo ENV PORT do Dockerfile..."
    sed -i.bak '/ENV PORT=/d' Dockerfile
fi

# 5. Deploy direto do código
echo ""
echo -e "${YELLOW}[2/3]${NC} Fazendo deploy no Cloud Run..."
echo -e "${BLUE}[INFO]${NC} Isso pode levar 3-5 minutos..."

if gcloud run deploy ${SERVICE_NAME} \
    --source . \
    --region ${REGION} \
    --allow-unauthenticated \
    --project ${PROJECT_ID} \
    --memory 512Mi \
    --cpu 1 \
    --timeout 300 \
    --min-instances 0 \
    --max-instances 10 \
    --set-env-vars "\
CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN},\
CLICKUP_LIST_ID=${CLICKUP_LIST_ID},\
CHATGURU__API_TOKEN=${CHATGURU_API_TOKEN},\
CHATGURU__API_ENDPOINT=${CHATGURU_API_ENDPOINT},\
CHATGURU__ACCOUNT_ID=${CHATGURU_ACCOUNT_ID},\
AI__ENABLED=${AI_ENABLED},\
GCP__PROJECT_ID=${PROJECT_ID},\
OPENAI_API_KEY=${OPENAI_API_KEY},\
RUST_LOG=info" \
    --quiet; then
    
    echo -e "${GREEN}✓${NC} Deploy concluído com sucesso!"
    
    # Restaurar Dockerfile
    if [ -f "Dockerfile.bak" ]; then
        mv Dockerfile.bak Dockerfile
    fi
else
    echo -e "${RED}✗${NC} Deploy falhou!"
    # Restaurar Dockerfile
    if [ -f "Dockerfile.bak" ]; then
        mv Dockerfile.bak Dockerfile
    fi
    exit 1
fi

# 6. Obter URL do serviço
echo ""
echo -e "${YELLOW}[3/3]${NC} Obtendo informações do serviço..."

SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --project ${PROJECT_ID} \
    --format 'value(status.url)')

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}         Deploy Completo!               ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${GREEN}📍 URL do Serviço:${NC}"
echo "   ${SERVICE_URL}"
echo ""
echo -e "${GREEN}🔗 Endpoints:${NC}"
echo "   Health: ${SERVICE_URL}/health"
echo "   Status: ${SERVICE_URL}/status"
echo "   Webhook: ${SERVICE_URL}/webhooks/chatguru"
echo ""

# 7. Testar health check
echo -e "${BLUE}[INFO]${NC} Testando serviço..."
sleep 5  # Aguardar um pouco para o serviço iniciar

if curl -s "${SERVICE_URL}/health" 2>/dev/null | grep -q "healthy"; then
    echo -e "${GREEN}✓${NC} Serviço está respondendo!"
    
    # Verificar status detalhado
    echo ""
    echo -e "${BLUE}[INFO]${NC} Status do serviço:"
    STATUS=$(curl -s "${SERVICE_URL}/status" 2>/dev/null)
    if command -v jq &> /dev/null; then
        echo "   ClickUp: $(echo $STATUS | jq -r '.clickup_connected')"
        echo "   AI: $(echo $STATUS | jq -r '.ai_enabled')"
        echo "   ChatGuru: $(echo $STATUS | jq -r '.chatguru_configured')"
    else
        echo "$STATUS"
    fi
else
    echo -e "${YELLOW}!${NC} Serviço ainda está iniciando..."
fi

echo ""
echo -e "${CYAN}Próximos passos:${NC}"
echo "1. Verificar configuração: ./verify-config.sh"
echo "2. Executar teste completo: node test-complete-flow.js"
echo "3. Ver logs: gcloud run logs tail ${SERVICE_NAME} --region ${REGION}"
echo ""