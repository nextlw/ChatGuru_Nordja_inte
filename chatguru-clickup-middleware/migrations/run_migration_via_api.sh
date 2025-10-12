#!/bin/bash
# Script para executar migração via endpoint admin (se existir)

SERVICE_URL="https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"

echo "Tentando executar migração via API do Cloud Run..."

# Criar payload com comandos SQL
SQL_COMMANDS=$(cat 003_fix_fallback_config.sql | grep -v '^--' | grep -v '^$' | tr '\n' ' ')

echo "SQL Commands preparados:"
echo "$SQL_COMMANDS"

echo ""
echo "NOTA: Se não existir endpoint /admin/migrate, você precisará:"
echo "1. Acessar Cloud Console: https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear"
echo "2. Clicar em 'OPEN CLOUD SHELL'"
echo "3. Executar: gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres"
echo "4. Colar os comandos SQL do arquivo: EXECUTE_MIGRATION_003_MANUAL.md"
