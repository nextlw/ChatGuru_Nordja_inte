/**
 * Script de teste para validar o padrão esperado do webhook ChatGuru
 * Este arquivo simula o envio do webhook conforme documentação oficial
 * 
 * Data: 23/09/2025
 * Autor: Elai
 * 
 * Uso: node test-webhook-chatguru-padrao.js
 */

const axios = require('axios');

// Configurações
const WEBHOOK_URL = process.env.WEBHOOK_URL || 'http://localhost:8080/webhooks/chatguru';
const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET || ''; // Se configurado

// Payload padrão esperado pelo ChatGuru
const chatguruPayload = {
  campanha_id: "CAMP-2025-001",
  campanha_nome: "Campanha de Vendas Q1 2025",
  origem: "ChatGuru",
  email: "556292650123@c.us",
  nome: "João Silva - Empresa XYZ",
  tags: [
    "🤖 Zap.Guru",
    "✅ Fechado e Ganho",
    "Origem: Instagram",
    "Prioridade: Alta"
  ],
  texto_mensagem: "Olá, gostaria de saber mais sobre o produto NCM. Preciso de uma cotação urgente para 100 unidades.",
  campos_personalizados: {
    email2: "joao.silva@empresa-xyz.com",
    Site: "https://www.empresa-xyz.com",
    Valor: "15.687,50",
    CNPJ: "12.345.678/0001-90",
    Empresa: "EMPRESA XYZ LTDA",
    Segmento: "Varejo",
    Porte: "Médio",
    UF: "SP"
  },
  bot_context: {
    ChatGuru: true,
    conversation_id: "conv_123456",
    flow_id: "flow_vendas_001",
    step: "cotacao"
  },
  responsavel_nome: "Maria Santos",
  responsavel_email: "maria.santos@nordja.com",
  link_chat: "https://app2.zap.guru/chats/5e370c9334a812e7e183f760",
  celular: "5511999887766"
};

// Função para gerar assinatura HMAC (se necessário)
function generateSignature(payload, secret) {
  if (!secret) return null;
  
  const crypto = require('crypto');
  const hmac = crypto.createHmac('sha256', secret);
  hmac.update(JSON.stringify(payload));
  return 'sha256=' + hmac.digest('hex');
}

// Função para enviar o webhook
async function sendWebhook() {
  console.log('🚀 Enviando webhook ChatGuru padrão...\n');
  console.log('📦 Payload:');
  console.log(JSON.stringify(chatguruPayload, null, 2));
  console.log('\n' + '='.repeat(80) + '\n');

  try {
    const headers = {
      'Content-Type': 'application/json',
      'User-Agent': 'ChatGuru-Webhook/1.0'
    };

    // Adicionar assinatura se configurada
    if (WEBHOOK_SECRET) {
      headers['X-ChatGuru-Signature'] = generateSignature(chatguruPayload, WEBHOOK_SECRET);
    }

    const startTime = Date.now();
    const response = await axios.post(WEBHOOK_URL, chatguruPayload, { headers });
    const responseTime = Date.now() - startTime;

    console.log('✅ Resposta recebida com sucesso!\n');
    console.log(`📊 Status: ${response.status} ${response.statusText}`);
    console.log(`⏱️  Tempo de resposta: ${responseTime}ms`);
    console.log('\n📄 Headers da resposta:');
    console.log(response.headers);
    console.log('\n📝 Corpo da resposta:');
    console.log(JSON.stringify(response.data, null, 2));

    // Verificar se a resposta indica sucesso
    if (response.data.success === true) {
      console.log('\n✅ TESTE PASSOU: Webhook processado com sucesso!');
      
      if (response.data.clickup_task_id) {
        console.log(`📋 Tarefa ClickUp criada/atualizada: ${response.data.clickup_task_id}`);
      }
    } else {
      console.log('\n⚠️ AVISO: Webhook processado mas com erros parciais');
      if (response.data.message) {
        console.log(`Mensagem: ${response.data.message}`);
      }
    }

  } catch (error) {
    console.error('❌ ERRO ao enviar webhook:\n');
    
    if (error.response) {
      // Servidor respondeu com erro
      console.error(`Status: ${error.response.status}`);
      console.error('Headers:', error.response.headers);
      console.error('Dados da resposta:', error.response.data);
      
      // Análise específica do erro
      if (error.response.status === 400) {
        console.error('\n⚠️ ERRO 400: Payload inválido');
        console.error('O servidor não conseguiu processar o payload.');
        console.error('Isso confirma que a estrutura atual NÃO é compatível!');
      } else if (error.response.status === 422) {
        console.error('\n⚠️ ERRO 422: Entidade não processável');
        console.error('O payload foi recebido mas não pôde ser processado.');
        console.error('Verifique os campos obrigatórios e tipos de dados.');
      } else if (error.response.status === 500) {
        console.error('\n⚠️ ERRO 500: Erro interno do servidor');
        console.error('O servidor encontrou um erro ao processar o webhook.');
        console.error('Provavelmente devido à incompatibilidade de estrutura.');
      }
    } else if (error.request) {
      // Requisição foi feita mas não houve resposta
      console.error('Nenhuma resposta recebida do servidor');
      console.error('Verifique se o servidor está rodando em:', WEBHOOK_URL);
    } else {
      // Erro ao configurar a requisição
      console.error('Erro ao configurar a requisição:', error.message);
    }
  }

  console.log('\n' + '='.repeat(80));
}

