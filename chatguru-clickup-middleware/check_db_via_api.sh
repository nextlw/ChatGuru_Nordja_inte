#!/bin/bash
# Script para verificar o banco de dados via endpoint /admin/db-check

set -e

echo "🔍 Verificando banco de dados via API..."
echo "=================================================="

# Opção 1: Usar serviço local (se estiver rodando)
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ Serviço local detectado em http://localhost:8080"
    SERVICE_URL="http://localhost:8080"
else
    # Opção 2: Usar Cloud Run
    echo "🌐 Buscando URL do Cloud Run..."
    SERVICE_URL=$(gcloud run services describe chatguru-clickup-middleware \
        --region=southamerica-east1 \
        --format='value(status.url)' \
        --project=buzzlightear 2>/dev/null || echo "")

    if [ -z "$SERVICE_URL" ]; then
        echo "❌ Erro: Serviço não encontrado"
        echo ""
        echo "Para verificar localmente:"
        echo "1. Execute: cd chatguru-clickup-middleware && cargo run"
        echo "2. Execute: ./check_db_via_api.sh"
        exit 1
    fi

    echo "✅ Cloud Run URL: $SERVICE_URL"
fi

echo ""
echo "📊 Consultando endpoint: $SERVICE_URL/admin/db-check"
echo "=================================================="
echo ""

# Fazer requisição
response=$(curl -s "$SERVICE_URL/admin/db-check")

# Verificar se jq está instalado
if command -v jq &> /dev/null; then
    # Exibir resposta formatada com jq
    echo "$response" | jq .

    # Exibir resumo
    echo ""
    echo "=================================================="
    echo "📋 RESUMO:"
    echo "=================================================="

    status=$(echo "$response" | jq -r '.status')
    total_spaces=$(echo "$response" | jq -r '.summary.total_spaces')
    active_spaces=$(echo "$response" | jq -r '.summary.active_spaces')
    total_folders=$(echo "$response" | jq -r '.summary.total_folders')
    active_folders=$(echo "$response" | jq -r '.summary.active_folders')
    total_lists=$(echo "$response" | jq -r '.summary.total_lists')
    active_lists=$(echo "$response" | jq -r '.summary.active_lists')

    echo "Status: $status"
    echo ""
    echo "Spaces:  $active_spaces/$total_spaces ativos"
    echo "Folders: $active_folders/$total_folders ativos"
    echo "Lists:   $active_lists/$total_lists ativos"
    echo ""

    # Verificar espaços faltando
    missing_spaces=$(echo "$response" | jq -r '.spaces.missing | length')
    if [ "$missing_spaces" -gt 0 ]; then
        echo "⚠️  Espaços faltando no banco:"
        echo "$response" | jq -r '.spaces.missing[]' | sed 's/^/  - /'
    else
        echo "✅ Todos os espaços esperados estão no banco!"
    fi
else
    # Se jq não estiver instalado, exibir JSON cru
    echo "$response"
    echo ""
    echo "=================================================="
    echo "💡 Instale 'jq' para visualização formatada:"
    echo "   brew install jq"
    echo "=================================================="
fi

echo ""
