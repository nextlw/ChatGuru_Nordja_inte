#!/bin/bash
# Script para monitorar mensagens no Pub/Sub Emulator
#
# Uso: ./monitor-pubsub.sh

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuração
export PUBSUB_EMULATOR_HOST="localhost:8085"
export PUBSUB_PROJECT_ID="local-dev"

PROJECT_ID="local-dev"
SUBSCRIPTION="chatguru-worker-sub"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           MONITORAMENTO PUB/SUB EMULATOR (TEMPO REAL)         ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}🔗 Emulator: ${YELLOW}${PUBSUB_EMULATOR_HOST}${NC}"
echo -e "${BLUE}📬 Subscription: ${YELLOW}${SUBSCRIPTION}${NC}"
echo -e "${BLUE}⏱️  Intervalo: ${YELLOW}2 segundos${NC}"
echo ""
echo -e "${YELLOW}Pressione Ctrl+C para parar${NC}"
echo ""

# Verificar se o emulador está rodando
if ! curl -s "http://${PUBSUB_EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${PUBSUB_EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi

# Watch com variáveis de ambiente corretas
watch -n 2 -c "export PUBSUB_EMULATOR_HOST=localhost:8085 && \
gcloud pubsub subscriptions pull ${SUBSCRIPTION} \
  --project=${PROJECT_ID} \
  --limit=5 \
  --format='table(message.data.decode(base64),message.publishTime,ackId)' 2>&1 || \
echo 'Aguardando mensagens...'"