// Função para testar variações do payload
async function testVariations() {
  console.log('\n📋 TESTANDO VARIAÇÕES DO PAYLOAD\n');
  
  // Teste 1: Payload mínimo
  const minimalPayload = {
    campanha_id: "TEST-MIN",
    campanha_nome: "Teste Mínimo",
    origem: "ChatGuru",
    email: "5511999999999@c.us",
    nome: "Teste Mínimo",
    tags: [],
    texto_mensagem: "Teste",
    link_chat: "https://teste.com",
    celular: "5511999999999"
  };
  
  console.log('1️⃣ Testando payload mínimo...');
  try {
    await axios.post(WEBHOOK_URL, minimalPayload);
    console.log('   ✅ Payload mínimo aceito');
  } catch (error) {
    console.log('   ❌ Payload mínimo rejeitado:', error.response?.status || error.message);
  }
  
  // Teste 2: Payload com campos extras
  const extraFieldsPayload = {
    ...chatguruPayload,
    campo_extra_1: "valor1",
    campo_extra_2: "valor2",
    timestamp_custom: new Date().toISOString()
  };
  
  console.log('2️⃣ Testando payload com campos extras...');
  try {
    await axios.post(WEBHOOK_URL, extraFieldsPayload);
    console.log('   ✅ Payload com campos extras aceito');
  } catch (error) {
    console.log('   ❌ Payload com campos extras rejeitado:', error.response?.status || error.message);
  }
  
  // Teste 3: Payload com campos personalizados complexos
  const complexPayload = {
    ...chatguruPayload,
    campos_personalizados: {
      ...chatguruPayload.campos_personalizados,
      dados_adicionais: {
        pedido_numero: "PED-2025-001",
        produtos: ["NCM-001", "NCM-002"],
        quantidade: 100
      }
    }
  };
  
  console.log('3️⃣ Testando payload com estrutura complexa...');
  try {
    await axios.post(WEBHOOK_URL, complexPayload);
    console.log('   ✅ Payload complexo aceito');
  } catch (error) {
    console.log('   ❌ Payload complexo rejeitado:', error.response?.status || error.message);
  }
}

// Função principal
async function main() {
  console.log('=' + '='.repeat(80));
  console.log('🧪 TESTE DE WEBHOOK CHATGURU - PADRÃO OFICIAL');
  console.log('=' + '='.repeat(80));
  console.log(`\n📍 URL do Webhook: ${WEBHOOK_URL}`);
  console.log(`🔐 Secret configurado: ${WEBHOOK_SECRET ? 'Sim' : 'Não'}`);
  console.log(`🕐 Timestamp: ${new Date().toISOString()}\n`);

  // Teste principal
  await sendWebhook();
  
  // Testes adicionais (descomente se quiser testar variações)
  // await testVariations();
  
  console.log('\n✅ Teste concluído!');
  console.log('=' + '='.repeat(80));
}

// Executar
main().catch(console.error);