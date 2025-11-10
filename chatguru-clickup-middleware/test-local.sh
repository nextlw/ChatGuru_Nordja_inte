#!/bin/bash
# Script para testar localhost
set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

LOCAL_URL="http://localhost:8080"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           TESTE LOCAL - ENVIO DE 8 MENSAGENS                  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Gerar ID único para este teste
TEST_ID="TEST-$(date +%s)"
CHAT_ID="${TEST_ID}@c.us"

echo -e "${BLUE}📋 Informações do Teste:${NC}"
echo -e "   Test ID: ${YELLOW}${TEST_ID}${NC}"
echo -e "   Chat ID: ${YELLOW}${CHAT_ID}${NC}"
echo -e "   Local: ${YELLOW}${LOCAL_URL}${NC}"
echo -e "   Timestamp: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Mensagens de tarefas para teste
declare -a TASK_MESSAGES=(
  "Preciso criar uma landing page para o novo produto. Deve ter formulário de captura de leads, vídeo de apresentação e integração com Mailchimp."
  "Desenvolver API REST para integração com sistema de pagamento. Precisa suportar cartão de crédito, PIX e boleto bancário."
  "Implementar dashboard analytics com gráficos de vendas mensais, métricas de conversão e relatórios exportáveis em PDF e Excel."
  "Criar sistema de notificações push para o app mobile. Deve enviar alertas de promoções, status de pedidos e mensagens importantes."
  "Desenvolver módulo de gerenciamento de estoque com controle de entrada/saída, alertas de estoque baixo e previsão de reposição."
  "Implementar sistema de chat ao vivo no site com suporte a múltiplos atendentes, histórico de conversas e integração com WhatsApp."
  "Criar fluxo de onboarding para novos usuários com tour guiado, vídeos tutoriais e checklist de configuração inicial."
  "Desenvolver relatório gerencial com KPIs de vendas, análise de clientes, ticket médio e projeções de crescimento."
)

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║          ENVIANDO 8 MENSAGENS (INTERVALO DE 2s)               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Enviar 8 mensagens com intervalo de 2s
for i in {1..8}; do
  MESSAGE_NUM=$i
  MESSAGE_TEXT="${TASK_MESSAGES[$((i-1))]}"

  echo -e "${YELLOW}📤 Enviando mensagem ${MESSAGE_NUM}/8...${NC}"

  PAYLOAD=$(cat <<EOF
{
  "chat_id": "${CHAT_ID}",
  "celular": "5511999999999",
  "sender_name": "William Duarte - Teste Local",
  "texto_mensagem": "[MSG ${MESSAGE_NUM}/8] ${MESSAGE_TEXT}",
  "message_type": "text",
  "campos_personalizados": {
    "Info_1": "Nexcode",
    "Info_2": "Tarefas"
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
)

  RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
    "${LOCAL_URL}/webhooks/chatguru" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" 2>&1)

  HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)

  if [ "$HTTP_CODE" = "200" ]; then
    echo -e "   ${GREEN}✅ Mensagem ${MESSAGE_NUM}/8 enviada (HTTP 200)${NC}"
  else
    echo -e "   ${RED}❌ Mensagem ${MESSAGE_NUM}/8 falhou (HTTP ${HTTP_CODE})${NC}"
    echo -e "   Response: $RESPONSE"
  fi

  # Aguardar 2 segundos antes da próxima mensagem (exceto na última)
  if [ $i -lt 8 ]; then
    echo -e "   ${BLUE}⏳ Aguardando 2 segundos...${NC}"
    sleep 2
  fi
done

echo ""
echo -e "${GREEN}✅ Todas as 8 mensagens foram enviadas!${NC}"
echo ""
echo -e "${BLUE}📋 Aguarde o processamento no terminal do cargo run...${NC}"
echo -e "${BLUE}   Chat ID: ${CHAT_ID}${NC}"
echo -e "${BLUE}   Busque por logs com '${TEST_ID}' ou 'William Duarte'${NC}"
echo ""
