#!/bin/bash

# Script de Testes Automatizados para o Middleware ChatGuru-ClickUp
# Uso: ./run-tests.sh [URL_DO_SERVICO]

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Função para imprimir com cor
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Verificar se a URL foi fornecida
if [ -z "$1" ]; then
    print_warning "URL não fornecida. Use: ./run-tests.sh https://your-service-url.run.app"
    print_warning "Aguardando conclusão do deploy para obter a URL..."
    exit 1
fi

SERVICE_URL="$1"
FAILED_TESTS=0
PASSED_TESTS=0

echo "========================================="
echo "   Testes do Middleware ChatGuru-ClickUp"
echo "========================================="
echo ""
echo "URL do Serviço: $SERVICE_URL"
echo ""

# Teste 1: Health Check
echo "1. Testando Health Check..."
if curl -s -f "$SERVICE_URL/health" > /dev/null 2>&1; then
    print_status "Health check OK"
    ((PASSED_TESTS++))
else
    print_error "Health check FALHOU"
    ((FAILED_TESTS++))
fi

# Teste 2: Ready Check
echo ""
echo "2. Testando Ready Check..."
if curl -s -f "$SERVICE_URL/ready" > /dev/null 2>&1; then
    print_status "Ready check OK"
    ((PASSED_TESTS++))
else
    print_error "Ready check FALHOU"
    ((FAILED_TESTS++))
fi

# Teste 3: Status Check
echo ""
echo "3. Testando Status Endpoint..."
RESPONSE=$(curl -s -w "\n%{http_code}" "$SERVICE_URL/status" 2>/dev/null)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    print_status "Status endpoint OK (HTTP 200)"
    ((PASSED_TESTS++))
else
    print_error "Status endpoint FALHOU (HTTP $HTTP_CODE)"
    ((FAILED_TESTS++))
fi

# Teste 4: Conexão com ClickUp
echo ""
echo "4. Testando Conexão com ClickUp..."
RESPONSE=$(curl -s -w "\n%{http_code}" "$SERVICE_URL/clickup/test" 2>/dev/null)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    print_status "Conexão com ClickUp OK"
    ((PASSED_TESTS++))
else
    print_error "Conexão com ClickUp FALHOU (HTTP $HTTP_CODE)"
    ((FAILED_TESTS++))
fi

# Teste 5: Listar Info da Lista
echo ""
echo "5. Testando Endpoint de Lista do ClickUp..."
RESPONSE=$(curl -s -w "\n%{http_code}" "$SERVICE_URL/clickup/list" 2>/dev/null)
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" = "200" ]; then
    print_status "Endpoint de lista OK"
    ((PASSED_TESTS++))
else
    print_error "Endpoint de lista FALHOU (HTTP $HTTP_CODE)"
    ((FAILED_TESTS++))
fi

# Teste 6: Webhook ChatGuru (teste básico)
echo ""
echo "6. Testando Webhook ChatGuru..."
RESPONSE=$(curl -s -X POST "$SERVICE_URL/webhooks/chatguru" \
  -H "Content-Type: application/json" \
  -d '{
    "event": "message.created",
    "message": {
      "text": "Teste automatizado",
      "sender": {
        "name": "Teste Script",
        "phone": "+5511999999999"
      }
    }
  }' -w "\n%{http_code}" 2>/dev/null)
  
HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ] || [ "$HTTP_CODE" = "202" ]; then
    print_status "Webhook processou evento OK"
    ((PASSED_TESTS++))
else
    print_error "Webhook FALHOU (HTTP $HTTP_CODE)"
    ((FAILED_TESTS++))
fi

# Teste 7: Performance - Tempo de Resposta
echo ""
echo "7. Testando Performance (tempo de resposta)..."
START_TIME=$(date +%s%N)
curl -s "$SERVICE_URL/health" > /dev/null 2>&1
END_TIME=$(date +%s%N)
ELAPSED=$((($END_TIME - $START_TIME) / 1000000))

if [ $ELAPSED -lt 500 ]; then
    print_status "Resposta rápida: ${ELAPSED}ms (< 500ms)"
    ((PASSED_TESTS++))
elif [ $ELAPSED -lt 1000 ]; then
    print_warning "Resposta moderada: ${ELAPSED}ms (< 1s)"
    ((PASSED_TESTS++))
else
    print_error "Resposta lenta: ${ELAPSED}ms (> 1s)"
    ((FAILED_TESTS++))
fi

# Resumo dos Testes
echo ""
echo "========================================="
echo "           RESUMO DOS TESTES"
echo "========================================="
echo ""
echo -e "${GREEN}Testes Aprovados:${NC} $PASSED_TESTS"
echo -e "${RED}Testes Falhados:${NC} $FAILED_TESTS"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ TODOS OS TESTES PASSARAM!${NC}"
    echo ""
    echo "O middleware está pronto para uso!"
    echo ""
    echo "Configure no ChatGuru:"
    echo "  URL: $SERVICE_URL/webhooks/chatguru"
    echo "  Método: POST"
    echo ""
    exit 0
else
    echo -e "${RED}❌ ALGUNS TESTES FALHARAM${NC}"
    echo ""
    echo "Verifique os logs com:"
    echo "  gcloud run services logs read chatguru-clickup-middleware \\"
    echo "    --project buzzlightear \\"
    echo "    --region southamerica-east1 \\"
    echo "    --limit 50"
    echo ""
    exit 1
fi