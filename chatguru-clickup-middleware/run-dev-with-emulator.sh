#!/bin/bash
# Script para rodar a aplicação em modo desenvolvimento COM Pub/Sub Emulator
#
# Uso: ./run-dev-with-emulator.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║        RODANDO APLICAÇÃO EM DEV COM PUB/SUB EMULATOR         ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verificar se o emulador está rodando
EMULATOR_HOST="localhost:8085"
if ! curl -s "http://${EMULATOR_HOST}" > /dev/null 2>&1; then
    echo -e "${RED}❌ Emulador não está rodando em ${EMULATOR_HOST}${NC}"
    echo ""
    echo -e "${YELLOW}Por favor, execute em outro terminal:${NC}"
    echo -e "   ${BLUE}Terminal 1:${NC} ./start-pubsub-emulator.sh"
    echo -e "   ${BLUE}Terminal 2:${NC} ./setup-pubsub-rest.sh"
    echo -e "   ${BLUE}Terminal 3:${NC} ./run-dev-with-emulator.sh ${YELLOW}(este script)${NC}"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ Emulador detectado em ${EMULATOR_HOST}${NC}"
echo ""

# Carregar variáveis do .env se existir
if [ -f .env ]; then
    echo -e "${BLUE}📄 Carregando variáveis do .env...${NC}"
    set -a
    source .env
    set +a
else
    echo -e "${YELLOW}⚠️  Arquivo .env não encontrado - usando variáveis do sistema${NC}"
fi

# Configurar variáveis para usar o emulator
export PUBSUB_EMULATOR_HOST="localhost:8085"
export PUBSUB_PROJECT_ID="local-dev"
export RUST_ENV="development"
export RUST_LOG="info,chatguru_clickup_middleware=debug"

# IMPORTANTE: Esta variável força o uso do Pub/Sub mesmo em dev
# Sem ela, o código chama o worker diretamente (linha 593 do main.rs)
export FORCE_PUBSUB="true"

echo -e "${BLUE}🔧 Configurações:${NC}"
echo -e "   PUBSUB_EMULATOR_HOST: ${YELLOW}${PUBSUB_EMULATOR_HOST}${NC}"
echo -e "   PUBSUB_PROJECT_ID: ${YELLOW}${PUBSUB_PROJECT_ID}${NC}"
echo -e "   RUST_ENV: ${YELLOW}${RUST_ENV}${NC}"
echo -e "   FORCE_PUBSUB: ${YELLOW}${FORCE_PUBSUB}${NC}"
echo ""

echo -e "${GREEN}🚀 Iniciando aplicação...${NC}"
echo -e "${YELLOW}   (Pressione Ctrl+C para parar)${NC}"
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Rodar aplicação
cargo run --bin chatguru-clickup-middleware
