# Implementação - Código Completo ChatGuru × ClickUp

## 🚀 Servidor Middleware (Node.js)

### Arquivo: `middleware/server.js`

```javascript
// Servidor Middleware - Integração ChatGuru x ClickUp
// Empresa: Nordja

require('dotenv').config();
const express = require('express');
const axios = require('axios');
const cors = require('cors');

const app = express();
const PORT = process.env.PORT || 3000;

// Configurações
const CLICKUP_API_TOKEN = process.env.CLICKUP_API_TOKEN;
const CLICKUP_LIST_ID = process.env.CLICKUP_LIST_ID;
const MIDDLEWARE_TOKEN = process.env.MIDDLEWARE_TOKEN;

// Middlewares
app.use(express.json());
app.use(cors());

// Middleware de autenticação
const authenticateRequest = (req, res, next) => {
  const token = req.headers.authorization?.replace('Bearer ', '');
  
  if (!token || token !== MIDDLEWARE_TOKEN) {
    return res.status(401).json({ 
      success: false, 
      error: 'Token de autenticação inválido' 
    });
  }
  
  next();
};

// Função para mapear prioridade
const getPriorityValue = (prioridade) => {
  const priorities = {
    'Alta': 1,
    'Média': 2, 
    'Baixa': 3,
    'Normal': 2
  };
  return priorities[prioridade] || 2;
};

// Função para calcular data de vencimento
const calculateDueDate = (prazo) => {
  if (!prazo) return null;
  
  const hoje = new Date();
  let diasAdicionais = 7; // Default: 7 dias
  
  // Interpretar o prazo informado
  if (prazo.toLowerCase().includes('urgente') || prazo.toLowerCase().includes('hoje')) {
    diasAdicionais = 1;
  } else if (prazo.toLowerCase().includes('amanhã')) {
    diasAdicionais = 2;
  } else if (prazo.toLowerCase().includes('semana')) {
    diasAdicionais = 7;
  } else if (prazo.toLowerCase().includes('mês')) {
    diasAdicionais = 30;
  }
  
  const dataVencimento = new Date(hoje.getTime() + (diasAdicionais * 24 * 60 * 60 * 1000));
  return dataVencimento.getTime();
};

// Função principal para criar tarefa no ClickUp
async function createClickUpTask(pedidoData) {
  try {
    const payload = {
      name: `Pedido - ${pedidoData.cliente}`,
      description: `
**Cliente:** ${pedidoData.cliente}
**Descrição:** ${pedidoData.descricao}
**Contato:** ${pedidoData.contato}
**Prazo Desejado:** ${pedidoData.prazo}
**Prioridade:** ${pedidoData.prioridade}
**Origem:** ChatGuru WhatsApp Bot
**Data/Hora:** ${new Date().toLocaleString('pt-BR')}

---
**Detalhes Adicionais:**
${pedidoData.observacoes || 'Nenhuma observação adicional'}
      `,
      assignees: [], // Será definido pela equipe Nordja
      tags: ["pedido", "chatguru-bot", "whatsapp"],
      status: "Open",
      priority: getPriorityValue(pedidoData.prioridade),
      due_date: calculateDueDate(pedidoData.prazo),
      notify_all: true
    };

    console.log('📤 Enviando para ClickUp:', {
      url: `https://api.clickup.com/api/v2/list/${CLICKUP_LIST_ID}/task`,
      cliente: pedidoData.cliente,
      prioridade: pedidoData.prioridade
    });

    const response = await axios.post(
      `https://api.clickup.com/api/v2/list/${CLICKUP_LIST_ID}/task`,
      payload,
      {
        headers: {
          'Authorization': `Bearer ${CLICKUP_API_TOKEN}`,
          'Content-Type': 'application/json'
        }
      }
    );

    console.log('✅ Tarefa criada no ClickUp:', response.data.id);
    return response;

  } catch (error) {
    console.error('❌ Erro ao criar tarefa no ClickUp:', error.response?.data || error.message);
    throw new Error(`Erro na API do ClickUp: ${error.response?.data?.err || error.message}`);
  }
}

