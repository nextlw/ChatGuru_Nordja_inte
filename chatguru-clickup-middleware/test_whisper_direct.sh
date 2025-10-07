#!/bin/bash

set -e

echo "🎤 Teste direto: Whisper API com arquivo .ogg"
echo ""

# Verificar se a chave OpenAI está configurada
OPENAI_KEY=$(gcloud secrets versions access latest --secret="openai-api-key" --project=buzzlightear 2>/dev/null || echo "")

if [ -z "$OPENAI_KEY" ]; then
    echo "❌ Erro: Chave OpenAI não encontrada"
    echo "   Configure com: gcloud secrets versions access latest --secret=\"openai-api-key\""
    exit 1
fi

echo "✅ Chave OpenAI encontrada"
echo ""

# Arquivo de áudio
AUDIO_FILE="WhatsApp Ptt 2025-10-07 at 14.19.26.ogg"

if [ ! -f "$AUDIO_FILE" ]; then
    echo "❌ Erro: Arquivo de áudio não encontrado: $AUDIO_FILE"
    exit 1
fi

echo "📁 Arquivo encontrado: $AUDIO_FILE ($(ls -lh "$AUDIO_FILE" | awk '{print $5}'))"
echo ""
echo "🔄 Enviando para Whisper API..."

# Chamar API do Whisper
RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
  https://api.openai.com/v1/audio/transcriptions \
  -H "Authorization: Bearer $OPENAI_KEY" \
  -F "file=@$AUDIO_FILE" \
  -F model=whisper-1 \
  -F language=pt \
  -F response_format=text)

HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
BODY=$(echo "$RESPONSE" | sed '/HTTP_CODE:/d')

echo ""
echo "📥 Resposta da API:"
echo "   Status: $HTTP_CODE"
echo ""

if [ "$HTTP_CODE" = "200" ]; then
    echo "✅ Transcrição bem-sucedida!"
    echo ""
    echo "📝 Texto transcrito:"
    echo "-----------------------------------"
    echo "$BODY"
    echo "-----------------------------------"
    echo ""
    echo "📊 Estatísticas:"
    echo "   - Tamanho: $(echo "$BODY" | wc -c) caracteres"
    echo "   - Palavras: $(echo "$BODY" | wc -w) palavras"
    exit 0
else
    echo "❌ Erro na transcrição!"
    echo "   Resposta:"
    echo "$BODY"
    exit 1
fi
