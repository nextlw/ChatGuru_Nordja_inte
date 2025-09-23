# Como Configurar o Diálogo nova_api para Criar Anotações

## 🎯 Objetivo
Configurar o diálogo `nova_api` no ChatGuru para que, ao ser acionado, crie automaticamente uma anotação formatada no chat.

## 📋 Opções para Criar Anotações

### Opção 1: Usar Ação de Anotação no Diálogo (RECOMENDADO)

No painel do ChatGuru, ao configurar o diálogo `nova_api`:

1. **Acesse o ChatGuru**
   - Entre no painel: https://app.chatguru.com ou https://s15.chatguru.app
   - Vá para Diálogos/Flows

2. **Edite o diálogo nova_api**

3. **Adicione uma Ação de Anotação:**
   ```
   Tipo de Ação: Adicionar Anotação
   Conteúdo da Anotação: 
   
   Tarefa: {{tarefa}}
   Tipo de Atividade: {{tipo_atividade}}
   Categoria: {{categoria}}
   Prioridade: {{prioridade}}
   Responsável: {{responsavel}}
   Descrição: {{descricao}}
   Subtarefas:
   - {{subtarefa1}}
   - {{subtarefa2}}
   ```

4. **Configure as Variáveis:**
   - Use extração de entidades da mensagem do usuário
   - Ou defina valores padrão para campos não identificados

### Opção 2: Usar Webhook com Resposta de Anotação

Configure o webhook para retornar uma instrução de anotação:

```javascript
// No seu webhook (buzzlightear ou outro)
app.post('/webhook', (req, res) => {
  const { chat_number, variables, dialog_id } = req.body;
  
  // Processar dados...
  
  // Retornar instrução para criar anotação
  res.json({
    action: "add_annotation",
    annotation: {
      text: `Tarefa: ${variables.tarefa || 'Não especificada'}
Tipo de Atividade: ${variables.tipo || 'Geral'}
Categoria: ${variables.categoria || 'Atividades em geral'}
Prioridade: ${variables.prioridade || 'Normal'}
Responsável: ${variables.responsavel || 'A definir'}
Descrição: ${variables.descricao || 'Sem descrição'}
Subtarefas:
- ${variables.subtarefa1 || 'Subtarefa 1'}
- ${variables.subtarefa2 || 'Subtarefa 2'}`
    }
  });
});
```

### Opção 3: Usar API do ChatGuru (Via Webhook)

Seu webhook pode chamar a API do ChatGuru para adicionar a anotação:

```javascript
// No webhook que recebe o evento
app.post('/webhook', async (req, res) => {
  const { chat_number, variables } = req.body;
  
  // Criar anotação via API do ChatGuru
  const annotation = `Tarefa: ${variables.tarefa}
Tipo de Atividade: ${variables.tipo}
Categoria: ${variables.categoria}
Prioridade: ${variables.prioridade}
Responsável: ${variables.responsavel}
Descrição: ${variables.descricao}`;

  // Chamar API do ChatGuru para adicionar anotação
  await fetch('https://s15.chatguru.app/note_add', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'APIKey': 'SUA_API_KEY'
    },
    body: JSON.stringify({
      chat_number: chat_number,
      note: annotation,
      key: 'SUA_API_KEY',
      account_id: 'SEU_ACCOUNT_ID',
      phone_id: 'SEU_PHONE_ID'
    })
  });
  
  res.json({ message: "Success" });
});
```

## 🔧 Configuração Passo a Passo no ChatGuru

### 1. Estrutura do Diálogo nova_api

```yaml
Nome: nova_api
Descrição: Identificar e processar tarefas

Gatilhos:
  - Palavras-chave: tarefa, atividade, fazer, buscar, criar, desenvolver
  - Intenção: task_creation

Etapas:
  1. Identificação:
     - Extrair entidade: tarefa (o que fazer)
     - Extrair entidade: tipo (tipo de atividade)
     - Extrair entidade: categoria
     - Extrair entidade: prioridade
  
  2. Processamento:
     - Webhook: https://buzzlightear.rj.r.appspot.com/webhook
     - Enviar variáveis extraídas
  
  3. Anotação:
     - Tipo: Adicionar Anotação
     - Template: [usar template acima]
  
  4. Resposta:
     - Mensagem: "✅ Tarefa registrada com sucesso!"
```