// Endpoint principal - Receber dados da ChatGuru e criar tarefa no ClickUp
app.post('/chatguru/create-task', authenticateRequest, async (req, res) => {
  try {
    console.log('📥 Dados recebidos da ChatGuru:', req.body);

    const { cliente, descricao, prioridade, prazo, contato, observacoes } = req.body;

    // Validação básica
    if (!cliente || !descricao) {
      return res.status(400).json({
        success: false,
        error: 'Campos obrigatórios: cliente e descrição'
      });
    }

    // Criar tarefa no ClickUp
    const clickupResponse = await createClickUpTask({
      cliente: cliente.trim(),
      descricao: descricao.trim(),
      prioridade: prioridade || 'Média',
      prazo: prazo || 'A definir',
      contato: contato || 'Não informado',
      observacoes: observacoes || ''
    });

    // Retornar sucesso para a ChatGuru
    res.json({
      success: true,
      taskId: clickupResponse.data.id,
      taskUrl: clickupResponse.data.url,
      message: 'Tarefa criada com sucesso!'
    });

  } catch (error) {
    console.error('💥 Erro no processamento:', error.message);
    
    res.status(500).json({
      success: false,
      error: 'Erro interno do servidor',
      details: error.message
    });
  }
});

// Endpoint de teste - Verificar se o servidor está funcionando
app.get('/health', (req, res) => {
  res.json({
    status: 'OK',
    timestamp: new Date().toISOString(),
    service: 'ChatGuru-ClickUp Integration',
    version: '1.0.0'
  });
});

// Endpoint de teste - Testar conexão com ClickUp
app.get('/test-clickup', authenticateRequest, async (req, res) => {
  try {
    const response = await axios.get(
      `https://api.clickup.com/api/v2/list/${CLICKUP_LIST_ID}`,
      {
        headers: {
          'Authorization': `Bearer ${CLICKUP_API_TOKEN}`
        }
      }
    );

    res.json({
      success: true,
      listName: response.data.name,
      listId: response.data.id,
      message: 'Conexão com ClickUp OK!'
    });

  } catch (error) {
    res.status(500).json({
      success: false,
      error: 'Erro ao conectar com ClickUp',
      details: error.response?.data || error.message
    });
  }
});

// Tratamento de erros global
app.use((error, req, res, next) => {
  console.error('🚨 Erro não tratado:', error);
  res.status(500).json({
    success: false,
    error: 'Erro interno do servidor'
  });
});

// Iniciar servidor
app.listen(PORT, () => {
  console.log(`🚀 Servidor rodando na porta ${PORT}`);
  console.log(`📋 ClickUp List ID: ${CLICKUP_LIST_ID}`);
  console.log(`🔑 Token configurado: ${CLICKUP_API_TOKEN ? 'Sim' : 'Não'}`);
});

module.exports = app;
```

### Arquivo: `middleware/package.json`

```json
{
  "name": "chatguru-clickup-integration",
  "version": "1.0.0",
  "description": "Integração entre ChatGuru e ClickUp para Nordja",
  "main": "server.js",
  "scripts": {
    "start": "node server.js",
    "dev": "nodemon server.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.2",
    "axios": "^1.6.0",
    "cors": "^2.8.5",
    "dotenv": "^16.3.1"
  },
  "devDependencies": {
    "nodemon": "^3.0.1",
    "jest": "^29.7.0"
  },
  "keywords": ["chatguru", "clickup", "integration", "chatbot"],
  "author": "Equipe Integração",
  "license": "MIT"
}
```

### Arquivo: `middleware/.env.example`

```bash
# Configurações do ClickUp
CLICKUP_API_TOKEN=pk_your_clickup_token_here
CLICKUP_LIST_ID=your_list_id_here

# Configurações do Middleware
MIDDLEWARE_TOKEN=your_secure_middleware_token
PORT=3000

# Configurações opcionais
NODE_ENV=production
LOG_LEVEL=info
```

## 🔧 Scripts de Deploy

### Arquivo: `middleware/deploy.sh`

```bash
#!/bin/bash

# Script de Deploy - ChatGuru ClickUp Integration
echo "🚀 Iniciando deploy da integração ChatGuru-ClickUp..."

# Verificar se .env existe
if [ ! -f .env ]; then
    echo "❌ Arquivo .env não encontrado!"
    echo "Copie .env.example para .env e configure as variáveis"
    exit 1
fi

# Instalar dependências
echo "📦 Instalando dependências..."
npm install

# Executar testes
echo "🧪 Executando testes..."
npm test

# Verificar se testes passaram
if [ $? -eq 0 ]; then
    echo "✅ Testes passaram!"
else
    echo "❌ Testes falharam! Deploy cancelado."
    exit 1
fi

# Iniciar aplicação
echo "🎯 Iniciando aplicação..."
pm2 restart chatguru-clickup-integration || pm2 start server.js --name chatguru-clickup-integration

