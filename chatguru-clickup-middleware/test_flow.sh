#!/bin/bash

# Script de teste de produção do fluxo ChatGuru -> ClickUp
# Este script simula cenários reais de produção

echo "=================================================="
echo "TESTE DE PRODUÇÃO - CHATGURU CLICKUP MIDDLEWARE"
echo "=================================================="
echo ""

# Detectar se está rodando local ou produção
if [[ "$1" == "prod" ]]; then
    BASE_URL="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"
    echo "🚀 Modo: PRODUÇÃO (Google Cloud Run)"
else
    BASE_URL="http://localhost:8080"
    echo "💻 Modo: LOCAL (desenvolvimento)"
fi

echo "📍 URL: $BASE_URL"
echo ""

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Função para fazer request e mostrar resultado
test_webhook() {
    local test_name="$1"
    local payload="$2"
    local expected_behavior="$3"
    
    echo -e "${BLUE}=================================================="
    echo -e "TESTE: $test_name"
    echo -e "==================================================${NC}"
    echo -e "${YELLOW}Comportamento esperado:${NC} $expected_behavior"
    echo ""
    
    # Log do payload apenas em modo verbose
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${YELLOW}Payload enviado:${NC}"
        echo "$payload" | jq '.'
        echo ""
    fi
    
    echo -e "${YELLOW}Enviando requisição...${NC}"
    
    response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$payload" \
        "$BASE_URL/webhooks/chatguru")
    
    # Verificar resposta
    if echo "$response" | jq -e '.message == "Success"' > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Resposta: Success${NC}"
    else
        echo -e "${RED}✗ Erro na resposta:${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
    fi
    
    # Aguardar processamento
    sleep 3
    echo ""
}

# Verificar saúde do servidor primeiro
echo -e "${YELLOW}Verificando status do servidor...${NC}"
health_check=$(curl -s "$BASE_URL/health" | jq -r '.status' 2>/dev/null)
if [[ "$health_check" == "healthy" ]]; then
    echo -e "${GREEN}✓ Servidor está saudável${NC}"
else
    echo -e "${RED}✗ Servidor não está respondendo corretamente${NC}"
    exit 1
fi
echo ""

# Teste 1: Atividade válida - Pedido de compra (formato Generic)
echo -e "${GREEN}>>> TESTE 1: PEDIDO DE COMPRA - PANELAS${NC}"
test_webhook \
    "Compra de Panelas de Ferro" \
    '{
        "celular": "+5511999998888",
        "nome": "João Silva - Restaurante",
        "mensagem": "Preciso comprar urgente 10 panelas de ferro fundido profissionais para o restaurante. Podem pesquisar fornecedores e enviar orçamentos com fotos dos modelos disponíveis?",
        "campanha": "WhatsApp Business",
        "origem": "ChatGuru",
        "extra": {
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
            "chat_id": "prod_test_001"
        }
    }' \
    "✓ Criar tarefa categoria Compras/Pesquisas"

# Teste 2: Modificação da tarefa anterior
echo -e "${GREEN}>>> TESTE 2: MODIFICAÇÃO DA TAREFA ANTERIOR${NC}"
test_webhook \
    "Mudança de ideia sobre panelas" \
    '{
        "conversation": {
            "id": "conv_123",
            "phone": "5511999998888"
        },
        "message": {
            "id": "msg_002",
            "text": "Na verdade, mudei de ideia. Prefiro panelas de cerâmica ao invés de ferro fundido",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "João Silva",
            "phone": "5511999998888"
        }
    }' \
    "Deve atualizar a tarefa existente (se estiver dentro do tempo limite)"

# Teste 3: Não é atividade - Conversa casual
echo -e "${GREEN}>>> TESTE 3: NÃO É ATIVIDADE - CONVERSA CASUAL${NC}"
test_webhook \
    "Conversa casual" \
    '{
        "conversation": {
            "id": "conv_124",
            "phone": "5511888887777"
        },
        "message": {
            "id": "msg_003",
            "text": "Bom dia! Como você está hoje?",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Maria Santos",
            "phone": "5511888887777"
        }
    }' \
    "Não deve criar tarefa - mensagem de saudação"

# Teste 4: Atividade de Agendamento
echo -e "${GREEN}>>> TESTE 4: ATIVIDADE DE AGENDAMENTO${NC}"
test_webhook \
    "Agendamento médico" \
    '{
        "conversation": {
            "id": "conv_125",
            "phone": "5511777776666"
        },
        "message": {
            "id": "msg_004",
            "text": "Preciso agendar uma consulta com o Dr. Carlos para dia 02 de dezembro às 16:05hs",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Pedro Oliveira",
            "phone": "5511777776666"
        }
    }' \
    "Deve criar tarefa com categoria Agendamento"

# Teste 5: Atividade de Reembolso
echo -e "${GREEN}>>> TESTE 5: ATIVIDADE DE REEMBOLSO MÉDICO${NC}"
test_webhook \
    "Reembolso de plano de saúde" \
    '{
        "conversation": {
            "id": "conv_126",
            "phone": "5511666665555"
        },
        "message": {
            "id": "msg_005",
            "text": "Preciso do reembolso da consulta médica. Vou enviar a nota fiscal para vocês processarem",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Ana Costa",
            "phone": "5511666665555"
        }
    }' \
    "Deve criar tarefa com categoria Plano de Saúde ou similar"

# Teste 6: Atividade de Pagamento
echo -e "${GREEN}>>> TESTE 6: ATIVIDADE DE PAGAMENTO${NC}"
test_webhook \
    "Solicitação de pagamento" \
    '{
        "conversation": {
            "id": "conv_127",
            "phone": "5511555554444"
        },
        "message": {
            "id": "msg_006",
            "text": "Preciso pagar a conta de luz que vence amanhã. Podem enviar o código de barras?",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Carlos Mendes",
            "phone": "5511555554444"
        }
    }' \
    "Deve criar tarefa com categoria Pagamentos"

# Teste 7: Atividade de Logística
echo -e "${GREEN}>>> TESTE 7: ATIVIDADE DE LOGÍSTICA${NC}"
test_webhook \
    "Solicitação de entrega" \
    '{
        "conversation": {
            "id": "conv_128",
            "phone": "5511444443333"
        },
        "message": {
            "id": "msg_007",
            "text": "Preciso enviar um documento urgente para o escritório central. Podem chamar um motoboy?",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Lucia Ferreira",
            "phone": "5511444443333"
        }
    }' \
    "Deve criar tarefa com categoria Logística"

# Teste 8: Não é atividade - Agradecimento
echo -e "${GREEN}>>> TESTE 8: NÃO É ATIVIDADE - AGRADECIMENTO${NC}"
test_webhook \
    "Agradecimento simples" \
    '{
        "conversation": {
            "id": "conv_129",
            "phone": "5511333332222"
        },
        "message": {
            "id": "msg_008",
            "text": "Muito obrigado pela ajuda!",
            "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'"
        },
        "contact": {
            "name": "Roberto Lima",
            "phone": "5511333332222"
        }
    }' \
    "Não deve criar tarefa - mensagem de agradecimento"

echo ""
echo -e "${GREEN}=================================================="
echo -e "TESTES CONCLUÍDOS"
echo -e "==================================================${NC}"
echo ""
echo "Verifique os logs do servidor para ver:"
echo "1. Como as anotações foram formatadas"
echo "2. Se as tarefas foram criadas no ClickUp"
echo "3. Se as classificações estão corretas"
echo "4. Se o ConversationTracker está funcionando"