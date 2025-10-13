#!/usr/bin/env python3
"""
Script para sincronizar dados do ClickUp para o banco PostgreSQL
Busca spaces, folders e lists do ClickUp e popula as tabelas corretas
"""

import requests
import psycopg2
import json
from datetime import datetime
import sys

# Configurações
CLICKUP_API_TOKEN = "106092691_5a823a64061246a2fb9498f37fecb77540478eb28423d778f37aabcafbd602b9"
TEAM_ID = "9013037641"
DB_CONFIG = {
    'host': '127.0.0.1',
    'port': '9470',
    'user': 'postgres',
    'password': '0Djn3a5CGGn7u1jTbsO0ZFRmXxo3idd+',
    'database': 'chatguru_middleware'
}

def get_clickup_data(url):
    """Busca dados da API do ClickUp"""
    headers = {
        'Authorization': f'Bearer {CLICKUP_API_TOKEN}',
        'Content-Type': 'application/json'
    }
    
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    return response.json()

def main():
    print("🔄 Iniciando sincronização dos dados do ClickUp...")
    
    # Conectar ao banco
    try:
        conn = psycopg2.connect(**DB_CONFIG)
        cursor = conn.cursor()
        print("✅ Conectado ao banco PostgreSQL")
    except Exception as e:
        print(f"❌ Erro ao conectar ao banco: {e}")
        return
    
    spaces_synced = 0
    folders_synced = 0
    lists_synced = 0
    
    try:
        # 1. Buscar todos os spaces
        print(f"📦 Buscando spaces do team {TEAM_ID}...")
        spaces_url = f"https://api.clickup.com/api/v2/team/{TEAM_ID}/space?archived=false"
        spaces_data = get_clickup_data(spaces_url)
        
        print(f"Encontrados {len(spaces_data['spaces'])} spaces")
        
        # 2. Para cada space, inserir/atualizar no banco
        for space in spaces_data['spaces']:
            print(f"📦 Sincronizando space: {space['name']} (ID: {space['id']})")
            
            # Inserir/atualizar space
            cursor.execute("""
                INSERT INTO spaces (space_id, space_name, team_id, raw_data, synced_at)
                VALUES (%s, %s, %s, %s, NOW())
                ON CONFLICT (space_id)
                DO UPDATE SET
                    space_name = EXCLUDED.space_name,
                    synced_at = NOW(),
                    updated_at = NOW()
            """, (space['id'], space['name'], TEAM_ID, json.dumps(space)))
            
            spaces_synced += 1
            
            # 3. Buscar folders do space
            folders_url = f"https://api.clickup.com/api/v2/space/{space['id']}/folder?archived=false"
            try:
                folders_data = get_clickup_data(folders_url)
                print(f"   Encontradas {len(folders_data['folders'])} folders")
                
                # 4. Para cada folder, inserir/atualizar
                for folder in folders_data['folders']:
                    print(f"   📁 Sincronizando folder: {folder['name']} (ID: {folder['id']})")
                    
                    task_count = folder.get('task_count', 0)
                    if isinstance(task_count, str):
                        try:
                            task_count = int(task_count)
                        except:
                            task_count = 0
                    
                    cursor.execute("""
                        INSERT INTO folders (folder_id, folder_name, space_id, is_hidden, is_archived, task_count, raw_data, synced_at)
                        VALUES (%s, %s, %s, %s, %s, %s, %s, NOW())
                        ON CONFLICT (folder_id)
                        DO UPDATE SET
                            folder_name = EXCLUDED.folder_name,
                            is_hidden = EXCLUDED.is_hidden,
                            is_archived = EXCLUDED.is_archived,
                            task_count = EXCLUDED.task_count,
                            synced_at = NOW(),
                            updated_at = NOW()
                    """, (
                        folder['id'], 
                        folder['name'], 
                        space['id'],
                        folder.get('hidden', False),
                        folder.get('archived', False),
                        task_count,
                        json.dumps(folder)
                    ))
                    
                    folders_synced += 1
                    
                    # 5. Buscar lists do folder
                    lists_url = f"https://api.clickup.com/api/v2/folder/{folder['id']}/list?archived=false"
                    try:
                        lists_data = get_clickup_data(lists_url)
                        print(f"      Encontradas {len(lists_data['lists'])} lists")
                        
                        for list_item in lists_data['lists']:
                            print(f"      📋 Sincronizando list: {list_item['name']} (ID: {list_item['id']})")
                            
                            list_task_count = list_item.get('task_count', 0)
                            if isinstance(list_task_count, str):
                                try:
                                    list_task_count = int(list_task_count)
                                except:
                                    list_task_count = 0
                            
                            cursor.execute("""
                                INSERT INTO lists (list_id, list_name, folder_id, space_id, is_archived, task_count, raw_data, synced_at)
                                VALUES (%s, %s, %s, %s, %s, %s, %s, NOW())
                                ON CONFLICT (list_id)
                                DO UPDATE SET
                                    list_name = EXCLUDED.list_name,
                                    is_archived = EXCLUDED.is_archived,
                                    task_count = EXCLUDED.task_count,
                                    synced_at = NOW(),
                                    updated_at = NOW()
                            """, (
                                list_item['id'],
                                list_item['name'],
                                folder['id'],
                                space['id'],
                                list_item.get('archived', False),
                                list_task_count,
                                json.dumps(list_item)
                            ))
                            
                            lists_synced += 1
                            
                    except Exception as e:
                        print(f"      ⚠️  Erro ao buscar lists do folder {folder['id']}: {e}")
                        
            except Exception as e:
                print(f"   ⚠️  Erro ao buscar folders do space {space['id']}: {e}")
            
            # 6. Buscar lists "folderless" (sem pasta) do space
            folderless_url = f"https://api.clickup.com/api/v2/space/{space['id']}/list?archived=false"
            try:
                folderless_data = get_clickup_data(folderless_url)
                print(f"   Encontradas {len(folderless_data['lists'])} lists folderless")
                
                for list_item in folderless_data['lists']:
                    print(f"   📋 Sincronizando folderless list: {list_item['name']} (ID: {list_item['id']})")
                    
                    list_task_count = list_item.get('task_count', 0)
                    if isinstance(list_task_count, str):
                        try:
                            list_task_count = int(list_task_count)
                        except:
                            list_task_count = 0
                    
                    cursor.execute("""
                        INSERT INTO lists (list_id, list_name, folder_id, space_id, is_folderless, is_archived, task_count, raw_data, synced_at)
                        VALUES (%s, %s, NULL, %s, true, %s, %s, %s, NOW())
                        ON CONFLICT (list_id)
                        DO UPDATE SET
                            list_name = EXCLUDED.list_name,
                            is_archived = EXCLUDED.is_archived,
                            task_count = EXCLUDED.task_count,
                            synced_at = NOW(),
                            updated_at = NOW()
                    """, (
                        list_item['id'],
                        list_item['name'],
                        space['id'],
                        list_item.get('archived', False),
                        list_task_count,
                        json.dumps(list_item)
                    ))
                    
                    lists_synced += 1
                    
            except Exception as e:
                print(f"   ⚠️  Erro ao buscar folderless lists do space {space['id']}: {e}")
        
        # Commit das alterações
        conn.commit()
        
        print(f"""
✅ Sincronização concluída com sucesso!
   📦 Spaces sincronizados: {spaces_synced}
   📁 Folders sincronizados: {folders_synced}
   📋 Lists sincronizadas: {lists_synced}
   🔢 Total de itens: {spaces_synced + folders_synced + lists_synced}
""")
        
    except Exception as e:
        print(f"❌ Erro durante a sincronização: {e}")
        conn.rollback()
        return 1
    
    finally:
        cursor.close()
        conn.close()
    
    return 0

if __name__ == "__main__":
    sys.exit(main())