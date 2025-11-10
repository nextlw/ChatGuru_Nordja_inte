#!/bin/bash
# Script para iniciar o Pub/Sub Emulator localmente
#
# Uso: ./start-pubsub-emulator.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║             INICIALIZANDO PUB/SUB EMULATOR LOCAL              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verificar se gcloud está instalado
if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}❌ gcloud CLI não encontrado!${NC}"
    echo -e "   Instale: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Verificar se o emulador está instalado
if ! gcloud components list --filter="id:pubsub-emulator" --format="value(state.name)" | grep -q "Installed"; then
    echo -e "${YELLOW}⚠️  Pub/Sub Emulator não instalado. Instalando...${NC}"
    gcloud components install pubsub-emulator
fi

# Configuração
PROJECT_ID="local-dev"
HOST="localhost"
PORT="8085"

echo -e "${BLUE}📋 Configuração:${NC}"
echo -e "   Project ID: ${YELLOW}${PROJECT_ID}${NC}"
echo -e "   Host: ${YELLOW}${HOST}:${PORT}${NC}"
echo -e "   Topics: ${YELLOW}chatguru-webhook-events, clickup-webhook-events${NC}"
echo ""

# Criar diretório de dados
DATA_DIR="./pubsub-data"
mkdir -p "$DATA_DIR"

echo -e "${BLUE}🚀 Iniciando emulador...${NC}"
echo -e "   ${YELLOW}Pressione Ctrl+C para parar${NC}"
echo ""

# Iniciar emulador
gcloud beta emulators pubsub start \
    --project="$PROJECT_ID" \
    --host-port="${HOST}:${PORT}" \
    --data-dir="$DATA_DIR"
