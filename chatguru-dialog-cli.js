#!/usr/bin/env node

/**
 * ChatGuru Dialog CLI
 * Ferramenta interativa para criar e gerenciar diálogos via API do ChatGuru
 * 
 * Funcionalidades:
 * - Listar todos os diálogos
 * - Ver detalhes de um diálogo específico
 * - Criar novo diálogo
 * - Atualizar diálogo existente
 * - Deletar diálogo
 * - Configurar webhooks
 * - Testar webhooks
 * - Executar diálogo para um contato
 */

const readline = require('readline');
const https = require('https');

// Configuração da API - Atualize com suas credenciais
const CONFIG = {
    API_BASE_URL: 'https://s12.chatguru.app/api/v1',  // Mudando de s15 para s12
    API_KEY: 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f',  // API Key correta do s12
    ACCOUNT_ID: '625584ce6fdcb7bda7d94aa8',
    PHONE_IDS: {
        'main': '6537de23b6d5b0bb0b80421a',  // +55 (11) 98834-8951
        'secondary': '62558780e2923cc4705beee1' // +55 (11) 97052-5814
    }
};

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

// Interface de linha de comando
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Função para fazer requisições à API
function apiRequest(method, endpoint, data = null) {
    return new Promise((resolve, reject) => {
        const url = new URL(`${CONFIG.API_BASE_URL}${endpoint}`);
        
        const options = {
            hostname: url.hostname,
            port: url.port || 443,
            path: url.pathname + url.search,
            method: method,
            headers: {
                'APIKey': CONFIG.API_KEY,
                'Content-Type': 'application/json'
            }
        };

        const req = https.request(options, (res) => {
            let responseData = '';

            res.on('data', (chunk) => {
                responseData += chunk;
            });

            res.on('end', () => {
                try {
                    const parsed = JSON.parse(responseData);
                    if (res.statusCode >= 200 && res.statusCode < 300) {
                        resolve(parsed);
                    } else {
                        reject({ status: res.statusCode, data: parsed });
                    }
                } catch (e) {
                    resolve({ status: res.statusCode, raw: responseData });
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

// Função para perguntar ao usuário
function ask(question) {
    return new Promise((resolve) => {
        rl.question(question, resolve);
    });
}

// Função para exibir o menu principal
function showMainMenu() {
    console.log('\n' + colors.cyan + '╔════════════════════════════════════════╗' + colors.reset);
    console.log(colors.cyan + '║     ChatGuru Dialog CLI Manager        ║' + colors.reset);
    console.log(colors.cyan + '╚════════════════════════════════════════╝' + colors.reset);
    console.log('\n' + colors.bright + 'Escolha uma opção:' + colors.reset);
    console.log(colors.green + '1.' + colors.reset + ' Listar todos os diálogos');
    console.log(colors.green + '2.' + colors.reset + ' Ver detalhes de um diálogo');
    console.log(colors.green + '3.' + colors.reset + ' Criar novo diálogo');
    console.log(colors.green + '4.' + colors.reset + ' Atualizar diálogo existente');
    console.log(colors.green + '5.' + colors.reset + ' Deletar diálogo');
    console.log(colors.green + '6.' + colors.reset + ' Configurar webhook em diálogo');
    console.log(colors.green + '7.' + colors.reset + ' Testar webhook');
    console.log(colors.green + '8.' + colors.reset + ' Executar diálogo (enviar para contato)');
    console.log(colors.green + '9.' + colors.reset + ' Configurações da API');
    console.log(colors.red + '0.' + colors.reset + ' Sair\n');
}

// Listar todos os diálogos
async function listDialogs() {
    try {
        console.log(colors.yellow + '\n📋 Buscando diálogos...' + colors.reset);
        const response = await apiRequest('GET', '/dialogs');
        
        // A API retorna um array direto, não um objeto com propriedade dialogs
        if (Array.isArray(response) && response.length > 0) {
            console.log(colors.green + `\n✅ Encontrados ${response.length} diálogos:\n` + colors.reset);
            
            response.forEach((dialog, index) => {
                console.log(colors.cyan + `${index + 1}. ${dialog.name || 'Sem nome'}` + colors.reset);
                console.log(`   ID: ${dialog.id}`);
                console.log(`   Status: ${dialog.active ? colors.green + '✅ Ativo' : colors.red + '❌ Inativo'}` + colors.reset);
                console.log(`   Descrição: ${dialog.description || 'Sem descrição'}`);
                
                // Verificar diferentes campos de webhook
                const webhookUrl = dialog.webhookUrl || dialog.webhook || dialog.url || null;
                if (webhookUrl) {
                    console.log(`   Webhook: ${colors.blue}${webhookUrl}${colors.reset}`);
                }
                console.log(colors.dim + '   ─────────────────────────' + colors.reset);
            });
        } else {
            console.log(colors.yellow + '⚠️ Nenhum diálogo encontrado' + colors.reset);
        }
    } catch (error) {
        console.log(colors.red + `❌ Erro ao buscar diálogos: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Ver detalhes de um diálogo
async function viewDialogDetails() {
    const dialogId = await ask(colors.cyan + 'Digite o ID do diálogo: ' + colors.reset);
    
    try {
        console.log(colors.yellow + '\n🔍 Buscando detalhes do diálogo...' + colors.reset);
        const response = await apiRequest('GET', `/dialogs/${dialogId}`);
        
        console.log(colors.green + '\n✅ Detalhes do diálogo:\n' + colors.reset);
        console.log(colors.cyan + 'ID: ' + colors.reset + response.id);
        console.log(colors.cyan + 'Nome: ' + colors.reset + (response.name || 'Sem nome'));
        console.log(colors.cyan + 'Descrição: ' + colors.reset + (response.description || 'Sem descrição'));
        console.log(colors.cyan + 'Status: ' + colors.reset + (response.active ? colors.green + 'Ativo' : colors.red + 'Inativo') + colors.reset);
        
        if (response.webhook || response.webhook_url) {
            console.log(colors.cyan + 'Webhook: ' + colors.reset + colors.blue + (response.webhook || response.webhook_url) + colors.reset);
        }
        
        if (response.actions && response.actions.length > 0) {
            console.log(colors.cyan + '\nAções configuradas:' + colors.reset);
            response.actions.forEach((action, index) => {
                console.log(`  ${index + 1}. Tipo: ${action.type}`);
                if (action.url) console.log(`     URL: ${action.url}`);
            });
        }
        
        if (response.annotations) {
            console.log(colors.cyan + '\nAnotações disponíveis:' + colors.reset);
            console.log(colors.dim + JSON.stringify(response.annotations, null, 2) + colors.reset);
        }
        
        if (response.triggers) {
            console.log(colors.cyan + '\nGatilhos configurados:' + colors.reset);
            console.log(colors.dim + JSON.stringify(response.triggers, null, 2) + colors.reset);
        }
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao buscar detalhes: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Criar novo diálogo
async function createDialog() {
    console.log(colors.cyan + '\n🆕 Criar Novo Diálogo\n' + colors.reset);
    
    const name = await ask('Nome do diálogo: ');
    const description = await ask('Descrição (opcional): ');
    const webhookUrl = await ask('URL do Webhook (opcional): ');
    
    const dialogData = {
        name: name,
        description: description || '',
        active: true,
        type: 'webhook'
    };
    
    if (webhookUrl) {
        dialogData.webhook = webhookUrl;
        dialogData.actions = [{
            type: 'webhook',
            url: webhookUrl,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            }
        }];
    }
    
    try {
        console.log(colors.yellow + '\n📤 Criando diálogo...' + colors.reset);
        const response = await apiRequest('POST', '/dialogs', dialogData);
        
        console.log(colors.green + '\n✅ Diálogo criado com sucesso!' + colors.reset);
        console.log('ID: ' + colors.cyan + response.id + colors.reset);
        console.log('Nome: ' + response.name);
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao criar diálogo: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Atualizar diálogo existente
async function updateDialog() {
    const dialogId = await ask(colors.cyan + 'ID do diálogo a atualizar: ' + colors.reset);
    
    console.log('\nO que deseja atualizar?');
    console.log('1. Nome');
    console.log('2. Descrição');
    console.log('3. Status (ativo/inativo)');
    console.log('4. Webhook URL');
    console.log('5. Todas as opções acima');
    
    const choice = await ask('\nEscolha: ');
    
    const updateData = {};
    
    if (choice === '1' || choice === '5') {
        updateData.name = await ask('Novo nome: ');
    }
    
    if (choice === '2' || choice === '5') {
        updateData.description = await ask('Nova descrição: ');
    }
    
    if (choice === '3' || choice === '5') {
        const statusChoice = await ask('Ativar diálogo? (s/n): ');
        updateData.active = statusChoice.toLowerCase() === 's';
    }
    
    if (choice === '4' || choice === '5') {
        const webhookUrl = await ask('Nova URL do webhook: ');
        updateData.webhook = webhookUrl;
        updateData.actions = [{
            type: 'webhook',
            url: webhookUrl,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            }
        }];
    }
    
    try {
        console.log(colors.yellow + '\n📝 Atualizando diálogo...' + colors.reset);
        const response = await apiRequest('PUT', `/dialogs/${dialogId}`, updateData);
        
        console.log(colors.green + '\n✅ Diálogo atualizado com sucesso!' + colors.reset);
        console.log('ID: ' + colors.cyan + response.id + colors.reset);
        console.log('Nome: ' + response.name);
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao atualizar diálogo: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Deletar diálogo
async function deleteDialog() {
    const dialogId = await ask(colors.cyan + 'ID do diálogo a deletar: ' + colors.reset);
    const confirm = await ask(colors.yellow + `⚠️  Tem certeza que deseja deletar o diálogo ${dialogId}? (s/n): ` + colors.reset);
    
    if (confirm.toLowerCase() !== 's') {
        console.log(colors.yellow + '❌ Operação cancelada' + colors.reset);
        return;
    }
    
    try {
        console.log(colors.yellow + '\n🗑️  Deletando diálogo...' + colors.reset);
        await apiRequest('DELETE', `/dialogs/${dialogId}`);
        
        console.log(colors.green + '\n✅ Diálogo deletado com sucesso!' + colors.reset);
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao deletar diálogo: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Configurar webhook em diálogo
async function configureWebhook() {
    const dialogId = await ask(colors.cyan + 'ID do diálogo: ' + colors.reset);
    const webhookUrl = await ask('URL do webhook: ');
    
    console.log('\nConfigurações adicionais:');
    const method = await ask('Método HTTP (POST/GET/PUT) [POST]: ') || 'POST';
    const addAuth = await ask('Adicionar autenticação? (s/n): ');
    
    const webhookConfig = {
        webhook: webhookUrl,
        actions: [{
            type: 'webhook',
            url: webhookUrl,
            method: method.toUpperCase(),
            headers: {
                'Content-Type': 'application/json'
            }
        }]
    };
    
    if (addAuth.toLowerCase() === 's') {
        const authType = await ask('Tipo de auth (1-Bearer Token, 2-API Key): ');
        const authValue = await ask('Valor da autenticação: ');
        
        if (authType === '1') {
            webhookConfig.actions[0].headers['Authorization'] = `Bearer ${authValue}`;
        } else {
            webhookConfig.actions[0].headers['X-API-Key'] = authValue;
        }
    }
    
    try {
        console.log(colors.yellow + '\n⚙️  Configurando webhook...' + colors.reset);
        const response = await apiRequest('PUT', `/dialogs/${dialogId}`, webhookConfig);
        
        console.log(colors.green + '\n✅ Webhook configurado com sucesso!' + colors.reset);
        console.log('URL: ' + colors.blue + webhookUrl + colors.reset);
        console.log('Método: ' + method.toUpperCase());
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao configurar webhook: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Testar webhook
async function testWebhook() {
    console.log(colors.cyan + '\n🧪 Teste de Webhook\n' + colors.reset);
    
    const webhookUrl = await ask('URL do webhook a testar: ');
    const testData = {
        annotation: {
            data: {
                tarefa: 'Teste de webhook',
                prioridade: 'Alta',
                responsavel: 'Teste CLI',
                descricao: 'Esta é uma tarefa de teste enviada pelo CLI'
            }
        },
        contact: {
            number: '5511999999999',
            name: 'Teste CLI User'
        },
        message: {
            text: 'Mensagem de teste',
            type: 'text',
            timestamp: new Date().toISOString()
        }
    };
    
    console.log(colors.yellow + '\n📤 Enviando teste para webhook...' + colors.reset);
    console.log(colors.dim + 'Dados: ' + JSON.stringify(testData, null, 2) + colors.reset);
    
    try {
        const url = new URL(webhookUrl);
        const options = {
            hostname: url.hostname,
            port: url.port || (url.protocol === 'https:' ? 443 : 80),
            path: url.pathname,
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(JSON.stringify(testData))
            }
        };

        const req = (url.protocol === 'https:' ? https : require('http')).request(options, (res) => {
            let responseData = '';
            res.on('data', chunk => responseData += chunk);
            res.on('end', () => {
                if (res.statusCode === 200 || res.statusCode === 201) {
                    console.log(colors.green + '\n✅ Teste enviado com sucesso!' + colors.reset);
                    console.log('Status:', res.statusCode);
                    console.log('Resposta:', responseData);
                } else {
                    console.log(colors.yellow + '\n⚠️ Webhook retornou status: ' + res.statusCode + colors.reset);
                    console.log('Resposta:', responseData);
                }
            });
        });

        req.on('error', error => {
            console.log(colors.red + `\n❌ Erro ao testar webhook: ${error.message}` + colors.reset);
        });

        req.write(JSON.stringify(testData));
        req.end();
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao testar webhook: ${error.message}` + colors.reset);
    }
}

// Executar diálogo para um contato
async function executeDialog() {
    console.log(colors.cyan + '\n📱 Executar Diálogo\n' + colors.reset);
    
    const dialogId = await ask('ID do diálogo: ');
    const phoneNumber = await ask('Número do WhatsApp (com código do país, ex: 5511999999999): ');
    
    console.log('\nVariáveis do diálogo (opcional):');
    const addVars = await ask('Adicionar variáveis? (s/n): ');
    
    const executeData = {
        chat_number: phoneNumber,
        dialog_id: dialogId,
        key: CONFIG.API_KEY,
        account_id: CONFIG.ACCOUNT_ID,
        phone_id: CONFIG.PHONE_IDS.main
    };
    
    if (addVars.toLowerCase() === 's') {
        const variables = {};
        
        let addMore = true;
        while (addMore) {
            const varName = await ask('Nome da variável: ');
            const varValue = await ask('Valor da variável: ');
            variables[varName] = varValue;
            
            const continueAdding = await ask('Adicionar mais variáveis? (s/n): ');
            addMore = continueAdding.toLowerCase() === 's';
        }
        
        executeData.variables = variables;
    }
    
    try {
        console.log(colors.yellow + '\n🚀 Executando diálogo...' + colors.reset);
        const response = await apiRequest('POST', '/dialog_execute', executeData);
        
        console.log(colors.green + '\n✅ Diálogo executado com sucesso!' + colors.reset);
        console.log('ID da execução:', response.execution_id || response.id);
        console.log('Status:', response.status || 'Iniciado');
        
    } catch (error) {
        console.log(colors.red + `❌ Erro ao executar diálogo: ${error.message}` + colors.reset);
        if (error.data) {
            console.log(colors.dim + JSON.stringify(error.data, null, 2) + colors.reset);
        }
    }
}

// Mostrar configurações da API
async function showApiConfig() {
    console.log(colors.cyan + '\n⚙️  Configurações Atuais da API\n' + colors.reset);
    console.log('URL Base:', colors.blue + CONFIG.API_BASE_URL + colors.reset);
    console.log('API Key:', colors.yellow + CONFIG.API_KEY.substring(0, 20) + '...' + colors.reset);
    console.log('Account ID:', CONFIG.ACCOUNT_ID);
    console.log('Phone IDs:');
    Object.entries(CONFIG.PHONE_IDS).forEach(([key, value]) => {
        console.log(`  - ${key}: ${value}`);
    });
    
    const change = await ask('\nDeseja alterar as configurações? (s/n): ');
    
    if (change.toLowerCase() === 's') {
        console.log('\nDeixe em branco para manter o valor atual:');
        
        const newApiUrl = await ask(`Nova URL Base [${CONFIG.API_BASE_URL}]: `);
        if (newApiUrl) CONFIG.API_BASE_URL = newApiUrl;
        
        const newApiKey = await ask('Nova API Key: ');
        if (newApiKey) CONFIG.API_KEY = newApiKey;
        
        const newAccountId = await ask(`Novo Account ID [${CONFIG.ACCOUNT_ID}]: `);
        if (newAccountId) CONFIG.ACCOUNT_ID = newAccountId;
        
        console.log(colors.green + '\n✅ Configurações atualizadas!' + colors.reset);
    }
}

// Loop principal do programa
async function main() {
    console.log(colors.bright + colors.cyan + '\n🤖 Bem-vindo ao ChatGuru Dialog CLI!' + colors.reset);
    console.log(colors.dim + 'Gerencie seus diálogos de forma simples e eficiente\n' + colors.reset);
    
    let running = true;
    
    while (running) {
        showMainMenu();
        const choice = await ask(colors.cyan + 'Opção: ' + colors.reset);
        
        switch (choice) {
            case '1':
                await listDialogs();
                break;
            case '2':
                await viewDialogDetails();
                break;
            case '3':
                await createDialog();
                break;
            case '4':
                await updateDialog();
                break;
            case '5':
                await deleteDialog();
                break;
            case '6':
                await configureWebhook();
                break;
            case '7':
                await testWebhook();
                break;
            case '8':
                await executeDialog();
                break;
            case '9':
                await showApiConfig();
                break;
            case '0':
                running = false;
                break;
            default:
                console.log(colors.red + '\n❌ Opção inválida!' + colors.reset);
        }
        
        if (running && choice !== '0') {
            await ask(colors.dim + '\nPressione ENTER para continuar...' + colors.reset);
        }
    }
    
    console.log(colors.green + '\n👋 Até logo!' + colors.reset);
    rl.close();
}

// Tratamento de erros globais
process.on('uncaughtException', (error) => {
    console.error(colors.red + '\n❌ Erro inesperado:' + colors.reset, error.message);
    console.log(colors.dim + 'Stack:', error.stack + colors.reset);
    rl.close();
    process.exit(1);
});

// Iniciar aplicação
main().catch(error => {
    console.error(colors.red + '\n❌ Erro fatal:' + colors.reset, error);
    rl.close();
    process.exit(1);
});