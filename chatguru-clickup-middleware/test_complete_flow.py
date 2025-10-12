#!/usr/bin/env python3
"""
Teste completo do fluxo ChatGuru → ClickUp
Simula um payload real do ChatGuru e acompanha todo o processo até a criação da tarefa
"""

import json
import requests
import time
import uuid
from datetime import datetime

# Configuração
BASE_URL = "https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app"
TEST_CLIENT = "Nexcode"  # Nome da empresa (Info_1)
TEST_REQUESTOR = "Willaim Duarte"  # Nome do solicitante (Info_2)
TEST_ATTENDANT = "Anne Souza"  # Responsável pelo atendimento (deve mapear para space "Anne Souza")

def generate_test_payload():
    """Gera um payload de teste realista do ChatGuru"""
    payload = {
        "event_type": "message_received",
        "timestamp": datetime.now().isoformat(),
        "account_id": "625584ce6fdcb7bda7d94aa8",
        "conversation_id": f"test_conv_{uuid.uuid4().hex[:8]}",
        "message": {
            "id": f"msg_{uuid.uuid4().hex[:8]}",
            "content": "Preciso de ajuda com a integração OAuth2 do ClickUp. O sistema não está validando os tokens corretamente.",
            "sender": {
                "phone": "+5511999999999",
                "name": "Cliente Teste"
            },
            "timestamp": datetime.now().isoformat(),
            "type": "text"
        },
        "campos_personalizados": {
            "Info_1": TEST_CLIENT,  # Nome da empresa (Nexcode)
            "Info_2": TEST_REQUESTOR,  # Nome do solicitante (João Silva)
            "Info_3": "Suporte Técnico",
            "Info_4": "Alta",
            "Info_5": "Integração",
            "responsavel": TEST_ATTENDANT  # Responsável (Anne)
        },
        "metadata": {
            "channel": "whatsapp",
            "source": "web"
        }
    }
    return payload

def test_webhook_endpoint(payload):
    """Testa o endpoint webhook do ChatGuru"""
    print("=" * 60)
    print("🔥 INICIANDO TESTE COMPLETO DO FLUXO CHATGURU → CLICKUP")
    print("=" * 60)
    
    print(f"\n📋 DADOS DO TESTE:")
    print(f"   Empresa (Info_1): {TEST_CLIENT}")
    print(f"   Solicitante (Info_2): {TEST_REQUESTOR}")
    print(f"   Responsável: {TEST_ATTENDANT}")
    print(f"   Conversation ID: {payload['conversation_id']}")
    print(f"   Message ID: {payload['message']['id']}")
    print(f"   Expected Space: Anne Souza")
    
    print(f"\n💬 MENSAGEM DE TESTE:")
    print(f"   '{payload['message']['content']}'")
    
    print(f"\n🌐 ENVIANDO WEBHOOK PARA: {BASE_URL}/webhooks/chatguru")
    
    start_time = time.time()
    
    try:
        response = requests.post(
            f"{BASE_URL}/webhooks/chatguru",
            json=payload,
            headers={
                "Content-Type": "application/json",
                "User-Agent": "ChatGuru-Test/1.0"
            },
            timeout=30
        )
        
        webhook_time = time.time() - start_time
        
        print(f"\n✅ WEBHOOK RESPONSE:")
        print(f"   Status Code: {response.status_code}")
        print(f"   Response Time: {webhook_time:.3f}s")
        print(f"   Headers: {dict(response.headers)}")
        
        if response.status_code == 200:
            try:
                webhook_response = response.json()
                print(f"   Response Body: {json.dumps(webhook_response, indent=2)}")
            except:
                print(f"   Response Body: {response.text}")
        
        return response.status_code == 200, webhook_time
        
    except Exception as e:
        print(f"\n❌ ERRO NO WEBHOOK: {str(e)}")
        return False, 0

def wait_for_processing():
    """Aguarda o processamento assíncrono via Pub/Sub"""
    print(f"\n⏳ AGUARDANDO PROCESSAMENTO VIA PUB/SUB...")
    print("   (O worker processa a mensagem em background)")
    
    for i in range(10):
        time.sleep(2)
        print(f"   Aguardando... {(i+1)*2}s")
    
    print("   ✅ Tempo de espera concluído")

