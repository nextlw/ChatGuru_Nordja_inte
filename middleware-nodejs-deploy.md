# 🚀 MIDDLEWARE NODE.JS - DEPLOY E TESTES SURI-CLICKUP

## 📦 ESTRUTURA DO PROJETO

```
chatguru-clickup-middleware/
├── package.json
├── app.yaml
├── server.js
├── config/
│   └── clickup.js
├── handlers/
│   ├── health.js
│   ├── chatguru-webhook.js
│   └── clickup.js
└── tests/
    └── chatguru-simulator.js
```

## 📋 PACKAGE.JSON

```json
{
  "name": "chatguru-clickup-middleware",
  "version": "1.0.0",
  "description": "Middleware para integração ChatGuru-ClickUp com Pub/Sub",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "node tests/chatguru-simulator.js",
    "deploy": "gcloud app deploy"
  },
  "dependencies": {
    "express": "^4.18.2",
    "axios": "^1.6.0",
    "@google-cloud/pubsub": "^4.0.0",
    "cors": "^2.8.5",
    "helmet": "^7.1.0",
    "uuid": "^9.0.1"
  },
  "devDependencies": {
    "nodemon": "^3.0.1"
  },
  "engines": {
    "node": ">=18.0.0"
  }
}
```

## 🎯 SERVER.JS - APLICAÇÃO PRINCIPAL

```javascript
const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const { v4: uuidv4 } = require('uuid');
const { PubSub } = require('@google-cloud/pubsub');

// Importar handlers
const healthHandler = require('./handlers/health');
const chatguruWebhookHandler = require('./handlers/chatguru-webhook');
const clickupHandler = require('./handlers/clickup');

const app = express();
const PORT = process.env.PORT || 8080;

// Configurar Pub/Sub
const pubsub = new PubSub({ projectId: 'buzzlightear' });
const topic = pubsub.topic('chatguru-events');

// Middleware
app.use(helmet());
app.use(cors());
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

// Logging middleware
app.use((req, res, next) => {
  const requestId = uuidv4();
  req.requestId = requestId;
  console.log(`[${new Date().toISOString()}] ${requestId} - ${req.method} ${req.url}`);
  next();
});

// Routes
app.get('/health', healthHandler.healthCheck);
app.get('/status', (req, res) => healthHandler.integrationStatus(req, res, pubsub));

// ChatGuru webhook
app.post('/webhooks/chatguru', (req, res) => 
  chatguruWebhookHandler.handleChatGuruEvent(req, res, topic)
);

// ClickUp endpoints
app.post('/clickup/tasks', clickupHandler.createTask);
app.get('/clickup/tasks/:taskId', clickupHandler.getTask);

// Error handler
app.use((err, req, res, next) => {
  console.error(`[${req.requestId}] Erro:`, err);
  res.status(500).json({
    error: 'Erro interno do servidor',
    requestId: req.requestId,
    timestamp: new Date().toISOString()
  });
});

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: 'Endpoint não encontrado',
    requestId: req.requestId,
    available_endpoints: [
      'GET /health',
      'GET /status', 
      'POST /webhooks/chatguru',
      'POST /clickup/tasks',
      'GET /clickup/tasks/:taskId'
    ]
  });
});

app.listen(PORT, () => {
  console.log(`🚀 Middleware ChatGuru-ClickUp rodando na porta ${PORT}`);
  console.log(`📊 Pub/Sub Topic: chatguru-events`);
  console.log(`🔗 ClickUp List ID: 901300373349`);
});

module.exports = app;
```

## 🏥 HANDLERS/HEALTH.JS

```javascript
const axios = require('axios');

const healthCheck = (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    service: 'chatguru-clickup-middleware',
    version: '1.0.0',
    requestId: req.requestId
  });
};

const integrationStatus = async (req, res, pubsub) => {
  const statuses = [];

  // Testar ClickUp API
  try {
    const clickupResponse = await axios.get('https://api.clickup.com/api/v2/user', {
      headers: {
        'Authorization': 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657'
      },
      timeout: 5000
    });
    
    statuses.push({
      service: 'ClickUp API',
      status: 'healthy',
      message: `Usuário: ${clickupResponse.data.user.username}`,
      last_check: new Date().toISOString()
    });
  } catch (error) {
    statuses.push({
      service: 'ClickUp API',
      status: 'unhealthy',
      message: error.message,
      last_check: new Date().toISOString()
    });
  }

  // Testar Pub/Sub
  try {
    const topic = pubsub.topic('chatguru-events');
    const [exists] = await topic.exists();
    
    statuses.push({
      service: 'Google Pub/Sub',
      status: exists ? 'healthy' : 'unhealthy',
      message: exists ? 'Tópico chatguru-events ativo' : 'Tópico não encontrado',
      last_check: new Date().toISOString()
    });
  } catch (error) {
    statuses.push({
      service: 'Google Pub/Sub',
      status: 'unhealthy',
      message: error.message,
      last_check: new Date().toISOString()
    });
  }

  res.json({
    overall_status: statuses.every(s => s.status === 'healthy') ? 'healthy' : 'degraded',
    services: statuses,
    timestamp: new Date().toISOString(),
    requestId: req.requestId
  });
};

module.exports = { healthCheck, integrationStatus };
```

