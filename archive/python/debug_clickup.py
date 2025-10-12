#!/usr/bin/env python3
import requests
import os

# Carrega o token da variável de ambiente
TOKEN = os.getenv("clickup_api_token", "pk_106092691_TV05R7R")
BASE_URL = "https://api.clickup.com/api/v2"
headers = {"Authorization": f"Bearer {TOKEN}"}

print(f"🔧 Token sendo usado: {TOKEN[:20]}...")

def test_folder(folder_id):
    print(f"\n🔍 Testando Folder ID: {folder_id}")
    try:
        # Testa se o folder existe
        response = requests.get(f"{BASE_URL}/folder/{folder_id}", headers=headers, timeout=5)
        print(f"GET /folder/{folder_id} → Status: {response.status_code}")
        if response.status_code != 200:
            print(f"❌ Erro: {response.text}")
            return False
        
        # Testa se pode criar lista no folder
        response = requests.get(f"{BASE_URL}/folder/{folder_id}/list", headers=headers, timeout=5)
        print(f"GET /folder/{folder_id}/list → Status: {response.status_code}")
        if response.status_code != 200:
            print(f"❌ Erro: {response.text}")
            return False
        
        print("✅ Folder válido")
        return True
    except Exception as e:
        print(f"❌ Exceção: {e}")
        return False

def test_list(list_id):
    print(f"\n🔍 Testando List ID: {list_id}")
    try:
        response = requests.get(f"{BASE_URL}/list/{list_id}", headers=headers, timeout=5)
        print(f"GET /list/{list_id} → Status: {response.status_code}")
        if response.status_code != 200:
            print(f"❌ Erro: {response.text}")
            return False
        print("✅ Lista válida")
        return True
    except Exception as e:
        print(f"❌ Exceção: {e}")
        return False

# Testa API básica
print("🔍 Testando conectividade API...")
try:
    response = requests.get(f"{BASE_URL}/user", headers=headers, timeout=5)
    if response.status_code == 200:
        print("✅ API conectada")
    else:
        print(f"❌ API erro: {response.status_code}")
        exit(1)
except Exception as e:
    print(f"❌ API exceção: {e}")
    exit(1)

# IDs para testar
folder_ids = ["90129889949"]
list_ids = ["9013037641", "90130037334"]

print("\n" + "="*50)
print("RESULTADOS:")
print("="*50)

for folder_id in folder_ids:
    test_folder(folder_id)

for list_id in list_ids:
    test_list(list_id)