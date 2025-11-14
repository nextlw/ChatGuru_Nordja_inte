#!/bin/bash
# Script para testar processamento de áudio no webhook local
set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

LOCAL_URL="http://localhost:8080"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     TESTE LOCAL - PROCESSAMENTO IMEDIATO DE ÁUDIO             ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Gerar ID único para este teste
TEST_ID="AUDIO-TEST-$(date +%s)"
CHAT_ID="${TEST_ID}@c.us"

echo -e "${BLUE}📋 Informações do Teste:${NC}"
echo -e "   Test ID: ${YELLOW}${TEST_ID}${NC}"
echo -e "   Chat ID: ${YELLOW}${CHAT_ID}${NC}"
echo -e "   Local: ${YELLOW}${LOCAL_URL}${NC}"
echo -e "   Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# URL de áudio de exemplo (nota: esta URL precisa ser válida e acessível)
# Como estamos testando localmente, usaremos uma URL pública de exemplo
AUDIO_URL="https://www2.cs.uic.edu/~i101/SoundFiles/BabyElephantWalk60.wav"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           ENVIANDO MENSAGEM COM ÁUDIO                          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${YELLOW}🎤 Enviando payload com áudio...${NC}"
echo -e "   URL do áudio: ${AUDIO_URL}"
echo ""

PAYLOAD=$(cat <<EOF
{
  "chat_id": "${CHAT_ID}",
  "celular": "5511999999999",
  "sender_name": "William Duarte - Teste Áudio",
  "nome": "William Duarte",
  "texto_mensagem": "Esta é uma mensagem de teste com áudio anexado",
  "message_type": "audio",
  "media_url": "${AUDIO_URL}",
  "media_type": "audio/wav",
  "campos_personalizados": {
    "Info_1": "Nexcode",
    "Info_2": "William"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

echo -e "${BLUE}📤 Enviando requisição...${NC}"
echo ""

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
  "${LOCAL_URL}/webhooks/chatguru" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" 2>&1)

HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)
RESPONSE_BODY=$(echo "$RESPONSE" | head -n -1)

echo -e "${BLUE}📥 Resposta:${NC}"
echo "$RESPONSE_BODY" | jq '.' 2>/dev/null || echo "$RESPONSE_BODY"
echo ""

if [ "$HTTP_CODE" = "200" ]; then
  echo -e "${GREEN}✅ Webhook retornou HTTP 200 - SUCESSO!${NC}"
  echo ""
  echo -e "${BLUE}🔍 O que deve acontecer nos logs do 'cargo run':${NC}"
  echo -e "   1. ${YELLOW}🔍 Verificando presença de mídia no payload...${NC}"
  echo -e "   2. ${YELLOW}📎 Mídia detectada: ${AUDIO_URL}${NC}"
  echo -e "   3. ${YELLOW}🎤 Processando áudio...${NC}"
  echo -e "   4. ${YELLOW}⬇️ Baixando áudio de: ${AUDIO_URL}${NC}"
  echo -e "   5. ${YELLOW}✅ Áudio baixado: XXX bytes${NC}"
  echo -e "   6. ${YELLOW}🎤 Transcrevendo áudio: XXX bytes${NC}"
  echo -e "   7. ${YELLOW}✅ Áudio transcrito: XXX caracteres${NC}"
  echo -e "   8. ${YELLOW}✅ Anotação enviada ao ChatGuru com sucesso${NC}"
  echo -e "   9. ${YELLOW}✅ Payload sintético criado com sucesso${NC}"
  echo -e "  10. ${YELLOW}📬 ADICIONANDO À FILA...${NC}"
  echo ""
  echo -e "${GREEN}✅ Teste enviado! Acompanhe os logs do servidor.${NC}"
else
  echo -e "${RED}❌ Webhook falhou com HTTP ${HTTP_CODE}${NC}"
  echo -e "   Response: $RESPONSE"
fi

echo ""
echo -e "${BLUE}💡 Dicas:${NC}"
echo -e "   • Busque nos logs por: '${TEST_ID}'"
echo -e "   • Busque nos logs por: 'Processando áudio'"
echo -e "   • Busque nos logs por: 'Payload sintético'"
echo -e "   • O processamento deve levar 2-5 segundos"
echo ""
