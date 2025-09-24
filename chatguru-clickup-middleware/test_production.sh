#!/bin/bash

# Script de teste de produção do fluxo ChatGuru -> ClickUp
# Simula cenários reais com payloads no formato correto

echo "=================================================="
echo "🚀 TESTE DE PRODUÇÃO - CHATGURU CLICKUP MIDDLEWARE"
echo "=================================================="
echo ""

# Configuração do ambiente
if [[ "$1" == "prod" ]]; then
    BASE_URL="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"
    echo "📍 Ambiente: PRODUÇÃO (Google Cloud Run)"
    DELAY=15  # Maior delay em produção
else
    BASE_URL="http://localhost:8080"
    echo "📍 Ambiente: LOCAL (desenvolvimento)"
    DELAY=5   # Menor delay local
fi

echo "🔗 URL: $BASE_URL"
echo "⏱️  Delay entre testes: ${DELAY}s"
echo ""

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Contador de sucessos/falhas
SUCCESS_COUNT=0
FAILURE_COUNT=0

# Função para fazer request e mostrar resultado
test_webhook() {
    local test_num="$1"
    local test_name="$2"
    local payload="$3"
    local expected="$4"
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "📋 TESTE #${test_num}: ${test_name}"
    echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}📍 Expectativa:${NC} $expected"
    echo ""
    
    # Enviar requisição
    echo -e "${YELLOW}📤 Enviando requisição...${NC}"
    
    response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "$BASE_URL/webhooks/chatguru" 2>&1)
    
    # Verificar resposta
    if echo "$response" | jq -e '.message == "Success"' > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Status: SUCCESS${NC}"
        ((SUCCESS_COUNT++))
    else
        echo -e "${RED}❌ Status: FAILED${NC}"
        echo -e "${RED}Resposta: $response${NC}"
        ((FAILURE_COUNT++))
    fi
    
    echo ""
    echo -e "${PURPLE}⏳ Aguardando ${DELAY}s para processamento...${NC}"
    sleep $DELAY
    echo ""
}

# Verificar saúde do servidor
echo -e "${YELLOW}🏥 Verificando saúde do servidor...${NC}"
health_response=$(curl -s "$BASE_URL/health" 2>&1)

if echo "$health_response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Servidor está saudável${NC}"
    version=$(echo "$health_response" | jq -r '.version' 2>/dev/null)
    echo -e "${BLUE}📌 Versão: $version${NC}"
else
    echo -e "${RED}❌ Servidor não está respondendo corretamente${NC}"
    echo "$health_response"
    exit 1
fi
echo ""

# ==================== INÍCIO DOS TESTES ====================

echo -e "${PURPLE}🔬 INICIANDO BATERIA DE TESTES DE PRODUÇÃO${NC}"
echo ""

# TESTE 1: Compra de Material
test_webhook 1 "COMPRA DE MATERIAL" \
'{
    "celular": "+5511999998888",
    "nome": "João Silva - Restaurante Premium",
    "mensagem": "Urgente! Preciso comprar 10 panelas de ferro fundido profissionais, 5 frigideiras antiaderentes e 3 conjuntos de facas. Por favor, pesquisem fornecedores e enviem orçamentos com fotos.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "priority": "high",
        "category": "restaurant_supplies"
    }
}' \
"Criar tarefa categoria COMPRAS com urgência alta"

# TESTE 2: Agendamento Médico
test_webhook 2 "AGENDAMENTO MÉDICO" \
'{
    "celular": "+5511888887777",
    "nome": "Maria Santos",
    "mensagem": "Preciso agendar consulta com o Dr. Carlos Mendes, neurologista, para dia 15 de janeiro às 14:30. É retorno da consulta anterior.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "type": "medical_appointment"
    }
}' \
"Criar tarefa categoria AGENDAMENTO"

# TESTE 3: Reembolso Plano de Saúde
test_webhook 3 "REEMBOLSO MÉDICO" \
'{
    "celular": "+5511777776666",
    "nome": "Ana Costa - Unimed",
    "mensagem": "Solicito reembolso da consulta médica realizada dia 10/12. Valor: R$ 450,00. Vou enviar a nota fiscal e recibo médico por email.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "value": 450.00,
        "type": "reimbursement"
    }
}' \
"Criar tarefa categoria PLANO DE SAÚDE"

