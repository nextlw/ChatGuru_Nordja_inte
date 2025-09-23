#!/usr/bin/env node

const axios = require('axios');

// URL do middleware (ajuste conforme necessário)
const MIDDLEWARE_URL = 'http://localhost:8080/webhooks/chatguru';

// Exemplo de payload do ChatGuru com dados DIFERENTES
const chatguruPayload = {
  "campanha_id": "789012",
  "campanha_nome": "Campanha de Suporte",
  "origem": "ChatGuru",
  "email": "5562999887766@c.us",
  "nome": "Maria Oliveira",
  "tags": [
    "🤖 Zap.Guru",
    "📞 Suporte Técnico",
    "Origem: WhatsApp"
  ],
  "texto_mensagem": "Preciso de ajuda com a configuração do sistema",
  "campos_personalizados": {
    "email2": "maria.oliveira@example.com",
    "Site": "https://www.mariatech.com",
    "Valor": "2500.00",
    "CNPJ": "35.222.333/0001-44",
    "Empresa": "MARIA TECH LTDA"
  },
  "bot_context": {
    "ChatGuru": true
  },
  "responsavel_nome": "Carlos Andrade",
  "responsavel_email": "carlos.andrade@empresa.com",
  "link_chat": "https://app2.zap.guru/chats/6e480d0445b923f8f294g871",
  "celular": "5562999887766"
};

async function testWebhook() {
  console.log('🚀 Testando webhook do ChatGuru para o middleware (DADOS DIFERENTES)...\n');
  console.log('URL:', MIDDLEWARE_URL);
  console.log('Payload:', JSON.stringify(chatguruPayload, null, 2));
  console.log('\n' + '='.repeat(60) + '\n');

  try {
    const response = await axios.post(MIDDLEWARE_URL, chatguruPayload, {
      headers: {
        'Content-Type': 'application/json',
      }
    });

    console.log('✅ Webhook processado com sucesso!\n');
    console.log('Status:', response.status);
    console.log('\nResposta do servidor:');
    console.log(JSON.stringify(response.data, null, 2));

    // Verifica se foi criado ou atualizado
    if (response.data.clickup_task_id) {
      console.log('\n📋 Tarefa no ClickUp:');
      console.log(`   ID: ${response.data.clickup_task_id}`);
      console.log(`   Ação: ${response.data.clickup_task_action || 'processada'}`);
    }

  } catch (error) {
    console.error('❌ Erro ao enviar webhook:');
    if (error.response) {
      console.log('Status:', error.response.status);
      console.log('Dados:', error.response.data);
    } else {
      console.log('Erro:', error.message);
    }
    
    if (error.response?.data?.error) {
      console.log('\nDetalhes do erro:');
      console.log('  -', error.response.data.error);
    }
  }
}

// Executar teste
testWebhook();