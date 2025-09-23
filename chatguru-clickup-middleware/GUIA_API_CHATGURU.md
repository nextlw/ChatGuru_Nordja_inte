# Guia Completo - Operação ChatGuru via API

## 1. Configuração Inicial da API

### Credenciais Necessárias
```bash
# Obtenha no painel do ChatGuru (página "Celulares")
API_KEY="sua_api_key"
ACCOUNT_ID="seu_account_id" 
PHONE_ID="seu_phone_id"
```

### Base URL da API
```
https://app.zap.guru/api/v1
```

## 2. Comandos CLI para Testar a API

### 2.1 Enviar Mensagem
```bash
curl -X POST https://app.zap.guru/api/v1/message_send \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "message": "Mensagem de teste",
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }'
```

### 2.2 Executar Diálogo (Flow/Chatbot)
```bash
# Para executar o diálogo "nova_api"
curl -X POST https://app.zap.guru/api/v1/dialog_execute \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "dialog_id": "nova_api",
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }'

# Para executar o diálogo "TESTE_API" 
curl -X POST https://app.zap.guru/api/v1/dialog_execute \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "dialog_id": "TESTE_API",
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }'
```

### 2.3 Adicionar Anotação (Note)
```bash
curl -X POST https://app.zap.guru/api/v1/note_add \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "note": "Tarefa: Verificar produto XYZ",
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }'
```

### 2.4 Atualizar Campos Personalizados
```bash
curl -X PUT https://app.zap.guru/api/v1/chat_update_custom_fields \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "custom_fields": {
      "tarefa": "Verificar produto XYZ",
      "prioridade": "Alta",
      "responsavel": "João"
    },
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }'
```

## 3. Diagnóstico - Diferenças entre nova_api e TESTE_API

### Possíveis Causas do Problema

#### 3.1 Configuração do Diálogo no ChatGuru
- **nova_api** pode não estar configurado corretamente para enviar webhooks
- Verifique no painel do ChatGuru:
  1. Entre no editor de diálogos
  2. Abra o diálogo "nova_api"
  3. Verifique se há ação de "Webhook" configurada
  4. Compare com as configurações do "TESTE_API"

#### 3.2 Formato do Webhook
O webhook deve enviar dados no formato esperado:
```json
{
  "event_type": "task_created",
  "id": "unique_event_id",
  "timestamp": "2024-01-20T10:00:00Z",
  "data": {
    "chat_number": "5511999999999",
    "message": "Descrição da tarefa",
    "custom_fields": {
      "tarefa": "Conteúdo da anotação"
    }
  }
}
```

#### 3.3 Ação de Webhook no Diálogo
No editor de diálogos do ChatGuru, a ação de webhook deve estar configurada assim:
- **URL**: https://seu-webhook-url/webhooks/chatguru
- **Método**: POST
- **Headers**: Content-Type: application/json
- **Body**: JSON com os dados da tarefa

## 4. Script de Teste Completo

Crie um arquivo `test-dialogs.sh`:

```bash
#!/bin/bash

# Configuração
API_KEY="sua_api_key"
ACCOUNT_ID="seu_account_id"
PHONE_ID="seu_phone_id"
CHAT_NUMBER="5511999999999"
BASE_URL="https://app.zap.guru/api/v1"

echo "========================================="
echo "Testando Diálogo: TESTE_API"
echo "========================================="

curl -X POST "$BASE_URL/dialog_execute" \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "'$CHAT_NUMBER'",
    "dialog_id": "TESTE_API",
    "variables": {
      "tarefa": "Teste via CLI - TESTE_API",
      "prioridade": "Alta"
    },
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }' | jq .

echo ""
echo "========================================="
echo "Testando Diálogo: nova_api"
echo "========================================="

curl -X POST "$BASE_URL/dialog_execute" \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "'$CHAT_NUMBER'",
    "dialog_id": "nova_api",
    "variables": {
      "tarefa": "Teste via CLI - nova_api",
      "prioridade": "Alta"  
    },
    "key": "'$API_KEY'",
    "account_id": "'$ACCOUNT_ID'",
    "phone_id": "'$PHONE_ID'"
  }' | jq .
```

## 5. Verificação do Webhook

### 5.1 Monitorar Logs do Webhook
```bash
# Se estiver rodando localmente
tail -f chatguru-clickup-middleware/logs/*.log

# Se estiver no GCP
gcloud run logs read --service chatguru-clickup-middleware --tail
```

### 5.2 Testar Webhook Diretamente
```bash
# Teste direto do webhook
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "task_created",
    "id": "test_'$(date +%s)'",
    "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
    "data": {
      "chat_number": "5511999999999",
      "message": "Teste direto do webhook",
      "custom_fields": {
        "tarefa": "Tarefa de teste via webhook direto"
      }
    }
  }'
```

## 6. Checklist de Verificação

### No ChatGuru:
- [ ] API está ativada na conta
- [ ] Credenciais estão corretas (key, account_id, phone_id)
- [ ] Diálogo "nova_api" existe e está ativo
- [ ] Diálogo "nova_api" tem ação de webhook configurada
- [ ] URL do webhook está correta
- [ ] Método HTTP está como POST
- [ ] Content-Type está como application/json

### No Middleware:
- [ ] Serviço está rodando (local ou GCP)
- [ ] Endpoint /webhooks/chatguru está acessível
- [ ] Logs mostram requisições chegando
- [ ] Formato do JSON está correto

### Teste de Comparação:
- [ ] TESTE_API envia webhook com sucesso
- [ ] nova_api não envia webhook
- [ ] Ambos retornam sucesso na API
- [ ] Diferença está na configuração do diálogo

## 7. Solução Provável

Se o "TESTE_API" funciona mas o "nova_api" não:

1. **Entre no editor de diálogos do ChatGuru**
2. **Abra o diálogo "TESTE_API"** 
3. **Exporte ou copie a configuração de webhook**
4. **Abra o diálogo "nova_api"**
5. **Adicione ou corrija a ação de webhook**:
   - Tipo: Webhook/HTTP Request
   - URL: mesma URL do TESTE_API
   - Método: POST
   - Headers: Content-Type: application/json
   - Body: JSON com estrutura correta

## 8. Template de Ação Webhook no Diálogo

```json
{
  "event_type": "task_created",
  "id": "{{dialog_execution_id}}",
  "timestamp": "{{current_timestamp}}",
  "data": {
    "chat_number": "{{chat_number}}",
    "message": "{{message}}",
    "sender_name": "{{sender_name}}",
    "custom_fields": {
      "tarefa": "{{variable.tarefa}}",
      "prioridade": "{{variable.prioridade}}",
      "responsavel": "{{variable.responsavel}}"
    }
  }
}
```

## 9. Comandos Úteis para Debug

```bash
# Verificar se o webhook está recebendo dados
nc -l 8080

# Monitorar tráfego HTTP
tcpdump -i any -A port 8080

# Ver logs do middleware em tempo real
journalctl -f -u chatguru-clickup-middleware

# Testar conectividade
curl -X GET http://localhost:8080/health
```

## 10. Contato e Suporte

- **Documentação ChatGuru**: https://oldwiki.chatguru.com.br/api/api-documentacao-v1
- **Suporte**: support@chatguru.com.br
- **Status do Webhook**: http://localhost:8080/status