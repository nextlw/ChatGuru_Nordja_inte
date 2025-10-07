#!/bin/bash

set -e

echo "🚀 Teste: Transcrição de Áudio com Whisper"
echo ""

# Verificar se o servidor já está rodando
if ! lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "📦 Compilando e iniciando servidor..."
    cargo build --release 2>&1 | tail -5

    # Iniciar servidor em background
    cargo run --release > /tmp/worker_audio_test.log 2>&1 &
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
            tail -50 /tmp/worker_audio_test.log
            kill $SERVER_PID 2>/dev/null || true
            exit 1
        fi
        sleep 0.5
    done
fi

echo ""
echo "🧪 Teste: Payload com áudio (simulando WhatsApp)"
echo "----------------------------------------"

# Payload do ChatGuru com áudio
CHATGURU_PAYLOAD='{
  "nome": "Leo de Sa",
  "celular": "558586736498",
  "texto_mensagem": "",
  "chat_id": "test_audio_123",
  "tipo_mensagem": "audio",
  "media_url": "https://s15.chatguru.app/media/audio_example.ogg",
  "media_type": "audio/ogg",
  "email": "558586736498",
  "origem": "whatsapp",
  "link_chat": "https://s15.chatguru.app/chats#test_audio_123",
  "phone_id": "62558780e2923cc4705beee1",
  "chat_created": "2025-10-07 18:57:35.545000",
  "datetime_post": "2025-10-07 14:19:12.808810"
}'

# Criar envelope interno
INNER_ENVELOPE=$(cat <<EOF
{
  "raw_payload": $(echo "$CHATGURU_PAYLOAD" | jq -Rs .),
  "received_at": "$(date -u +%Y-%m-%dT%H:%M:%S.000000Z)",
  "source": "test-audio"
}
EOF
)

echo "📦 Envelope interno criado"

# Encode em base64
DATA_BASE64=$(echo -n "$INNER_ENVELOPE" | base64)

# Criar envelope do Pub/Sub
PUBSUB_ENVELOPE=$(cat <<EOF
{
  "message": {
    "data": "$DATA_BASE64",
    "messageId": "test-audio-123",
    "publishTime": "$(date -u +%Y-%m-%dT%H:%M:%S.000000Z)"
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
echo "   (últimas 20 linhas relevantes)"
grep -E "(audio|transcri|Whisper|OpenAI|classificação)" /tmp/worker_audio_test.log | tail -20 || echo "   (nenhum log relevante encontrado)"

echo ""
echo "📊 Análise:"
if [[ "$HTTP_CODE" == "200" ]]; then
    echo "✅ PASSOU: Requisição processada com sucesso!"
    if echo "$BODY" | grep -qi "transcrição"; then
        echo "   ✨ Transcrição encontrada na anotação!"
    else
        echo "   ⚠️  Transcrição não encontrada na anotação"
    fi
    EXIT_CODE=0
elif [[ "$HTTP_CODE" == "500" ]] || [[ "$HTTP_CODE" == "400" ]]; then
    echo "❌ FALHOU: Erro no processamento"
    EXIT_CODE=1
else
    echo "⚠️  Status inesperado: $HTTP_CODE"
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
echo "📝 Logs completos em: /tmp/worker_audio_test.log"
echo ""

exit $EXIT_CODE