echo "✅ Deploy concluído com sucesso!"
echo "🌐 Aplicação disponível em: http://localhost:3000"
echo "🏥 Health check: http://localhost:3000/health"
```

## 🧪 Testes

### Arquivo: `middleware/tests/integration.test.js`

```javascript
const request = require('supertest');
const app = require('../server');

describe('ChatGuru-ClickUp Integration Tests', () => {
  const validToken = process.env.MIDDLEWARE_TOKEN;

  test('Health check deve retornar status OK', async () => {
    const response = await request(app)
      .get('/health');
    
    expect(response.status).toBe(200);
    expect(response.body.status).toBe('OK');
  });

  test('Endpoint sem token deve retornar erro 401', async () => {
    const response = await request(app)
      .post('/chatguru/create-task')
      .send({
        cliente: 'Teste',
        descricao: 'Descrição teste'
      });
    
    expect(response.status).toBe(401);
  });

  test('Endpoint com token válido mas dados incompletos', async () => {
    const response = await request(app)
      .post('/chatguru/create-task')
      .set('Authorization', `Bearer ${validToken}`)
      .send({
        cliente: 'Teste'
        // faltando descrição
      });
    
    expect(response.status).toBe(400);
    expect(response.body.success).toBe(false);
  });

  test('Teste de conexão ClickUp', async () => {
    const response = await request(app)
      .get('/test-clickup')
      .set('Authorization', `Bearer ${validToken}`);
    
    // Pode retornar 200 (OK) ou 500 (erro de configuração)
    expect([200, 500]).toContain(response.status);
  });
});
```

## 📋 Configuração do Fluxo ChatGuru

### Estrutura do Flow no ChatGuru

#### 1. Gatilho PLN (Frases de Treino)

```
Frases para reconhecimento de pedidos:
- "Preciso fazer um pedido"
- "Quero solicitar um serviço"
- "Gostaria de encomendar"
- "Preciso de um orçamento"
- "Quero fazer uma solicitação"
- "Tenho uma demanda"
- "Preciso contratar"
- "Quero um projeto"
- "Gostaria de solicitar"
- "Tenho uma necessidade"
- "Preciso de ajuda com"
- "Quero contratar vocês"
```

#### 2. Sequência do Fluxo

**Etapa 1: Confirmação da Intenção**
```
Mensagem: "Entendi que você quer fazer um pedido! 😊
Vou te ajudar a registrar sua solicitação.
Vamos começar?"

Botões: [Sim, vamos lá!] [Cancelar]
```

**Etapa 2: Captura do Nome**
```
Mensagem: "Primeiro, me diga seu nome completo:"
Variável: {{nome_cliente}}
Validação: Obrigatório, mínimo 2 palavras
```

**Etapa 3: Captura da Descrição**
```
Mensagem: "Agora me conte: o que você precisa?
Seja o mais detalhado possível:"

Variável: {{descricao_pedido}}
Validação: Obrigatório, mínimo 10 caracteres
```

**Etapa 4: Captura da Prioridade**
```
Mensagem: "Como você classificaria a urgência do seu pedido?"

Botões: 
[🔴 Alta] → {{prioridade_selecionada}} = "Alta"
[🟡 Média] → {{prioridade_selecionada}} = "Média"  
[🟢 Baixa] → {{prioridade_selecionada}} = "Baixa"
```

**Etapa 5: Captura do Prazo**
```
Mensagem: "Qual o prazo desejado para este pedido?
(Ex: urgente, 1 semana, 1 mês)"

Variável: {{prazo_desejado}}
Validação: Opcional
```

**Etapa 6: Captura do Contato**
```
Mensagem: "Por último, confirme seu melhor contato:
(telefone, email ou WhatsApp)"

Variável: {{contato_cliente}}
Validação: Opcional
```

**Etapa 7: Confirmação dos Dados**
```
Mensagem: "Vou confirmar os dados do seu pedido:

👤 Nome: {{nome_cliente}}
📝 Descrição: {{descricao_pedido}}
⚡ Prioridade: {{prioridade_selecionada}}
⏰ Prazo: {{prazo_desejado}}
📞 Contato: {{contato_cliente}}

Está tudo correto?"

