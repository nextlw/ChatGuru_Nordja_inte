#!/usr/bin/env node

// ================================================
// TESTE RÁPIDO - ChatGuru ClickUp Middleware
// ================================================
// Script simplificado para testes rápidos
// ================================================

const https = require('https');
const crypto = require('crypto');

// Configuração
const PROD_URL = 'chatguru-clickup-middleware-707444002434.southamerica-east1.run.app';
const LOCAL_URL = 'localhost';
const USE_PROD = false; // Mudando para testar localmente

const HOST = USE_PROD ? PROD_URL : LOCAL_URL;
const PROTOCOL = USE_PROD ? 'https' : 'http';

// Cores para output
const colors = {
    reset: '\x1b[0m',
    green: '\x1b[32m',
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
    bold: '\x1b[1m'
};

const log = {
    success: (msg) => console.log(`${colors.green}✓${colors.reset} ${msg}`),
    error: (msg) => console.log(`${colors.red}✗${colors.reset} ${msg}`),
    info: (msg) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
    header: (msg) => console.log(`\n${colors.cyan}${colors.bold}${'='.repeat(50)}${colors.reset}\n${colors.cyan}${colors.bold}${msg}${colors.reset}\n${colors.cyan}${colors.bold}${'='.repeat(50)}${colors.reset}`)
};

// Função para fazer requisição HTTP/HTTPS
function makeRequest(options, data = null) {
    return new Promise((resolve, reject) => {
        const client = USE_PROD ? https : require('http');
        
        const req = client.request(options, (res) => {
            let body = '';
            
            res.on('data', (chunk) => {
                body += chunk;
            });
            
            res.on('end', () => {
                try {
                    const response = {
                        status: res.statusCode,
                        headers: res.headers,
                        body: body ? JSON.parse(body) : null
                    };
                    resolve(response);
                } catch (e) {
                    resolve({
                        status: res.statusCode,
                        headers: res.headers,
                        body: body
                    });
                }
            });
        });
        
        req.on('error', reject);
        
        if (data) {
            req.write(JSON.stringify(data));
        }
        
        req.end();
    });
}

// Criar evento de teste
function createTestEvent() {
    const timestamp = new Date().toISOString();
    
    return {
        event_id: `test_${Date.now()}`,
        event_type: 'payment_created',
        timestamp: timestamp,
        data: {
            amount: 100.00,
            customer_name: 'Teste Rápido',
            payment_method: 'pix',
            transaction_id: `txn_${Date.now()}`
        },
        source: 'quick_test',
        metadata: {
            test_time: timestamp
        }
    };
}

// Criar assinatura HMAC
function createSignature(payload, secret = 'test-secret') {
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(JSON.stringify(payload));
    return 'sha256=' + hmac.digest('hex');
}

// Teste 1: Health Check
async function testHealth() {
    log.header('TESTE 1: Health Check');
    
    try {
        const response = await makeRequest({
            hostname: HOST,
            port: USE_PROD ? 443 : 8080,
            path: '/health',
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        if (response.status === 200) {
            log.success(`Health Check OK - Status: ${response.status}`);
            log.info(`Resposta: ${JSON.stringify(response.body)}`);
            return true;
        } else {
            log.error(`Health Check falhou - Status: ${response.status}`);
            return false;
        }
    } catch (error) {
        log.error(`Erro no Health Check: ${error.message}`);
        return false;
    }
}

// Teste 2: Webhook Válido
async function testValidWebhook() {
    log.header('TESTE 2: Webhook Válido');
    
    const event = createTestEvent();
    const signature = createSignature(event);
    
    try {
        const response = await makeRequest({
            hostname: HOST,
            port: USE_PROD ? 443 : 8080,
            path: '/webhooks/chatguru',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-ChatGuru-Signature': signature,
                'Content-Length': Buffer.byteLength(JSON.stringify(event))
            }
        }, event);
        
        if (response.status === 200) {
            log.success(`Webhook processado com sucesso - Status: ${response.status}`);
            if (response.body && response.body.clickup_task_id) {
                log.info(`Task criada no ClickUp: ${response.body.clickup_task_id}`);
            }
            return true;
        } else if (response.status === 500) {
            log.error(`Webhook processado mas ClickUp falhou - Status: ${response.status}`);
            if (response.body && response.body.clickup_error) {
                log.error(`Erro do ClickUp: ${response.body.clickup_error}`);
            }
            return false;
        } else {
            log.error(`Webhook falhou - Status: ${response.status}`);
            return false;
        }
    } catch (error) {
        log.error(`Erro no Webhook: ${error.message}`);
        return false;
    }
}

// Teste 3: Webhook Inválido
async function testInvalidWebhook() {
    log.header('TESTE 3: Webhook Inválido (sem event_type)');
    
    const event = createTestEvent();
    delete event.event_type; // Remover campo obrigatório
    
    const signature = createSignature(event);
    
    try {
        const response = await makeRequest({
            hostname: HOST,
            port: USE_PROD ? 443 : 8080,
            path: '/webhooks/chatguru',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-ChatGuru-Signature': signature,
                'Content-Length': Buffer.byteLength(JSON.stringify(event))
            }
        }, event);
        
        if (response.status === 400) {
            log.success(`Validação funcionou corretamente - Status: ${response.status}`);
            log.info(`Erro esperado: ${JSON.stringify(response.body)}`);
            return true;
        } else {
            log.error(`Validação falhou - Status inesperado: ${response.status}`);
            return false;
        }
    } catch (error) {
        log.error(`Erro no teste: ${error.message}`);
        return false;
    }
}