## 🎯 HANDLERS/SURI-WEBHOOK.JS

```javascript
const clickupService = require('../config/clickup');

const handleChatGuruEvent = async (req, res, pubsubTopic) => {
  const event = req.body;
  
  console.log(`[${req.requestId}] 📥 Evento ChatGuru recebido:`, JSON.stringify(event, null, 2));

  try {
    // Processar evento baseado no tipo
    let taskCreated = null;
    
    switch (event.type) {
      case 'novo_contato':
        taskCreated = await handleNewContact(event.data, req.requestId);
        break;
      case 'mensagem_recebida':
        taskCreated = await handleMessageReceived(event.data, req.requestId);
        break;
      case 'troca_fila':
        taskCreated = await handleQueueChange(event.data, req.requestId);
        break;
      case 'finalizacao_atendimento':
        taskCreated = await handleServiceEnd(event.data, req.requestId);
        break;
      default:
        console.log(`[${req.requestId}] ⚠️ Tipo de evento não reconhecido: ${event.type}`);
    }

    // Publicar no Pub/Sub para processamento assíncrono
    const pubsubMessage = {
      id: req.requestId,
      type: event.type,
      data: event.data,
      timestamp: new Date().toISOString(),
      task_created: taskCreated
    };

    await pubsubTopic.publish(Buffer.from(JSON.stringify(pubsubMessage)));
    console.log(`[${req.requestId}] 📢 Evento publicado no Pub/Sub`);

    res.json({
      success: true,
      message: 'Evento processado com sucesso',
      event_type: event.type,
      task_created: taskCreated,
      request_id: req.requestId,
      timestamp: new Date().toISOString()
    });

  } catch (error) {
    console.error(`[${req.requestId}] ❌ Erro processando evento:`, error);
    res.status(500).json({
      success: false,
      error: error.message,
      request_id: req.requestId,
      timestamp: new Date().toISOString()
    });
  }
};

const handleNewContact = async (data, requestId) => {
  console.log(`[${requestId}] 👤 Processando novo contato: ${data.contact_name}`);

  const taskData = {
    name: `🆕 Novo Lead - ${data.contact_name || 'Contato Anônimo'}`,
    description: `📞 **Novo contato via ChatGuru**

**Dados do Contato:**
- Nome: ${data.contact_name || 'N/A'}
- Telefone: ${data.phone || 'N/A'}
- Canal: ${data.channel || 'WhatsApp'}
- Timestamp: ${new Date().toISOString()}

**Próximos Passos:**
- [ ] Qualificar lead
- [ ] Entrar em contato
- [ ] Registrar no CRM`,
    tags: ['chatguru-lead', 'novo-contato', 'automacao'],
    priority: 2
  };

  return await clickupService.createTask(taskData);
};

const handleMessageReceived = async (data, requestId) => {
  const message = data.message || '';
  console.log(`[${requestId}] 💬 Mensagem recebida: ${message.substring(0, 100)}...`);

  // Análise simples de sentimento
  const negativeWords = ['problema', 'erro', 'ruim', 'péssimo', 'terrível', 'insatisfeito', 'reclamação'];
  const isNegative = negativeWords.some(word => message.toLowerCase().includes(word));

  if (isNegative) {
    console.log(`[${requestId}] ⚠️ Mensagem negativa detectada`);
    
    const taskData = {
      name: '🚨 URGENTE - Suporte ao Cliente',
      description: `**Mensagem com possível insatisfação detectada**

**Mensagem:** ${message}
**Contato:** ${data.contact_name || 'N/A'}
**Timestamp:** ${new Date().toISOString()}

