#!/bin/bash

# ============================================
# Script de Teste para Diálogos ChatGuru
# ============================================

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configurações do ChatGuru - SUBSTITUA COM SUAS CREDENCIAIS
API_KEY="${CHATGURU_API_KEY:-TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK}"
ACCOUNT_ID="${CHATGURU_ACCOUNT_ID:-625584ce6fdcb7bda7d94aa8}"
PHONE_ID="${CHATGURU_PHONE_ID:-6537de23b6d5b0bb0b80421a}"
CHAT_NUMBER="${TEST_PHONE:-5585989530473}"
BASE_URL="https://s15.chatguru.app/api/v1"
WEBHOOK_URL="${WEBHOOK_URL:-https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru}"

# Timestamp para logs
TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

echo -e "${CYAN}╔════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   TESTE DE DIÁLOGOS CHATGURU VIA CLI          ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════╝${NC}"
echo ""

# Verificar se as credenciais estão configuradas
if [ "$API_KEY" = "sua_api_key_aqui" ]; then
    echo -e "${RED}❌ ERRO: Configure suas credenciais do ChatGuru!${NC}"
    echo -e "   Exporte as variáveis de ambiente:"
    echo -e "   ${YELLOW}export CHATGURU_API_KEY='sua_chave'${NC}"
    echo -e "   ${YELLOW}export CHATGURU_ACCOUNT_ID='seu_id'${NC}"
    echo -e "   ${YELLOW}export CHATGURU_PHONE_ID='seu_phone_id'${NC}"
    echo -e "   ${YELLOW}export TEST_PHONE='5511999999999'${NC}"
    exit 1
fi

