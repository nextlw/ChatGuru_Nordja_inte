#!/usr/bin/env python3
"""
Aplicar Migration 010 no Cloud SQL via connection string
"""

import os
import sys
import psycopg2
from pathlib import Path

def main():
    # Ler migration SQL
    migration_file = Path(__file__).parent / "010_populate_mappings_with_aliases.sql"

    with open(migration_file, 'r') as f:
        migration_sql = f.read()

    # Connection string do Cloud SQL
    # Formato: postgresql://USER:PASSWORD@/DATABASE?host=/cloudsql/PROJECT:REGION:INSTANCE
    conn_str = os.getenv('DATABASE_URL')

    if not conn_str:
        print("❌ DATABASE_URL não configurada")
        print("Configure com:")
        print("export DATABASE_URL='postgresql://postgres:SENHA@/chatguru_middleware?host=/cloudsql/buzzlightear:southamerica-east1:chatguru-middleware-db'")
        sys.exit(1)

    print("🚀 Aplicando Migration 010...")
    print(f"📡 Conectando ao banco...")

    try:
        conn = psycopg2.connect(conn_str)
        cur = conn.cursor()

        # Executar migration
        cur.execute(migration_sql)
        conn.commit()

        # Verificar resultado
        cur.execute("SELECT COUNT(*) FROM attendant_mappings")
        attendant_count = cur.fetchone()[0]

        cur.execute("SELECT COUNT(*) FROM client_mappings")
        client_count = cur.fetchone()[0]

        print(f"✅ Migration 010 aplicada com sucesso!")
        print(f"   - Attendants: {attendant_count}")
        print(f"   - Clients: {client_count}")

        cur.close()
        conn.close()

    except Exception as e:
        print(f"❌ Erro ao aplicar migration: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
