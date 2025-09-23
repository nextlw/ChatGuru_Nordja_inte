#!/usr/bin/env node

/**
 * Script para descobrir os endpoints corretos da API ChatGuru
 */

const https = require('https');

// Configurações
const config = {
    apiKey: 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
    accountId: '625584ce6fdcb7bda7d94aa8',
    phoneId: '6537de23b6d5b0bb0b80421a',
    chatNumber: '5585989530473'
};

// Cores para console
const colors = {
    reset: '\x1b[0m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m'
};

console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
console.log(`${colors.cyan}║   DESCOBRINDO ENDPOINTS CHATGURU               ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Lista de possíveis variações de URLs e endpoints
const urlVariations = [
    // Variações com s15
    { base: 'https://s15.chatguru.app', paths: [
        '',
        '/api',
        '/api/v1',
        '/api/v2',
        '/v1',
        '/webhook',
        '/webhooks'
    ]},
    // Variações com api
    { base: 'https://api.chatguru.app', paths: [
        '',
        '/v1',
        '/v2',
        '/api/v1'
    ]},
    // Variações com app
    { base: 'https://app.chatguru.com', paths: [
        '/api',
        '/api/v1',
        '/v1'
    ]},
    // Variações com s15 sem app
    { base: 'https://s15.chatguru.com', paths: [
        '/api',
        '/api/v1',
        '/v1'
    ]}
];

// Endpoints específicos para testar
const specificEndpoints = [
    '/dialog_execute',
    '/dialogs/list',
    '/messages',
    '/note_add',
    '/annotation',
    '/webhook',
    '/health',
    '/status',
    '/ping'
];

// Função para testar uma URL
function testUrl(url, method = 'GET', payload = null) {
    return new Promise((resolve) => {
        const urlObj = new URL(url);
        
        const options = {
            hostname: urlObj.hostname,
            port: 443,
            path: urlObj.pathname,
            method: method,
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'ChatGuru-Endpoint-Discovery/1.0'
            },
            timeout: 3000,
            rejectUnauthorized: false
        };

        if (payload) {
            options.headers['Content-Length'] = Buffer.byteLength(JSON.stringify(payload));
        }

        const req = https.request(options, (res) => {
            let data = '';
            
            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                resolve({
                    url,
                    method,
                    status: res.statusCode,
                    headers: res.headers,
                    data: data.substring(0, 100)
                });
            });
        });

        req.on('error', (err) => {
            resolve({
                url,
                method,
                status: 'ERROR',
                error: err.message
            });
        });

        req.on('timeout', () => {
            req.destroy();
            resolve({
                url,
                method,
                status: 'TIMEOUT'
            });
        });

        if (payload) {
            req.write(JSON.stringify(payload));
        }
        req.end();
    });
}

// Função principal
async function discoverEndpoints() {
    const results = [];
    
    console.log(`${colors.blue}Fase 1: Testando URLs base${colors.reset}`);
    console.log('');
    
    // Testar URLs base
    for (const variation of urlVariations) {
        for (const path of variation.paths) {
            const url = `${variation.base}${path}`;
            process.stdout.write(`Testando ${url}... `);
            
            const result = await testUrl(url);
            
            if (result.status === 'ERROR' || result.status === 'TIMEOUT') {
                console.log(`${colors.red}${result.status}${colors.reset}`);
            } else if (result.status === 404) {
                console.log(`${colors.yellow}404${colors.reset}`);
            } else if (result.status >= 200 && result.status < 300) {
                console.log(`${colors.green}${result.status} ✓${colors.reset}`);
                results.push(result);
            } else if (result.status === 401 || result.status === 403) {
                console.log(`${colors.cyan}${result.status} (Auth)${colors.reset}`);
                results.push(result);
            } else {
                console.log(`${colors.yellow}${result.status}${colors.reset}`);
            }
        }
    }
    
    console.log('');
    console.log(`${colors.blue}Fase 2: Testando endpoints específicos com POST${colors.reset}`);
    console.log('');
    
    // Para URLs promissoras, testar endpoints específicos
    const promisingBases = [
        'https://s15.chatguru.app',
        'https://s15.chatguru.app/api/v1',
        'https://api.chatguru.app',
        'https://api.chatguru.app/v1'
    ];
    
    for (const base of promisingBases) {
        for (const endpoint of specificEndpoints) {
            const url = `${base}${endpoint}`;
            process.stdout.write(`POST ${url}... `);
            
            const payload = {
                key: config.apiKey,
                account_id: config.accountId,
                phone_id: config.phoneId
            };
            
            if (endpoint.includes('dialog')) {
                payload.chat_number = config.chatNumber;
                payload.dialog_id = 'TESTE_API';
            }
            
            const result = await testUrl(url, 'POST', payload);
            
            if (result.status === 'ERROR' || result.status === 'TIMEOUT') {
                console.log(`${colors.red}${result.status}${colors.reset}`);
            } else if (result.status === 404) {
                console.log(`${colors.yellow}404${colors.reset}`);
            } else if (result.status >= 200 && result.status < 300) {
                console.log(`${colors.green}${result.status} ✓ ENCONTRADO!${colors.reset}`);
                results.push(result);
                console.log(`  Resposta: ${result.data}`);
            } else if (result.status === 401 || result.status === 403) {
                console.log(`${colors.cyan}${result.status} (Requer Auth)${colors.reset}`);
                results.push(result);
            } else {
                console.log(`${colors.yellow}${result.status}${colors.reset}`);
                if (result.data) {
                    console.log(`  Resposta: ${result.data}`);
                }
            }
        }
    }
    
    console.log('');
    console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║   RESUMO DOS ENDPOINTS ENCONTRADOS             ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
    console.log('');
    
    const successfulEndpoints = results.filter(r => 
        r.status >= 200 && r.status < 500 && r.status !== 404
    );
    
    if (successfulEndpoints.length > 0) {
        console.log(`${colors.green}Endpoints acessíveis:${colors.reset}`);
        successfulEndpoints.forEach(endpoint => {
            const statusColor = endpoint.status < 300 ? colors.green : 
                               endpoint.status < 400 ? colors.yellow : colors.cyan;
            console.log(`  ${statusColor}[${endpoint.status}]${colors.reset} ${endpoint.method} ${endpoint.url}`);
            if (endpoint.data) {
                console.log(`       Data: ${endpoint.data}`);
            }
        });
    } else {
        console.log(`${colors.red}Nenhum endpoint acessível encontrado${colors.reset}`);
    }
    
    console.log('');
    console.log(`${colors.yellow}Observações:${colors.reset}`);
    console.log('- Status 2xx = Endpoint funcional');
    console.log('- Status 401/403 = Endpoint existe mas requer autenticação');
    console.log('- Status 404 = Endpoint não existe');
    console.log('- ERROR/TIMEOUT = Servidor não responde');
}

// Executar
discoverEndpoints().catch(console.error);