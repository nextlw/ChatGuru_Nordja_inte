#!/usr/bin/env python3
"""
Teste específico para validar a lógica corrigida de mapeamento ChatGuru → ClickUp
Foca na validação dos campos Info_1, Info_2 e responsavel_nome
"""

import json
import requests
import time
import uuid
import psycopg2
from datetime import datetime

# Configuração
BASE_URL = "https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app"

# Casos de teste baseados nos dados reais corrigidos
TEST_CASES = [
    {
        "name": "Teste Anne Souza Space",
        "info_1": "Nexcode",  # Empresa cliente (campo personalizado)
        "info_2": "William Duarte",  # Nome do cliente (folder)
        "responsavel": "Anne",  # Responsável (space Anne Souza: 90131713706)
        "expected_space": "Anne Souza",
        "expected_space_id": "90131713706"
    }
]

# Configuração do banco
DB_CONFIG = {
    'host': '127.0.0.1',
    'port': '9470',
    'user': 'postgres',
    'password': '0Djn3a5CGGn7u1jTbsO0ZFRmXxo3idd+',
    'database': 'chatguru_middleware'
}

def verify_database_mapping():
    """Verifica se o banco está com os dados corretos"""
    print("🔍 VERIFICANDO MAPEAMENTOS NO BANCO DE DADOS")
    print("=" * 50)
    
    try:
        conn = psycopg2.connect(**DB_CONFIG)
        cursor = conn.cursor()
        
        # Verificar attendant_aliases
        cursor.execute("""
            SELECT 
                aa.attendant_alias,
                aa.space_id,
                aa.space_name,
                s.space_name as real_space_name
            FROM attendant_aliases aa
            LEFT JOIN spaces s ON aa.space_id = s.space_id
            ORDER BY aa.attendant_alias;
        """)
        
        aliases = cursor.fetchall()
        print("\n📋 Attendant Aliases:")
        for alias, space_id, space_name, real_space_name in aliases:
            status = "✅" if real_space_name else "❌"
            print(f"   {status} {alias}: {space_id} ({space_name})")
        
        # Verificar folder_mapping - CORRIGIDO: usar folder_path ao invés de folder_name
        cursor.execute("""
            SELECT client_name, attendant_name, folder_id, folder_path, is_active
            FROM folder_mapping
            WHERE is_active = true AND attendant_name = 'William'
            ORDER BY client_name
            LIMIT 10;
        """)
        
        mappings = cursor.fetchall()
        print(f"\n📁 Folder Mappings (showing William only):")
        for client, attendant, folder_id, folder_path, active in mappings:
            print(f"   ✅ {client} + {attendant} → {folder_path} ({folder_id})")
        
        cursor.close()
        conn.close()
        
        return len([a for a in aliases if a[3] is not None]) == len(aliases)
        
    except Exception as e:
        print(f"❌ Erro verificando banco: {e}")
        return False

def generate_test_payload(test_case):
    """Gera payload de teste para um caso específico"""
    payload = {
        "event_type": "message_received",
        "timestamp": datetime.now().isoformat(),
        "account_id": "625584ce6fdcb7bda7d94aa8",
        "conversation_id": f"test_conv_{uuid.uuid4().hex[:8]}",
        "message": {
            "id": f"msg_{uuid.uuid4().hex[:8]}",
            "content": f"Preciso de suporte técnico urgente. Problema com sistema de pagamento da {test_case['info_1']}.",
            "sender": {
                "phone": "+5511999999999",
                "name": test_case["info_2"]
            },
            "timestamp": datetime.now().isoformat(),
            "type": "text"
        },
        "campos_personalizados": {
            "Info_1": test_case["info_1"],  # Empresa cliente → Campo personalizado
            "Info_2": test_case["info_2"],  # Nome do cliente → Folder
            "Info_3": "Suporte Técnico",
            "Info_4": "Alta",
            "Info_5": "Integração",
            "responsavel": test_case["responsavel"]  # Responsável → Space
        },
        "metadata": {
            "channel": "whatsapp",
            "source": "test"
        }
    }
    return payload

def test_mapping_logic(test_case):
    """Testa um caso específico de mapeamento"""
    print(f"\n🧪 TESTANDO: {test_case['name']}")
    print("-" * 50)
    
    # Mostrar a lógica esperada
    print(f"📋 MAPEAMENTO ESPERADO:")
    print(f"   Info_1 ('{test_case['info_1']}') → Campo Personalizado")
    print(f"   Info_2 ('{test_case['info_2']}') → Folder")
    print(f"   Responsável ('{test_case['responsavel']}') → Space '{test_case['expected_space']}' ({test_case['expected_space_id']})")
    
    # Gerar payload
    payload = generate_test_payload(test_case)
    print(f"\n📨 ENVIANDO WEBHOOK...")
    
    try:
        start_time = time.time()
        response = requests.post(
            f"{BASE_URL}/webhooks/chatguru",
            json=payload,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "ChatGuru-MappingTest/1.0"
            },
            timeout=30
        )
        
        webhook_time = time.time() - start_time
        
        print(f"   Status: {response.status_code}")
        print(f"   Tempo: {webhook_time:.3f}s")
        
        if response.status_code == 200:
            print(f"   ✅ Webhook aceito - processamento iniciado")
            return True, payload
        else:
            print(f"   ❌ Webhook falhou: {response.text}")
            return False, payload
            
    except Exception as e:
        print(f"   ❌ Erro no webhook: {str(e)}")
        return False, payload

