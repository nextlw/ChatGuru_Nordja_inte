#!/bin/bash
# Script para aplicar migrações SQL via psql local ou API
set -e

echo "🔧 Aplicando migrações SQL ao banco de dados..."
echo "=================================================="

# Opção 1: usar gcloud sql connect (requer acesso IPv4)
# Opção 2: usar psql com DATABASE_URL
# Opção 3: executar SQL via script Python

MIGRATIONS=(
  "001_create_tables.sql"
  "002_populate_initial.sql"
  "003_fix_fallback_config.sql"
  "004_add_missing_attendants.sql"
  "005_update_space_ids.sql"
  "006_create_system_config.sql"
)

echo "📁 Migrações a serem aplicadas:"
for migration in "${MIGRATIONS[@]}"; do
  echo "  - $migration"
done

echo ""
echo "📊 Método: Usar migrations/FULL_MIGRATION_ALL.sql (todas as migrações combinadas)"
echo ""

# Criar script Python para aplicar via API ou conexão direta
cat > /tmp/apply_migrations.py <<'PYTHON'
#!/usr/bin/env python3
import psycopg2
import os
import sys

# Tentar obter DATABASE_URL do ambiente ou usar padrão
DATABASE_URL = os.getenv("DATABASE_URL")

if not DATABASE_URL:
    print("❌ DATABASE_URL não definida")
    print("")
    print("Para aplicar via Cloud SQL Proxy:")
    print("  1. Inicie o proxy:")
    print("     cloud-sql-proxy buzzlightear:southamerica-east1:chatguru-middleware-db")
    print("  2. Defina DATABASE_URL:")
    print("     export DATABASE_URL='postgresql://postgres:SENHA@localhost:5432/chatguru_middleware'")
    print("  3. Execute novamente este script")
    sys.exit(1)

print(f"✅ Conectando ao banco: {DATABASE_URL.split('@')[1] if '@' in DATABASE_URL else 'local'}")

try:
    conn = psycopg2.connect(DATABASE_URL)
    cursor = conn.cursor()

    # Ler e executar migration completa
    migration_file = "migrations/FULL_MIGRATION_ALL.sql"
    print(f"📖 Lendo {migration_file}...")

    with open(migration_file, "r") as f:
        sql = f.read()

    print("⚙️  Executando migrações...")
    cursor.execute(sql)
    conn.commit()

    print("✅ Migrações aplicadas com sucesso!")

    # Verificar tabelas criadas
    cursor.execute("""
        SELECT table_name FROM information_schema.tables
        WHERE table_schema = 'public'
        ORDER BY table_name;
    """)
    tables = cursor.fetchall()

    print(f"\n📋 Tabelas no banco ({len(tables)}):")
    for table in tables:
        print(f"  ✓ {table[0]}")

    cursor.close()
    conn.close()

except Exception as e:
    print(f"❌ Erro: {e}")
    sys.exit(1)
PYTHON

chmod +x /tmp/apply_migrations.py

echo "🚀 Executando script Python..."
python3 /tmp/apply_migrations.py
