const axios = require('axios');

// Configuração
const WEBHOOK_URL = 'http://localhost:8081/webhooks/chatguru';
const WORKER_URL = 'http://localhost:8081/worker/process';

// Dados de teste do ChatGuru
const testPayload = {
  campanha_id: "12345",
  campanha_nome: "WhatsApp Bot Test",
  origem: "whatsapp",
  email: "joao@teste.com",
  nome: "João Silva - TESTE FLUXO COMPLETO",
  tags: ["teste", "integracao"],
  texto_mensagem: "Preciso agendar uma consulta médica para amanhã às 14h. É urgente, por favor me ajudem a encontrar um clínico geral disponível.",
  media_url: null,
  media_type: null,
  campos_personalizados: {
    "Info_1": "Conta Premium",
    "Info_2": "João Silva"
  },
  bot_context: {
    "ChatGuru": true
  },
  responsavel_nome: "Atendente Bot",
  responsavel_email: "bot@chatguru.app",
  link_chat: "https://s15.chatguru.app/chat/12345",
  celular: "+5511999887766",
  phone_id: "phone_12345",
  chat_id: "chat_67890",
  chat_created: "2025-10-07T12:00:00Z"
};

async function testWebhookFlow() {
  console.log('🧪 TESTE COMPLETO: Webhook → Pub/Sub → Worker → ClickUp');
  console.log('================================================================\n');

  try {
    // 1. Testar webhook ChatGuru
    console.log('📥 1. Enviando webhook ChatGuru...');
    const webhookResponse = await axios.post(WEBHOOK_URL, testPayload, {
      headers: { 'Content-Type': 'application/json' },
      timeout: 30000
    });

    console.log('✅ Webhook recebido com sucesso!');
    console.log(`   Status: ${webhookResponse.status}`);
    console.log(`   Response: ${JSON.stringify(webhookResponse.data, null, 2)}`);

    // 2. Simular processamento do worker
    console.log('\n🔄 2. Simulando processamento do Worker...');
    
    // Criar envelope Pub/Sub simulado
    const pubsubEnvelope = {
      raw_payload: JSON.stringify(testPayload),
      timestamp: new Date().toISOString(),
      source: "test"
    };

    const workerResponse = await axios.post(WORKER_URL, pubsubEnvelope, {
      headers: { 
        'Content-Type': 'application/json',
        'X-CloudTasks-TaskName': 'test-task-123',
        'X-CloudTasks-QueueName': 'chatguru-webhook-queue'
      },
      timeout: 60000
    });

    console.log('✅ Worker processou com sucesso!');
    console.log(`   Status: ${workerResponse.status}`);
    console.log(`   Response: ${JSON.stringify(workerResponse.data, null, 2)}`);

    // 3. Verificar se tarefa foi criada no ClickUp
    if (workerResponse.data.task_id) {
      console.log(`\n🎯 3. Tarefa criada no ClickUp!`);
      console.log(`   Task ID: ${workerResponse.data.task_id}`);
      console.log(`   Anotação: ${workerResponse.data.annotation}`);
    } else {
      console.log('\n⚠️  3. Nenhuma tarefa foi criada (pode ser que não foi classificada como atividade)');
    }

    console.log('\n🎉 TESTE COMPLETO EXECUTADO COM SUCESSO!');
    console.log('================================================================');

    return true;

  } catch (error) {
    console.error('\n❌ ERRO NO TESTE:');
    console.error(`   Status: ${error.response?.status}`);
    console.error(`   Mensagem: ${error.message}`);
    
    if (error.response?.data) {
      console.error(`   Detalhes: ${JSON.stringify(error.response.data, null, 2)}`);
    }
    
    console.error(`   URL: ${error.config?.url}`);
    return false;
  }
}

// Executar teste
testWebhookFlow().then(success => {
  process.exit(success ? 0 : 1);
});
