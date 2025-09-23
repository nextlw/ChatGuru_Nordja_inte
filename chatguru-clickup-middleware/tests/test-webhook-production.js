#!/usr/bin/env node

const axios = require('axios');

// URL do middleware em PRODUÇÃO (Google Cloud Run)
const PRODUCTION_URL = 'https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/webhooks/chatguru';

// Exemplo de payload do ChatGuru com dados DIFERENTES para teste em produção
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

async function testWebhookProduction() {
  console.log('🚀 Testando webhook do ChatGuru em PRODUÇÃO...\n');
  console.log('🌐 URL: ' + PRODUCTION_URL);
  console.log('📦 Payload:', JSON.stringify(chatguruPayload, null, 2));
  console.log('\n' + '='.repeat(60) + '\n');

  try {
    const startTime = Date.now();
    
    const response = await axios.post(PRODUCTION_URL, chatguruPayload, {
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000 // 30 segundos de timeout
    });

    const responseTime = Date.now() - startTime;

    console.log('✅ Webhook processado com sucesso em produção!\n');
    console.log('⏱️  Tempo de resposta: ' + responseTime + 'ms');
    console.log('📊 Status HTTP: ' + response.status);
    console.log('\n📨 Resposta do servidor:');
    console.log(JSON.stringify(response.data, null, 2));

    // Verifica se foi criado ou atualizado
    if (response.data.clickup_task_id) {
      console.log('\n✅ Tarefa no ClickUp:');
      console.log(`   📌 ID: ${response.data.clickup_task_id}`);
      console.log(`   🔄 Ação: ${response.data.clickup_task_action || 'processada'}`);
      
      if (response.data.clickup_task_action === 'created') {
        console.log('   ✨ Nova tarefa criada com sucesso!');
      } else if (response.data.clickup_task_action === 'updated') {
        console.log('   ♻️  Tarefa existente atualizada!');
      }
    }

    // Verificar outros endpoints
    console.log('\n🔍 Verificando outros endpoints de produção...\n');
    
    // Health check
    try {
      const healthResponse = await axios.get('https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/health');
      console.log('✅ Health Check:', healthResponse.data);
    } catch (error) {
      console.log('❌ Health Check falhou');
    }

    // Ready check
    try {
      const readyResponse = await axios.get('https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/ready');
      console.log('✅ Ready Check:', readyResponse.data);
    } catch (error) {
      console.log('❌ Ready Check falhou');
    }

  } catch (error) {
    console.error('❌ ERRO ao enviar webhook para produção:');
    
    if (error.code === 'ECONNABORTED') {
      console.log('⏱️  Timeout - a requisição demorou mais de 30 segundos');
    } else if (error.response) {
      console.log('📊 Status HTTP:', error.response.status);
      console.log('📨 Resposta de erro:', error.response.data);
      
      if (error.response.status === 500) {
        console.log('\n⚠️  Erro interno do servidor - verificar logs no GCP:');
        console.log('   gcloud run logs read chatguru-clickup-middleware --region southamerica-east1');
      } else if (error.response.status === 400) {
        console.log('\n⚠️  Erro de validação - verificar payload');
      }
    } else if (error.request) {
      console.log('🌐 Erro de rede - não foi possível conectar ao servidor');
      console.log('   Verifique sua conexão com a internet');
    } else {
      console.log('❓ Erro desconhecido:', error.message);
    }
    
    if (error.response?.data?.error) {
      console.log('\n📝 Detalhes do erro:');
      console.log('  -', error.response.data.error);
      if (error.response.data.message) {
        console.log('  - Mensagem:', error.response.data.message);
      }
    }
  }

  console.log('\n' + '='.repeat(60));
  console.log('📌 URL do serviço em produção:');
  console.log('   https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app');
  console.log('\n💡 Para configurar no ChatGuru:');
  console.log('   URL: https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/webhooks/chatguru');
  console.log('   Método: POST');
  console.log('   Content-Type: application/json');
}

// Executar teste
console.log('====================================');
console.log('   TESTE DE PRODUÇÃO - ChatGuru    ');
console.log('====================================\n');

testWebhookProduction().then(() => {
  console.log('\n✅ Teste finalizado!');
}).catch((error) => {
  console.error('\n❌ Erro crítico no teste:', error);
  process.exit(1);
});