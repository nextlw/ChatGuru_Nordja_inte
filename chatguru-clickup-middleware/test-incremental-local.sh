#!/bin/bash
# Script para testar análise incremental localmente
#
# Uso: ./test-incremental-local.sh
#
# Pré-requisitos:
# 1. Criar arquivo .env com OPENAI_API_KEY
# 2. Ter Pub/Sub emulator rodando (ou usar Google Cloud)

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         TESTE LOCAL - ANÁLISE INCREMENTAL (gpt-4o-mini)       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verificar se .env existe
if [ ! -f .env ]; then
    echo -e "${RED}❌ Arquivo .env não encontrado!${NC}"
    echo -e "${YELLOW}ℹ️  Crie um arquivo .env com:${NC}"
    echo -e "   OPENAI_API_KEY=sk-..."
    echo -e "   DATABASE_URL=postgresql://..."
    echo -e "   CLICKUP_CLIENT_ID=..."
    echo -e "   CLICKUP_CLIENT_SECRET=..."
    exit 1
fi

# Verificar se OPENAI_API_KEY está configurado
source .env
if [ -z "$OPENAI_API_KEY" ]; then
    echo -e "${RED}❌ OPENAI_API_KEY não encontrado no .env!${NC}"
    exit 1
fi

echo -e "${BLUE}📋 Configurações:${NC}"
echo -e "   Análise Incremental: ${GREEN}HABILITADA${NC}"
echo -e "   Modelo: ${YELLOW}gpt-4o-mini${NC} (~\$0.0003/mensagem)"
echo -e "   Threshold: ${YELLOW}80% confiança${NC}"
echo -e "   Fallback: ${YELLOW}SmartContextManager${NC}"
echo ""

# Teste 1: Conversa completa (deve processar rápido)
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  TESTE 1: Conversa Completa (conclusão explícita)             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Simulando conversa:${NC}"
echo -e "  Msg 1: 'Preciso criar uma landing page para o novo produto'"
echo -e "  Msg 2: 'Deve ter formulário de captura de leads'"
echo -e "  Msg 3: 'Ok, pode criar a task' ${GREEN}← CONCLUSÃO${NC}"
echo ""
echo -e "${BLUE}➡️  Esperado: Processar IMEDIATAMENTE após msg 3 (análise incremental)${NC}"
echo ""

# Teste 2: Conversa incompleta (deve aguardar)
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  TESTE 2: Conversa Incompleta (aguardando mais info)          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Simulando conversa:${NC}"
echo -e "  Msg 1: 'Olá, preciso de ajuda'"
echo -e "  Msg 2: 'Como faço para...'"
echo ""
echo -e "${BLUE}➡️  Esperado: NÃO processar (aguardando mais mensagens)${NC}"
echo ""

# Instruções para compilar e rodar
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  COMO EXECUTAR O TESTE                         ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${YELLOW}Passo 1:${NC} Compilar o projeto"
echo -e "  ${BLUE}cargo build${NC}"
echo ""

echo -e "${YELLOW}Passo 2:${NC} Rodar o middleware com análise incremental HABILITADA"
echo -e "  ${BLUE}ENABLE_INCREMENTAL_ANALYSIS=true RUST_LOG=info cargo run${NC}"
echo ""

echo -e "${YELLOW}Passo 3:${NC} Em outro terminal, enviar mensagens de teste"
echo -e "  ${BLUE}# Teste 1: Conversa completa${NC}"
echo -e "  curl -X POST http://localhost:8080/webhooks/chatguru \\"
echo -e "    -H 'Content-Type: application/json' \\"
echo -e "    -d '{\"chat_id\":\"test1@c.us\",\"texto_mensagem\":\"Preciso criar uma landing page\",\"campos_personalizados\":{\"Info_1\":\"Nexcode\",\"Info_2\":\"William\"}}'"
echo ""
echo -e "  curl -X POST http://localhost:8080/webhooks/chatguru \\"
echo -e "    -H 'Content-Type: application/json' \\"
echo -e "    -d '{\"chat_id\":\"test1@c.us\",\"texto_mensagem\":\"Deve ter formulário de leads\",\"campos_personalizados\":{\"Info_1\":\"Nexcode\",\"Info_2\":\"William\"}}'"
echo ""
echo -e "  curl -X POST http://localhost:8080/webhooks/chatguru \\"
echo -e "    -H 'Content-Type: application/json' \\"
echo -e "    -d '{\"chat_id\":\"test1@c.us\",\"texto_mensagem\":\"Ok, pode criar a task\",\"campos_personalizados\":{\"Info_1\":\"Nexcode\",\"Info_2\":\"William\"}}'"
echo ""

echo -e "${YELLOW}Passo 4:${NC} Observar os logs"
echo -e "  ${GREEN}✅${NC} Procure por: ${BLUE}⚡ Análise incremental${NC}"
echo -e "  ${GREEN}✅${NC} Procure por: ${BLUE}complete=true, confidence=XX%${NC}"
echo -e "  ${GREEN}✅${NC} Procure por: ${BLUE}Análise incremental triggered${NC}"
echo ""

echo -e "${YELLOW}Passo 5:${NC} Para DESABILITAR análise incremental (usar só SmartContextManager)"
echo -e "  ${BLUE}ENABLE_INCREMENTAL_ANALYSIS=false RUST_LOG=info cargo run${NC}"
echo -e "  ${BLUE}# ou simplesmente: RUST_LOG=info cargo run (default: false)${NC}"
echo ""

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                   MÉTRICAS ESPERADAS                           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${BLUE}Latência:${NC} ~300ms por análise (gpt-4o-mini)"
echo -e "  ${BLUE}Custo:${NC} ~\$0.0003 por mensagem (vs \$0.002 no gpt-4o)"
echo -e "  ${BLUE}Redução:${NC} 85% mais barato que análise completa"
echo -e "  ${BLUE}Taxa de acerto:${NC} ~80% das conversas detectadas corretamente"
echo -e "  ${BLUE}Fallback:${NC} SmartContextManager para casos não detectados"
echo ""

echo -e "${GREEN}✅ Guia de teste carregado!${NC}"
echo -e "${YELLOW}ℹ️  Execute os passos acima para testar a análise incremental.${NC}"
