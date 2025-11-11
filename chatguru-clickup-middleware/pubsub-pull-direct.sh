#!/bin/bash
# Script para fazer pull de mensagens do Pub/Sub Emulator usando API REST
# Esta versão usa curl diretamente e não depende do gcloud
#
# Uso: ./pubsub-pull-direct.sh [quantidade]

# Configuração
EMULATOR_HOST="localhost:8085"
PROJECT_ID="local-dev"
SUBSCRIPTION="chatguru-worker-sub"
LIMIT="${1:-10}"

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}📬 Fazendo pull de até ${LIMIT} mensagens via API REST...${NC}"
echo ""

# Verificar se o emulador está rodando
if ! curl -s "http://${EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Emulador detectado em ${EMULATOR_HOST}${NC}"

# URL da API REST do Pub/Sub Emulator
PULL_URL="http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/${SUBSCRIPTION}:pull"

# Fazer pull de mensagens
echo -e "${BLUE}🔄 Consultando subscription '${SUBSCRIPTION}'...${NC}"
echo ""

RESPONSE=$(curl -s -X POST "$PULL_URL" \
  -H "Content-Type: application/json" \
  -d "{\"maxMessages\": ${LIMIT}}")

# Verificar se há erro
if echo "$RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
    ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
    echo -e "${RED}❌ Erro ao fazer pull:${NC}"
    echo -e "   $ERROR_MSG"
    exit 1
fi

# Contar mensagens
MSG_COUNT=$(echo "$RESPONSE" | jq '.receivedMessages | length' 2>/dev/null || echo "0")

if [ "$MSG_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ ${MSG_COUNT} mensagem(ns) encontrada(s):${NC}"
    echo ""

    # Exibir mensagens de forma legível
    echo "$RESPONSE" | jq -C '.receivedMessages[] | {
        ackId: .ackId,
        publishTime: .message.publishTime,
        data: (.message.data | @base64d),
        attributes: .message.attributes
    }'

    echo ""
    echo -e "${YELLOW}💡 Para fazer ACK das mensagens:${NC}"
    echo -e "   curl -X POST http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/${SUBSCRIPTION}:acknowledge \\"
    echo -e "     -H 'Content-Type: application/json' \\"
    echo -e "     -d '{\"ackIds\": [\"ACK_ID_AQUI\"]}'"
else
    echo -e "${YELLOW}📭 Nenhuma mensagem na fila (subscription vazia)${NC}"
fi

echo ""
echo -e "${BLUE}💡 Dica:${NC}"
echo -e "   - Para monitorar: ${YELLOW}watch -n 2 './pubsub-pull-direct.sh 5'${NC}"
echo -e "   - Para mais mensagens: ${YELLOW}./pubsub-pull-direct.sh 50${NC}"
