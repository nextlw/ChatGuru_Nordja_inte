#!/usr/bin/env node

const readline = require('readline');
const https = require('https');

// Cores para o terminal
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    dim: '\x1b[2m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m',
    white: '\x1b[37m'
};

// Configuração da API
let CONFIG = {
    API_BASE_URL: 'https://s15.chatguru.app/api/v1',  // Endpoint correto do s15
    API_KEY: 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
    ACCOUNT_ID: '625584ce6fdcb7bda7d94aa8',
    PHONE_ID: '6537de23b6d5b0bb0b80421a'  // Default: +55 (11) 98834-8951
    // Alternativo: '62558780e2923cc4705beee1' para +55 (11) 97052-5814
};

// Interface readline
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Função para fazer perguntas
function ask(question) {
    return new Promise(resolve => {
        rl.question(`${colors.cyan}${question}${colors.reset}`, answer => {
            resolve(answer.trim());
        });
    });
}

// Função para fazer requisições HTTP
function makeRequest(endpoint, method = 'GET', data = null) {
    return new Promise((resolve, reject) => {
        const url = new URL(`${CONFIG.API_BASE_URL}${endpoint}`);
        
        const options = {
            hostname: url.hostname,
            port: 443,
            path: url.pathname,
            method: method,
            headers: {
                'Content-Type': 'application/json'
            }
        };

        // Adicionar dados da API key no body para requests POST
        if (method === 'POST' && data) {
            data.key = CONFIG.API_KEY;
            if (CONFIG.ACCOUNT_ID) data.account_id = CONFIG.ACCOUNT_ID;
            if (CONFIG.PHONE_ID) data.phone_id = CONFIG.PHONE_ID;
        }

        const req = https.request(options, (res) => {
            let responseData = '';

            res.on('data', (chunk) => {
                responseData += chunk;
            });

            res.on('end', () => {
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    try {
                        resolve(JSON.parse(responseData));
                    } catch {
                        resolve(responseData);
                    }
                } else {
                    reject(new Error(`HTTP ${res.statusCode}: ${responseData}`));
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

// Função principal do menu
async function showMenu() {
    console.log(`\n${colors.cyan}╔════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║     ChatGuru CLI Tool                  ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════╝${colors.reset}\n`);
    
    console.log(`${colors.bright}Escolha uma opção:${colors.reset}`);
    console.log(`${colors.green}1.${colors.reset} Enviar mensagem de teste`);
    console.log(`${colors.green}2.${colors.reset} Executar diálogo (nova_api)`);
    console.log(`${colors.green}3.${colors.reset} Executar diálogo (TESTE_API)`);
    console.log(`${colors.green}4.${colors.reset} Adicionar anotação em contato`);
    console.log(`${colors.green}5.${colors.reset} Atualizar campos personalizados`);
    console.log(`${colors.green}6.${colors.reset} Testar webhook diretamente`);
    console.log(`${colors.green}7.${colors.reset} Configurar credenciais`);
    console.log(`${colors.green}8.${colors.reset} Verificar status da integração`);
    console.log(`${colors.red}0.${colors.reset} Sair\n`);
}

// 1. Enviar mensagem
async function sendMessage() {
    console.log(`\n${colors.yellow}📱 Enviar Mensagem${colors.reset}`);
    
    if (!CONFIG.ACCOUNT_ID || !CONFIG.PHONE_ID) {
        console.log(`${colors.red}❌ Configure primeiro as credenciais (opção 7)${colors.reset}`);
        return;
    }
    
    const phoneNumber = await ask('Número do WhatsApp (com código do país, ex: 5511999999999): ');
    const message = await ask('Mensagem a enviar: ');
    
    try {
        console.log(`${colors.yellow}Enviando mensagem...${colors.reset}`);
        
        const result = await makeRequest('/message_send', 'POST', {
            chat_number: phoneNumber,
            message: message
        });
        
        console.log(`${colors.green}✅ Mensagem enviada com sucesso!${colors.reset}`);
        console.log('Resposta:', JSON.stringify(result, null, 2));
    } catch (error) {
        console.log(`${colors.red}❌ Erro ao enviar mensagem:${colors.reset}`, error.message);
    }
}

// 2. Executar diálogo
async function executeDialog(dialogId) {
    console.log(`\n${colors.yellow}🤖 Executar Diálogo: ${dialogId}${colors.reset}`);
    
    if (!CONFIG.ACCOUNT_ID || !CONFIG.PHONE_ID) {
        console.log(`${colors.red}❌ Configure primeiro as credenciais (opção 7)${colors.reset}`);
        return;
    }
    
    const phoneNumber = await ask('Número do WhatsApp (com código do país, ex: 5511999999999): ');
    const taskDescription = await ask('Descrição da tarefa: ');
    const priority = await ask('Prioridade (Alta/Média/Baixa): ');
    
    try {
        console.log(`${colors.yellow}Executando diálogo...${colors.reset}`);
        
        const result = await makeRequest('/dialog_execute', 'POST', {
            chat_number: phoneNumber,
            dialog_id: dialogId,
            variables: {
                tarefa: taskDescription,
                prioridade: priority,
                responsavel: 'Sistema CLI'
            }
        });
        
        console.log(`${colors.green}✅ Diálogo executado com sucesso!${colors.reset}`);
        console.log('Resposta:', JSON.stringify(result, null, 2));
    } catch (error) {
        console.log(`${colors.red}❌ Erro ao executar diálogo:${colors.reset}`, error.message);
    }
}

// 3. Adicionar anotação
async function addNote() {
    console.log(`\n${colors.yellow}📝 Adicionar Anotação${colors.reset}`);
    
    if (!CONFIG.ACCOUNT_ID || !CONFIG.PHONE_ID) {
        console.log(`${colors.red}❌ Configure primeiro as credenciais (opção 7)${colors.reset}`);
        return;
    }
    
    const phoneNumber = await ask('Número do WhatsApp (com código do país): ');
    const note = await ask('Anotação: ');
    
    try {
        console.log(`${colors.yellow}Adicionando anotação...${colors.reset}`);
        
        const result = await makeRequest('/note_add', 'POST', {
            chat_number: phoneNumber,
            note: note
        });
        
        console.log(`${colors.green}✅ Anotação adicionada!${colors.reset}`);
        console.log('Resposta:', JSON.stringify(result, null, 2));
    } catch (error) {
        console.log(`${colors.red}❌ Erro:${colors.reset}`, error.message);
    }
}

// 4. Atualizar campos personalizados
async function updateCustomFields() {
    console.log(`\n${colors.yellow}🔧 Atualizar Campos Personalizados${colors.reset}`);
    
    if (!CONFIG.ACCOUNT_ID || !CONFIG.PHONE_ID) {
        console.log(`${colors.red}❌ Configure primeiro as credenciais (opção 7)${colors.reset}`);
        return;
    }
    
    const phoneNumber = await ask('Número do WhatsApp: ');
    const tarefa = await ask('Tarefa: ');
    const prioridade = await ask('Prioridade: ');
    const responsavel = await ask('Responsável: ');
    
    try {
        console.log(`${colors.yellow}Atualizando campos...${colors.reset}`);
        
        const result = await makeRequest('/chat_update_custom_fields', 'POST', {
            chat_number: phoneNumber,
            custom_fields: {
                tarefa,
                prioridade,
                responsavel
            }
        });
        
        console.log(`${colors.green}✅ Campos atualizados!${colors.reset}`);
    } catch (error) {
        console.log(`${colors.red}❌ Erro:${colors.reset}`, error.message);
    }
}

// 5. Testar webhook
async function testWebhook() {
    console.log(`\n${colors.yellow}🔗 Testar Webhook Diretamente${colors.reset}`);
    
    const webhookUrl = await ask('URL do webhook (default: http://localhost:8080/webhooks/chatguru): ') 
        || 'http://localhost:8080/webhooks/chatguru';
    
    const testData = {
        event_type: 'task_created',
        id: `test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        data: {
            chat_number: '5511999999999',
            message: 'Teste direto do webhook via CLI',
            custom_fields: {
                tarefa: 'Tarefa de teste via CLI',
                prioridade: 'Alta',
                responsavel: 'Sistema'
            }
        }
    };
    
    console.log(`\n${colors.yellow}Enviando dados para o webhook...${colors.reset}`);
    console.log('Dados:', JSON.stringify(testData, null, 2));
    
    const urlParts = new URL(webhookUrl);
    const options = {
        hostname: urlParts.hostname,
        port: urlParts.port || (urlParts.protocol === 'https:' ? 443 : 80),
        path: urlParts.pathname,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        }
    };
    
    const protocol = urlParts.protocol === 'https:' ? https : require('http');
    
    try {
        const req = protocol.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    console.log(`${colors.green}✅ Webhook testado com sucesso!${colors.reset}`);
                    console.log(`Status: ${res.statusCode}`);
                    if (data) console.log('Resposta:', data);
                } else {
                    console.log(`${colors.red}❌ Erro no webhook: ${res.statusCode}${colors.reset}`);
                    if (data) console.log('Resposta:', data);
                }
            });
        });
        
        req.on('error', (err) => {
            console.log(`${colors.red}❌ Erro de conexão:${colors.reset}`, err.message);
        });
        
        req.write(JSON.stringify(testData));
        req.end();
    } catch (error) {
        console.log(`${colors.red}❌ Erro:${colors.reset}`, error.message);
    }
}

// 6. Configurar credenciais
async function configureCredentials() {
    console.log(`\n${colors.yellow}⚙️  Configurar Credenciais${colors.reset}`);
    console.log(`${colors.dim}Obtenha essas informações no painel do ChatGuru${colors.reset}\n`);
    
    CONFIG.API_KEY = await ask(`API Key (atual: ${CONFIG.API_KEY.substring(0, 10)}...): `) || CONFIG.API_KEY;
    CONFIG.ACCOUNT_ID = await ask(`Account ID (atual: ${CONFIG.ACCOUNT_ID || 'não configurado'}): `) || CONFIG.ACCOUNT_ID;
    CONFIG.PHONE_ID = await ask(`Phone ID (atual: ${CONFIG.PHONE_ID || 'não configurado'}): `) || CONFIG.PHONE_ID;
    
    console.log(`\n${colors.green}✅ Credenciais atualizadas!${colors.reset}`);
    
    if (!CONFIG.ACCOUNT_ID || !CONFIG.PHONE_ID) {
        console.log(`${colors.yellow}⚠️  Atenção: Account ID e Phone ID são necessários para usar a API${colors.reset}`);
        console.log(`${colors.dim}Você pode encontrá-los no painel do ChatGuru, na página "Celulares"${colors.reset}`);
    }
}

// 7. Verificar status
async function checkStatus() {
    console.log(`\n${colors.yellow}📊 Verificar Status da Integração${colors.reset}\n`);
    
    // Verificar credenciais
    console.log(`${colors.bright}Credenciais Configuradas:${colors.reset}`);
    console.log(`  API Key: ${colors.green}✓ ${CONFIG.API_KEY.substring(0, 20)}...${colors.reset}`);
    console.log(`  Account ID: ${colors.green}✓ ${CONFIG.ACCOUNT_ID}${colors.reset}`);
    console.log(`  Phone ID: ${colors.green}✓ ${CONFIG.PHONE_ID}${colors.reset}`);
    
    // Mostrar números disponíveis
    console.log(`\n${colors.bright}Números WhatsApp Disponíveis:${colors.reset}`);
    console.log(`  1. ${colors.green}+55 (11) 98834-8951${colors.reset} (ID: 6537de23b6d5b0bb0b80421a) ${CONFIG.PHONE_ID === '6537de23b6d5b0bb0b80421a' ? colors.cyan + '← Ativo' + colors.reset : ''}`);
    console.log(`  2. ${colors.green}+55 (11) 97052-5814${colors.reset} (ID: 62558780e2923cc4705beee1) ${CONFIG.PHONE_ID === '62558780e2923cc4705beee1' ? colors.cyan + '← Ativo' + colors.reset : ''}`);
    
    // Verificar webhook local
    console.log(`\n${colors.bright}Webhook Local:${colors.reset}`);
    try {
        const http = require('http');
        const req = http.request({
            hostname: 'localhost',
            port: 8080,
            path: '/health',
            method: 'GET'
        }, (res) => {
            if (res.statusCode === 200) {
                console.log(`  ${colors.green}✓ Middleware rodando em http://localhost:8080${colors.reset}`);
            } else {
                console.log(`  ${colors.yellow}⚠ Middleware respondeu com status ${res.statusCode}${colors.reset}`);
            }
        });
        
        req.on('error', () => {
            console.log(`  ${colors.red}✗ Middleware não está rodando localmente${colors.reset}`);
        });
        
        req.end();
    } catch (error) {
        console.log(`  ${colors.red}✗ Erro ao verificar webhook local${colors.reset}`);
    }
    
    // URLs importantes
    console.log(`\n${colors.bright}URLs da Integração:${colors.reset}`);
    console.log(`  API ChatGuru: ${CONFIG.API_BASE_URL}`);
    console.log(`  Webhook Local: http://localhost:8080/webhooks/chatguru`);
    console.log(`  Webhook Cloud: https://buzzlightear.rj.r.appspot.com/webhooks/chatguru`);
}

// Menu principal
async function main() {
    console.log(`${colors.bright}${colors.cyan}\n🤖 Bem-vindo ao ChatGuru CLI Tool!${colors.reset}`);
    console.log(`${colors.dim}Ferramenta para gerenciar diálogos e testar integração\n${colors.reset}`);
    
    let running = true;
    
    while (running) {
        await showMenu();
        const option = await ask('Opção: ');
        
        switch (option) {
            case '1':
                await sendMessage();
                break;
            case '2':
                await executeDialog('nova_api');
                break;
            case '3':
                await executeDialog('TESTE_API');
                break;
            case '4':
                await addNote();
                break;
            case '5':
                await updateCustomFields();
                break;
            case '6':
                await testWebhook();
                break;
            case '7':
                await configureCredentials();
                break;
            case '8':
                await checkStatus();
                break;
            case '0':
                running = false;
                console.log(`\n${colors.green}👋 Até logo!${colors.reset}\n`);
                break;
            default:
                console.log(`${colors.red}❌ Opção inválida!${colors.reset}`);
        }
        
        if (running && option !== '0') {
            await ask('\nPressione Enter para continuar...');
        }
    }
    
    rl.close();
}

// Tratamento de erros
process.on('unhandledRejection', (error) => {
    console.error(`${colors.red}❌ Erro não tratado:${colors.reset}`, error);
    rl.close();
    process.exit(1);
});

// Executar aplicação
main().catch(error => {
    console.error(`${colors.red}❌ Erro fatal:${colors.reset}`, error);
    rl.close();
    process.exit(1);
});