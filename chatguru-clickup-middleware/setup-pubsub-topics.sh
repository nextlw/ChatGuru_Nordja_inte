#!/bin/bash
# Script para criar tópicos e subscriptions no Pub/Sub Emulator
#
# IMPORTANTE: Execute este script DEPOIS de iniciar o emulador (start-pubsub-emulator.sh)
#
# Uso:
#   1. Terminal 1: ./start-pubsub-emulator.sh
#   2. Terminal 2: ./setup-pubsub-topics.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║            CONFIGURANDO TÓPICOS NO PUB/SUB EMULATOR           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuração
export PUBSUB_EMULATOR_HOST="localhost:8085"
export PUBSUB_PROJECT_ID="local-dev"
# Importante: desabilitar autenticação para o emulador
export CLOUDSDK_AUTH_ACCESS_TOKEN=""

PROJECT_ID="local-dev"
TOPIC_CHATGURU="chatguru-webhook-events"
TOPIC_CLICKUP="clickup-webhook-events"
SUBSCRIPTION_WORKER="chatguru-worker-sub"

echo -e "${BLUE}🔗 Conectando ao emulador: ${YELLOW}${PUBSUB_EMULATOR_HOST}${NC}"

# Verificar se o emulador está rodando
if ! curl -s "http://${PUBSUB_EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${PUBSUB_EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi
echo -e "   ${GREEN}✅${NC} Emulador está rodando"
echo ""

# Criar tópicos
echo -e "${BLUE}📝 Criando tópicos...${NC}"

gcloud pubsub topics create "$TOPIC_CHATGURU" \
    --project="$PROJECT_ID" 2>/dev/null \
    && echo -e "   ${GREEN}✅${NC} Tópico '${TOPIC_CHATGURU}' criado" \
    || echo -e "   ${YELLOW}⚠️${NC}  Tópico '${TOPIC_CHATGURU}' já existe"

gcloud pubsub topics create "$TOPIC_CLICKUP" \
    --project="$PROJECT_ID" 2>/dev/null \
    && echo -e "   ${GREEN}✅${NC} Tópico '${TOPIC_CLICKUP}' criado" \
    || echo -e "   ${YELLOW}⚠️${NC}  Tópico '${TOPIC_CLICKUP}' já existe"

echo ""

# Criar subscription
echo -e "${BLUE}📬 Criando subscription...${NC}"

gcloud pubsub subscriptions create "$SUBSCRIPTION_WORKER" \
    --topic="$TOPIC_CHATGURU" \
    --project="$PROJECT_ID" \
    --ack-deadline=600 \
    2>/dev/null \
    && echo -e "   ${GREEN}✅${NC} Subscription '${SUBSCRIPTION_WORKER}' criada" \
    || echo -e "   ${YELLOW}⚠️${NC}  Subscription '${SUBSCRIPTION_WORKER}' já existe"

echo ""

# Listar recursos criados
echo -e "${BLUE}📋 Recursos criados:${NC}"
echo ""
echo -e "${YELLOW}Tópicos:${NC}"
if ! gcloud pubsub topics list --project="$PROJECT_ID" --format="table(name)" 2>/dev/null; then
    echo -e "   ${YELLOW}⚠️${NC}  Não foi possível listar (use curl para verificar)"
    echo -e "   - $TOPIC_CHATGURU"
    echo -e "   - $TOPIC_CLICKUP"
fi
echo ""
echo -e "${YELLOW}Subscriptions:${NC}"
if ! gcloud pubsub subscriptions list --project="$PROJECT_ID" --format="table(name,topic)" 2>/dev/null; then
    echo -e "   ${YELLOW}⚠️${NC}  Não foi possível listar (use curl para verificar)"
    echo -e "   - $SUBSCRIPTION_WORKER → $TOPIC_CHATGURU"
fi

echo ""
echo -e "${GREEN}✅ Configuração concluída!${NC}"
echo ""
echo -e "${BLUE}💡 Variáveis de ambiente para desenvolvimento local:${NC}"
echo -e "   ${YELLOW}export PUBSUB_EMULATOR_HOST=localhost:8085${NC}"
echo -e "   ${YELLOW}export PUBSUB_PROJECT_ID=local-dev${NC}"
echo ""
echo -e "${BLUE}💡 Para testar publicação:${NC}"
echo -e "   ${YELLOW}gcloud pubsub topics publish ${TOPIC_CHATGURU} --message='test' --project=${PROJECT_ID}${NC}"
