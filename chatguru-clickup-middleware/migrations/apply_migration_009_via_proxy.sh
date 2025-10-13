#!/bin/bash

# Script para aplicar Migration 009 via Cloud SQL Proxy
# Uso: ./apply_migration_009_via_proxy.sh

echo "🚀 Iniciando aplicação da Migration 009..."

# Adicionar Cloud SQL Proxy ao PATH
export PATH="/opt/homebrew/share/google-cloud-sdk/bin:$PATH"

# Iniciar Cloud SQL Proxy em background
echo "🔌 Iniciando Cloud SQL Proxy..."
cloud_sql_proxy -instances=buzzlightear:southamerica-east1:chatguru-middleware-db=tcp:9470 -credential_file=$HOME/.config/gcloud/legacy_credentials/voilaassist@gmail.com/adc.json &
PROXY_PID=$!

# Aguardar proxy estar pronto
echo "⏳ Aguardando proxy estar pronto..."
sleep 5

# Aplicar migração via psql
echo "📋 Aplicando Migration 009..."
PGPASSWORD="${DB_PASSWORD:-}" psql -h 127.0.0.1 -p 9470 -U postgres -d chatguru_middleware -f 009_correct_mapping_logic.sql

# Salvar resultado
MIGRATION_RESULT=$?

# Parar Cloud SQL Proxy
echo "🛑 Parando Cloud SQL Proxy..."
kill $PROXY_PID

if [ $MIGRATION_RESULT -eq 0 ]; then
    echo "✅ Migration 009 aplicada com sucesso!"
else
    echo "❌ Erro ao aplicar Migration 009"
    exit 1
fi

echo "🎯 Para testar o fluxo completo:"
echo "   1. Faça deploy da aplicação com as correções"
echo "   2. Envie um payload de teste via webhook"
echo "   3. Verifique se responsavel_nome → Space, Info_2 → Folder"