const axios = require('axios');

// Testes de categorização automática
const TEST_CASES = [
  {
    nome: "João Silva",
    mensagem: "Preciso enviar um reembolso via motoboy",
    expected: {
      categoria: "Logística",
      subcategoria: "Corrida de motoboy",
      estrelas: 1
    }
  },
  {
    nome: "Maria Santos",
    mensagem: "Envio de reembolso bradesco saúde via sedex",
    expected: {
      categoria: "Plano de Saúde",
      subcategoria: "Reembolso Médico",
      estrelas: 2
    }
  },
  {
    nome: "Ana Costa",
    mensagem: "Planejamento de festa de aniversário",
    expected: {
      categoria: "Lazer",
      subcategoria: "Planejamento de festas",
      estrelas: 4
    }
  },
  {
    nome: "Pedro Oliveira",
    mensagem: "Consulta médica na próxima semana",
    expected: {
      categoria: "Agendamentos",
      subcategoria: "Consultas",
      estrelas: 1
    }
  }
];

async function testAutoCategorization() {
  console.log('🧪 Testando categorização automática no middleware Rust...\n');

  const WEBHOOK_URL = 'http://localhost:8080/webhooks/chatguru';

  for (const [index, testCase] of TEST_CASES.entries()) {
    console.log(`\nTeste ${index + 1}/${TEST_CASES.length}: ${testCase.mensagem}`);
    console.log(`Esperado: ${testCase.expected.categoria} > ${testCase.expected.subcategoria} (${testCase.expected.estrelas}⭐)`);

    const payload = {
      campanha_id: "test",
      campanha_nome: "Teste Categorização",
      origem: "test",
      email: "test@example.com",
      nome: testCase.nome,
      tags: [],
      texto_mensagem: testCase.mensagem,
      celular: "+5511999999999",
      phone_id: "test123",
      chat_id: "test_chat",
      campos_personalizados: {}
    };

    try {
      const response = await axios.post(WEBHOOK_URL, payload, {
        headers: { 'Content-Type': 'application/json' },
        timeout: 30000
      });

      console.log(`✅ Status: ${response.status}`);

      if (response.data.task_id) {
        console.log(`   Task ID: ${response.data.task_id}`);

        // Verificar campos customizados se retornados
        if (response.data.custom_fields) {
          console.log(`   Categoria: ${response.data.custom_fields.categoria || 'N/A'}`);
          console.log(`   SubCategoria: ${response.data.custom_fields.subcategoria || 'N/A'}`);
          console.log(`   Estrelas: ${response.data.custom_fields.estrelas || 'N/A'}⭐`);
        }
      }

    } catch (error) {
      if (error.code === 'ECONNREFUSED') {
        console.log('❌ Middleware não está rodando em http://localhost:8080');
        console.log('   Execute: cd chatguru-clickup-middleware && cargo run\n');
        break;
      } else if (error.response) {
        console.log(`❌ Erro ${error.response.status}: ${error.response.data?.error || error.message}`);
      } else {
        console.log(`❌ Erro: ${error.message}`);
      }
    }

    // Delay entre testes
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  console.log('\n✅ Testes concluídos!\n');
  console.log('📋 Para verificar as tarefas criadas:');
  console.log('   https://app.clickup.com/9013037641/v/l/8ckg2j9-61473\n');
}

// Executar testes
testAutoCategorization();