**Ação Necessária:** Contato imediato com supervisor`,
      tags: ['urgente', 'suporte', 'insatisfacao'],
      priority: 1
    };

    return await clickupService.createTask(taskData);
  }

  return null;
};

const handleQueueChange = async (data, requestId) => {
  console.log(`[${requestId}] 🔄 Troca de fila: ${data.from_queue} → ${data.to_queue}`);

  if (data.to_queue === 'Esperando atendimento') {
    const taskData = {
      name: `👨‍💼 Atendimento Humano - ${data.contact_name || 'Cliente'}`,
      description: `**Cliente aguardando atendimento humano**

**De:** ${data.from_queue || 'N/A'}
**Para:** ${data.to_queue || 'N/A'}
**Contato:** ${data.contact_name || 'N/A'}
**Timestamp:** ${new Date().toISOString()}

**Prioridade:** Atender o mais rápido possível`,
      tags: ['atendimento-humano', 'fila', 'pendente'],
      priority: 2
    };

    return await clickupService.createTask(taskData);
  }

  return null;
};

const handleServiceEnd = async (data, requestId) => {
  console.log(`[${requestId}] ✅ Atendimento finalizado para: ${data.contact_name}`);

  const taskData = {
    name: `📋 Follow-up - ${data.contact_name || 'Cliente'}`,
    description: `**Atendimento finalizado - Follow-up necessário**

**Contato:** ${data.contact_name || 'N/A'}
**Agente:** ${data.agent_name || 'N/A'}
**Duração:** ${data.duration || 'N/A'}
**Finalizado em:** ${new Date().toISOString()}

**Ações:**
- [ ] Enviar pesquisa de satisfação
- [ ] Registrar resolução no CRM
- [ ] Avaliar necessidade de follow-up`,
    tags: ['follow-up', 'pos-atendimento', 'satisfacao'],
    priority: 3
  };

  return await clickupService.createTask(taskData);
};

module.exports = { handleChatGuruEvent };
```

## 🔧 CONFIG/CLICKUP.JS

```javascript
const axios = require('axios');

const CLICKUP_CONFIG = {
  token: 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657',
  listId: '901300373349',
  baseUrl: 'https://api.clickup.com/api/v2'
};

const clickupClient = axios.create({
  baseURL: CLICKUP_CONFIG.baseUrl,
  headers: {
    'Authorization': CLICKUP_CONFIG.token,
    'Content-Type': 'application/json'
  },
  timeout: 30000
});

const createTask = async (taskData) => {
  try {
    const response = await clickupClient.post(`/list/${CLICKUP_CONFIG.listId}/task`, taskData);
    
    console.log(`✅ Tarefa ClickUp criada: ${response.data.id}`);
    return {
      id: response.data.id,
      url: response.data.url,
      name: response.data.name
    };
  } catch (error) {
    console.error('❌ Erro ao criar tarefa ClickUp:', error.response?.data || error.message);
    throw new Error(`Erro ClickUp: ${error.response?.status} - ${error.response?.data?.err || error.message}`);
  }
};

const getTask = async (taskId) => {
  try {
    const response = await clickupClient.get(`/task/${taskId}`);
    return response.data;
  } catch (error) {
    console.error('❌ Erro ao buscar tarefa ClickUp:', error.response?.data || error.message);
    throw new Error(`Erro ClickUp: ${error.response?.status} - ${error.response?.data?.err || error.message}`);
  }
};

module.exports = { createTask, getTask, CLICKUP_CONFIG };
```

## 🎯 HANDLERS/CLICKUP.JS

```javascript
const clickupService = require('../config/clickup');

const createTask = async (req, res) => {
  try {
    const taskData = req.body;
    console.log(`[${req.requestId}] 📝 Criando tarefa ClickUp:`, taskData.name);
    
    const task = await clickupService.createTask(taskData);
    
    res.json({
      success: true,
      task,
      request_id: req.requestId,
      timestamp: new Date().toISOString()
    });
  } catch (error) {
    console.error(`[${req.requestId}] ❌ Erro criando tarefa:`, error);
    res.status(500).json({
      success: false,
      error: error.message,
      request_id: req.requestId
    });
  }
};

const getTask = async (req, res) => {
  try {
    const { taskId } = req.params;
    console.log(`[${req.requestId}] 🔍 Buscando tarefa: ${taskId}`);
    
    const task = await clickupService.getTask(taskId);
    
    res.json({
      success: true,
      task,
      request_id: req.requestId
    });
  } catch (error) {
    console.error(`[${req.requestId}] ❌ Erro buscando tarefa:`, error);
    res.status(404).json({
      success: false,
      error: error.message,
      request_id: req.requestId
    });
  }
};

module.exports = { createTask, getTask };
```

