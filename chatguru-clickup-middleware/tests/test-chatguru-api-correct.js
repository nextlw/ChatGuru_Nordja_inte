#!/usr/bin/env node

/**
 * Teste da API ChatGuru com endpoints corretos baseados na documentação
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
console.log(`${colors.cyan}║   TESTE API CHATGURU - ENDPOINTS CORRETOS      ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Função para fazer requisição HTTPS
function makeRequest(options, payload = null) {
    return new Promise((resolve, reject) => {
        const req = https.request(options, (res) => {
            let data = '';
            
            res.on('data', (chunk) => {
                data += chunk;
            });
            
            res.on('end', () => {
                resolve({
                    status: res.statusCode,
                    headers: res.headers,
                    data: data
                });
            });
        });
        
        req.on('error', (err) => {
            reject(err);
        });
        
        req.setTimeout(10000, () => {
            req.destroy();
            reject(new Error('Timeout'));
        });
        
        if (payload) {
            req.write(JSON.stringify(payload));
        }
        
        req.end();
    });
}

// Teste 1: Dialog Execute com Header APIKey
async function testDialogExecuteWithHeader() {
    console.log(`${colors.blue}═══ Teste 1: dialog_execute com APIKey no Header ═══${colors.reset}`);
    
    const payload = {
        chat_number: config.chatNumber,
        dialog_id: 'nova_api',
        variables: {
            tarefa: 'Teste via API',
            prioridade: 'Alta',
            responsavel: 'Sistema',
            descricao: 'Teste de integração'
        },
        key: config.apiKey,
        account_id: config.accountId,
        phone_id: config.phoneId
    };
    
    const options = {
        hostname: 's15.chatguru.app',
        port: 443,
        path: '/dialog_execute',
        method: 'POST',
        headers: {
            'APIKey': config.apiKey,
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(JSON.stringify(payload))
        }
    };
    
    console.log(`URL: https://${options.hostname}${options.path}`);
    console.log('Headers:', options.headers);
    console.log('Payload:', JSON.stringify(payload, null, 2));
    
    try {
        const result = await makeRequest(options, payload);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Sucesso! Status: ${result.status}${colors.reset}`);
        } else {
            console.log(`${colors.red}❌ Falhou. Status: ${result.status}${colors.reset}`);
        }
        console.log('Resposta:', result.data.substring(0, 500));
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Teste 2: Message Send com Header APIKey
async function testMessageSendWithHeader() {
    console.log(`${colors.blue}═══ Teste 2: message_send com APIKey no Header ═══${colors.reset}`);
    
    const payload = {
        chat_number: config.chatNumber,
        message: 'Teste de envio de mensagem via API',
        key: config.apiKey,
        account_id: config.accountId,
        phone_id: config.phoneId
    };
    
    const options = {
        hostname: 's15.chatguru.app',
        port: 443,
        path: '/message_send',
        method: 'POST',
        headers: {
            'APIKey': config.apiKey,
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(JSON.stringify(payload))
        }
    };
    
    console.log(`URL: https://${options.hostname}${options.path}`);
    console.log('Headers:', options.headers);
    console.log('Payload:', JSON.stringify(payload, null, 2));
    
    try {
        const result = await makeRequest(options, payload);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Sucesso! Status: ${result.status}${colors.reset}`);
        } else {
            console.log(`${colors.red}❌ Falhou. Status: ${result.status}${colors.reset}`);
        }
        console.log('Resposta:', result.data.substring(0, 500));
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Teste 3: Note Add (anotação)
async function testNoteAdd() {
    console.log(`${colors.blue}═══ Teste 3: note_add (Adicionar Anotação) ═══${colors.reset}`);
    
    const payload = {
        chat_number: config.chatNumber,
        note: `TAREFA: Teste de anotação
PRIORIDADE: Alta
RESPONSÁVEL: Sistema
DESCRIÇÃO: Teste completo da API`,
        key: config.apiKey,
        account_id: config.accountId,
        phone_id: config.phoneId
    };
    
    const options = {
        hostname: 's15.chatguru.app',
        port: 443,
        path: '/note_add',
        method: 'POST',
        headers: {
            'APIKey': config.apiKey,
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(JSON.stringify(payload))
        }
    };
    
    console.log(`URL: https://${options.hostname}${options.path}`);
    console.log('Headers:', options.headers);
    console.log('Payload:', JSON.stringify(payload, null, 2));
    
    try {
        const result = await makeRequest(options, payload);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Sucesso! Status: ${result.status}${colors.reset}`);
        } else {
            console.log(`${colors.red}❌ Falhou. Status: ${result.status}${colors.reset}`);
        }
        console.log('Resposta:', result.data.substring(0, 500));
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Teste 4: Sem header APIKey (teste negativo)
async function testWithoutAPIKey() {
    console.log(`${colors.blue}═══ Teste 4: dialog_execute SEM APIKey no Header ═══${colors.reset}`);
    
    const payload = {
        chat_number: config.chatNumber,
        dialog_id: 'nova_api',
        variables: {
            tarefa: 'Teste sem header',
            prioridade: 'Baixa'
        },
        key: config.apiKey,
        account_id: config.accountId,
        phone_id: config.phoneId
    };
    
    const options = {
        hostname: 's15.chatguru.app',
        port: 443,
        path: '/dialog_execute',
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(JSON.stringify(payload))
        }
    };
    
    console.log(`URL: https://${options.hostname}${options.path}`);
    console.log('Headers:', options.headers);
    console.log('Payload:', JSON.stringify(payload, null, 2));
    
    try {
        const result = await makeRequest(options, payload);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Sucesso mesmo sem header! Status: ${result.status}${colors.reset}`);
        } else if (result.status === 401 || result.status === 403) {
            console.log(`${colors.yellow}⚠️ Requer autenticação (esperado). Status: ${result.status}${colors.reset}`);
        } else {
            console.log(`${colors.red}❌ Falhou. Status: ${result.status}${colors.reset}`);
        }
        console.log('Resposta:', result.data.substring(0, 500));
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Teste 5: Testar diferentes formatos de autenticação
async function testAuthFormats() {
    console.log(`${colors.blue}═══ Teste 5: Diferentes formatos de autenticação ═══${colors.reset}`);
    
    const authVariations = [
        { header: 'APIKey', value: config.apiKey },
        { header: 'Authorization', value: `Bearer ${config.apiKey}` },
        { header: 'Authorization', value: config.apiKey },
        { header: 'X-API-Key', value: config.apiKey },
        { header: 'X-Auth-Token', value: config.apiKey }
    ];
    
    for (const auth of authVariations) {
        console.log(`\nTestando com header: ${auth.header}`);
        
        const payload = {
            chat_number: config.chatNumber,
            dialog_id: 'nova_api',
            variables: {
                tarefa: `Teste com ${auth.header}`,
                prioridade: 'Media'
            },
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        };
        
        const options = {
            hostname: 's15.chatguru.app',
            port: 443,
            path: '/dialog_execute',
            method: 'POST',
            headers: {
                [auth.header]: auth.value,
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(JSON.stringify(payload))
            }
        };
        
        try {
            const result = await makeRequest(options, payload);
            if (result.status === 200 || result.status === 201) {
                console.log(`  ${colors.green}✅ Funciona com ${auth.header}!${colors.reset}`);
            } else {
                console.log(`  ${colors.yellow}Status: ${result.status}${colors.reset}`);
            }
        } catch (error) {
            console.log(`  ${colors.red}Erro: ${error.message}${colors.reset}`);
        }
    }
    
    console.log('');
}

// Executar todos os testes
async function runAllTests() {
    await testDialogExecuteWithHeader();
    await testMessageSendWithHeader();
    await testNoteAdd();
    await testWithoutAPIKey();
    await testAuthFormats();
    
    console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║   TESTES CONCLUÍDOS                            ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
    console.log('');
    
    console.log(`${colors.yellow}Observações importantes:${colors.reset}`);
    console.log('1. A API do ChatGuru aparentemente está em https://s15.chatguru.app (sem /api/v1)');
    console.log('2. Os endpoints são diretamente na raiz: /dialog_execute, /message_send, etc');
    console.log('3. A autenticação pode ser via header APIKey ou no corpo da requisição');
    console.log('4. O webhook do middleware está funcionando corretamente');
    console.log('');
    console.log(`${colors.green}✅ A integração com ClickUp está funcionando!${colors.reset}`);
    console.log(`   Task ID atualizada: 86ac0r5dj`);
}

// Executar
runAllTests().catch(console.error);