#!/bin/bash
# Script para testar o fluxo completo local com Pub/Sub Emulator
#
# IMPORTANTE: Execute este script DEPOIS de:
#   1. Terminal 1: ./start-pubsub-emulator.sh
#   2. Terminal 2: ./setup-pubsub-topics.sh
#   3. Terminal 3: source .env.local && cargo run
#   4. Terminal 4: ./test-local-pubsub.sh
#
# Uso: ./test-local-pubsub.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          TESTE LOCAL COM PUB/SUB EMULATOR + WEBHOOK           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuração
export PUBSUB_EMULATOR_HOST="localhost:8085"
export PUBSUB_PROJECT_ID="local-dev"
LOCAL_URL="http://localhost:8080"
PROJECT_ID="local-dev"
TOPIC="chatguru-webhook-events"

# Gerar ID único para este teste
TEST_ID="LOCAL-TEST-$(date +%s)"

echo -e "${BLUE}📋 Informações do Teste:${NC}"
echo -e "   Test ID: ${YELLOW}${TEST_ID}${NC}"
echo -e "   Local URL: ${YELLOW}${LOCAL_URL}${NC}"
echo -e "   Pub/Sub: ${YELLOW}${PUBSUB_EMULATOR_HOST}${NC}"
echo -e "   Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Verificar se o servidor local está rodando
echo -e "${BLUE}🔍 Verificando servidor local...${NC}"
if ! curl -s "${LOCAL_URL}/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Servidor não está rodando em ${LOCAL_URL}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}source .env.local && cargo run${NC}"
    exit 1
fi
echo -e "   ${GREEN}✅${NC} Servidor local está respondendo"
echo ""

# Criar payload de teste
PAYLOAD=$(cat <<EOF
{
  "chat_id": "${TEST_ID}@c.us",
  "celular": "5511999999999",
  "sender_name": "Teste Local",
  "texto_mensagem": "[${TEST_ID}] Teste local com emulador Pub/Sub - Sistema completo de gestão de tarefas",
  "message_type": "text",
  "campos_personalizados": {
    "Info_1": "Nexcode",
    "Info_2": "Tarefas"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

echo -e "${BLUE}📤 Payload a ser enviado:${NC}"
echo "$PAYLOAD" | jq '.' | sed 's/^/   /'
echo ""

# Enviar para webhook local
echo -e "${BLUE}🚀 Enviando para webhook local...${NC}"
START_TIME=$(date +%s)

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
  "${LOCAL_URL}/webhooks/chatguru" \
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
    echo -e "${RED}❌ Webhook retornou status ${HTTP_CODE}${NC}"
    exit 1
fi

# Aguardar batch timeout ou mensagens suficientes
echo -e "${BLUE}⏳ Aguardando 5 segundos para verificar logs...${NC}"
sleep 5
echo ""

# Verificar mensagens no Pub/Sub Emulator (tópico)
echo -e "${BLUE}📬 Verificando mensagens no Pub/Sub...${NC}"

# Garantir que variáveis estão exportadas para o gcloud
export PUBSUB_EMULATOR_HOST="localhost:8085"

MESSAGES=$(gcloud pubsub subscriptions pull chatguru-worker-sub \
    --project="$PROJECT_ID" \
    --limit=10 \
    --format=json 2>/dev/null || echo "[]")

if [ "$MESSAGES" == "[]" ] || [ -z "$MESSAGES" ]; then
    echo -e "   ${YELLOW}⚠️${NC}  Nenhuma mensagem na subscription ainda (aguardando batch)"
    echo -e "   ${BLUE}💡${NC} A mensagem foi enfileirada e aguarda:"
    echo -e "      - 8 mensagens acumuladas OU"
    echo -e "      - 180 segundos (3 minutos) de timeout"
    echo ""
    echo -e "   ${BLUE}📝 Logs do servidor local:${NC}"
    echo -e "      Verifique os logs do ${YELLOW}cargo run${NC} para ver a mensagem enfileirada"
else
    echo -e "   ${GREEN}✅${NC} Mensagem(ns) encontrada(s) no Pub/Sub!"
    echo "$MESSAGES" | jq '.' | sed 's/^/      /'
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                      TESTE CONCLUÍDO                           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}💡 Próximos passos:${NC}"
echo -e "   1. Verifique os logs do servidor: ${YELLOW}cargo run${NC}"
echo -e "   2. Envie mais mensagens para completar o batch (8 mensagens)"
echo -e "   3. Ou aguarde 180s para o timeout automático"
echo -e "   4. Para simular worker, implemente subscriber no /worker/process"
echo ""
echo -e "${BLUE}📚 Comandos úteis:${NC}"
echo -e "   ${YELLOW}# Ver mensagens (uma vez)${NC}"
echo -e "   ./pubsub-pull.sh"
echo ""
echo -e "   ${YELLOW}# Monitorar em tempo real${NC}"
echo -e "   ./monitor-pubsub.sh"
echo ""
echo -e "   ${YELLOW}# Comandos gcloud manuais (precisa exportar variável antes!)${NC}"
echo -e "   export PUBSUB_EMULATOR_HOST=localhost:8085"
echo -e "   gcloud pubsub subscriptions pull chatguru-worker-sub --project=local-dev --limit=10"
echo -e "   gcloud pubsub topics publish ${TOPIC} --message='test' --project=${PROJECT_ID}"
