#!/bin/bash
# Script para criar tópicos e subscriptions no Pub/Sub Emulator via API REST
#
# IMPORTANTE: O emulator NÃO suporta comandos 'gcloud pubsub'!
# Referência: https://cloud.google.com/pubsub/docs/emulator
#
# Uso:
#   1. Terminal 1: ./start-pubsub-emulator.sh
#   2. Terminal 2: ./setup-pubsub-rest.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         CONFIGURANDO PUB/SUB EMULATOR (API REST)              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuração
EMULATOR_HOST="localhost:8085"
PROJECT_ID="local-dev"
TOPIC_CHATGURU="chatguru-webhook-events"
TOPIC_CLICKUP="clickup-webhook-events"
SUBSCRIPTION_WORKER="chatguru-worker-sub"

echo -e "${BLUE}🔗 Conectando ao emulador: ${YELLOW}${EMULATOR_HOST}${NC}"

# Verificar se o emulador está rodando
if ! curl -s "http://${EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi
echo -e "   ${GREEN}✅${NC} Emulador está rodando"
echo ""

# Função para criar um tópico
create_topic() {
    local topic_name=$1
    local url="http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics/${topic_name}"

    echo -e "${BLUE}📝 Criando tópico '${topic_name}'...${NC}"

    response=$(curl -s -X PUT "$url" \
        -H "Content-Type: application/json" \
        -d '{}')

    if echo "$response" | jq -e '.error' > /dev/null 2>&1; then
        error_msg=$(echo "$response" | jq -r '.error.message')
        if [[ "$error_msg" == *"ALREADY_EXISTS"* ]]; then
            echo -e "   ${YELLOW}⚠️${NC}  Tópico '${topic_name}' já existe"
        else
            echo -e "   ${RED}❌${NC} Erro: $error_msg"
            return 1
        fi
    else
        echo -e "   ${GREEN}✅${NC} Tópico '${topic_name}' criado"
    fi
}

# Função para criar uma subscription
create_subscription() {
    local subscription_name=$1
    local topic_name=$2
    local url="http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/${subscription_name}"

    echo -e "${BLUE}📬 Criando subscription '${subscription_name}'...${NC}"

    response=$(curl -s -X PUT "$url" \
        -H "Content-Type: application/json" \
        -d "{
            \"topic\": \"projects/${PROJECT_ID}/topics/${topic_name}\",
            \"ackDeadlineSeconds\": 600
        }")

    if echo "$response" | jq -e '.error' > /dev/null 2>&1; then
        error_msg=$(echo "$response" | jq -r '.error.message')
        if [[ "$error_msg" == *"ALREADY_EXISTS"* ]]; then
            echo -e "   ${YELLOW}⚠️${NC}  Subscription '${subscription_name}' já existe"
        else
            echo -e "   ${RED}❌${NC} Erro: $error_msg"
            return 1
        fi
    else
        echo -e "   ${GREEN}✅${NC} Subscription '${subscription_name}' criada"
    fi
}

# Criar tópicos
echo -e "${BLUE}📝 Criando tópicos...${NC}"
create_topic "$TOPIC_CHATGURU"
create_topic "$TOPIC_CLICKUP"
echo ""

# Criar subscriptions
echo -e "${BLUE}📬 Criando subscriptions...${NC}"
create_subscription "$SUBSCRIPTION_WORKER" "$TOPIC_CHATGURU"
echo ""

# Listar recursos criados via API REST
echo -e "${BLUE}📋 Recursos criados:${NC}"
echo ""

echo -e "${YELLOW}Tópicos:${NC}"
topics_response=$(curl -s "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/topics")
echo "$topics_response" | jq -r '.topics[]?.name // empty' | sed 's|projects/.*/topics/|  - |'

echo ""
echo -e "${YELLOW}Subscriptions:${NC}"
subs_response=$(curl -s "http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions")
echo "$subs_response" | jq -r '.subscriptions[]? | "  - \(.name | sub("projects/.*/subscriptions/"; "")) → \(.topic | sub("projects/.*/topics/"; ""))"'

echo ""
echo -e "${GREEN}✅ Configuração concluída!${NC}"
echo ""
echo -e "${BLUE}💡 Para desenvolvimento local, use estas variáveis:${NC}"
echo -e "   ${YELLOW}export PUBSUB_EMULATOR_HOST=localhost:8085${NC}"
echo -e "   ${YELLOW}export PUBSUB_PROJECT_ID=local-dev${NC}"
echo ""
echo -e "${BLUE}💡 Para testar:${NC}"
echo -e "   ${YELLOW}./pubsub-publish-test.sh${NC}"
echo -e "   ${YELLOW}./pubsub-pull-direct.sh${NC}"