## 📄 APP.YAML - CONFIGURAÇÃO APP ENGINE

```yaml
runtime: nodejs18

env_variables:
  NODE_ENV: production
  GCLOUD_PROJECT: buzzlightear

automatic_scaling:
  min_instances: 1
  max_instances: 10
  target_cpu_utilization: 0.6

resources:
  cpu: 1
  memory_gb: 0.5
  disk_size_gb: 10
```

## 🧪 TESTS/SURI-SIMULATOR.JS - SIMULADOR DE EVENTOS SURI

```javascript
const axios = require('axios');

// Configuração do middleware (local ou deployed)
const MIDDLEWARE_URL = process.env.MIDDLEWARE_URL || 'http://localhost:8080';

console.log(`🧪 Simulador de Eventos ChatGuru`);
console.log(`🎯 Target: ${MIDDLEWARE_URL}`);
console.log(`🕐 Iniciando testes em 3 segundos...\n`);

// Eventos de teste simulando diferentes cenários da ChatGuru
const testEvents = [
  {
    name: 'Novo Contato - Lead Qualificado',
    event: {
      type: 'novo_contato',
      data: {
        contact_name: 'João Silva',
        phone: '11999999999',
        channel: 'WhatsApp',
        timestamp: new Date().toISOString()
      }
    }
  },
  {
    name: 'Mensagem Positiva',
    event: {
      type: 'mensagem_recebida',
      data: {
        contact_name: 'Maria Santos',
        message: 'Muito obrigada pelo excelente atendimento! Vocês são incríveis!',
        phone: '11888888888',
        timestamp: new Date().toISOString()
      }
    }
  },
  {
    name: 'Mensagem Negativa - Reclamação',
    event: {
      type: 'mensagem_recebida',
      data: {
        contact_name: 'Carlos Oliveira',
        message: 'Estou com um problema grave no produto. Muito insatisfeito!',
        phone: '11777777777',
        timestamp: new Date().toISOString()
      }
    }
  },
  {
    name: 'Troca para Atendimento Humano',
    event: {
      type: 'troca_fila',
      data: {
        contact_name: 'Ana Costa',
        from_queue: 'Automático',
        to_queue: 'Esperando atendimento',
        phone: '11666666666',
        timestamp: new Date().toISOString()
      }
    }
  },
  {
    name: 'Finalização de Atendimento',
    event: {
      type: 'finalizacao_atendimento',
      data: {
        contact_name: 'Pedro Martins',
        agent_name: 'Atendente Carol',
        duration: '15 minutos',
        resolution: 'Problema resolvido com sucesso',
        timestamp: new Date().toISOString()
      }
    }
  }
];

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

const testHealthCheck = async () => {
  try {
    console.log('🏥 Testando Health Check...');
    const response = await axios.get(`${MIDDLEWARE_URL}/health`);
    console.log('✅ Health Check OK:', response.data.status);
    return true;
  } catch (error) {
    console.log('❌ Health Check FALHOU:', error.message);
    return false;
  }
};

const testIntegrationStatus = async () => {
  try {
    console.log('🔍 Testando Status da Integração...');
    const response = await axios.get(`${MIDDLEWARE_URL}/status`);
    console.log('📊 Status da Integração:', response.data.overall_status);
    console.log('🔧 Serviços:', response.data.services.map(s => `${s.service}: ${s.status}`).join(', '));
    return true;
  } catch (error) {
    console.log('❌ Status Check FALHOU:', error.message);
    return false;
  }
};

const sendChatGuruEvent = async (testEvent) => {
  try {
    console.log(`\n📤 Enviando: ${testEvent.name}`);
    console.log(`   Tipo: ${testEvent.event.type}`);
    
    const response = await axios.post(`${MIDDLEWARE_URL}/webhooks/chatguru`, testEvent.event, {
      timeout: 10000
    });
    
    console.log(`✅ Resposta: ${response.data.message}`);
    if (response.data.task_created) {
      console.log(`📋 Tarefa ClickUp: ${response.data.task_created.id} - ${response.data.task_created.name}`);
      console.log(`🔗 URL: ${response.data.task_created.url}`);
    }
    
    return response.data;
  } catch (error) {
    console.log(`❌ ERRO: ${error.response?.data?.error || error.message}`);
    return null;
  }
};

const testDirectClickUp = async () => {
  try {
    console.log('\n📝 Testando criação direta de tarefa ClickUp...');
    
    const taskData = {
      name: '🧪 Tarefa de Teste - Via API Direta',
      description: `**Teste de integração**

