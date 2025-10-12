#!/usr/bin/env python3
"""
Script para executar migração 003 via Cloud SQL Admin API
"""
import subprocess
import sys

def run_migration():
    # Ler o arquivo SQL
    with open('003_fix_fallback_config.sql', 'r') as f:
        sql_content = f.read()

    # Limpar comentários e espaços extras
    lines = []
    for line in sql_content.split('\n'):
        line = line.strip()
        if line and not line.startswith('--'):
            lines.append(line)

    sql_clean = ' '.join(lines)

    # Executar via gcloud (usando echo e pipe)
    cmd = f'''echo "{sql_clean}" | gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres --project=buzzlightear --quiet 2>&1'''

    print(f"Executando migração via gcloud sql connect...")
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

    print("STDOUT:", result.stdout)
    print("STDERR:", result.stderr)
    print("Return code:", result.returncode)

    return result.returncode == 0

if __name__ == '__main__':
    success = run_migration()
    sys.exit(0 if success else 1)
