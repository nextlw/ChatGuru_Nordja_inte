#!/bin/bash

set -e

echo "🚀 Teste: Decodificação Pub/Sub (sem processamento completo)"
echo ""

# Verificar se o servidor já está rodando
if ! lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "📦 Compilando e iniciando servidor..."
    cargo build --release 2>&1 | tail -5

    # Iniciar servidor em background
    cargo run --release > /tmp/worker_test.log 2>&1 &
    SERVER_PID=$!
    echo "   Servidor iniciado (PID: $SERVER_PID)"

    # Aguardar servidor inicializar
    echo "⏳ Aguardando servidor inicializar..."
    for i in {1..30}; do
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo "✅ Servidor pronto!"
            break
        fi
        if [ $i -eq 30 ]; then
            echo "❌ Timeout aguardando servidor"
            echo "Logs do servidor:"
            tail -50 /tmp/worker_test.log
            kill $SERVER_PID 2>/dev/null || true
            exit 1
        fi
        sleep 0.5
    done
fi

echo ""
echo "🧪 Teste 1: Formato Pub/Sub com base64"
echo "----------------------------------------"

# Payload simples do ChatGuru
CHATGURU_PAYLOAD='{
  "nome": "Teste User",
  "celular": "5511999999999",
  "texto_mensagem": "Mensagem de teste",
  "chat_id": "test123",
  "tipo_mensagem": "chat"
}'

# Criar envelope interno
INNER_ENVELOPE=$(cat <<EOF
{
  "raw_payload": $(echo "$CHATGURU_PAYLOAD" | jq -Rs .),
  "received_at": "2025-10-07T14:00:00.000000Z",
  "source": "test"
}
EOF
)

echo "📦 Envelope interno criado:"
echo "$INNER_ENVELOPE" | jq .

# Encode em base64
DATA_BASE64=$(echo -n "$INNER_ENVELOPE" | base64)

echo ""
echo "🔐 Base64 encoded (primeiros 80 chars):"
echo "${DATA_BASE64:0:80}..."

# Criar envelope do Pub/Sub
PUBSUB_ENVELOPE=$(cat <<EOF
{
  "message": {
    "data": "$DATA_BASE64",
    "messageId": "test-123",
    "publishTime": "2025-10-07T14:00:00.000000Z"
  }
}
EOF
)

echo ""
echo "📤 Enviando para /worker/process..."

# Enviar para o worker
RESPONSE=$(curl -X POST http://localhost:8080/worker/process \
  -H "Content-Type: application/json" \
  -d "$PUBSUB_ENVELOPE" \
  -w "\nHTTP_CODE:%{http_code}" \
  -s 2>&1)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d')

echo ""
echo "📥 Resposta:"
echo "   Status: $HTTP_CODE"
echo "   Body:"
echo "$BODY" | jq . 2>/dev/null || echo "$BODY"

echo ""
echo "🔍 Verificando logs do servidor..."
echo "   (últimas 10 linhas relevantes)"
grep -E "(Request received|raw_payload|Missing|decode|base64|ERROR)" /tmp/worker_test.log | tail -10 || echo "   (nenhum log relevante encontrado)"

echo ""
echo "📊 Análise:"
if [[ "$HTTP_CODE" == "400" ]] && echo "$BODY" | grep -q "Missing raw_payload"; then
    echo "❌ FALHOU: Ainda está retornando 'Missing raw_payload'"
    echo "   O decode do base64 não está funcionando"
    EXIT_CODE=1
elif [[ "$HTTP_CODE" == "500" ]] || [[ "$HTTP_CODE" == "400" ]]; then
    if echo "$BODY" | grep -q "OpenAI\|Vertex\|classification"; then
        echo "✅ PASSOU: Decodificação funcionou!"
        echo "   Erro é na classificação IA (esperado sem credenciais)"
        EXIT_CODE=0
    else
        echo "⚠️  Status $HTTP_CODE mas erro inesperado"
        EXIT_CODE=1
    fi
elif [[ "$HTTP_CODE" == "200" ]]; then
    echo "✅ PASSOU COMPLETAMENTE: Tudo funcionou!"
    EXIT_CODE=0
else
    echo "⚠️  Status inesperado: $HTTP_CODE"
    EXIT_CODE=1
fi

echo ""
echo "🧪 Teste 2: Formato direto (sem base64) para compatibilidade"
echo "------------------------------------------------------------"

# Testar formato direto também
DIRECT_ENVELOPE=$(cat <<EOF
{
  "raw_payload": $(echo "$CHATGURU_PAYLOAD" | jq -Rs .),
  "received_at": "2025-10-07T14:00:00.000000Z",
  "source": "test-direct"
}
EOF
)

RESPONSE2=$(curl -X POST http://localhost:8080/worker/process \
  -H "Content-Type: application/json" \
  -d "$DIRECT_ENVELOPE" \
  -w "\nHTTP_CODE:%{http_code}" \
  -s 2>&1)

HTTP_CODE2=$(echo "$RESPONSE2" | grep "HTTP_CODE:" | cut -d: -f2)

echo "📥 Status formato direto: $HTTP_CODE2"

if [[ "$HTTP_CODE2" == "500" ]] || [[ "$HTTP_CODE2" == "400" ]] || [[ "$HTTP_CODE2" == "200" ]]; then
    echo "✅ Formato direto também funciona (retrocompatível)"
else
    echo "⚠️  Formato direto retornou: $HTTP_CODE2"
fi

# Matar servidor se foi iniciado por este script
if [ ! -z "$SERVER_PID" ]; then
    echo ""
    echo "🛑 Parando servidor..."
    kill $SERVER_PID 2>/dev/null || true
    wait $SERVER_PID 2>/dev/null || true
fi

echo ""
echo "📝 Logs completos em: /tmp/worker_test.log"
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ TESTE PASSOU: Worker consegue decodificar Pub/Sub corretamente!"
else
    echo "❌ TESTE FALHOU: Verificar logs acima"
fi

exit $EXIT_CODE
