#!/usr/bin/env python3
"""
Aplica migrations via API, dividindo SQL em statements inteligentemente.
Respeita blocos de funções PL/pgSQL (CREATE FUNCTION ... $$ ... $$).
"""
import re
import subprocess
import sys
import json

def split_sql_statements(sql_content):
    """
    Divide SQL em statements respeitando:
    - Dollar-quoted strings ($$)
    - Funções PL/pgSQL
    - Triggers
    """
    statements = []
    current = []
    in_dollar_quote = False
    dollar_tag = None

    lines = sql_content.split('\n')

    for line in lines:
        stripped = line.strip()

        # Ignorar linhas vazias e comentários standalone
        if not stripped or stripped.startswith('--'):
            continue

        # Detectar início de dollar-quoted string
        dollar_match = re.search(r'\$(\w*)\$', line)
        if dollar_match and not in_dollar_quote:
            in_dollar_quote = True
            dollar_tag = dollar_match.group(0)
            current.append(line)
            continue

        # Detectar fim de dollar-quoted string
        if in_dollar_quote and dollar_tag and dollar_tag in line:
            current.append(line)
            # Verificar se tem ; após o $$
            if ';' in line.split(dollar_tag)[-1]:
                statements.append('\n'.join(current))
                current = []
                in_dollar_quote = False
                dollar_tag = None
            continue

        # Se estiver dentro de dollar quote, apenas acumular
        if in_dollar_quote:
            current.append(line)
            continue

        # Fora de dollar quote, procurar por ;
        current.append(line)
        if ';' in line:
            statements.append('\n'.join(current))
            current = []

    # Se sobrou algo no buffer
    if current:
        statements.append('\n'.join(current))

    return [s.strip() for s in statements if s.strip()]

def execute_migration():
    # Ler arquivo SQL
    with open('FULL_MIGRATION_ALL.sql', 'r') as f:
        sql_content = f.read()

    # Dividir em statements
    statements = split_sql_statements(sql_content)

    print(f"📊 Total de statements a executar: {len(statements)}")
    print("=" * 80)

    # Executar via psql direto no Cloud SQL (usando a conexão Unix socket do Cloud Run)
    # Como não temos acesso direto, vamos usar a API do Cloud SQL Admin

    # Alternativamente, podemos fazer statement por statement via nossa API
    url = "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"

    executed = 0
    errors = []

    for idx, stmt in enumerate(statements, 1):
        # Escapar para JSON
        stmt_escaped = stmt.replace('"', '\\"').replace('\n', '\\n')

        # Fazer POST para endpoint customizado
        cmd = f'''curl -s -X POST {url}/admin/execute-sql \\
            -H "Content-Type: application/json" \\
            -d '{{"sql": "{stmt_escaped}"}}'
        '''

        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)

        try:
            response = json.loads(result.stdout)
            if response.get('status') == 'success':
                executed += 1
                print(f"✅ Statement {idx}/{len(statements)}: OK")
            else:
                error_msg = response.get('error', 'Unknown error')
                errors.append(f"Statement {idx}: {error_msg}")
                print(f"❌ Statement {idx}/{len(statements)}: {error_msg[:100]}")
        except json.JSONDecodeError:
            errors.append(f"Statement {idx}: Invalid JSON response")
            print(f"⚠️ Statement {idx}/{len(statements)}: Response não é JSON")

    print("=" * 80)
    print(f"✅ Executados: {executed}/{len(statements)}")
    print(f"❌ Erros: {len(errors)}")

    if errors:
        print("\n⚠️ ERROS:")
        for error in errors[:10]:  # Mostrar apenas os primeiros 10
            print(f"  - {error}")

    return len(errors) == 0

if __name__ == '__main__':
    success = execute_migration()
    sys.exit(0 if success else 1)
