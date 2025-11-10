#!/bin/bash
# Script para testar produção e monitorar logs em tempo real
#
# Uso: ./test-production-live.sh

set -e

# Cores
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PRODUCTION_URL="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"
SERVICE_NAME="chatguru-clickup-middleware"
REGION="southamerica-east1"

echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║       TESTE EM PRODUÇÃO - ENVIO + MONITORAMENTO LOGS          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Gerar ID único para este teste
TEST_ID="TEST-$(date +%s)"
CHAT_ID="${TEST_ID}@c.us"

echo -e "${BLUE}📋 Informações do Teste:${NC}"
echo -e "   Test ID: ${YELLOW}${TEST_ID}${NC}"
echo -e "   Chat ID: ${YELLOW}${CHAT_ID}${NC}"
echo -e "   Produção: ${YELLOW}${PRODUCTION_URL}${NC}"
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
echo -e "${GREEN}║          ENVIANDO 8 MENSAGENS (INTERVALO DE 3s)               ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Enviar 8 mensagens com intervalo de 3s
for i in {1..8}; do
  MESSAGE_NUM=$i
  MESSAGE_TEXT="${TASK_MESSAGES[$((i-1))]}"

  echo -e "${YELLOW}📤 Enviando mensagem ${MESSAGE_NUM}/8...${NC}"

  PAYLOAD=$(cat <<EOF
{
  "chat_id": "${CHAT_ID}",
  "celular": "5511999999999",
  "sender_name": "William Duarte - Teste",
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
    "${PRODUCTION_URL}/webhooks/chatguru" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD")

  HTTP_CODE=$(echo "$RESPONSE" | tail -n 1)

  if [ "$HTTP_CODE" = "200" ]; then
    echo -e "   ${GREEN}✅ Mensagem ${MESSAGE_NUM}/8 enviada (HTTP 200)${NC}"
  else
    echo -e "   ${RED}❌ Mensagem ${MESSAGE_NUM}/8 falhou (HTTP ${HTTP_CODE})${NC}"
  fi

  # Aguardar 3 segundos antes da próxima mensagem (exceto na última)
  if [ $i -lt 8 ]; then
    echo -e "   ${BLUE}⏳ Aguardando 3 segundos...${NC}"
    sleep 3
  fi
done

echo ""
echo -e "${GREEN}✅ Todas as 8 mensagens foram enviadas!${NC}"
echo ""

# Aguardar um pouco antes de começar a monitorar logs
echo -e "${BLUE}⏳ Aguardando 3 segundos antes de iniciar monitoramento de logs...${NC}"
sleep 3
echo ""

# Monitorar logs em tempo real
echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              📋 LOGS EM TEMPO REAL (Cloud Run)                 ║${NC}"
echo -e "${GREEN}╠════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${GREEN}║  Filtrando por: ${TEST_ID}                                      ║${NC}"
echo -e "${GREEN}║  Pressione Ctrl+C para parar                                   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Tail dos logs (filtrando por eventos relevantes)
gcloud beta run services logs tail ${SERVICE_NAME} \
  --region=${REGION} \
  --format="value(textPayload)" \
  | grep --line-buffered -E "${TEST_ID}|agrupada recebida|mensagens na fila|Executando verificar|Aguardando mais mensagens|SmartContextManager ativado|Batch.*publicado|Worker|Mensagem recebida|Atendente:|Cliente encontrado|Task criada|OpenAI" \
  || echo -e "${YELLOW}ℹ️  Nenhum log relevante ainda (aguardando processamento)...${NC}"

echo ""
echo -e "${GREEN}✅ Monitoramento finalizado${NC}"
