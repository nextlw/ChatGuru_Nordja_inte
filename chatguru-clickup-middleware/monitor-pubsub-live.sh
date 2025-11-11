#!/bin/bash
# Script para monitorar mensagens do Pub/Sub Emulator em TEMPO REAL
# Usa API REST diretamente (não depende de gcloud)
#
# Uso: ./monitor-pubsub-live.sh

# Configuração
EMULATOR_HOST="localhost:8085"
PROJECT_ID="local-dev"
SUBSCRIPTION="chatguru-worker-sub"
INTERVAL=2  # segundos entre cada pull

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Limpar tela
clear

echo -e "${GREEN}${BOLD}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}${BOLD}║         MONITORAMENTO PUB/SUB EMULATOR (TEMPO REAL)           ║${NC}"
echo -e "${GREEN}${BOLD}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verificar se o emulador está rodando
if ! curl -s "http://${EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${EMULATOR_HOST}${NC}"
    echo -e "   Execute em outro terminal: ${YELLOW}./start-pubsub-emulator.sh${NC}"
    exit 1
fi

echo -e "${BLUE}🔗 Emulator: ${YELLOW}${EMULATOR_HOST}${NC}"
echo -e "${BLUE}📬 Subscription: ${YELLOW}${SUBSCRIPTION}${NC}"
echo -e "${BLUE}⏱️  Intervalo: ${YELLOW}${INTERVAL}s${NC}"
echo ""
echo -e "${YELLOW}${BOLD}Pressione Ctrl+C para parar${NC}"
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Contador de mensagens
TOTAL_MESSAGES=0

# URL da API REST
PULL_URL="http://${EMULATOR_HOST}/v1/projects/${PROJECT_ID}/subscriptions/${SUBSCRIPTION}:pull"

# Trap para limpar na saída
trap 'echo ""; echo -e "${YELLOW}👋 Monitoramento finalizado. Total de mensagens vistas: ${TOTAL_MESSAGES}${NC}"; exit 0' INT TERM

# Loop de monitoramento
while true; do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

    # Fazer pull de mensagens (sem ACK automático para poder ver repetidas vezes)
    RESPONSE=$(curl -s -X POST "$PULL_URL" \
        -H "Content-Type: application/json" \
        -d '{"maxMessages": 5}')

    # Verificar se há erro
    if echo "$RESPONSE" | jq -e '.error' > /dev/null 2>&1; then
        ERROR_MSG=$(echo "$RESPONSE" | jq -r '.error.message')
        echo -e "${RED}[${TIMESTAMP}] ❌ Erro: ${ERROR_MSG}${NC}"
        sleep $INTERVAL
        continue
    fi

    # Contar mensagens
    MSG_COUNT=$(echo "$RESPONSE" | jq '.receivedMessages | length' 2>/dev/null || echo "0")

    if [ "$MSG_COUNT" -gt 0 ]; then
        TOTAL_MESSAGES=$((TOTAL_MESSAGES + MSG_COUNT))

        echo -e "${GREEN}[${TIMESTAMP}] 📨 ${MSG_COUNT} mensagem(ns) encontrada(s)${NC}"
        echo ""

        # Processar cada mensagem
        echo "$RESPONSE" | jq -r '.receivedMessages[] | @json' | while read -r msg; do
            # Extrair campos
            MESSAGE_ID=$(echo "$msg" | jq -r '.message.messageId')
            PUBLISH_TIME=$(echo "$msg" | jq -r '.message.publishTime')
            DATA=$(echo "$msg" | jq -r '.message.data' | base64 -d 2>/dev/null || echo "(erro ao decodificar)")
            ATTRIBUTES=$(echo "$msg" | jq -c '.message.attributes // {}')
            ACK_ID=$(echo "$msg" | jq -r '.ackId')

            # Exibir mensagem formatada
            echo -e "${MAGENTA}  ┌─ Mensagem ${MESSAGE_ID:0:16}...${NC}"
            echo -e "${BLUE}  │ ⏰ Publicada em: ${YELLOW}${PUBLISH_TIME}${NC}"

            if [ "$ATTRIBUTES" != "{}" ]; then
                echo -e "${BLUE}  │ 🏷️  Atributos: ${CYAN}${ATTRIBUTES}${NC}"
            fi

            echo -e "${BLUE}  │ 📄 Dados:${NC}"

            # Tentar formatar como JSON se possível
            if echo "$DATA" | jq . > /dev/null 2>&1; then
                echo "$DATA" | jq -C '.' | sed 's/^/  │     /'
            else
                echo -e "${CYAN}  │     ${DATA}${NC}"
            fi

            echo -e "${MAGENTA}  └─ ACK ID: ${ACK_ID:0:32}...${NC}"
            echo ""
        done

        echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
    else
        # Sem mensagens - exibir status silencioso
        echo -ne "\r${BLUE}[${TIMESTAMP}] ⏳ Aguardando mensagens... (Total visto: ${TOTAL_MESSAGES})${NC}  "
    fi

    sleep $INTERVAL
done
