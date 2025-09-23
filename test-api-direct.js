#!/usr/bin/env node

const https = require('https');

// Configurações
const CONFIG = {
    API_BASE_URL: 'https://s12.chatguru.app/api/v1',
    API_KEY: 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f'
};

function makeRequest(path, method = 'GET') {
    return new Promise((resolve, reject) => {
        const url = new URL(`${CONFIG.API_BASE_URL}${path}`);
        
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

        console.log(`\n📡 Fazendo requisição:`, {
            url: url.toString(),
            method: method,
            headers: options.headers
        });

        const req = https.request(options, (res) => {
            let data = '';

            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                console.log(`\n📊 Status: ${res.statusCode}`);
                console.log(`📋 Headers:`, res.headers);
                
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    try {
                        const parsed = JSON.parse(data);
                        console.log(`\n✅ Resposta parseada com sucesso!`);
                        console.log(`📦 Tipo de resposta:`, Array.isArray(parsed) ? 'Array' : typeof parsed);
                        if (Array.isArray(parsed)) {
                            console.log(`📌 Total de itens:`, parsed.length);
                        }
                        resolve(parsed);
                    } catch (err) {
                        console.log(`\n⚠️ Resposta não é JSON válido:`, data.substring(0, 200));
                        resolve(data);
                    }
                } else {
                    console.log(`\n❌ Erro HTTP ${res.statusCode}:`, data.substring(0, 500));
                    reject(new Error(`HTTP ${res.statusCode}: ${data}`));
                }
            });
        });

        req.on('error', (err) => {
            console.log(`\n❌ Erro de conexão:`, err.message);
            reject(err);
        });

        req.end();
    });
}

async function testApi() {
    console.log('🚀 Iniciando teste direto da API ChatGuru\n');
    console.log('📍 Configuração:');
    console.log(`   - URL Base: ${CONFIG.API_BASE_URL}`);
    console.log(`   - API Key: ${CONFIG.API_KEY.substring(0, 10)}...`);
    
    try {
        // Teste 1: Listar diálogos
        console.log('\n' + '='.repeat(50));
        console.log('📝 TESTE 1: Listar Diálogos');
        console.log('='.repeat(50));
        
        const dialogs = await makeRequest('/dialogs');
        
        if (Array.isArray(dialogs) && dialogs.length > 0) {
            console.log(`\n✅ Encontrados ${dialogs.length} diálogos!`);
            
            // Mostrar primeiro diálogo como exemplo
            console.log('\n📌 Exemplo do primeiro diálogo:');
            const firstDialog = dialogs[0];
            console.log(JSON.stringify(firstDialog, null, 2));
            
            // Listar todos os IDs e nomes
            console.log('\n📋 Lista de todos os diálogos:');
            dialogs.forEach((dialog, index) => {
                console.log(`   ${index + 1}. ID: ${dialog.id || dialog._id || 'sem id'} - Nome: ${dialog.name || dialog.title || 'sem nome'}`);
            });
        } else if (Array.isArray(dialogs)) {
            console.log('\n⚠️ Array vazio retornado - nenhum diálogo encontrado');
        } else {
            console.log('\n❓ Resposta inesperada:');
            console.log(JSON.stringify(dialogs, null, 2));
        }
        
        // Teste 2: Tentar outros endpoints
        console.log('\n' + '='.repeat(50));
        console.log('📝 TESTE 2: Testar endpoint /dialog (singular)');
        console.log('='.repeat(50));
        
        try {
            const dialog = await makeRequest('/dialog');
            console.log('Resposta /dialog:', JSON.stringify(dialog, null, 2).substring(0, 500));
        } catch (err) {
            console.log('Endpoint /dialog não disponível:', err.message);
        }
        
        // Teste 3: Tentar com query params
        console.log('\n' + '='.repeat(50));
        console.log('📝 TESTE 3: Listar com query params');
        console.log('='.repeat(50));
        
        try {
            const dialogsWithLimit = await makeRequest('/dialogs?limit=10');
            console.log('Resposta com limit:', Array.isArray(dialogsWithLimit) ? `Array com ${dialogsWithLimit.length} itens` : typeof dialogsWithLimit);
        } catch (err) {
            console.log('Erro com query params:', err.message);
        }
        
    } catch (err) {
        console.log('\n❌ Erro durante os testes:', err.message);
        if (err.stack) {
            console.log('\nStack trace:', err.stack);
        }
    }
    
    console.log('\n✨ Teste concluído!');
}

// Executar teste
testApi().catch(err => {
    console.error('Erro fatal:', err);
    process.exit(1);
});