### 2. Configurar Extração de Entidades

No ChatGuru, configure o NLP para identificar:

```javascript
// Exemplos de frases e entidades
"Pode buscar documentações de API para mim?" 
→ tarefa: "buscar documentações de API"
→ tipo: "Pesquisa"
→ categoria: "Documentação"

"Criar uma landing page com formulário de contato"
→ tarefa: "criar landing page"
→ tipo: "Desenvolvimento"
→ categoria: "Frontend"

"Desenvolver integração com Stripe urgente"
→ tarefa: "desenvolver integração com Stripe"
→ tipo: "Desenvolvimento"
→ prioridade: "Alta"
```

## 📝 Template de Anotação Recomendado

```text
📋 NOVA TAREFA IDENTIFICADA
━━━━━━━━━━━━━━━━━━━━━━
📌 Tarefa: {{tarefa}}
📊 Tipo: {{tipo_atividade}}
📁 Categoria: {{categoria}}
🔴 Prioridade: {{prioridade}}
👤 Responsável: {{responsavel}}
📝 Descrição: {{descricao}}

📍 Subtarefas:
{{#if subtarefas}}
{{#each subtarefas}}
  • {{this}}
{{/each}}
{{else}}
  • Análise inicial
  • Implementação
  • Testes
{{/if}}

⏰ Criado em: {{timestamp}}
💬 Chat: {{chat_number}}
━━━━━━━━━━━━━━━━━━━━━━
```

## 🚀 Script de Teste

Salve como `test-nova-api-annotation.js`:

```javascript
const axios = require('axios');

async function testNovaApiWithAnnotation() {
  // Simular mensagem do usuário
  const userMessage = "Pode buscar documentações de API para mim?";
  
  // Webhook do ChatGuru simula o diálogo nova_api
  const webhookPayload = {
    event_type: 'dialog.executed',
    dialog_id: 'nova_api',
    chat_number: '5585989530473',
    variables: {
      tarefa: 'Buscar documentações de API',
      tipo_atividade: 'Pesquisa',
      categoria: 'Documentação',
      prioridade: 'Normal',
      responsavel: 'Time de Dev',
      descricao: 'Pesquisar e compilar documentações de APIs relevantes',
      subtarefa1: 'Identificar APIs necessárias',
      subtarefa2: 'Organizar documentação'
    },
    timestamp: new Date().toISOString()
  };
  
  // Enviar para webhook
  const response = await axios.post(
    'https://buzzlightear.rj.r.appspot.com/webhook',
    webhookPayload
  );
  
  console.log('Resposta do webhook:', response.data);
  
  // Se o webhook retornar instrução de anotação
  if (response.data.action === 'add_annotation') {
    console.log('Anotação a ser criada:', response.data.annotation.text);
  }
}

testNovaApiWithAnnotation();
```

## ⚙️ Integração com seu Middleware

Seu middleware já está configurado para receber eventos de anotação e criar tarefas no ClickUp. O fluxo completo será:

1. **Usuário** envia mensagem
2. **ChatGuru** identifica o diálogo nova_api
3. **ChatGuru** executa o diálogo:
   - Envia webhook para buzzlightear
   - Cria anotação no chat
4. **ChatGuru** dispara evento `annotation.added`
5. **Seu middleware** recebe o evento e cria tarefa no ClickUp

## 🎯 Resultado Esperado

Quando o usuário enviar uma mensagem que acione o nova_api, verá no chat:

```
Usuário: Pode buscar documentações de API para mim?

Chatbot: ✅ Tarefa registrada com sucesso!

Chatbot anotou:
📋 NOVA TAREFA IDENTIFICADA
━━━━━━━━━━━━━━━━━━━━━━
📌 Tarefa: Buscar documentações de API
📊 Tipo: Pesquisa
📁 Categoria: Documentação
🔴 Prioridade: Normal
👤 Responsável: Time de Dev
📝 Descrição: Pesquisar e compilar documentações

📍 Subtarefas:
  • Identificar APIs necessárias
  • Organizar documentação
━━━━━━━━━━━━━━━━━━━━━━
```

E automaticamente será criada uma tarefa no ClickUp com esses dados!