#!/bin/bash

# Script para testar o webhook no ambiente de teste GCP

# Cores para output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# URL do serviço de teste (será preenchida após o deploy)
if [ -z "$1" ]; then
    echo "Uso: ./test-webhook-gcp.sh <SERVICE_URL>"
    echo "Exemplo: ./test-webhook-gcp.sh https://chatguru-middleware-test-xxx.run.app"
    exit 1
fi

SERVICE_URL=$1

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Testando Webhook no GCP Test${NC}"
echo -e "${GREEN}========================================${NC}"

# 1. Teste de Health Check
echo -e "\n${YELLOW}1. Testando Health Check...${NC}"
curl -s ${SERVICE_URL}/health | jq .

# 2. Teste de Status
echo -e "\n${YELLOW}2. Testando Status...${NC}"
curl -s ${SERVICE_URL}/status | jq .

# 3. Teste de Webhook com payload ChatGuru
echo -e "\n${YELLOW}3. Enviando webhook de teste...${NC}"

PAYLOAD='{
  "event_type": "message.created",
  "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
  "id": "test-'$(date +%s)'",
  "data": {
    "lead_name": "[TESTE GCP] Cliente Teste",
    "phone": "5511999998888",
    "message": "Teste de integração no ambiente GCP Test",
    "chat_id": "test-chat-'$(date +%s)'",
    "annotation": "Pedido de teste no ambiente GCP"
  }
}'

echo "Payload:"
echo $PAYLOAD | jq .

echo -e "\n${YELLOW}Enviando requisição...${NC}"
RESPONSE=$(curl -s -X POST ${SERVICE_URL}/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD")

echo -e "\n${YELLOW}Resposta:${NC}"
echo $RESPONSE | jq .

# 4. Verificar logs (opcional)
echo -e "\n${YELLOW}Para ver os logs do serviço, execute:${NC}"
echo "gcloud run logs read --service chatguru-middleware-test --region southamerica-east1 --tail 50"

echo -e "\n${GREEN}Teste concluído!${NC}"