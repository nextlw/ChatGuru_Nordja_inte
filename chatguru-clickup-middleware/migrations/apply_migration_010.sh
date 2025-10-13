#!/bin/bash
# Aplicar Migration 010 via Cloud SQL Proxy

set -e

echo "🚀 Aplicando Migration 010..."

# Ler a migration
MIGRATION_SQL=$(cat migrations/010_populate_mappings_with_aliases.sql)

# Aplicar via psql (assumindo que há Cloud SQL Proxy rodando ou conexão configurada)
# Usando a connection string do ambiente
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL não configurada"
    echo "Configure com: export DATABASE_URL='postgresql://user:pass@host:port/dbname'"
    exit 1
fi

echo "📡 Conectando ao banco..."
psql "$DATABASE_URL" -c "$MIGRATION_SQL"

echo "✅ Migration 010 aplicada com sucesso!"