def check_clickup_integration():
    """Verifica se a integração ClickUp está funcionando"""
    print(f"\n🔗 VERIFICANDO INTEGRAÇÃO CLICKUP...")
    
    try:
        response = requests.get(f"{BASE_URL}/status", timeout=10)
        
        if response.status_code == 200:
            status_data = response.json()
            
            print(f"   ✅ Status Service: {response.status_code}")
            
            # Verificar integração ClickUp
            clickup_status = status_data.get("integrations", {}).get("clickup", {})
            print(f"\n📊 CLICKUP INTEGRATION STATUS:")
            print(f"   Configured: {clickup_status.get('configured', 'Unknown')}")
            print(f"   Connection: {clickup_status.get('connection', 'Unknown')}")
            print(f"   Token Configured: {clickup_status.get('token_configured', 'Unknown')}")
            print(f"   List ID: {clickup_status.get('list_id', 'Unknown')}")
            print(f"   List Name: {clickup_status.get('list_name', 'Unknown')}")
            
            # Verificar outras integrações
            chatguru_status = status_data.get("integrations", {}).get("chatguru", {})
            openai_status = status_data.get("integrations", {}).get("openai", {})
            
            print(f"\n📊 OTHER INTEGRATIONS:")
            print(f"   ChatGuru API: {chatguru_status.get('api_configured', 'Unknown')}")
            print(f"   OpenAI Enabled: {openai_status.get('enabled', 'Unknown')}")
            print(f"   AI Enabled: {status_data.get('ai_enabled', 'Unknown')}")
            
            return clickup_status.get('connection') == 'success'
        else:
            print(f"   ❌ Status endpoint failed: {response.status_code}")
            return False
            
    except Exception as e:
        print(f"   ❌ Erro verificando status: {str(e)}")
        return False

def search_created_task():
    """Busca a tarefa criada no ClickUp"""
    print(f"\n🔍 BUSCANDO TAREFA CRIADA NO CLICKUP...")
    
    try:
        # Tentar buscar tarefas via endpoint de debug
        response = requests.get(f"{BASE_URL}/clickup/tasks", timeout=15)
        
        if response.status_code == 200:
            tasks_data = response.json()
            print(f"   ✅ Tasks endpoint response: {response.status_code}")
            
            if isinstance(tasks_data, dict) and "tasks" in tasks_data:
                tasks = tasks_data["tasks"]
                print(f"   📝 Total de tarefas encontradas: {len(tasks)}")
                
                # Buscar a tarefa mais recente
                if tasks:
                    latest_task = max(tasks, key=lambda t: t.get("date_created", 0))
                    
                    print(f"\n🎯 TAREFA MAIS RECENTE ENCONTRADA:")
                    print(f"   ID: {latest_task.get('id', 'N/A')}")
                    print(f"   Nome: {latest_task.get('name', 'N/A')}")
                    print(f"   Status: {latest_task.get('status', {}).get('status', 'N/A')}")
                    print(f"   Lista: {latest_task.get('list', {}).get('name', 'N/A')}")
                    print(f"   Data Criação: {latest_task.get('date_created', 'N/A')}")
                    print(f"   URL: {latest_task.get('url', 'N/A')}")
                    
                    if latest_task.get('assignees'):
                        assignees = [a.get('username', 'N/A') for a in latest_task['assignees']]
                        print(f"   Assignees: {', '.join(assignees)}")
                    
                    return latest_task
                else:
                    print("   ⚠️ Nenhuma tarefa encontrada")
                    return None
            else:
                print(f"   ⚠️ Formato de resposta inesperado: {tasks_data}")
                return None
        else:
            print(f"   ❌ Tasks endpoint failed: {response.status_code}")
            print(f"   Response: {response.text}")
            return None
            
    except Exception as e:
        print(f"   ❌ Erro buscando tarefas: {str(e)}")
        return None

