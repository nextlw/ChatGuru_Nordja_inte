#!/usr/bin/env node

const axios = require('axios');

// URL do middleware (ajuste conforme necessário)
const MIDDLEWARE_URL = 'http://localhost:8080/webhooks/chatguru';

// Exemplo de payload do ChatGuru conforme documentação
const chatguruPayload = {
  "campanha_id": "123456",
  "campanha_nome": "Campanha de Vendas",
  "origem": "ChatGuru",
  "email": "556292650123@c.us",
  "nome": "João Silva",
  "tags": [
    "🤖 Zap.Guru",
    "✅ Fechado e Ganho",
    "Origem: Instagram"
  ],
  "texto_mensagem": "Gostaria de saber mais sobre o produto X e valores",
  "campos_personalizados": {
    "email2": "joao.silva@email.com",
    "Site": "https://www.empresa.com",
    "Valor": "1567.87",
    "CNPJ": "24.111.111/0001-01",
    "Empresa": "EMPRESA S.A"
  },
  "bot_context": {
    "ChatGuru": true
  },
  "responsavel_nome": "Maria Santos",
  "responsavel_email": "maria.santos@empresa.com",
  "link_chat": "https://app2.zap.guru/chats/5e370c9334a812e7e183f760",
  "celular": "556212650015"
};

async function testWebhook() {
  console.log('🚀 Testando webhook do ChatGuru para o middleware...\n');
  console.log('URL:', MIDDLEWARE_URL);
  console.log('Payload:', JSON.stringify(chatguruPayload, null, 2));
  console.log('\n' + '='.repeat(60) + '\n');

  try {
    const response = await axios.post(MIDDLEWARE_URL, chatguruPayload, {
      headers: {
        'Content-Type': 'application/json',
        // Se houver autenticação/assinatura, adicione aqui
        // 'X-ChatGuru-Signature': 'sha256=...'
      }
    });

    console.log('✅ Webhook processado com sucesso!');
    console.log('\nStatus:', response.status);
    console.log('\nResposta do servidor:');
    console.log(JSON.stringify(response.data, null, 2));

    // Verificar se a tarefa foi criada no ClickUp
    if (response.data.clickup_task_id) {
      console.log('\n📋 Tarefa criada no ClickUp:');
      console.log('   ID:', response.data.clickup_task_id);
      console.log('   Ação:', response.data.clickup_task_action || 'created');
    }

  } catch (error) {
    console.error('❌ Erro ao enviar webhook:');
    
    if (error.response) {
      // Erro retornado pelo servidor
      console.error('Status:', error.response.status);
      console.error('Dados:', error.response.data);
      console.error('\nDetalhes do erro:');
      if (error.response.data.error) {
        console.error('  -', error.response.data.error);
      }
      if (error.response.data.details) {
        console.error('  Detalhes:', error.response.data.details);
      }
    } else if (error.request) {
      // Requisição foi feita mas não houve resposta
      console.error('Sem resposta do servidor. Verifique se o middleware está rodando.');
      console.error('Execute: cargo run');
    } else {
      // Erro ao configurar a requisição
      console.error('Erro:', error.message);
    }
  }
}

// Função para testar múltiplos cenários
async function testMultipleScenarios() {
  console.log('🧪 Testando múltiplos cenários...\n');

  // Cenário 1: Webhook normal
  console.log('1. Webhook com dados completos:');
  await testWebhook();

  // Aguardar 2 segundos
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Cenário 2: Webhook sem responsável
  console.log('\n' + '='.repeat(60) + '\n');
  console.log('2. Webhook sem responsável:');
  const payloadSemResponsavel = { ...chatguruPayload };
  delete payloadSemResponsavel.responsavel_nome;
  delete payloadSemResponsavel.responsavel_email;
  
  try {
    const response = await axios.post(MIDDLEWARE_URL, payloadSemResponsavel);
    console.log('✅ Processado com sucesso');
  } catch (error) {
    console.error('❌ Erro:', error.response?.data || error.message);
  }

  // Aguardar 2 segundos
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Cenário 3: Webhook com a mesma campanha/nome (deve atualizar)
  console.log('\n' + '='.repeat(60) + '\n');
  console.log('3. Webhook duplicado (deve atualizar tarefa existente):');
  const payloadDuplicado = { 
    ...chatguruPayload,
    texto_mensagem: "ATUALIZAÇÃO: Cliente confirmou interesse e pediu orçamento"
  };
  
  try {
    const response = await axios.post(MIDDLEWARE_URL, payloadDuplicado);
    console.log('✅ Processado com sucesso');
    if (response.data.clickup_task_action === 'updated') {
      console.log('   ✓ Tarefa atualizada corretamente');
    }
  } catch (error) {
    console.error('❌ Erro:', error.response?.data || error.message);
  }
}

// Verificar parâmetros da linha de comando
const args = process.argv.slice(2);
if (args.includes('--multiple') || args.includes('-m')) {
  testMultipleScenarios();
} else {
  testWebhook();
}