// Teste 4: Status da Aplicação
async function testStatus() {
    log.header('TESTE 4: Status da Aplicação');
    
    try {
        const response = await makeRequest({
            hostname: HOST,
            port: USE_PROD ? 443 : 8080,
            path: '/status',
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        if (response.status === 200) {
            log.success(`Status OK - Status: ${response.status}`);
            const body = response.body;
            if (body) {
                log.info(`Versão: ${body.version || 'N/A'}`);
                log.info(`Uptime: ${body.uptime || 'N/A'}`);
                log.info(`Environment: ${body.environment || 'N/A'}`);
            }
            return true;
        } else {
            log.error(`Status falhou - Status: ${response.status}`);
            return false;
        }
    } catch (error) {
        log.error(`Erro no Status: ${error.message}`);
        return false;
    }
}

// Teste 5: Conexão com ClickUp
async function testClickUpConnection() {
    log.header('TESTE 5: Conexão com ClickUp');
    
    try {
        const response = await makeRequest({
            hostname: HOST,
            port: USE_PROD ? 443 : 8080,
            path: '/clickup/test',
            method: 'GET',
            headers: {
                'Content-Type': 'application/json'
            }
        });
        
        if (response.status === 200) {
            log.success(`Conexão com ClickUp OK - Status: ${response.status}`);
            if (response.body && response.body.user) {
                log.info(`Usuário ClickUp: ${response.body.user.username || response.body.user.email}`);
            }
            return true;
        } else {
            log.error(`Conexão com ClickUp falhou - Status: ${response.status}`);
            return false;
        }
    } catch (error) {
        log.error(`Erro na conexão: ${error.message}`);
        return false;
    }
}

// Executar todos os testes
async function runTests() {
    console.log(`${colors.cyan}${colors.bold}\n${'='.repeat(60)}${colors.reset}`);
    console.log(`${colors.cyan}${colors.bold}TESTE RÁPIDO - ChatGuru ClickUp Middleware${colors.reset}`);
    console.log(`${colors.cyan}${colors.bold}${'='.repeat(60)}${colors.reset}`);
    console.log(`\nAmbiente: ${USE_PROD ? 'PRODUÇÃO (GCP)' : 'LOCAL'}`);
    console.log(`URL: ${PROTOCOL}://${HOST}`);
    console.log(`Horário: ${new Date().toISOString()}\n`);
    
    const results = [];
    
    // Executar testes
    results.push(await testHealth());
    results.push(await testStatus());
    results.push(await testValidWebhook());
    results.push(await testInvalidWebhook());
    results.push(await testClickUpConnection());
    
    // Relatório final
    console.log(`\n${colors.cyan}${colors.bold}${'='.repeat(60)}${colors.reset}`);
    console.log(`${colors.cyan}${colors.bold}RESULTADO FINAL${colors.reset}`);
    console.log(`${colors.cyan}${colors.bold}${'='.repeat(60)}${colors.reset}`);
    
    const passed = results.filter(r => r === true).length;
    const total = results.length;
    const percentage = ((passed / total) * 100).toFixed(0);
    
    console.log(`\nTestes aprovados: ${passed}/${total} (${percentage}%)`);
    
    if (passed === total) {
        console.log(`\n${colors.green}${colors.bold}✨ TODOS OS TESTES PASSARAM! ✨${colors.reset}`);
        process.exit(0);
    } else if (passed >= total * 0.6) {
        console.log(`\n${colors.yellow}${colors.bold}⚠️ ALGUNS TESTES FALHARAM ⚠️${colors.reset}`);
        process.exit(1);
    } else {
        console.log(`\n${colors.red}${colors.bold}❌ MAIORIA DOS TESTES FALHOU ❌${colors.reset}`);
        process.exit(1);
    }
}

// Executar
runTests();