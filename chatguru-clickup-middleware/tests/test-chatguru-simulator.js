#!/usr/bin/env node

/**
 * ChatGuru Event Simulator
 * Simula eventos do ChatGuru para testar o middleware Rust
 */

const http = require('http');
const crypto = require('crypto');

// Configurações
const MIDDLEWARE_URL = 'http://localhost:8080';  // URL do middleware Rust
const WEBHOOK_SECRET = 'test-secret-key';

// Eventos ChatGuru simulados
const chatguruEvents = {
  new_lead: {
    event_type: "new_lead",
    event_id: "lead_12345",
    timestamp: new Date().toISOString(),
    lead_id: "12345",
    lead_name: "João Silva",
    lead_email: "joao@email.com",
    lead_phone: "+55 85 99999-9999",
    lead_source: "Website",
    lead_status: "new",
    project_name: "Projeto Residencial XYZ",
    project_value: "350000.00",
    lead_notes: "Cliente interessado em apartamento de 2 quartos",
    assigned_consultant: "Maria Santos"
  },

  status_change: {
    event_type: "status_change",
    event_id: "status_67890",
    timestamp: new Date().toISOString(),
    lead_id: "67890",
    lead_name: "Ana Costa",
    previous_status: "contacted",
    new_status: "qualified",
    changed_by: "Carlos Oliveira",
    change_reason: "Cliente confirmou interesse e orçamento aprovado",
    next_action: "Agendar visita técnica"
  },

  appointment_scheduled: {
    event_type: "appointment_scheduled",
    event_id: "appt_111213",
    timestamp: new Date().toISOString(),
    lead_id: "111213",
    lead_name: "Pedro Lima",
    appointment_date: "2024-01-20T10:00:00Z",
    appointment_type: "site_visit",
    consultant: "Lucas Fernandes",
    location: "Rua das Flores, 123 - Fortaleza/CE",
    notes: "Cliente quer ver apartamento modelo"
  }
};

/**
 * Gera assinatura HMAC para o webhook
 */
function generateSignature(payload, secret) {
  return crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
}

/**
 * Envia evento para o middleware
 */
async function sendEvent(eventType, eventData) {
  return new Promise((resolve, reject) => {
    const payload = JSON.stringify(eventData);
    const signature = generateSignature(payload, WEBHOOK_SECRET);
    
    const options = {
      hostname: 'localhost',
      port: 8080,
      path: '/webhook/chatguru',
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(payload),
        'X-ChatGuru-Signature': signature,
        'User-Agent': 'ChatGuru-Webhook/1.0'
      }
    };

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        console.log(`✅ ${eventType}: Status ${res.statusCode}`);
        if (data) {
          console.log(`   Response: ${data}`);
        }
        resolve({
          statusCode: res.statusCode,
          data: data
        });
      });
    });

    req.on('error', (error) => {
      console.error(`❌ ${eventType}: ${error.message}`);
      reject(error);
    });

    req.write(payload);
    req.end();
  });
}

/**
 * Testa saúde do middleware
 */
async function testHealth() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 8080,
      path: '/health',
      method: 'GET'
    };

    const req = http.request(options, (res) => {
      let data = '';
      
      res.on('data', (chunk) => {
        data += chunk;
      });
      
      res.on('end', () => {
        console.log(`🏥 Health Check: Status ${res.statusCode}`);
        if (data) {
          console.log(`   Response: ${data}`);
        }
        resolve({
          statusCode: res.statusCode,
          data: data
        });
      });
    });

    req.on('error', (error) => {
      console.error(`❌ Health Check: ${error.message}`);
      reject(error);
    });

    req.end();
  });
}

/**
 * Executa todos os testes
 */
async function runTests() {
  console.log('🚀 Iniciando testes do Middleware Rust ChatGuru-ClickUp\n');
  
  try {
    // Teste de saúde
    console.log('1. Testando Health Check...');
    await testHealth();
    console.log('');

    // Teste de eventos
    console.log('2. Testando eventos ChatGuru...');
    
    for (const [eventType, eventData] of Object.entries(chatguruEvents)) {
      console.log(`   Enviando: ${eventType}`);
      await sendEvent(eventType, eventData);
      await new Promise(resolve => setTimeout(resolve, 1000)); // Pausa entre eventos
    }

    console.log('\n✅ Todos os testes foram executados!');
    
  } catch (error) {
    console.error('\n❌ Erro durante os testes:', error.message);
    process.exit(1);
  }
}

/**
 * Testa evento específico
 */
async function testSpecificEvent(eventType) {
  if (!chatguruEvents[eventType]) {
    console.error(`❌ Tipo de evento '${eventType}' não encontrado.`);
    console.log(`📋 Eventos disponíveis: ${Object.keys(chatguruEvents).join(', ')}`);
    process.exit(1);
  }

  console.log(`🎯 Testando evento específico: ${eventType}\n`);
  
  try {
    await sendEvent(eventType, chatguruEvents[eventType]);
    console.log('\n✅ Teste do evento concluído!');
  } catch (error) {
    console.error('\n❌ Erro durante o teste:', error.message);
    process.exit(1);
  }
}

/**
 * Mostra ajuda
 */
function showHelp() {
  console.log(`
🔧 ChatGuru Event Simulator - Teste do Middleware Rust

Uso:
  node test-chatguru-simulator.js [comando] [opções]

Comandos:
  test                    Executa todos os testes
  health                 Testa apenas o health check
  event <tipo>           Testa um evento específico
  list                   Lista todos os eventos disponíveis
  help                   Mostra esta ajuda

Eventos disponíveis:
  - new_lead             Novo lead no ChatGuru
  - status_change        Mudança de status do lead
  - appointment_scheduled Agendamento marcado

Exemplos:
  node test-chatguru-simulator.js test
  node test-chatguru-simulator.js event new_lead
  node test-chatguru-simulator.js health

Configurações:
  Middleware URL: ${MIDDLEWARE_URL}
  Webhook Secret: ${WEBHOOK_SECRET}
  
🔗 O middleware deve estar rodando em ${MIDDLEWARE_URL}
`);
}

// CLI
const args = process.argv.slice(2);
const command = args[0] || 'help';

switch (command) {
  case 'test':
    runTests();
    break;
    
  case 'health':
    testHealth().then(() => {
      console.log('✅ Health check concluído!');
    }).catch(console.error);
    break;
    
  case 'event':
    const eventType = args[1];
    if (!eventType) {
      console.error('❌ Especifique o tipo de evento.');
      console.log(`📋 Eventos disponíveis: ${Object.keys(chatguruEvents).join(', ')}`);
      process.exit(1);
    }
    testSpecificEvent(eventType);
    break;
    
  case 'list':
    console.log('📋 Eventos disponíveis para teste:');
    Object.keys(chatguruEvents).forEach(event => {
      console.log(`  - ${event}`);
    });
    break;
    
  case 'help':
  default:
    showHelp();
    break;
}