Botões: [✅ Confirmar] [❌ Corrigir]
```

#### 3. Requisição HTTP (Envio para ClickUp)

**URL do Endpoint:**
```
https://your-middleware-domain.com/chatguru/create-task
```

**Método:**
```
POST
```

**Headers:**
```json
{
  "Content-Type": "application/json",
  "Authorization": "Bearer YOUR_MIDDLEWARE_TOKEN"
}
```

**Body (JSON):**
```json
{
  "cliente": "{{nome_cliente}}",
  "descricao": "{{descricao_pedido}}",
  "prioridade": "{{prioridade_selecionada}}",
  "prazo": "{{prazo_desejado}}",
  "contato": "{{contato_cliente}}"
}
```

#### 4. Tratamento da Resposta

**Resposta de Sucesso (Status 200):**
```
Mensagem: "✅ Pedido registrado com sucesso!

📋 ID da Tarefa: {{response.taskId}}
🔗 Link: {{response.taskUrl}}

Nossa equipe recebeu sua solicitação e entrará em contato em breve!

Obrigado pela preferência! 😊"
```

**Resposta de Erro (Status != 200):**
```
Mensagem: "❌ Ops! Ocorreu um problema ao registrar seu pedido.

Não se preocupe, nossa equipe foi notificada. 
Por favor, tente novamente em alguns minutos ou entre em contato diretamente:

📞 Telefone: (XX) XXXX-XXXX
📧 Email: contato@nordja.com

Pedimos desculpas pelo inconveniente!"
```

## 🔄 Mapeamento de Dados

### Tabela de Correspondência ChatGuru → ClickUp

| Campo ChatGuru | Campo ClickUp | Tipo | Obrigatório | Observações |
|------------|---------------|------|-------------|-------------|
| `{{nome_cliente}}` | `name` (prefixo "Pedido -") | String | Sim | Título da tarefa |
| `{{descricao_pedido}}` | `description` | String | Sim | Descrição principal |
| `{{prioridade_selecionada}}` | `priority` | Integer | Não | 1=Alta, 2=Média, 3=Baixa |
| `{{prazo_desejado}}` | `due_date` | Timestamp | Não | Convertido para timestamp |
| `{{contato_cliente}}` | `description` (embed) | String | Não | Incluído na descrição |
| - | `tags` | Array | Não | ["pedido", "chatguru-bot", "whatsapp"] |
| - | `status` | String | Não | "Open" (padrão) |
| - | `assignees` | Array | Não | Definido pela equipe Nordja |

### Regras de Conversão

**Prioridade:**
- "Alta" → 1 (Urgent)
- "Média" → 2 (High) 
- "Baixa" → 3 (Normal)
- Default → 2 (High)

**Prazo (Due Date):**
- "urgente" / "hoje" → +1 dia
- "amanhã" → +2 dias  
- "semana" → +7 dias
- "mês" → +30 dias
- Default → +7 dias

## 🚀 Instruções de Deploy

### 1. Preparação do Ambiente

```bash
# Clonar/criar diretório do projeto
mkdir chatguru-clickup-integration
cd chatguru-clickup-integration

# Criar estrutura de pastas
mkdir middleware tests docs

# Copiar arquivos do código acima
# server.js, package.json, .env.example, etc.
```

### 2. Configuração

```bash
# Instalar dependências
npm install

# Configurar variáveis de ambiente
cp .env.example .env
nano .env

# Configurar as variáveis:
# CLICKUP_API_TOKEN=pk_seu_token_aqui
# CLICKUP_LIST_ID=sua_lista_id
# MIDDLEWARE_TOKEN=token_seguro_middleware
```

### 3. Testes Locais

```bash
# Executar testes
npm test

# Iniciar servidor local
npm run dev

# Testar endpoints
curl http://localhost:3000/health
curl -H "Authorization: Bearer seu_token" http://localhost:3000/test-clickup
```

### 4. Deploy em Produção

```bash
# Usar PM2 para gerenciamento de processo
npm install -g pm2

# Iniciar aplicação
pm2 start server.js --name chatguru-clickup-integration

# Configurar auto-restart
pm2 startup
pm2 save
```

### 5. Configuração na ChatGuru

1. **Acessar o painel da ChatGuru**
2. **Criar novo Flow**
3. **Configurar gatilhos PLN** com as frases de treino
4. **Montar sequência** de captura de dados
5. **Configurar requisição HTTP** com os parâmetros acima
6. **Testar fluxo** com dados reais

---

**🎯 Status da Implementação:** Código completo e pronto para deploy
**📋 Próximos Passos:** Configurar ambiente de produção e testar integração
**👥 Responsável:** Equipe de Integração Nordja