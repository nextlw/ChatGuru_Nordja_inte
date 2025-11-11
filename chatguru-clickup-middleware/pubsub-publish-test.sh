#!/bin/bash
# Script para publicar mensagem de teste no Pub/Sub Emulator
#
# Uso: ./pubsub-publish-test.sh ["mensagem opcional"]

# Configuração
EMULATOR_HOST="localhost:8085"
PROJECT_ID="local-dev"
TOPIC="chatguru-webhook-events"

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Mensagem padrão ou customizada
if [ -n "$1" ]; then
    MESSAGE="$1"
else
    MESSAGE='{"tipo": "teste", "timestamp": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'", "mensagem": "Teste do emulator local"}'
fi

echo -e "${BLUE}📤 Publicando mensagem de teste...${NC}"
echo ""

# Verificar se o emulador está rodando
if ! curl -s "http://${EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Emulador detectado${NC}"
echo -e "${BLUE}📝 Tópico: ${YELLOW}${TOPIC}${NC}"
echo -e "${BLUE}💬 Mensagem:${NC}"
echo -e "   ${YELLOW}${MESSAGE}${NC}"
echo ""

# Codificar mensagem em base64
MESSAGE_BASE64=$(echo -n "$MESSAGE" | base64)

# URL da API REST do Pub/Sub Emulator
PUBLISH_URL="http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics/${TOPIC}:publish"

# Publicar mensagem
RESPONSE=$(curl -s -X POST "$PUBLISH_URL" \
  -H "Content-Type: application/json" \
  -d "{
    \"messages\": [
      {
        \"data\": \"${MESSAGE_BASE64}\",
        \"attributes\": {
          \"source\": \"test-script\",
          \"environment\": \"local\"
        }
      }
    ]
  }")

# Verificar se há erro
if echo "$RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
    echo -e "${RED}❌ Erro ao publicar:${NC}"
    echo -e "   $ERROR_MSG"
    exit 1
fi

# Obter ID da mensagem publicada
MESSAGE_ID=$(echo "$RESPONSE" | jq -r '.messageIds[0]')

if [ -n "$MESSAGE_ID" ] && [ "$MESSAGE_ID" != "null" ]; then
    echo -e "${GREEN}✅ Mensagem publicada com sucesso!${NC}"
    echo -e "   Message ID: ${YELLOW}${MESSAGE_ID}${NC}"
else
    echo -e "${YELLOW}⚠️  Mensagem publicada mas sem ID retornado${NC}"
fi

echo ""
echo -e "${BLUE}💡 Para verificar a mensagem:${NC}"
echo -e "   ${YELLOW}./pubsub-pull-direct.sh${NC}"
echo -e "   ou"
echo -e "   ${YELLOW}./pubsub-pull.sh${NC}"
