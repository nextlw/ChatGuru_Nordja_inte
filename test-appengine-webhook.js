#!/usr/bin/env node

const axios = require('axios');

// URL do webhook no App Engine (sistema antigo em produção)
const APPENGINE_URL = 'https://buzzlightear.rj.r.appspot.com/webhook';

// Payload similar ao que usamos para o middleware
const testPayload = {
  "campanha_id": "123456",
  "campanha_nome": "Teste App Engine",
  "origem": "ChatGuru",
  "email": "5562999887766@c.us",
  "nome": "Teste Sistema Antigo",
  "tags": [
    "🤖 Zap.Guru",
    "📞 Teste",
    "Origem: WhatsApp"
  ],
  "texto_mensagem": "Teste de webhook no sistema antigo App Engine",
  "campos_personalizados": {
    "email2": "teste@example.com",
    "Site": "https://www.teste.com",
    "Valor": "1000.00",
    "CNPJ": "00.000.000/0001-00",
    "Empresa": "TESTE LTDA"
  },
  "bot_context": {
    "ChatGuru": true
  },
  "responsavel_nome": "Teste Responsavel",
  "responsavel_email": "responsavel@teste.com",
  "link_chat": "https://app2.zap.guru/chats/teste123",
  "celular": "5562999887766"
};

// Também testar com estrutura alternativa (event_type)
const alternativePayload = {
  "id": "evt_test_123",
  "event_type": "new_lead",
  "timestamp": new Date().toISOString(),
  "data": {
    "lead_name": "Teste Sistema Antigo",
    "phone": "+5562999887766",
    "email": "teste@example.com",
    "project_name": "Teste App Engine",
    "task_title": "Teste de webhook - App Engine",
    "annotation": "Testando sistema antigo",
    "amount": 1000.00,
    "status": "new",
    "custom_data": {
      "source": "WhatsApp",
      "campaign": "TesteAppEngine"
    }
  }
};

async function testWebhook(payload, description) {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`🧪 ${description}`);
  console.log('='.repeat(60));
  console.log('🌐 URL:', APPENGINE_URL);
  console.log('📦 Payload:', JSON.stringify(payload, null, 2));
  console.log('-'.repeat(60));

  try {
    const startTime = Date.now();
    
    const response = await axios.post(APPENGINE_URL, payload, {
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'ChatGuru-Test/1.0'
      },
      timeout: 30000,
      validateStatus: function (status) {
        return true; // Aceita qualquer status para análise
      }
    });

    const responseTime = Date.now() - startTime;

    console.log('\n📊 Resposta:');
    console.log(`   Status HTTP: ${response.status} (${response.statusText})`);
    console.log(`   Tempo de resposta: ${responseTime}ms`);
    console.log(`   Headers:`, response.headers);
    
    if (response.data) {
      console.log('\n📨 Dados da resposta:');
      if (typeof response.data === 'object') {
        console.log(JSON.stringify(response.data, null, 2));
      } else {
        console.log(response.data);
      }
    }

    // Análise do resultado
    if (response.status === 200 || response.status === 201) {
      console.log('\n✅ Webhook processado com sucesso!');
    } else if (response.status === 400) {
      console.log('\n⚠️  Bad Request - Payload inválido');
    } else if (response.status === 401 || response.status === 403) {
      console.log('\n🔒 Erro de autenticação/autorização');
    } else if (response.status === 404) {
      console.log('\n❌ Endpoint não encontrado');
    } else if (response.status === 500) {
      console.log('\n💥 Erro interno do servidor');
    }

  } catch (error) {
    console.error('\n❌ ERRO na requisição:');
    
    if (error.code === 'ECONNABORTED') {
      console.log('⏱️  Timeout - requisição demorou mais de 30 segundos');
    } else if (error.code === 'ENOTFOUND') {
      console.log('🌐 Host não encontrado - verificar URL');
    } else if (error.code === 'ECONNREFUSED') {
      console.log('🚫 Conexão recusada - servidor pode estar offline');
    } else if (error.response) {
      console.log('📊 Status:', error.response.status);
      console.log('📨 Resposta:', error.response.data);
    } else if (error.request) {
      console.log('🌐 Erro de rede:', error.message);
    } else {
      console.log('❓ Erro:', error.message);
    }
  }
}

async function runAllTests() {
  console.log('====================================');
  console.log('   TESTE WEBHOOK APP ENGINE        ');
  console.log('   URL: ' + APPENGINE_URL);
  console.log('====================================');

  // Teste 1: Estrutura ChatGuru padrão
  await testWebhook(testPayload, 'Teste 1: Estrutura ChatGuru (campanha_id, nome, etc)');
  
  // Aguardar um pouco entre os testes
  await new Promise(resolve => setTimeout(resolve, 2000));
  
  // Teste 2: Estrutura com event_type
  await testWebhook(alternativePayload, 'Teste 2: Estrutura com event_type');
  
  // Teste 3: Payload mínimo
  const minimalPayload = {
    "nome": "Teste Mínimo",
    "celular": "5562999887766"
  };
  
  await new Promise(resolve => setTimeout(resolve, 2000));
  await testWebhook(minimalPayload, 'Teste 3: Payload Mínimo');
  
  console.log('\n' + '='.repeat(60));
  console.log('📌 Resumo dos Testes');
  console.log('='.repeat(60));
  console.log('\nSistema testado: App Engine (buzzlightear.rj.r.appspot.com)');
  console.log('Endpoint: /webhook');
  console.log('\n💡 Compare essas respostas com o novo middleware para garantir compatibilidade!');
}

// Executar testes
runAllTests().then(() => {
  console.log('\n✅ Todos os testes finalizados!');
}).catch((error) => {
  console.error('\n❌ Erro crítico:', error);
  process.exit(1);
});