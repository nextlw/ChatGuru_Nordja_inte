#!/bin/bash

set -e

echo "🚀 Iniciando teste local do worker..."
echo ""

# Verificar se o servidor já está rodando
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "⚠️  Servidor já está rodando na porta 8080"
    echo "   Usando servidor existente..."
else
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
            cat /tmp/worker_test.log
            kill $SERVER_PID 2>/dev/null || true
            exit 1
        fi
        sleep 0.5
    done
fi

echo ""
echo "🧪 Preparando payload de teste..."

# Payload de exemplo do ChatGuru
CHATGURU_PAYLOAD='{
  "campanha_id": "",
  "campanha_nome": "",
  "origem": "",
  "email": "558586736498",
  "nome": "Leo de Sa",
  "tags": [],
  "texto_mensagem": "teste de integração local",
  "campos_personalizados": {},
  "bot_context": {},
  "responsavel_nome": "",
  "responsavel_email": "",
  "link_chat": "https://s15.chatguru.app/chats#test123",
  "celular": "558586736498",
  "phone_id": "62558780e2923cc4705beee1",
  "chat_id": "test123",
  "chat_created": "2025-10-07 18:57:35.545000",
  "datetime_post": "2025-10-07 13:59:12.808810",
  "tipo_mensagem": "chat"
}'

# Criar envelope interno (como o webhook cria)
INNER_ENVELOPE=$(cat <<EOF
{
  "raw_payload": $(echo "$CHATGURU_PAYLOAD" | jq -Rs .),
  "received_at": "$(date -u +%Y-%m-%dT%H:%M:%S.000000Z)",
  "source": "chatguru-webhook"
}
EOF
)

# Encode em base64 (como o Pub/Sub faz)
DATA_BASE64=$(echo -n "$INNER_ENVELOPE" | base64)

# Criar envelope do Pub/Sub
PUBSUB_ENVELOPE=$(cat <<EOF
{
  "message": {
    "data": "$DATA_BASE64",
    "messageId": "test-message-id",
    "publishTime": "$(date -u +%Y-%m-%dT%H:%M:%S.000000Z)"
  },
  "subscription": "projects/buzzlightear/subscriptions/chatguru-webhook-worker-sub"
}
EOF
)

echo ""
echo "📤 Enviando requisição para o worker..."
echo ""

# Enviar para o worker local
RESPONSE=$(curl -X POST http://localhost:8080/worker/process \
  -H "Content-Type: application/json" \
  -d "$PUBSUB_ENVELOPE" \
  -w "\nHTTP_CODE:%{http_code}" \
  -s)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d')

echo "📥 Resposta do servidor:"
echo "   Status: $HTTP_CODE"
echo "   Body: $BODY"
echo ""

# Verificar resultado
if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Teste PASSOU! Worker processou com sucesso"
    EXIT_CODE=0
else
    echo "❌ Teste FALHOU! Esperado status 200, recebido $HTTP_CODE"
    EXIT_CODE=1
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

exit $EXIT_CODE
