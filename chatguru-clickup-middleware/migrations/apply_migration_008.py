#!/usr/bin/env python3
"""
Script para aplicar Migration 008: Correção da lógica de mapeamento Info_1 vs Info_2
"""

import subprocess
import sys
import json
import re

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

def apply_migration_008():
    """Aplica a Migration 008 para corrigir a lógica de mapeamento."""
    
    print("🚀 Aplicando Migration 008: Correção da lógica de mapeamento")
    print("=" * 60)
    print("📝 Corrigindo lógica: Info_1 (responsável) → Space, Info_2 (cliente) → Folder")
    
    # Ler o arquivo SQL da migração
    try:
        with open('008_fix_mapping_logic.sql', 'r', encoding='utf-8') as f:
            sql_content = f.read()
    except FileNotFoundError:
        print("❌ Arquivo 008_fix_mapping_logic.sql não encontrado!")
        return False
    
    # Dividir em statements
    statements = split_sql_statements(sql_content)
    
    print(f"📊 Total de statements a executar: {len(statements)}")
    print("=" * 60)
    
    # URL da API do middleware
    url = "https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app"
    
    executed = 0
    errors = []
    
    for idx, stmt in enumerate(statements, 1):
        print(f"🔄 Executando statement {idx}/{len(statements)}...")
        
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
            print(f"Raw response: {result.stdout[:200]}")
    
    print("=" * 60)
    print(f"✅ Executados: {executed}/{len(statements)}")
    print(f"❌ Erros: {len(errors)}")
    
    if errors:
        print("\n⚠️ ERROS:")
        for error in errors[:10]:  # Mostrar apenas os primeiros 10
            print(f"  - {error}")
    else:
        print("\n🎉 LÓGICA CORRIGIDA COM SUCESSO!")
        print("   Info_1 (responsável) → Determina SPACE via attendant_mappings")
        print("   Info_2 (cliente) → Determina FOLDER via client_mappings")
        print("\n🔄 Reinicie a aplicação Rust para aplicar as mudanças no código.")
    
    return len(errors) == 0

if __name__ == "__main__":
    success = apply_migration_008()
    sys.exit(0 if success else 1)