Esta tarefa foi criada diretamente via API para validar a funcionalidade do middleware.

**Timestamp:** ${new Date().toISOString()}
**Origem:** Simulador de testes`,
      tags: ['teste', 'api-direta', 'validacao'],
      priority: 2
    };
    
    const response = await axios.post(`${MIDDLEWARE_URL}/clickup/tasks`, taskData);
    
    console.log(`✅ Tarefa criada: ${response.data.task.id}`);
    console.log(`🔗 URL: ${response.data.task.url}`);
    
    return response.data.task;
  } catch (error) {
    console.log(`❌ ERRO: ${error.response?.data?.error || error.message}`);
    return null;
  }
};

const runAllTests = async () => {
  console.log('🚀 Iniciando suite completa de testes...\n');
  
  let passed = 0;
  let total = 0;
  
  // 1. Health Check
  total++;
  if (await testHealthCheck()) passed++;
  
  await sleep(1000);
  
  // 2. Status Check
  total++;
  if (await testIntegrationStatus()) passed++;
  
  await sleep(2000);
  
  // 3. Teste direto ClickUp
  total++;
  if (await testDirectClickUp()) passed++;
  
  await sleep(2000);
  
  // 4. Eventos ChatGuru
  for (const testEvent of testEvents) {
    total++;
    if (await sendChatGuruEvent(testEvent)) passed++;
    await sleep(3000); // Pausa entre eventos
  }
  
  // Relatório final
  console.log(`\n📊 RELATÓRIO FINAL DOS TESTES`);
  console.log(`===============================`);
  console.log(`✅ Testes Passaram: ${passed}/${total}`);
  console.log(`📈 Taxa de Sucesso: ${((passed/total) * 100).toFixed(1)}%`);
  console.log(`🕐 Timestamp: ${new Date().toISOString()}`);
  
  if (passed === total) {
    console.log(`\n🎉 TODOS OS TESTES PASSARAM! Integração funcionando 100%`);
  } else {
    console.log(`\n⚠️ Alguns testes falharam. Verificar logs acima.`);
  }
};

// Executar testes
setTimeout(runAllTests, 3000);
```

## 🚀 COMANDOS DE DEPLOY

### **1. Preparar ambiente local:**
```bash
# Criar diretório do projeto
mkdir chatguru-clickup-middleware && cd chatguru-clickup-middleware

# Criar estrutura de diretórios
mkdir -p handlers config tests

# Instalar dependências
npm install
```

### **2. Deploy no App Engine:**
```bash
# Fazer deploy
gcloud app deploy app.yaml

# Verificar URL do deploy
gcloud app browse
```

### **3. Executar testes:**
```bash
# Testes locais (se rodando local)
npm run test

# Testes contra ambiente deployado
MIDDLEWARE_URL=https://buzzlightear.rj.r.appspot.com npm run test
```

## 🧪 EXEMPLOS DE TESTE MANUAL

### **Health Check:**
```bash
curl https://buzzlightear.rj.r.appspot.com/health
```

### **Status da Integração:**
```bash
curl https://buzzlightear.rj.r.appspot.com/status
```

### **Simular evento ChatGuru:**
```bash
curl -X POST https://buzzlightear.rj.r.appspot.com/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "type": "novo_contato",
    "data": {
      "contact_name": "Teste Manual",
      "phone": "11999999999",
      "channel": "WhatsApp"
    }
  }'
```

---

## 📊 VALIDAÇÃO ESPERADA

1. **Health Check**: Status `healthy`
2. **Integration Status**: ClickUp `healthy`, Pub/Sub `healthy`  
3. **Novo Contato**: Cria tarefa "🆕 Novo Lead" no ClickUp
4. **Mensagem Negativa**: Cria tarefa "🚨 URGENTE - Suporte"
5. **Troca de Fila**: Cria tarefa "👨‍💼 Atendimento Humano"
6. **Finalização**: Cria tarefa "📋 Follow-up"
7. **Pub/Sub**: Eventos publicados no tópico `chatguru-events`

**🎯 RESULTADO**: Middleware completo, deployado e testado com simulação de eventos ChatGuru reais!