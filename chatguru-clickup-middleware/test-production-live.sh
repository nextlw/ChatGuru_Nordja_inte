#!/bin/bash
# Script para testar produção e monitorar logs em tempo real
#
# Uso: ./test-production-live.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PRODUCTION_URL="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"
SERVICE_NAME="chatguru-clickup-middleware"
REGION="southamerica-east1"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       TESTE EM PRODUÇÃO - ENVIO + MONITORAMENTO LOGS          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Gerar ID único para este teste
TEST_ID="TEST-$(date +%s)"

echo -e "${BLUE}📋 Informações do Teste:${NC}"
echo -e "   Test ID: ${YELLOW}${TEST_ID}${NC}"
echo -e "   Produção: ${YELLOW}${PRODUCTION_URL}${NC}"
echo -e "   Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Criar payload
PAYLOAD=$(cat <<EOF
{
  "chat_id": "${TEST_ID}@c.us",
  "celular": "5511999999999",
  "sender_name": "Teste Automático",
  "texto_mensagem": "[${TEST_ID}] Preciso criar um sistema de gerenciamento de estoque completo. Deve incluir dashboard analytics, controle de entrada/saída, alertas de estoque baixo e relatórios PDF exportáveis. Sistema deve ser web-based com backend em Node.js e frontend em React.",
  "message_type": "text",
  "campos_personalizados": {
    "Info_1": "Nexcode",
    "Info_2": "William Duarte"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

echo -e "${BLUE}📤 Payload a ser enviado:${NC}"
echo "$PAYLOAD" | jq '.' | sed 's/^/   /'
echo ""

# Enviar para webhook
echo -e "${BLUE}🚀 Enviando para webhook de produção...${NC}"
START_TIME=$(date +%s)

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
  "${PRODUCTION_URL}/webhooks/chatguru" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
BODY=$(echo "$RESPONSE" | sed '$d')

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "${GREEN}✅ Webhook respondeu em ${DURATION}s${NC}"
echo -e "   HTTP Status: ${HTTP_CODE}"
echo -e "   Response:"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY" | sed 's/^/      /'
echo ""

if [ "$HTTP_CODE" != "200" ]; then
    echo -e "${YELLOW}⚠️  Webhook retornou status ${HTTP_CODE}${NC}"
fi

# Aguardar um pouco antes de começar a monitorar logs
echo -e "${BLUE}⏳ Aguardando 3 segundos antes de iniciar monitoramento de logs...${NC}"
sleep 3
echo ""

# Monitorar logs em tempo real
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              📋 LOGS EM TEMPO REAL (Cloud Run)                 ║${NC}"
echo -e "${GREEN}╠════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${GREEN}║  Filtrando por: ${TEST_ID}                                      ║${NC}"
echo -e "${GREEN}║  Pressione Ctrl+C para parar                                   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Tail dos logs (filtrando pelo TEST_ID)
gcloud beta run services logs tail ${SERVICE_NAME} \
  --region=${REGION} \
  --format="value(textPayload)" \
  | grep --line-buffered -E "${TEST_ID}|Task criada|Classificando|Mensagem recebida|ClickUp|OpenAI" \
  || echo -e "${YELLOW}ℹ️  Nenhum log relevante ainda (aguardando processamento)...${NC}"

echo ""
echo -e "${GREEN}✅ Monitoramento finalizado${NC}"
