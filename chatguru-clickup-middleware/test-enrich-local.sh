#!/bin/bash
# Teste local do Job de Enriquecimento
#
# Simula uma mensagem do Pub/Sub para testar o endpoint /enrich

set -e

PORT="${PORT:-8080}"
TASK_ID="${1:-}"

if [ -z "$TASK_ID" ]; then
    echo "❌ Uso: $0 <task_id>"
    echo ""
    echo "Exemplo:"
    echo "   $0 901322079100"
    exit 1
fi

# Criar log entry simulado
LOG_ENTRY="🎉 Task criada - ID: $TASK_ID"

# Codificar em base64
LOG_BASE64=$(echo -n "$LOG_ENTRY" | base64)

echo "🧪 Testando Job de Enriquecimento"
echo "   Task ID: $TASK_ID"
echo "   Log Entry: $LOG_ENTRY"
echo "   Base64: $LOG_BASE64"
echo ""

# Payload do Pub/Sub
PAYLOAD=$(cat <<EOF
{
  "message": {
    "data": "$LOG_BASE64",
    "messageId": "test-$(date +%s)",
    "attributes": {}
  }
}
EOF
)

echo "📤 Enviando para http://localhost:$PORT/enrich"
echo ""

curl -X POST "http://localhost:$PORT/enrich" \
    -H "Content-Type: application/json" \
    -d "$PAYLOAD" \
    -w "\n\nHTTP Status: %{http_code}\n" | jq . 2>/dev/null || cat

echo ""
echo "✅ Teste concluído"

