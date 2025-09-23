#!/usr/bin/env node

/**
 * Script de Diagnóstico da API do ChatGuru
 * Verifica endpoints e credenciais
 */

const https = require('https');

// Configurações
const config = {
    apiKey: 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
    accountId: '625584ce6fdcb7bda7d94aa8',
    phoneId: '6537de23b6d5b0bb0b80421a',
    chatNumber: '5585989530473'
};

// Lista de possíveis endpoints do ChatGuru
const endpoints = [
    { url: 'https://s15.chatguru.app/api/v1', name: 'API v1' },
    { url: 'https://api.chatguru.app/v1', name: 'API Principal' },
    { url: 'https://app.chatguru.com/api/v1', name: 'App API' },
    { url: 'https://s15.chatguru.app/webhook', name: 'Webhook Endpoint' }
];

// Cores para console
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m'
};

console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
console.log(`${colors.cyan}║   DIAGNÓSTICO DA API CHATGURU                 ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Função para testar endpoint
async function testEndpoint(endpoint) {
    return new Promise((resolve) => {
        console.log(`${colors.blue}Testando: ${endpoint.name}${colors.reset}`);
        console.log(`URL: ${endpoint.url}`);
        
        const url = new URL(endpoint.url);
        
        const options = {
            hostname: url.hostname,
            port: 443,
            path: url.pathname,
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'ChatGuru-Diagnostic/1.0'
            },
            timeout: 5000
        };

        const req = https.request(options, (res) => {
            console.log(`Status: ${res.statusCode}`);
            console.log(`Headers:`, JSON.stringify(res.headers, null, 2).substring(0, 200));
            
            let data = '';
            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                if (res.statusCode === 200 || res.statusCode === 201) {
                    console.log(`${colors.green}✅ Endpoint acessível${colors.reset}`);
                } else if (res.statusCode === 404) {
                    console.log(`${colors.red}❌ Endpoint não encontrado (404)${colors.reset}`);
                } else if (res.statusCode === 401 || res.statusCode === 403) {
                    console.log(`${colors.yellow}⚠️ Endpoint requer autenticação${colors.reset}`);
                } else {
                    console.log(`${colors.yellow}⚠️ Status inesperado: ${res.statusCode}${colors.reset}`);
                }
                
                if (data) {
                    console.log(`Resposta: ${data.substring(0, 200)}...`);
                }
                
                console.log('---');
                console.log('');
                resolve();
            });
        });

        req.on('error', (err) => {
            console.log(`${colors.red}❌ Erro: ${err.message}${colors.reset}`);
            console.log('');
            resolve();
        });

        req.on('timeout', () => {
            console.log(`${colors.red}❌ Timeout após 5 segundos${colors.reset}`);
            console.log('');
            req.destroy();
            resolve();
        });

        req.end();
    });
}

// Função para testar dialog_execute
async function testDialogExecute() {
    console.log(`${colors.cyan}═══ Testando dialog_execute ═══${colors.reset}`);
    console.log('');
    
    const possibleUrls = [
        'https://s15.chatguru.app/api/v1/dialog_execute',
        'https://api.chatguru.app/v1/dialog_execute',
        'https://app.chatguru.com/api/v1/dialog_execute'
    ];

    for (const urlString of possibleUrls) {
        console.log(`${colors.blue}Testando POST: ${urlString}${colors.reset}`);
        
        const payload = JSON.stringify({
            chat_number: config.chatNumber,
            dialog_id: "TESTE_API",
            variables: {
                tarefa: "Teste de diagnóstico",
                prioridade: "Alta",
                responsavel: "Sistema",
                descricao: "Teste de endpoint"
            },
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        });

        await new Promise((resolve) => {
            const url = new URL(urlString);
            
            const options = {
                hostname: url.hostname,
                port: 443,
                path: url.pathname,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(payload),
                    'User-Agent': 'ChatGuru-Diagnostic/1.0'
                },
                timeout: 5000
            };

            const req = https.request(options, (res) => {
                console.log(`Status: ${res.statusCode}`);
                
                let data = '';
                res.on('data', (chunk) => {
                    data += chunk;
                });

                res.on('end', () => {
                    if (res.statusCode === 200 || res.statusCode === 201) {
                        console.log(`${colors.green}✅ Sucesso!${colors.reset}`);
                    } else {
                        console.log(`${colors.red}❌ Falhou: ${res.statusCode}${colors.reset}`);
                    }
                    
                    if (data) {
                        try {
                            const parsed = JSON.parse(data);
                            console.log('Resposta:', JSON.stringify(parsed, null, 2).substring(0, 300));
                        } catch {
                            console.log('Resposta:', data.substring(0, 300));
                        }
                    }
                    
                    console.log('---');
                    console.log('');
                    resolve();
                });
            });

            req.on('error', (err) => {
                console.log(`${colors.red}❌ Erro: ${err.message}${colors.reset}`);
                console.log('');
                resolve();
            });

            req.on('timeout', () => {
                console.log(`${colors.red}❌ Timeout após 5 segundos${colors.reset}`);
                console.log('');
                req.destroy();
                resolve();
            });

            req.write(payload);
            req.end();
        });
    }
}

// Função para testar mensagens
async function testSendMessage() {
    console.log(`${colors.cyan}═══ Testando envio de mensagem ═══${colors.reset}`);
    console.log('');
    
    const possibleUrls = [
        'https://s15.chatguru.app/api/v1/messages',
        'https://api.chatguru.app/v1/messages',
        'https://app.chatguru.com/api/v1/messages'
    ];

    for (const urlString of possibleUrls) {
        console.log(`${colors.blue}Testando POST: ${urlString}${colors.reset}`);
        
        const payload = JSON.stringify({
            number: config.chatNumber,
            message: "Teste de diagnóstico da API",
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        });

        await new Promise((resolve) => {
            const url = new URL(urlString);
            
            const options = {
                hostname: url.hostname,
                port: 443,
                path: url.pathname,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(payload),
                    'Authorization': `Bearer ${config.apiKey}`,
                    'User-Agent': 'ChatGuru-Diagnostic/1.0'
                },
                timeout: 5000
            };

            const req = https.request(options, (res) => {
                console.log(`Status: ${res.statusCode}`);
                
                let data = '';
                res.on('data', (chunk) => {
                    data += chunk;
                });

                res.on('end', () => {
                    if (res.statusCode === 200 || res.statusCode === 201) {
                        console.log(`${colors.green}✅ Sucesso!${colors.reset}`);
                    } else {
                        console.log(`${colors.red}❌ Falhou: ${res.statusCode}${colors.reset}`);
                    }
                    
                    if (data) {
                        try {
                            const parsed = JSON.parse(data);
                            console.log('Resposta:', JSON.stringify(parsed, null, 2).substring(0, 300));
                        } catch {
                            console.log('Resposta:', data.substring(0, 300));
                        }
                    }
                    
                    console.log('---');
                    console.log('');
                    resolve();
                });
            });

            req.on('error', (err) => {
                console.log(`${colors.red}❌ Erro: ${err.message}${colors.reset}`);
                console.log('');
                resolve();
            });

            req.on('timeout', () => {
                console.log(`${colors.red}❌ Timeout após 5 segundos${colors.reset}`);
                console.log('');
                req.destroy();
                resolve();
            });

            req.write(payload);
            req.end();
        });
    }
}

// Executar testes
async function runTests() {
    console.log(`${colors.cyan}═══ TESTANDO ENDPOINTS GERAIS ═══${colors.reset}`);
    console.log('');
    
    for (const endpoint of endpoints) {
        await testEndpoint(endpoint);
    }
    
    console.log(`${colors.cyan}═══ TESTANDO ENDPOINTS ESPECÍFICOS ═══${colors.reset}`);
    console.log('');
    
    await testDialogExecute();
    await testSendMessage();
    
    console.log(`${colors.cyan}═══ TESTES CONCLUÍDOS ═══${colors.reset}`);
    console.log('');
    console.log(`${colors.yellow}Dica: Se todos os endpoints falharem, verifique:${colors.reset}`);
    console.log('1. A chave API está correta');
    console.log('2. O account_id e phone_id estão corretos');
    console.log('3. Você tem permissões para acessar a API');
    console.log('4. O servidor/instância S15 está ativo');
}

// Executar
runTests().catch(console.error);