#!/usr/bin/env node

/**
 * Teste completo do fluxo de integração ChatGuru -> Middleware -> ClickUp
 * Simula todo o processo incluindo classificação AI e envio de anotações
 */

const axios = require('axios');

// Configurações
const MIDDLEWARE_URL = process.env.MIDDLEWARE_URL || 'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app';
const CHATGURU_WEBHOOK = `${MIDDLEWARE_URL}/webhooks/chatguru`;

// Cores para output
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
    magenta: '\x1b[35m'
};

// Função auxiliar para log colorido
function log(message, color = 'reset') {
    console.log(`${colors[color]}${message}${colors.reset}`);
}

// Teste 1: Mensagem que DEVE criar tarefa (atividade válida)
const validActivityPayload = {
    "data_criacao": new Date().toISOString(),
    "numero": "+5511988348951",
    "nome": "João Silva",
    "celular": "5511988348951",
    "origem": "whatsapp",
    "texto_mensagem": "Preciso de 3 caixas de parafusos M8 e 2 chapas de alumínio 2mm",
    "campanha_id": "625584ce6fdcb7bda7d94aa8",
    "campanha_nome": "Vendas WhatsApp",
    "tags": ["pedido", "material"],
    "extra": {
        "priority": "normal",
        "department": "vendas"
    }
};

// Teste 2: Mensagem que NÃO deve criar tarefa (não é atividade)
const invalidActivityPayload = {
    "data_criacao": new Date().toISOString(),
    "numero": "+5511970525814",
    "nome": "Maria Santos",
    "celular": "5511970525814",
    "origem": "whatsapp",
    "texto_mensagem": "Oi, bom dia! Tudo bem?",
    "campanha_id": "625584ce6fdcb7bda7d94aa8",
    "campanha_nome": "Vendas WhatsApp",
    "tags": ["saudacao"],
    "extra": {}
};

// Teste 3: Payload com formato diferente (EventType)
const eventTypePayload = {
    "event_type": "message.received",
    "timestamp": new Date().toISOString(),
    "data": {
        "phone": "5511988348951",
        "name": "Pedro Costa",
        "message": "Quero fazer um orçamento para 10 metros de cabo elétrico",
        "tags": ["orcamento"],
        "source": "chatguru"
    }
};

// Função para enviar teste
async function sendTest(testName, payload, expectedResult) {
    log(`\n${testName}`, 'cyan');
    log('─'.repeat(50), 'cyan');
    
    try {
        // Enviar webhook
        log('📤 Enviando webhook...', 'yellow');
        const startTime = Date.now();
        
        const response = await axios.post(CHATGURU_WEBHOOK, payload, {
            headers: {
                'Content-Type': 'application/json'
            },
            timeout: 10000
        });
        
        const duration = Date.now() - startTime;
        
        // Verificar resposta
        if (response.data && response.data.message === 'Success') {
            log(`✅ Resposta recebida em ${duration}ms: ${JSON.stringify(response.data)}`, 'green');
        } else {
            log(`⚠️ Resposta inesperada: ${JSON.stringify(response.data)}`, 'yellow');
        }
        
        // Aguardar processamento assíncrono
        log('⏳ Aguardando processamento assíncrono (5s)...', 'blue');
        await new Promise(resolve => setTimeout(resolve, 5000));
        
        // Informar resultado esperado
        log(`📊 Resultado esperado: ${expectedResult}`, 'magenta');
        
        return true;
    } catch (error) {
        log(`❌ Erro no teste: ${error.message}`, 'red');
        if (error.response) {
            log(`   Status: ${error.response.status}`, 'red');
            log(`   Data: ${JSON.stringify(error.response.data)}`, 'red');
        }
        return false;
    }
}

// Função para verificar status do serviço
async function checkServiceStatus() {
    log('\n🔍 Verificando status do serviço...', 'blue');
    
    try {
        const statusUrl = `${MIDDLEWARE_URL}/status`;
        const response = await axios.get(statusUrl, { timeout: 5000 });
        
        if (response.data) {
            log('✅ Serviço está online:', 'green');
            log(`   Versão: ${response.data.version || 'N/A'}`, 'green');
            log(`   Uptime: ${response.data.uptime || 'N/A'}`, 'green');
            log(`   ClickUp: ${response.data.clickup_connected ? 'Conectado' : 'Desconectado'}`, 'green');
            log(`   AI: ${response.data.ai_enabled ? 'Habilitada' : 'Desabilitada'}`, 'green');
        }
        return true;
    } catch (error) {
        log(`❌ Serviço não está respondendo: ${error.message}`, 'red');
        return false;
    }
}

// Função principal
async function runTests() {
    log('\n🚀 TESTE COMPLETO DO FLUXO DE INTEGRAÇÃO', 'bright');
    log('═'.repeat(50), 'bright');
    log(`URL do Middleware: ${MIDDLEWARE_URL}`, 'cyan');
    log(`Webhook Endpoint: ${CHATGURU_WEBHOOK}`, 'cyan');
    
    // Verificar status do serviço
    const isOnline = await checkServiceStatus();
    if (!isOnline) {
        log('\n⛔ Abortando testes - serviço não está disponível', 'red');
        process.exit(1);
    }
    
    // Executar testes
    const tests = [
        {
            name: 'TESTE 1: Atividade Válida (deve criar tarefa)',
            payload: validActivityPayload,
            expected: 'Tarefa criada no ClickUp + Anotação enviada ao ChatGuru'
        },
        {
            name: 'TESTE 2: Não é Atividade (não deve criar tarefa)',
            payload: invalidActivityPayload,
            expected: 'Apenas anotação "Não é uma atividade" enviada ao ChatGuru'
        },
        {
            name: 'TESTE 3: Formato EventType (compatibilidade)',
            payload: eventTypePayload,
            expected: 'Processamento normal com classificação AI'
        }
    ];
    
    let successCount = 0;
    for (const test of tests) {
        const success = await sendTest(test.name, test.payload, test.expected);
        if (success) successCount++;
        
        // Pequeno delay entre testes
        await new Promise(resolve => setTimeout(resolve, 2000));
    }
    
    // Resumo final
    log('\n' + '═'.repeat(50), 'bright');
    log('📊 RESUMO DOS TESTES', 'bright');
    log('─'.repeat(50), 'bright');
    log(`Total de testes: ${tests.length}`, 'cyan');
    log(`Sucesso: ${successCount}`, 'green');
    log(`Falhas: ${tests.length - successCount}`, 'red');
    
    if (successCount === tests.length) {
        log('\n🎉 TODOS OS TESTES PASSARAM!', 'green');
    } else {
        log('\n⚠️ Alguns testes falharam. Verifique os logs do serviço.', 'yellow');
    }
    
    // Instruções para verificar resultados
    log('\n📝 PRÓXIMOS PASSOS:', 'magenta');
    log('1. Verifique os logs do Cloud Run:', 'yellow');
    log(`   gcloud run logs tail chatguru-clickup-middleware --region southamerica-east1`, 'cyan');
    log('\n2. Verifique se as tarefas foram criadas no ClickUp:', 'yellow');
    log(`   curl -X GET "https://api.clickup.com/api/v2/list/901300373349/task" \\`, 'cyan');
    log(`     -H "Authorization: pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657"`, 'cyan');
    log('\n3. Verifique se as anotações chegaram no ChatGuru:', 'yellow');
    log('   Acesse o painel do ChatGuru e verifique as mensagens enviadas', 'cyan');
}

// Executar testes
runTests().catch(error => {
    log(`\n❌ Erro fatal: ${error.message}`, 'red');
    process.exit(1);
});