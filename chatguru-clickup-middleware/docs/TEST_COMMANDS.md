# 🧪 Comandos de Teste para o Middleware no Cloud Run

## 📌 IMPORTANTE: Aguardar URL do Serviço
O deploy está em andamento. Quando concluir, substitua `SERVICE_URL` pela URL real fornecida pelo Cloud Run.

**Formato esperado**: `https://chatguru-clickup-middleware-[HASH]-rj.a.run.app`

## 🔧 Variáveis de Ambiente
```bash
# Definir a URL do serviço (substituir após deploy)
export SERVICE_URL="https://chatguru-clickup-middleware-XXXXX-rj.a.run.app"

# Token do ClickUp (já configurado no serviço)
export CLICKUP_TOKEN="pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657"

# List ID do ClickUp (já configurado no serviço)
export LIST_ID="901300373349"
```

## 1️⃣ Testes de Health Check

### Verificar se o serviço está online
```bash
curl -X GET "$SERVICE_URL/health"
```
**Resposta esperada**: `{"status":"healthy"}`

### Verificar prontidão do serviço
```bash
curl -X GET "$SERVICE_URL/ready"
```
**Resposta esperada**: `{"ready":true}`

### Status completo da aplicação
```bash
curl -X GET "$SERVICE_URL/status" | jq '.'
```
**Resposta esperada**: JSON com informações do serviço

## 2️⃣ Testes de Conectividade com ClickUp

### Testar conexão com ClickUp API
```bash
curl -X GET "$SERVICE_URL/clickup/test" | jq '.'
```
**Resposta esperada**: 
```json
{
  "status": "success",
  "message": "ClickUp connection successful",
  "list_id": "901300373349"
}
```

### Obter informações da lista do ClickUp
```bash
curl -X GET "$SERVICE_URL/clickup/list" | jq '.'
```
**Resposta esperada**: Detalhes da lista configurada

### Listar tasks existentes
```bash
curl -X GET "$SERVICE_URL/clickup/tasks" | jq '.'
```
**Resposta esperada**: Array com tasks da lista

## 3️⃣ Teste do Webhook ChatGuru

### Simular evento de mensagem recebida
```bash
curl -X POST "$SERVICE_URL/webhooks/chatguru" \
  -H "Content-Type: application/json" \
  -d '{
    "event": "message.created",
    "message": {
      "timestamp": 1703095200,
      "text": "Teste de integração - criar task urgente",
      "sender": {
        "name": "Cliente Teste",
        "phone": "+5511999999999"
      }
    },
    "sender": {
      "name": "Cliente Teste",
      "phone": "+5511999999999"
    },
    "data": {
      "text": "Preciso de um orçamento para 100 unidades do produto X"
    }
  }' | jq '.'
```

### Simular pedido completo
```bash
curl -X POST "$SERVICE_URL/webhooks/chatguru" \
  -H "Content-Type: application/json" \
  -d '{
    "event": "message.created",
    "timestamp": "'$(date +%s)'",
    "message": {
      "text": "Novo pedido: 50 camisetas modelo premium",
      "sender": {
        "name": "João Silva",
        "phone": "+5521987654321"
      }
    },
    "sender": {
      "name": "João Silva",
      "phone": "+5521987654321",
      "email": "joao.silva@example.com"
    },
    "data": {
      "order_details": {
        "product": "Camiseta Premium",
        "quantity": 50,
        "urgent": true
      }
    }
  }' | jq '.'
```

## 4️⃣ Testes de Carga e Performance

### Teste de resposta rápida
```bash
time curl -X GET "$SERVICE_URL/health"
```

### Múltiplas requisições simultâneas
```bash
for i in {1..5}; do
  curl -X GET "$SERVICE_URL/health" &
done
wait
```

## 5️⃣ Monitoramento de Logs

### Ver logs em tempo real (via gcloud)
```bash
gcloud run services logs read chatguru-clickup-middleware \
  --project buzzlightear \
  --region southamerica-east1 \
  --limit 50
```

### Logs contínuos
```bash
gcloud run services logs tail chatguru-clickup-middleware \
  --project buzzlightear \
  --region southamerica-east1
```

## 6️⃣ Teste de Criação de Task Direta

### Criar task via API interna (debug)
```bash
curl -X POST "$SERVICE_URL/debug/create-task" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Task de Teste via API",
    "description": "Criada para validar integração",
    "priority": 2,
    "status": "Open"
  }' | jq '.'
```

## 7️⃣ Validação no ClickUp

Após executar os testes, verificar no ClickUp:

1. **Acesse o ClickUp**: https://app.clickup.com/
2. **Navegue até a Lista**: ID `901300373349`
3. **Verifique as tasks criadas**:
   - Título deve conter informações do cliente
   - Descrição deve ter detalhes da mensagem
   - Status deve ser "Open" ou "To Do"
   - Prioridade conforme configurado

## 8️⃣ Troubleshooting

### Se o serviço não responder
```bash
# Verificar status do serviço
gcloud run services describe chatguru-clickup-middleware \
  --project buzzlightear \
  --region southamerica-east1 \
  --format json | jq '.status'
```

### Se houver erro 500
```bash
# Ver logs de erro detalhados
gcloud run services logs read chatguru-clickup-middleware \
  --project buzzlightear \
  --region southamerica-east1 \
  --limit 100 | grep -i error
```

### Verificar variáveis de ambiente
```bash
gcloud run services describe chatguru-clickup-middleware \
  --project buzzlightear \
  --region southamerica-east1 \
  --format json | jq '.spec.template.spec.containers[0].env'
```

## 📊 Métricas de Sucesso

✅ **Deploy Bem-sucedido**:
- [ ] Health check retorna 200 OK
- [ ] Ready check retorna true
- [ ] Teste de conexão ClickUp passa
- [ ] Lista do ClickUp é acessível

✅ **Integração Funcionando**:
- [ ] Webhook recebe e processa eventos
- [ ] Tasks são criadas no ClickUp
- [ ] Logs mostram processamento correto
- [ ] Sem erros 500 ou timeouts

## 🔔 Configuração no ChatGuru

Após todos os testes passarem, configure no ChatGuru:

```
URL do Webhook: [SERVICE_URL]/webhooks/chatguru
Método: POST
Headers: 
  - Content-Type: application/json
  - X-ChatGuru-Signature: [opcional]
```

---

**Última atualização**: 22/09/2025 16:04 (horário de Brasília)
**Status**: Aguardando conclusão do deploy no Cloud Run