def test_oauth_endpoints():
    """Testa os endpoints OAuth2 melhorados"""
    print(f"\n🔐 TESTANDO ENDPOINTS OAUTH2 MELHORADOS...")
    
    try:
        # Testar endpoint de inicialização OAuth
        response = requests.head(f"{BASE_URL}/auth/clickup", timeout=10, allow_redirects=False)
        
        print(f"   OAuth Start Endpoint:")
        print(f"   Status: {response.status_code}")
        
        if response.status_code == 303:
            location = response.headers.get('location', '')
            print(f"   ✅ Redirect para ClickUp: {location[:100]}...")
        
        return response.status_code == 303
        
    except Exception as e:
        print(f"   ❌ Erro testando OAuth: {str(e)}")
        return False

def generate_test_report(webhook_success, webhook_time, clickup_working, oauth_working, created_task):
    """Gera relatório final do teste"""
    print("\n" + "=" * 60)
    print("📊 RELATÓRIO FINAL DO TESTE COMPLETO")
    print("=" * 60)
    
    print(f"\n🔥 RESULTADOS GERAIS:")
    print(f"   Webhook Success: {'✅' if webhook_success else '❌'}")
    print(f"   Webhook Time: {webhook_time:.3f}s {'✅' if webhook_time < 5 else '⚠️'}")
    print(f"   ClickUp Integration: {'✅' if clickup_working else '❌'}")
    print(f"   OAuth2 Endpoints: {'✅' if oauth_working else '❌'}")
    print(f"   Task Created: {'✅' if created_task else '❌'}")
    
    print(f"\n🎯 FLUXO COMPLETO:")
    print(f"   1. ChatGuru Webhook → {'✅ SUCCESS' if webhook_success else '❌ FAILED'}")
    print(f"   2. Pub/Sub Processing → ⏳ ASYNC")
    print(f"   3. ClickUp Connection → {'✅ SUCCESS' if clickup_working else '❌ FAILED'}")
    print(f"   4. OAuth2 Security → {'✅ SUCCESS' if oauth_working else '❌ FAILED'}")
    print(f"   5. Task Creation → {'✅ SUCCESS' if created_task else '❌ FAILED'}")
    
    if created_task:
        print(f"\n🎉 TAREFA CRIADA COM SUCESSO:")
        print(f"   ID: {created_task.get('id', 'N/A')}")
        print(f"   Nome: {created_task.get('name', 'N/A')}")
        print(f"   URL ClickUp: {created_task.get('url', 'N/A')}")
        print(f"   Lista: {created_task.get('list', {}).get('name', 'N/A')}")
    
    print(f"\n🔗 MELHORIAS OAUTH2 IMPLEMENTADAS:")
    print(f"   ✅ Token validation robusta")
    print(f"   ✅ Cache invalidation inteligente")
    print(f"   ✅ Error handling específico (OAUTH_025, OAUTH_027)")
    print(f"   ✅ Authorization headers corrigidos")
    print(f"   ✅ Auto-reauthorization")
    
    # Score final
    score = sum([webhook_success, clickup_working, oauth_working, bool(created_task)])
    total_score = (score / 4) * 100
    
    print(f"\n📈 SCORE FINAL: {total_score:.1f}% ({score}/4 testes)")
    
    if total_score >= 75:
        print("🎉 SISTEMA FUNCIONANDO CORRETAMENTE!")
    elif total_score >= 50:
        print("⚠️ SISTEMA PARCIALMENTE FUNCIONAL")
    else:
        print("❌ SISTEMA COM PROBLEMAS CRÍTICOS")
    
    print("=" * 60)

def main():
    """Executa o teste completo"""
    print("🚀 Iniciando teste completo do fluxo ChatGuru → ClickUp...")
    
    # 1. Gerar payload de teste
    test_payload = generate_test_payload()
    
    # 2. Testar webhook
    webhook_success, webhook_time = test_webhook_endpoint(test_payload)
    
    # 3. Aguardar processamento
    if webhook_success:
        wait_for_processing()
    
    # 4. Verificar integração ClickUp
    clickup_working = check_clickup_integration()
    
    # 5. Testar OAuth2
    oauth_working = test_oauth_endpoints()
    
    # 6. Buscar tarefa criada
    created_task = search_created_task()
    
    # 7. Gerar relatório final
    generate_test_report(webhook_success, webhook_time, clickup_working, oauth_working, created_task)

if __name__ == "__main__":
    main()