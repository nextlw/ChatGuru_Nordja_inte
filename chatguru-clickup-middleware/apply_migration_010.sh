#!/bin/bash
# Aplicar Migration 010 diretamente via gcloud sql execute-sql

set -e

PROJECT_ID="buzzlightear"
INSTANCE_NAME="chatguru-middleware-db"
DATABASE_NAME="chatguru_middleware"

echo "🚀 Aplicando Migration 010 no Cloud SQL..."
echo "📡 Instance: ${INSTANCE_NAME}"
echo "🗄️  Database: ${DATABASE_NAME}"
echo ""

# Executar migration via gcloud sql execute-sql
gcloud sql execute-sql "${INSTANCE_NAME}" \
  --project="${PROJECT_ID}" \
  --database="${DATABASE_NAME}" \
  --query-file="chatguru-clickup-middleware/migrations/010_populate_mappings_with_aliases.sql"

echo ""
echo "✅ Migration 010 aplicada com sucesso!"
echo ""
echo "📊 Verificando dados inseridos..."

# Verificar counts
gcloud sql execute-sql "${INSTANCE_NAME}" \
  --project="${PROJECT_ID}" \
  --database="${DATABASE_NAME}" \
  --query="SELECT 'Attendants: ' || COUNT(*)::text FROM attendant_mappings UNION ALL SELECT 'Clients: ' || COUNT(*)::text FROM client_mappings;"

echo ""
echo "✅ Concluído!"
