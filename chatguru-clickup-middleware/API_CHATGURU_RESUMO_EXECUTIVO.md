# 📚 API ChatGuru - Resumo Executivo

## 🔑 Informações Essenciais

### Endpoints Base
- **S12 Server**: `https://s12.chatguru.app/api/v1`
- **App Server**: `https://app.zap.guru/api/v1`

### Autenticação
```javascript
// Header obrigatório para todas as requisições
headers: {
  'APIKey': 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f',
  'Content-Type': 'application/json'
}
```

## 📡 Webhooks do ChatGuru

### Formato Enviado pelo ChatGuru
O ChatGuru envia webhooks no seguinte formato:

```json
{
  // Campo event_id NÃO é enviado pelo ChatGuru
  "annotation": {
    "data": {
      "tarefa": "Descrição da tarefa",
      "prioridade": "Alta",
      "responsavel": "João",
      // Outros campos customizados configurados no diálogo
    }
  },
  "contact": {
    "number": "5511999999999",
    "name": "Nome do Contato"
  },
  "message": {
    "text": "Texto da mensagem",
    "type": "text"
  }
}
```

⚠️ **IMPORTANTE**: 
- O ChatGuru NÃO envia o campo `event_id`
- O ChatGuru NÃO envia o campo `data` direto
- Os dados vêm dentro de `annotation.data`

## 🤖 Configuração de Diálogos

### Via API - Endpoints

#### 1. Listar Diálogos
```bash
curl -X GET https://s12.chatguru.app/api/v1/dialogs \
  -H "APIKey: c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f"
```

#### 2. Obter Detalhes de um Diálogo
```bash
curl -X GET https://s12.chatguru.app/api/v1/dialogs/{dialog_id} \
  -H "APIKey: c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f"
```

#### 3. Atualizar Webhook do Diálogo
```bash
curl -X PUT https://s12.chatguru.app/api/v1/dialogs/{dialog_id} \
  -H "APIKey: c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f" \
  -H "Content-Type: application/json" \
  -d '{
    "webhook": "https://nova-url/webhooks/chatguru",
    "actions": [{
      "type": "webhook",
      "url": "https://nova-url/webhooks/chatguru",
      "method": "POST",
      "headers": {
        "Content-Type": "application/json"
      }
    }]
  }'
```

## 🔍 Diagnóstico Rápido

### Problema: nova_api não envia webhook

**Verificações:**

1. **Diálogo existe?**
```javascript
// Script: check-nova-api-dialog.js
node check-nova-api-dialog.js
```

2. **Webhook está configurado?**
```javascript
// Script: update-dialog-webhook.js
node update-dialog-webhook.js
```

3. **URL está correta?**
- ❌ Antigo: `https://buzzlightear-ek3kpvifpq-ue.a.run.app/webhooks/chatguru`
- ✅ Novo: `https://chatguru-clickup-middleware-xxxxx-uc.a.run.app/webhooks/chatguru`

## 🛠️ Solução Passo a Passo

### 1. Identificar o Diálogo
```javascript
const fetch = require('node-fetch');

const findDialog = async () => {
  const response = await fetch('https://s12.chatguru.app/api/v1/dialogs', {
    headers: { 'APIKey': 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f' }
  });
  
  const dialogs = await response.json();
  const novaApi = dialogs.find(d => d.name === 'nova_api');
  
  console.log('Dialog ID:', novaApi?.id);
  console.log('Current webhook:', novaApi?.webhook);
};
```

### 2. Atualizar o Webhook
```javascript
const updateWebhook = async (dialogId, newUrl) => {
  const response = await fetch(`https://s12.chatguru.app/api/v1/dialogs/${dialogId}`, {
    method: 'PUT',
    headers: {
      'APIKey': 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      webhook: newUrl,
      actions: [{
        type: 'webhook',
        url: newUrl,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      }]
    })
  });
  
  return response.ok;
};
```

### 3. Testar o Webhook
```bash
# Testar diretamente
curl -X POST https://sua-url/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "annotation": {
      "data": {
        "tarefa": "Teste direto"
      }
    }
  }'
```

## 📝 Checklist de Correção

- [ ] Deploy do middleware concluído no Cloud Run
- [ ] URL do Cloud Run obtida
- [ ] Diálogo nova_api localizado via API
- [ ] Webhook atualizado para nova URL
- [ ] Teste enviando mensagem no WhatsApp
- [ ] Tarefa criada no ClickUp com sucesso

## 🚨 Erros Comuns e Soluções

### Erro: "missing field event_id"
**Solução**: Tornar o campo opcional no middleware
```rust
pub id: Option<String>,  // Não String
```

### Erro: "missing field data"
**Solução**: Aceitar dados de annotation.data
```rust
// Extrair de annotation.data se data não existir
let data = event.data.or_else(|| {
    event.annotation.map(|a| a.data)
});
```

### Erro: Webhook não enviado
**Solução**: Verificar configuração do diálogo
1. Listar diálogos
2. Verificar se webhook está configurado
3. Atualizar com URL correta

## 🔗 Scripts Disponíveis

1. **check-nova-api-dialog.js** - Verifica status do diálogo
2. **update-dialog-webhook.js** - Atualiza webhook via API
3. **test-webhook-simple.js** - Testa webhook localmente
4. **test-dialogs.sh** - Testa execução dos diálogos

## 📞 URLs Importantes

- **Buzzlightear (antigo)**: https://buzzlightear-ek3kpvifpq-ue.a.run.app
- **Middleware (novo)**: Aguardando deploy...
- **ChatGuru S12**: https://s12.chatguru.app
- **Documentação**: https://oldwiki.chatguru.com.br/api/api-documentacao-v1

---

*Use este resumo para diagnosticar e corrigir rapidamente problemas com a integração ChatGuru-ClickUp*