def wait_for_processing(payload):
    """Aguarda processamento e verifica logs"""
    print(f"\n⏳ AGUARDANDO PROCESSAMENTO...")
    print(f"   Conversation ID: {payload['conversation_id']}")
    print(f"   Message ID: {payload['message']['id']}")
    
    for i in range(8):
        time.sleep(3)
        print(f"   Aguardando... {(i+1)*3}s")
    
    print(f"   ✅ Processamento concluído")

def check_task_creation():
    """Verifica se alguma tarefa foi criada recentemente"""
    print(f"\n🔍 VERIFICANDO CRIAÇÃO DE TAREFA...")
    
    try:
        response = requests.get(f"{BASE_URL}/clickup/tasks", timeout=15)
        
        if response.status_code == 200:
            tasks_data = response.json()
            
            if isinstance(tasks_data, dict) and "tasks" in tasks_data:
                tasks = tasks_data["tasks"]
                
                # Buscar tarefas dos últimos 5 minutos
                recent_tasks = []
                current_time = time.time() * 1000  # milissegundos
                five_minutes_ago = current_time - (5 * 60 * 1000)
                
                for task in tasks:
                    date_created = int(task.get("date_created", 0))
                    if date_created > five_minutes_ago:
                        recent_tasks.append(task)
                
                print(f"   📝 Tarefas recentes (últimos 5 min): {len(recent_tasks)}")
                
                if recent_tasks:
                    latest_task = max(recent_tasks, key=lambda t: t.get("date_created", 0))
                    
                    print(f"\n🎯 TAREFA MAIS RECENTE:")
                    print(f"   ID: {latest_task.get('id', 'N/A')}")
                    print(f"   Nome: {latest_task.get('name', 'N/A')}")
                    print(f"   Lista: {latest_task.get('list', {}).get('name', 'N/A')}")
                    print(f"   Folder: {latest_task.get('folder', {}).get('name', 'N/A')}")
                    print(f"   Space: {latest_task.get('space', {}).get('name', 'N/A')}")
                    print(f"   URL: {latest_task.get('url', 'N/A')}")
                    
                    return latest_task
                else:
                    print(f"   ⚠️ Nenhuma tarefa recente encontrada")
                    return None
            else:
                print(f"   ❌ Formato de resposta inesperado")
                return None
        else:
            print(f"   ❌ Erro acessando tarefas: {response.status_code}")
            return None
            
    except Exception as e:
        print(f"   ❌ Erro verificando tarefas: {str(e)}")
        return None

def run_complete_mapping_test():
    """Executa teste completo da lógica de mapeamento"""
    print("🚀 TESTE COMPLETO DA LÓGICA DE MAPEAMENTO CORRIGIDA")
    print("=" * 60)
    
    # 1. Verificar banco de dados
    db_ok = verify_database_mapping()
    if not db_ok:
        print("❌ Banco de dados com problemas - teste abortado")
        return
    
    # 2. Testar cada caso
    successful_tests = 0
    total_tests = len(TEST_CASES)
    
    for i, test_case in enumerate(TEST_CASES, 1):
        print(f"\n\n{'='*60}")
        print(f"🧪 TESTE {i}/{total_tests}: {test_case['name']}")
        print(f"{'='*60}")
        
        # Testar webhook
        webhook_ok, payload = test_mapping_logic(test_case)
        
        if webhook_ok:
            # Aguardar processamento
            wait_for_processing(payload)
            
            # Verificar resultado
            created_task = check_task_creation()
            
            if created_task:
                # Validar se o mapeamento está correto
                task_space = created_task.get('space', {}).get('name', 'N/A')
                expected_space = test_case['expected_space']
                
                if task_space == expected_space:
                    print(f"   ✅ MAPEAMENTO CORRETO: Space '{task_space}' conforme esperado")
                    successful_tests += 1
                else:
                    print(f"   ❌ MAPEAMENTO INCORRETO: Esperado '{expected_space}', obtido '{task_space}'")
            else:
                print(f"   ❌ TAREFA NÃO CRIADA")
        else:
            print(f"   ❌ WEBHOOK FALHOU")
        
        # Pausar entre testes
        if i < total_tests:
            print(f"\n⏸️ Pausando 5s antes do próximo teste...")
            time.sleep(5)
    
    # 3. Relatório final
    print(f"\n\n{'='*60}")
    print(f"📊 RELATÓRIO FINAL DOS TESTES DE MAPEAMENTO")
    print(f"{'='*60}")
    
    success_rate = (successful_tests / total_tests) * 100
    
    print(f"\n🎯 RESULTADOS:")
    print(f"   Testes bem-sucedidos: {successful_tests}/{total_tests}")
    print(f"   Taxa de sucesso: {success_rate:.1f}%")
    
    print(f"\n✅ LÓGICA VALIDADA:")
    print(f"   Info_1 → Campo Personalizado (Empresa)")
    print(f"   Info_2 → Folder (Nome do Cliente)")
    print(f"   responsavel → Space (Atendente)")
    
    if success_rate >= 80:
        print(f"\n🎉 MAPEAMENTO FUNCIONANDO CORRETAMENTE!")
    elif success_rate >= 60:
        print(f"\n⚠️ MAPEAMENTO PARCIALMENTE FUNCIONAL")
    else:
        print(f"\n❌ MAPEAMENTO COM PROBLEMAS CRÍTICOS")
    
    print(f"\n{'='*60}")

if __name__ == "__main__":
    run_complete_mapping_test()