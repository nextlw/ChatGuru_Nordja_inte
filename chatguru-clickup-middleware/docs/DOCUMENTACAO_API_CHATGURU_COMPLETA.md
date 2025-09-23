# Documentação Completa da API ChatGuru

## 📋 Índice
1. [Visão Geral](#visão-geral)
2. [Autenticação](#autenticação)
3. [Endpoints da API](#endpoints-da-api)
4. [Diálogos (Flows/Chatbots)](#diálogos-flowschatbots)
5. [Webhooks](#webhooks)
6. [Formato de Dados](#formato-de-dados)
7. [Integração com Middleware](#integração-com-middleware)
8. [Troubleshooting](#troubleshooting)

## 🔍 Visão Geral

A API do ChatGuru permite integração completa com a plataforma de automação WhatsApp, incluindo:
- Envio e recebimento de mensagens
- Execução de diálogos (flows)
- Gerenciamento de contatos
- Configuração de webhooks
- Anotações e campos personalizados

### Servidores Disponíveis
- **S12**: https://s12.chatguru.app/api/v1
- **App**: https://app.zap.guru/api/v1
- **Principal**: https://api.chatguru.app/v1

## 🔐 Autenticação

### Obtendo Credenciais
1. Acesse o painel do ChatGuru
2. Navegue até "Configurações" → "API"
3. Copie suas credenciais:

```javascript
const credentials = {
  apiKey: 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f',  // Exemplo
  accountId: 'seu_account_id',
  phoneId: 'seu_phone_id'
};
```

### Headers Necessários
```javascript
const headers = {
  'APIKey': apiKey,           // Para S12
  'Authorization': apiKey,    // Alternativa
  'Content-Type': 'application/json'
};
```

## 📡 Endpoints da API

### 1. Listar Diálogos
```javascript
GET /dialogs
Headers: { APIKey: 'sua_api_key' }

Resposta:
[
  {
    "id": "dialog_id",
    "name": "nome_do_dialogo",
    "active": true,
    "description": "Descrição do diálogo",
    "webhook": "https://webhook.url/endpoint"
  }
]
```

### 2. Obter Detalhes do Diálogo
```javascript
GET /dialogs/{dialog_id}
Headers: { APIKey: 'sua_api_key' }

Resposta:
{
  "id": "dialog_id",
  "name": "nome_do_dialogo",
  "active": true,
  "description": "Descrição",
  "webhook": "https://webhook.url",
  "actions": [...],
  "annotations": [...],
  "triggers": [...]
}
```

### 3. Atualizar Diálogo
```javascript
PUT /dialogs/{dialog_id}
Headers: { APIKey: 'sua_api_key' }
Body: {
  "webhook": "https://nova-url-webhook.com/endpoint",
  "active": true,
  "actions": [
    {
      "type": "webhook",
      "url": "https://webhook.url",
      "method": "POST",
      "headers": {
        "Content-Type": "application/json"
      }
    }
  ]
}
```

### 4. Executar Diálogo
```javascript
POST /dialog_execute
Headers: { APIKey: 'sua_api_key' }
Body: {
  "chat_number": "5511999999999",
  "dialog_id": "nova_api",
  "variables": {
    "tarefa": "Descrição da tarefa",
    "prioridade": "Alta"
  },
  "key": "api_key",
  "account_id": "account_id",
  "phone_id": "phone_id"
}
```

### 5. Enviar Mensagem
```javascript
POST /message_send
Headers: { APIKey: 'sua_api_key' }
Body: {
  "chat_number": "5511999999999",
  "message": "Mensagem de texto",
  "key": "api_key",
  "account_id": "account_id",
  "phone_id": "phone_id"
}
```

## 🤖 Diálogos (Flows/Chatbots)

### Estrutura de um Diálogo

```json
{
  "id": "nova_api",
  "name": "Nova API",
  "description": "Diálogo para criar tarefas",
  "trigger": {
    "type": "keyword",
    "keywords": ["tarefa", "task", "criar tarefa"]
  },
  "steps": [
    {
      "id": "step1",
      "type": "message",
      "content": "Qual é a descrição da tarefa?"
    },
    {
      "id": "step2",
      "type": "input",
      "variable": "tarefa_descricao",
      "validation": "text"
    },
    {
      "id": "step3",
      "type": "webhook",
      "config": {
        "url": "https://chatguru-clickup-middleware.run.app/webhooks/chatguru",
        "method": "POST",
        "headers": {
          "Content-Type": "application/json"
        },
        "body": {
          "annotation": {
            "data": {
              "tarefa": "{{tarefa_descricao}}",
              "prioridade": "{{prioridade}}",
              "responsavel": "{{responsavel}}"
            }
          },
          "contact": {
            "number": "{{chat_number}}",
            "name": "{{contact_name}}"
          }
        }
      }
    }
  ]
}
```

### Configurando Webhook no Diálogo

1. **Via API**:
```javascript
const updateDialog = async (dialogId, webhookUrl) => {
  const response = await fetch(`https://s12.chatguru.app/api/v1/dialogs/${dialogId}`, {
    method: 'PUT',
    headers: {
      'APIKey': 'sua_api_key',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      webhook: webhookUrl,
      actions: [
        {
          type: 'webhook',
          url: webhookUrl,
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          }
        }
      ]
    })
  });
  
  return response.json();
};
```

2. **Via Interface do ChatGuru**:
- Acesse o editor de diálogos
- Selecione o diálogo
- Adicione uma ação de "Webhook/HTTP Request"
- Configure a URL e o formato

## 🔔 Webhooks

### Formato do Webhook ChatGuru

O ChatGuru envia webhooks no seguinte formato:

```json
{
  "event_type": "message_received",
  "timestamp": "2024-01-20T10:00:00Z",
  "annotation": {
    "data": {
      "tarefa": "Descrição da tarefa",
      "prioridade": "Alta",
      "responsavel": "João",
      "custom_field_1": "valor1",
      "custom_field_2": "valor2"
    }
  },
  "contact": {
    "number": "5511999999999",
    "name": "Nome do Contato",
    "profilePicUrl": "https://..."
  },
  "message": {
    "text": "Texto da mensagem",
    "type": "text",
    "timestamp": "2024-01-20T10:00:00Z"
  },
  "dialog": {
    "id": "nova_api",
    "name": "Nova API",
    "execution_id": "exec_123456"
  }
}
```

### Configurando Webhook para Cloud Run

```javascript
const WEBHOOK_URL = "https://chatguru-clickup-middleware-xxxxx-uc.a.run.app/webhooks/chatguru";

// Configurar webhook no diálogo
const configureWebhook = {
  url: WEBHOOK_URL,
  method: "POST",
  headers: {
    "Content-Type": "application/json"
  },
  retry: {
    attempts: 3,
    delay: 1000
  }
};
```

## 📦 Formato de Dados

### Annotation Data
```javascript
{
  "annotation": {
    "data": {
      // Campos personalizados do diálogo
      "campo1": "valor1",
      "campo2": "valor2",
      // Campos específicos para ClickUp
      "tarefa": "Título da tarefa",
      "descricao": "Descrição detalhada",
      "prioridade": "Alta|Normal|Baixa",
      "responsavel": "email@example.com"
    }
  }
}
```

### Contact Data
```javascript
{
  "contact": {
    "number": "5511999999999",
    "name": "Nome do Contato",
    "firstName": "Nome",
    "lastName": "Sobrenome",
    "profilePicUrl": "https://...",
    "customFields": {
      "empresa": "Nome da Empresa",
      "cargo": "Cargo"
    }
  }
}
```

## 🔧 Integração com Middleware

### Middleware esperado pelo ChatGuru

O middleware deve aceitar webhooks no formato:

```rust
#[derive(Deserialize)]
pub struct ChatGuruEvent {
    pub event_type: Option<String>,
    pub id: Option<String>,  // Opcional - gerado se não fornecido
    pub timestamp: Option<String>,
    pub annotation: Option<AnnotationData>,
    pub contact: Option<ContactData>,
    pub data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct AnnotationData {
    pub data: HashMap<String, String>,
}
```

### Processamento no Middleware

```rust
// Extrair dados do webhook
let task_data = event.extract_data();

// Criar tarefa no ClickUp
let clickup_task = ClickUpTask {
    name: task_data.get("tarefa").unwrap_or("Nova Tarefa"),
    description: task_data.get("descricao"),
    priority: map_priority(task_data.get("prioridade")),
    assignees: vec![task_data.get("responsavel")],
};
```

## 🐛 Troubleshooting

### Problema: Webhook não está sendo enviado

1. **Verificar configuração do diálogo**:
```javascript
// Script para verificar
const checkDialog = async (dialogId) => {
  const response = await fetch(`https://s12.chatguru.app/api/v1/dialogs/${dialogId}`, {
    headers: { 'APIKey': 'sua_api_key' }
  });
  
  const dialog = await response.json();
  console.log('Webhook configurado:', dialog.webhook);
  console.log('Ações:', dialog.actions);
  
  // Verificar se tem ação de webhook
  const hasWebhook = dialog.actions?.some(a => a.type === 'webhook');
  console.log('Tem ação webhook?', hasWebhook);
};
```

2. **Testar webhook diretamente**:
```bash
# Testar se o webhook está acessível
curl -X POST https://sua-url-webhook/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "annotation": {
      "data": {
        "tarefa": "Teste direto"
      }
    }
  }'
```

### Problema: Erro 400 - Missing field

**Solução**: Tornar campos opcionais no middleware:
```rust
pub id: Option<String>,  // Ao invés de String
pub event_id: Option<String>,
```

### Problema: Diálogo não encontrado

**Verificar**:
1. Nome exato do diálogo (case sensitive)
2. Se o diálogo está ativo
3. Se está no workspace correto

```javascript
// Listar todos os diálogos
const listDialogs = async () => {
  const response = await fetch('https://s12.chatguru.app/api/v1/dialogs', {
    headers: { 'APIKey': 'sua_api_key' }
  });
  
  const dialogs = await response.json();
  dialogs.forEach(d => {
    console.log(`${d.name} (${d.id}) - Ativo: ${d.active}`);
  });
};
```

## 📝 Checklist de Configuração

- [ ] API Key obtida do ChatGuru
- [ ] Diálogo criado no ChatGuru
- [ ] Webhook configurado no diálogo
- [ ] URL do webhook aponta para o serviço correto
- [ ] Middleware aceita formato do ChatGuru
- [ ] Campos são opcionais no middleware
- [ ] Logs configurados para debug
- [ ] Teste end-to-end funcionando

## 🔗 Links Úteis

- **Documentação Antiga**: https://oldwiki.chatguru.com.br/api/api-documentacao-v1
- **Painel ChatGuru**: https://app.chatguru.app
- **Status da API**: https://status.chatguru.app
- **Suporte**: support@chatguru.com.br

## 📞 Suporte

Para problemas específicos:
1. Verifique os logs do middleware
2. Teste o webhook isoladamente
3. Confirme as credenciais da API
4. Entre em contato com o suporte do ChatGuru

---

*Última atualização: Dezembro 2024*