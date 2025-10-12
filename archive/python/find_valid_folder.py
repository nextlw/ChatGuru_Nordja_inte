#!/usr/bin/env python3
"""
Script para encontrar folders válidos no workspace ClickUp atual
e substituir o folder_id hardcoded inválido no código Rust.
"""

import requests
import json
import subprocess

def get_oauth_token():
    """Recupera o token OAuth2 do Google Secret Manager via gcloud"""
    try:
        project_id = "buzzlightear"
        secret_name = "clickup-oauth-token"
        
        # Usar gcloud CLI para acessar o secret
        cmd = [
            "gcloud", "secrets", "versions", "access", "latest",
            "--secret", secret_name,
            "--project", project_id
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        token = result.stdout.strip()
        print(f"✅ Token OAuth2 recuperado: {token[:20]}...")
        return token
    except subprocess.CalledProcessError as e:
        print(f"❌ Erro ao executar gcloud: {e}")
        print(f"❌ Stderr: {e.stderr}")
        return None
    except Exception as e:
        print(f"❌ Erro ao recuperar token: {e}")
        return None

def get_team_info(token):
    """Recupera informações do team/workspace"""
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    try:
        # Listar teams
        response = requests.get("https://api.clickup.com/api/v2/team", headers=headers)
        if response.status_code == 200:
            teams = response.json()["teams"]
            print(f"✅ Teams encontrados: {len(teams)}")
            for team in teams:
                print(f"  - Team ID: {team['id']}, Nome: {team['name']}")
            return teams[0] if teams else None
        else:
            print(f"❌ Erro ao listar teams: {response.status_code} - {response.text}")
            return None
    except Exception as e:
        print(f"❌ Erro na requisição: {e}")
        return None

def get_spaces_and_folders(token, team_id):
    """Recupera spaces e folders do team"""
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    try:
        # Listar spaces
        response = requests.get(f"https://api.clickup.com/api/v2/team/{team_id}/space", headers=headers)
        if response.status_code == 200:
            spaces = response.json()["spaces"]
            print(f"✅ Spaces encontrados: {len(spaces)}")
            
            all_folders = []
            for space in spaces:
                print(f"\n📁 Space: {space['name']} (ID: {space['id']})")
                
                # Listar folders do space
                folders_response = requests.get(f"https://api.clickup.com/api/v2/space/{space['id']}/folder", headers=headers)
                if folders_response.status_code == 200:
                    folders = folders_response.json()["folders"]
                    print(f"  └── Folders: {len(folders)}")
                    
                    for folder in folders:
                        print(f"    📂 {folder['name']} (ID: {folder['id']})")
                        all_folders.append({
                            "space_name": space['name'],
                            "space_id": space['id'],
                            "folder_name": folder['name'],
                            "folder_id": folder['id']
                        })
                        
                        # Testar se consegue acessar o folder
                        test_response = requests.get(f"https://api.clickup.com/api/v2/folder/{folder['id']}", headers=headers)
                        if test_response.status_code == 200:
                            print(f"      ✅ Folder acessível")
                        else:
                            print(f"      ❌ Folder inacessível: {test_response.status_code}")
                else:
                    print(f"  └── ❌ Erro ao listar folders: {folders_response.status_code}")
            
            return all_folders
        else:
            print(f"❌ Erro ao listar spaces: {response.status_code} - {response.text}")
            return []
    except Exception as e:
        print(f"❌ Erro na requisição: {e}")
        return []

def test_folder_permissions(token, folder_id):
    """Testa se consegue criar uma lista no folder"""
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    test_list_data = {
        "name": "TEST_LIST_DELETE_ME",
        "content": "Lista de teste - pode deletar"
    }
    
    try:
        # Tentar criar lista
        response = requests.post(f"https://api.clickup.com/api/v2/folder/{folder_id}/list", 
                               headers=headers, json=test_list_data)
        
        if response.status_code == 200:
            list_id = response.json()["id"]
            print(f"  ✅ Consegue criar listas (lista teste criada: {list_id})")
            
            # Deletar lista de teste
            delete_response = requests.delete(f"https://api.clickup.com/api/v2/list/{list_id}", headers=headers)
            if delete_response.status_code == 200:
                print(f"  🗑️ Lista de teste deletada")
            
            return True
        else:
            print(f"  ❌ Não consegue criar listas: {response.status_code} - {response.text}")
            return False
    except Exception as e:
        print(f"  ❌ Erro ao testar permissões: {e}")
        return False

def suggest_best_folder(folders):
    """Sugere o melhor folder para usar como 'Clientes Inativos'"""
    print("\n🎯 SUGESTÕES DE FOLDERS PARA 'CLIENTES INATIVOS':")
    
    # Procurar folders que já tenham nome similar
    candidates = []
    for folder in folders:
        name_lower = folder['folder_name'].lower()
        if any(word in name_lower for word in ['inativo', 'cliente', 'geral', 'misc', 'outros']):
            candidates.append(folder)
    
    if candidates:
        print("\n🔍 Folders com nomes similares encontrados:")
        for candidate in candidates:
            print(f"  📂 {candidate['folder_name']} (ID: {candidate['folder_id']}) - Space: {candidate['space_name']}")
    
    # Se não encontrou candidatos, sugerir os primeiros folders válidos
    if not candidates and folders:
        print("\n📋 Primeiros folders disponíveis:")
        for i, folder in enumerate(folders[:3]):
            print(f"  {i+1}. 📂 {folder['folder_name']} (ID: {folder['folder_id']}) - Space: {folder['space_name']}")
    
    return candidates if candidates else folders[:3]

def main():
    print("🔍 DESCOBRINDO FOLDER VÁLIDO PARA CLIENTES INATIVOS")
    print("=" * 60)
    
    # 1. Recuperar token OAuth2
    token = get_oauth_token()
    if not token:
        return
    
    # 2. Obter informações do team
    team = get_team_info(token)
    if not team:
        return
    
    team_id = team['id']
    print(f"\n🏢 Usando Team: {team['name']} (ID: {team_id})")
    
    # 3. Listar spaces e folders
    folders = get_spaces_and_folders(token, team_id)
    if not folders:
        print("❌ Nenhum folder encontrado")
        return
    
    # 4. Testar permissões em alguns folders
    print(f"\n🧪 TESTANDO PERMISSÕES EM {min(5, len(folders))} FOLDERS:")
    valid_folders = []
    
    for i, folder in enumerate(folders[:5]):  # Testar apenas os primeiros 5
        print(f"\n📂 Testando: {folder['folder_name']} (ID: {folder['folder_id']})")
        if test_folder_permissions(token, folder['folder_id']):
            valid_folders.append(folder)
    
    # 5. Sugerir melhor folder
    if valid_folders:
        print(f"\n✅ FOLDERS VÁLIDOS ENCONTRADOS: {len(valid_folders)}")
        suggestions = suggest_best_folder(valid_folders)
        
        if suggestions:
            best_folder = suggestions[0]
            print(f"\n🎯 RECOMENDAÇÃO:")
            print(f"Usar folder: {best_folder['folder_name']}")
            print(f"Folder ID: {best_folder['folder_id']}")
            print(f"Space: {best_folder['space_name']}")
            
            print(f"\n📝 CÓDIGO PARA ATUALIZAR:")
            print(f"Em src/services/estrutura.rs, linha ~340:")
            print(f"const INACTIVE_CLIENTS_FOLDER_ID: &str = \"{best_folder['folder_id']}\";")
        
    else:
        print("\n❌ NENHUM FOLDER VÁLIDO ENCONTRADO")
        print("Verifique as permissões do token OAuth2")

if __name__ == "__main__":
    main()