# TESTE 4: Logística - Entrega Urgente
test_webhook 4 "LOGÍSTICA URGENTE" \
'{
    "celular": "+5511666665555",
    "nome": "Pedro Oliveira - Escritório Central",
    "mensagem": "Preciso enviar documentos urgentes para o escritório da Av. Paulista 1000. Chamem um motoboy para retirada imediata. Os documentos precisam chegar até 16h.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "delivery_type": "express",
        "deadline": "16:00"
    }
}' \
"Criar tarefa categoria LOGÍSTICA com prioridade"

# TESTE 5: Pagamento de Conta
test_webhook 5 "PAGAMENTO DE CONTA" \
'{
    "celular": "+5511555554444",
    "nome": "Carlos Mendes",
    "mensagem": "Preciso pagar a conta de luz que vence amanhã. Valor R$ 342,50. Podem enviar o código de barras ou PIX?",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "bill_type": "electricity",
        "amount": 342.50
    }
}' \
"Criar tarefa categoria PAGAMENTOS"

# TESTE 6: Pesquisa de Fornecedores
test_webhook 6 "PESQUISA/ORÇAMENTO" \
'{
    "celular": "+5511444443333",
    "nome": "Lucia Ferreira - Eventos",
    "mensagem": "Preciso de orçamento para decoração de casamento para 200 convidados, dia 15 de março. Tema: Garden Party. Incluir flores, iluminação e mobiliário.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "event_date": "2025-03-15",
        "guests": 200
    }
}' \
"Criar tarefa categoria PESQUISAS/ORÇAMENTOS"

# TESTE 7: Viagem Corporativa
test_webhook 7 "VIAGEM CORPORATIVA" \
'{
    "celular": "+5511333332222",
    "nome": "Roberto Lima - Diretor Comercial",
    "mensagem": "Preciso de passagens aéreas São Paulo - Rio de Janeiro, ida dia 20/01 volta 22/01, voo pela manhã. Hotel próximo ao centro de convenções.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "travel_type": "business",
        "dates": "20-22/01"
    }
}' \
"Criar tarefa categoria VIAGENS"

# TESTE 8: Documentação
test_webhook 8 "DOCUMENTAÇÃO" \
'{
    "celular": "+5511222221111",
    "nome": "Patricia Souza",
    "mensagem": "Preciso da segunda via do contrato de locação e do comprovante de pagamento do IPTU dos últimos 3 meses.",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru",
    "extra": {
        "document_type": "contract"
    }
}' \
"Criar tarefa categoria DOCUMENTOS"

# TESTE 9: NÃO É ATIVIDADE - Saudação
test_webhook 9 "SAUDAÇÃO (NÃO-ATIVIDADE)" \
'{
    "celular": "+5511111110000",
    "nome": "Cliente Teste",
    "mensagem": "Bom dia! Como vocês estão?",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru"
}' \
"NÃO criar tarefa - apenas saudação"

# TESTE 10: NÃO É ATIVIDADE - Agradecimento
test_webhook 10 "AGRADECIMENTO (NÃO-ATIVIDADE)" \
'{
    "celular": "+5511000009999",
    "nome": "Cliente Satisfeito",
    "mensagem": "Muito obrigado pela ajuda! Vocês são ótimos!",
    "campanha": "WhatsApp Business",
    "origem": "ChatGuru"
}' \
"NÃO criar tarefa - apenas agradecimento"

# ==================== RELATÓRIO FINAL ====================

echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "📊 RELATÓRIO FINAL"
echo -e "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

TOTAL=$((SUCCESS_COUNT + FAILURE_COUNT))
if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$((SUCCESS_COUNT * 100 / TOTAL))
else
    SUCCESS_RATE=0
fi

echo -e "${GREEN}✅ Sucessos: $SUCCESS_COUNT${NC}"
echo -e "${RED}❌ Falhas: $FAILURE_COUNT${NC}"
echo -e "${BLUE}📈 Taxa de sucesso: ${SUCCESS_RATE}%${NC}"
echo ""

if [ $FAILURE_COUNT -eq 0 ]; then
    echo -e "${GREEN}🎉 TODOS OS TESTES PASSARAM COM SUCESSO!${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠️  Alguns testes falharam. Verifique os logs para mais detalhes.${NC}"
    exit 1
fi