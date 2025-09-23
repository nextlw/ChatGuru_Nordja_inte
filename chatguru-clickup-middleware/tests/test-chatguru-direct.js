#!/usr/bin/env node

/**
 * Teste direto da API do ChatGuru
 * Verifica endpoints e configurações
 */

const axios = require('axios');

// Configurações
const config = {
    apiKey: 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
    accountId: '625584ce6fdcb7bda7d94aa8',
    phoneId: '6537de23b6d5b0bb0b80421a',
    chatNumber: '5585989530473',
    baseUrl: 'https://s15.chatguru.app/api/v1'
};

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
console.log(`${colors.cyan}║   TESTE DIRETO DA API CHATGURU                ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Função para testar endpoint
async function testEndpoint(name, method, endpoint, data = null) {
    console.log(`${colors.blue}═══ Testando: ${name} ═══${colors.reset}`);
    console.log(`Método: ${method}`);
    console.log(`URL: ${config.baseUrl}${endpoint}`);
    
    try {
        const axiosConfig = {
            method,
            url: `${config.baseUrl}${endpoint}`,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            },
            timeout: 10000
        };

        if (data) {
            axiosConfig.data = data;
            console.log('Payload:', JSON.stringify(data, null, 2));
        }

        const response = await axios(axiosConfig);
        
        console.log(`${colors.green}✅ Status: ${response.status}${colors.reset}`);
        console.log(`Resposta:`, JSON.stringify(response.data, null, 2).substring(0, 500));
        
        return { success: true, status: response.status, data: response.data };
    } catch (error) {
        if (error.response) {
            console.log(`${colors.red}❌ Erro HTTP: ${error.response.status}${colors.reset}`);
            console.log(`Mensagem:`, error.response.statusText);
            if (error.response.data) {
                console.log(`Dados:`, JSON.stringify(error.response.data, null, 2).substring(0, 500));
            }
        } else if (error.request) {
            console.log(`${colors.red}❌ Sem resposta do servidor${colors.reset}`);
            console.log(`Erro:`, error.message);
        } else {
            console.log(`${colors.red}❌ Erro na configuração${colors.reset}`);
            console.log(`Erro:`, error.message);
        }
        
        return { success: false, error: error.message };
    } finally {
        console.log('');
        console.log('');
    }
}

// Função principal
async function main() {
    const results = [];
    
    // 1. Testar health check básico
    console.log(`${colors.cyan}1. TESTE DE CONECTIVIDADE${colors.reset}`);
    console.log('');
    results.push(await testEndpoint(
        'Health Check',
        'GET',
        '/health'
    ));
    
    // 2. Testar autenticação
    console.log(`${colors.cyan}2. TESTE DE AUTENTICAÇÃO${colors.reset}`);
    console.log('');
    results.push(await testEndpoint(
        'Verificar Conta',
        'POST',
        '/account/verify',
        {
            key: config.apiKey,
            account_id: config.accountId
        }
    ));
    
    // 3. Testar listagem de diálogos
    console.log(`${colors.cyan}3. TESTE DE DIÁLOGOS${colors.reset}`);
    console.log('');
    results.push(await testEndpoint(
        'Listar Diálogos',
        'POST',
        '/dialogs/list',
        {
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        }
    ));
    
    // 4. Testar execução de diálogo TESTE_API
    console.log(`${colors.cyan}4. TESTE DE EXECUÇÃO DE DIÁLOGO${colors.reset}`);
    console.log('');
    results.push(await testEndpoint(
        'Executar TESTE_API',
        'POST',
        '/dialog_execute',
        {
            chat_number: config.chatNumber,
            dialog_id: 'TESTE_API',
            variables: {
                tarefa: 'Teste direto da API',
                prioridade: 'Alta',
                responsavel: 'Sistema',
                descricao: 'Verificando funcionamento da API'
            },
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        }
    ));
    
    // 5. Testar adição de anotação
    console.log(`${colors.cyan}5. TESTE DE ANOTAÇÃO${colors.reset}`);
    console.log('');
    results.push(await testEndpoint(
        'Adicionar Anotação',
        'POST',
        '/note_add',
        {
            chat_number: config.chatNumber,
            note: `Anotação de teste - ${new Date().toISOString()}`,
            key: config.apiKey,
            account_id: config.accountId,
            phone_id: config.phoneId
        }
    ));
    
    // 6. Testar webhook direto no middleware
    console.log(`${colors.cyan}6. TESTE DO WEBHOOK DO MIDDLEWARE${colors.reset}`);
    console.log('');
    
    const webhookUrl = 'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru';
    console.log(`${colors.blue}═══ Testando: Webhook Direto ═══${colors.reset}`);
    console.log(`URL: ${webhookUrl}`);
    
    try {
        const webhookPayload = {
            event_type: 'annotation.added',
            id: `test_${Date.now()}`,
            timestamp: new Date().toISOString(),
            account: {
                id: config.accountId,
                name: 'Nordja Test'
            },
            phone: {
                id: config.phoneId,
                number: '+558512345678'
            },
            chat: {
                number: config.chatNumber,
                contact: {
                    name: 'Cliente Teste',
                    number: config.chatNumber
                }
            },
            annotation: {
                id: `ann_${Date.now()}`,
                text: 'TAREFA: Teste de integração\nPRIORIDADE: Alta\nRESPONSÁVEL: Sistema\nDESCRIÇÃO: Teste completo da integração ChatGuru-ClickUp',
                created_at: new Date().toISOString()
            }
        };
        
        console.log('Payload do Webhook:', JSON.stringify(webhookPayload, null, 2));
        
        const webhookResponse = await axios.post(webhookUrl, webhookPayload, {
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            },
            timeout: 10000
        });
        
        console.log(`${colors.green}✅ Status: ${webhookResponse.status}${colors.reset}`);
        console.log(`Resposta:`, JSON.stringify(webhookResponse.data, null, 2));
        
        results.push({ success: true, status: webhookResponse.status });
    } catch (error) {
        if (error.response) {
            console.log(`${colors.red}❌ Erro HTTP: ${error.response.status}${colors.reset}`);
            console.log(`Mensagem:`, error.response.statusText);
            if (error.response.data) {
                console.log(`Dados:`, JSON.stringify(error.response.data, null, 2));
            }
        } else if (error.request) {
            console.log(`${colors.red}❌ Sem resposta do servidor${colors.reset}`);
            console.log(`Erro:`, error.message);
        } else {
            console.log(`${colors.red}❌ Erro na configuração${colors.reset}`);
            console.log(`Erro:`, error.message);
        }
        results.push({ success: false, error: error.message });
    }
    
    // Resumo dos resultados
    console.log('');
    console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║   RESUMO DOS TESTES                            ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
    console.log('');
    
    const successful = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success).length;
    
    console.log(`${colors.green}✅ Bem-sucedidos: ${successful}${colors.reset}`);
    console.log(`${colors.red}❌ Falhados: ${failed}${colors.reset}`);
    
    console.log('');
    console.log(`${colors.yellow}CONFIGURAÇÕES UTILIZADAS:${colors.reset}`);
    console.log(`- API Key: ${config.apiKey.substring(0, 10)}...`);
    console.log(`- Account ID: ${config.accountId}`);
    console.log(`- Phone ID: ${config.phoneId}`);
    console.log(`- Chat Number: ${config.chatNumber}`);
    console.log(`- Base URL: ${config.baseUrl}`);
    
    if (failed > 0) {
        console.log('');
        console.log(`${colors.yellow}⚠️ Alguns testes falharam. Verifique:${colors.reset}`);
        console.log('1. As credenciais estão corretas');
        console.log('2. A instância S15 está ativa');
        console.log('3. O middleware está rodando corretamente');
        console.log('4. Os endpoints da API mudaram');
    }
}

// Executar testes
main().catch(error => {
    console.error(`${colors.red}❌ Erro fatal:${colors.reset}`, error);
    process.exit(1);
});