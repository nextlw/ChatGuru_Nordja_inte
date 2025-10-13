#!/bin/bash
# Aplicar Migration 010 usando psql via stdin

set -e

echo "🚀 Aplicando Migration 010..."

# Ler a migration e aplicar via gcloud sql connect
gcloud sql connect chatguru-middleware-db \
  --user=postgres \
  --database=chatguru_middleware \
  --project=buzzlightear \
  --quiet < chatguru-clickup-middleware/migrations/010_populate_mappings_with_aliases.sql

echo "✅ Migration aplicada com sucesso!"
