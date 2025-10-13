#!/bin/bash

# Script para aplicar Migration 010 via Cloud SQL Proxy
# Uso: DB_PASSWORD='senha' ./apply_migration_010_via_proxy.sh

echo "🚀 Iniciando aplicação da Migration 010..."

# Adicionar Cloud SQL Proxy ao PATH
export PATH="/opt/homebrew/share/google-cloud-sdk/bin:$PATH"

# Verificar se cloud_sql_proxy existe
if ! command -v cloud_sql_proxy &> /dev/null; then
    echo "❌ cloud_sql_proxy não encontrado no PATH"
    echo "Certifique-se de que o Google Cloud SDK está instalado"
    exit 1
fi

# Iniciar Cloud SQL Proxy em background
echo "🔌 Iniciando Cloud SQL Proxy..."
cloud_sql_proxy -instances=buzzlightear:southamerica-east1:chatguru-middleware-db=tcp:9470 -credential_file=$HOME/.config/gcloud/legacy_credentials/voilaassist@gmail.com/adc.json &
PROXY_PID=$!

# Aguardar proxy estar pronto
echo "⏳ Aguardando proxy estar pronto..."
sleep 5

# Aplicar migração via psql
echo "📋 Aplicando Migration 010..."
cd "$(dirname "$0")"
PGPASSWORD="${DB_PASSWORD:-0Djn3a5CGGn7u1jTbsO0ZFRmXxo3idd+}" psql -h 127.0.0.1 -p 9470 -U postgres -d chatguru_middleware -f 010_populate_mappings_with_aliases.sql

# Salvar resultado
MIGRATION_RESULT=$?

# Parar Cloud SQL Proxy
echo "🛑 Parando Cloud SQL Proxy..."
kill $PROXY_PID 2>/dev/null || true

if [ $MIGRATION_RESULT -eq 0 ]; then
    echo "✅ Migration 010 aplicada com sucesso!"
    echo ""
    echo "📊 Verificando dados inseridos..."

    # Reconectar para verificar
    cloud_sql_proxy -instances=buzzlightear:southamerica-east1:chatguru-middleware-db=tcp:9470 -credential_file=$HOME/.config/gcloud/legacy_credentials/voilaassist@gmail.com/adc.json &
    PROXY_PID=$!
    sleep 3

    PGPASSWORD="${DB_PASSWORD:-0Djn3a5CGGn7u1jTbsO0ZFRmXxo3idd+}" psql -h 127.0.0.1 -p 9470 -U postgres -d chatguru_middleware << 'EOSQL'
SELECT 'Attendants: ' || COUNT(*)::text as info FROM attendant_mappings
UNION ALL
SELECT 'Clients: ' || COUNT(*)::text FROM client_mappings;

-- Mostrar alguns exemplos de aliases
SELECT 'Exemplo de aliases:' as info;
SELECT attendant_key, attendant_aliases FROM attendant_mappings LIMIT 3;
EOSQL

    kill $PROXY_PID 2>/dev/null || true
else
    echo "❌ Erro ao aplicar Migration 010"
    exit 1
fi

echo ""
echo "🎯 Próximos passos:"
echo "   1. Deploy da nova imagem com logs de debug"
echo "   2. Testar com webhook real contendo 'william duarte' + 'anne'"
echo "   3. Verificar nos logs se aliases são encontrados"
