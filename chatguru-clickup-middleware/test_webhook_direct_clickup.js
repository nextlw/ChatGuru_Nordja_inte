const axios = require('axios');

// Configuração
const WORKER_URL = 'http://localhost:8081/worker/process';

// Dados de teste com classificação manual (sem OpenAI)
const testPayload = {
  campanha_id: "12345",
  campanha_nome: "WhatsApp Bot Test",
  origem: "whatsapp",
  email: "joao@teste.com",
  nome: "João Silva - TESTE DIRETO CLICKUP",
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

async function testDirectClickUp() {
  console.log('🧪 TESTE DIRETO: Worker → ClickUp (SEM OpenAI)');
  console.log('=====================================================\n');

  try {
    // Criar envelope Pub/Sub simulado com classificação forçada
    const pubsubEnvelope = {
      raw_payload: JSON.stringify(testPayload),
      timestamp: new Date().toISOString(),
      source: "test",
      // Adicionar classificação manual para bypassar OpenAI
      force_classification: {
        campanha: "Atendimento",
        description: "Agendamento de consulta médica",
        info_1: "Conta Premium", 
        info_2: "João Silva",
        is_task_worthy: true,
        priority: 2
      }
    };

    console.log('🔄 1. Enviando payload direto para Worker...');
    console.log(`   Payload: ${JSON.stringify(pubsubEnvelope, null, 2)}`);

    const workerResponse = await axios.post(WORKER_URL, pubsubEnvelope, {
      headers: { 
        'Content-Type': 'application/json',
        'X-CloudTasks-TaskName': 'test-direct-clickup-123',
        'X-CloudTasks-QueueName': 'chatguru-webhook-queue'
      },
      timeout: 60000
    });

    console.log('\n✅ Worker processou com sucesso!');
    console.log(`   Status: ${workerResponse.status}`);
    console.log(`   Response: ${JSON.stringify(workerResponse.data, null, 2)}`);

    // Verificar se tarefa foi criada no ClickUp
    if (workerResponse.data.task_id) {
      console.log(`\n🎯 2. Tarefa criada no ClickUp!`);
      console.log(`   Task ID: ${workerResponse.data.task_id}`);
      console.log(`   URL: https://app.clickup.com/t/${workerResponse.data.task_id}`);
      console.log(`   Anotação: ${workerResponse.data.annotation}`);
    } else {
      console.log('\n⚠️  2. Nenhuma tarefa foi criada');
    }

    console.log('\n🎉 TESTE DIRETO EXECUTADO COM SUCESSO!');
    console.log('=====================================================');

    return true;

  } catch (error) {
    console.error('\n❌ ERRO NO TESTE DIRETO:');
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
testDirectClickUp().then(success => {
  process.exit(success ? 0 : 1);
});