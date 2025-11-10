#!/bin/bash
# Script para fazer pull de mensagens do Pub/Sub Emulator (uma vez só)
#
# Uso: ./pubsub-pull.sh [quantidade]

# Configuração
export PUBSUB_EMULATOR_HOST="localhost:8085"
export PUBSUB_PROJECT_ID="local-dev"

PROJECT_ID="local-dev"
SUBSCRIPTION="chatguru-worker-sub"
LIMIT="${1:-10}"  # Default: 10 mensagens

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}📬 Fazendo pull de até ${LIMIT} mensagens da subscription '${SUBSCRIPTION}'...${NC}"
echo ""

# Pull mensagens
RESULT=$(gcloud pubsub subscriptions pull "$SUBSCRIPTION" \
  --project="$PROJECT_ID" \
  --limit="$LIMIT" \
  --format=json 2>&1)

if [ $? -eq 0 ]; then
    # Parse JSON e verificar se há mensagens
    MSG_COUNT=$(echo "$RESULT" | jq 'length' 2>/dev/null || echo "0")

    if [ "$MSG_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✅ ${MSG_COUNT} mensagem(ns) encontrada(s):${NC}"
        echo ""
        echo "$RESULT" | jq -C '.' || echo "$RESULT"
    else
        echo -e "${YELLOW}📭 Nenhuma mensagem na fila (subscription vazia)${NC}"
    fi
else
    echo -e "${RED}❌ Erro ao fazer pull:${NC}"
    echo "$RESULT"
    exit 1
fi

echo ""
echo -e "${BLUE}💡 Dica:${NC}"
echo -e "   - Para monitorar em tempo real: ${YELLOW}./monitor-pubsub.sh${NC}"
echo -e "   - Para especificar quantidade: ${YELLOW}./pubsub-pull.sh 50${NC}"