# Função para testar diálogo
test_dialog() {
    local dialog_id=$1
    local description=$2
    
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Testando Diálogo: ${dialog_id}${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "Descrição: ${description}"
    echo -e "Timestamp: ${TIMESTAMP}"
    echo ""
    
    # Criar payload JSON
    local payload=$(cat <<EOF
{
    "chat_number": "${CHAT_NUMBER}",
    "dialog_id": "${dialog_id}",
    "variables": {
        "tarefa": "Teste via CLI - ${dialog_id} - ${TIMESTAMP}",
        "prioridade": "Alta",
        "responsavel": "Sistema de Teste",
        "descricao": "${description}"
    },
    "key": "${API_KEY}",
    "account_id": "${ACCOUNT_ID}",
    "phone_id": "${PHONE_ID}"
}
EOF
)
    
    echo -e "${CYAN}📤 Enviando requisição...${NC}"
    echo -e "URL: ${BASE_URL}/dialog_execute"
    echo -e "Payload:"
    echo "$payload" | jq . 2>/dev/null || echo "$payload"
    echo ""
    
    # Executar chamada
    response=$(curl -s -X POST "${BASE_URL}/dialog_execute" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    # Verificar resposta
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Resposta recebida:${NC}"
        echo "$response" | jq . 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Erro ao executar diálogo${NC}"
    fi
    
    echo ""
}

# Função para testar webhook diretamente
test_webhook() {
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Testando Webhook Diretamente${NC}"
    echo -e "${BLUE}=========================================${NC}"
    echo -e "URL: ${WEBHOOK_URL}"
    echo ""
    
    # Criar payload do webhook
    local webhook_payload=$(cat <<EOF
{
    "event_type": "task_created",
    "id": "test_$(date +%s)",
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "data": {
        "chat_number": "${CHAT_NUMBER}",
        "message": "Teste direto do webhook - ${TIMESTAMP}",
        "custom_fields": {
            "tarefa": "Tarefa de teste via webhook direto",
            "prioridade": "Alta",
            "responsavel": "Sistema"
        }
    }
}
EOF
)
    
    echo -e "${CYAN}🔧 Enviando webhook...${NC}"
    echo "Payload:"
    echo "$webhook_payload" | jq . 2>/dev/null || echo "$webhook_payload"
    echo ""
    
    # Executar chamada
    response=$(curl -s -X POST "${WEBHOOK_URL}" \
        -H "Content-Type: application/json" \
        -d "$webhook_payload")
    
    # Verificar resposta
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Webhook processado:${NC}"
        echo "$response" | jq . 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Erro ao testar webhook${NC}"
    fi
    
    echo ""
}

# Função para adicionar nota
add_note() {
    local note_content=$1
    
    echo -e "${BLUE}=========================================${NC}"
    echo -e "${YELLOW}Adicionando Anotação${NC}"
    echo -e "${BLUE}=========================================${NC}"
    
    local payload=$(cat <<EOF
{
    "chat_number": "${CHAT_NUMBER}",
    "note": "${note_content}",
    "key": "${API_KEY}",
    "account_id": "${ACCOUNT_ID}",
    "phone_id": "${PHONE_ID}"
}
EOF
)
    
    echo -e "${CYAN}📝 Enviando anotação...${NC}"
    
    response=$(curl -s -X POST "${BASE_URL}/note_add" \
        -H "Content-Type: application/json" \
        -d "$payload")
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Anotação adicionada${NC}"
        echo "$response" | jq . 2>/dev/null || echo "$response"
    else
        echo -e "${RED}❌ Erro ao adicionar anotação${NC}"
    fi
    
    echo ""
}

# Menu principal
show_menu() {
    echo -e "${CYAN}Escolha uma opção:${NC}"
    echo "1) Testar TESTE_API"
    echo "2) Testar nova_api"
    echo "3) Testar ambos os diálogos"
    echo "4) Testar webhook diretamente"
    echo "5) Adicionar anotação de teste"
    echo "6) Teste completo (todos os itens)"
    echo "7) Configurar credenciais"
    echo "0) Sair"
    echo ""
}

# Função para configurar credenciais
configure_credentials() {
    echo -e "${YELLOW}Configure suas credenciais:${NC}"
    read -p "API Key: " API_KEY
    read -p "Account ID: " ACCOUNT_ID
    read -p "Phone ID: " PHONE_ID
    read -p "Número WhatsApp para teste (com código do país): " CHAT_NUMBER
    
    echo ""
    echo -e "${GREEN}Credenciais configuradas para esta sessão.${NC}"
    echo -e "Para torná-las permanentes, adicione ao seu ~/.bashrc ou ~/.zshrc:"
    echo -e "${CYAN}export CHATGURU_API_KEY='${API_KEY}'${NC}"
    echo -e "${CYAN}export CHATGURU_ACCOUNT_ID='${ACCOUNT_ID}'${NC}"
    echo -e "${CYAN}export CHATGURU_PHONE_ID='${PHONE_ID}'${NC}"
    echo -e "${CYAN}export TEST_PHONE='${CHAT_NUMBER}'${NC}"
    echo ""
}

# Loop principal
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Uso: $0 [opção]"
    echo ""
    echo "Opções:"
    echo "  1 - Testar TESTE_API"
    echo "  2 - Testar nova_api"
    echo "  3 - Testar ambos"
    echo "  4 - Testar webhook"
    echo "  5 - Adicionar nota"
    echo "  6 - Teste completo"
    echo ""
    echo "Variáveis de ambiente:"
    echo "  CHATGURU_API_KEY    - Chave da API"
    echo "  CHATGURU_ACCOUNT_ID - ID da conta"
    echo "  CHATGURU_PHONE_ID   - ID do telefone"
    echo "  TEST_PHONE          - Número para teste"
    exit 0
fi

# Se passou argumento, executar direto
if [ -n "$1" ]; then
    case $1 in
        1) test_dialog "TESTE_API" "Testando diálogo TESTE_API" ;;
        2) test_dialog "nova_api" "Testando diálogo nova_api" ;;
        3) 
            test_dialog "TESTE_API" "Testando diálogo TESTE_API"
            sleep 2
            test_dialog "nova_api" "Testando diálogo nova_api"
            ;;
        4) test_webhook ;;
        5) add_note "Anotação de teste - $TIMESTAMP" ;;
        6)
            test_dialog "TESTE_API" "Teste completo - TESTE_API"
            sleep 2
            test_dialog "nova_api" "Teste completo - nova_api"
            sleep 2
            test_webhook
            sleep 1
            add_note "Anotação do teste completo - $TIMESTAMP"
            ;;
        *) echo -e "${RED}Opção inválida${NC}" ;;
    esac
    exit 0
fi

# Menu interativo
while true; do
    show_menu
    read -p "Opção: " choice
    
    case $choice in
        1) test_dialog "TESTE_API" "Testando diálogo TESTE_API" ;;
        2) test_dialog "nova_api" "Testando diálogo nova_api" ;;
        3) 
            test_dialog "TESTE_API" "Testando diálogo TESTE_API"
            sleep 2
            test_dialog "nova_api" "Testando diálogo nova_api"
            ;;
        4) test_webhook ;;
        5) 
            read -p "Digite o conteúdo da anotação: " note_text
            add_note "$note_text"
            ;;
        6)
            echo -e "${CYAN}Executando teste completo...${NC}"
            test_dialog "TESTE_API" "Teste completo - TESTE_API"
            sleep 2
            test_dialog "nova_api" "Teste completo - nova_api"
            sleep 2
            test_webhook
            sleep 1
            add_note "Anotação do teste completo - $TIMESTAMP"
            echo -e "${GREEN}Teste completo concluído!${NC}"
            ;;
        7) configure_credentials ;;
        0) 
            echo -e "${GREEN}Saindo...${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Opção inválida!${NC}"
            ;;
    esac
    
    echo ""
    read -p "Pressione Enter para continuar..."
    echo ""
done