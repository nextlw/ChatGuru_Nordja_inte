#!/usr/bin/env node

const https = require('https');

// Configurações
const API_KEY = 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f';

function makeRequest(url) {
    return new Promise((resolve, reject) => {
        const options = {
            headers: {
                'APIKey': API_KEY,
                'Content-Type': 'application/json'
            }
        };

        https.get(url, options, (res) => {
            let data = '';

            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                if (res.statusCode >= 200 && res.statusCode < 300) {
                    try {
                        resolve(JSON.parse(data));
                    } catch (err) {
                        resolve(data);
                    }
                } else {
                    reject(new Error(`HTTP ${res.statusCode}: ${data}`));
                }
            });
        }).on('error', reject);
    });
}

async function testAPIs() {
    console.log('🚀 Testando APIs do ChatGuru\n');
    console.log('=' .repeat(60));
    
    // Lista de servidores para testar
    const servers = [
        's12',
        's15',
        's10',
        's11',
        's13',
        's14'
    ];
    
    console.log('📝 Testando diferentes servidores ChatGuru:\n');
    
    for (const server of servers) {
        const url = `https://${server}.chatguru.app/api/v1/dialogs`;
        console.log(`\nTestando ${server}:`);
        console.log(`URL: ${url}`);
        
        try {
            const startTime = Date.now();
            const response = await makeRequest(url);
            const elapsed = Date.now() - startTime;
            
            if (Array.isArray(response)) {
                console.log(`✅ Sucesso! Retornou array com ${response.length} diálogos (${elapsed}ms)`);
                
                if (response.length > 0) {
                    console.log('   Primeiro diálogo:');
                    const first = response[0];
                    console.log(`   - ID: ${first.id || first._id || 'sem id'}`);
                    console.log(`   - Nome: ${first.name || first.title || 'sem nome'}`);
                    
                    // Procurar por nova_api
                    const novaApi = response.find(d => 
                        d.name === 'nova_api' || 
                        d.name === 'Nova API' ||
                        (d.description && d.description.toLowerCase().includes('nova'))
                    );
                    
                    if (novaApi) {
                        console.log('\n   🎯 Diálogo "nova_api" encontrado!');
                        console.log(`      ID: ${novaApi.id}`);
                        console.log(`      Nome: ${novaApi.name}`);
                        if (novaApi.webhook || novaApi.webhookUrl || novaApi.url) {
                            console.log(`      Webhook: ${novaApi.webhook || novaApi.webhookUrl || novaApi.url}`);
                        }
                    }
                }
            } else if (typeof response === 'object' && response.dialogs) {
                console.log(`✅ Sucesso! Retornou objeto com propriedade dialogs: ${response.dialogs.length} itens (${elapsed}ms)`);
            } else {
                console.log(`⚠️ Resposta inesperada (${elapsed}ms):`, JSON.stringify(response).substring(0, 100));
            }
            
        } catch (err) {
            console.log(`❌ Erro: ${err.message.substring(0, 100)}`);
        }
    }
    
    console.log('\n' + '=' .repeat(60));
    console.log('\n📊 Resumo dos testes concluído!\n');
}

// Executar testes
testAPIs().catch(err => {
    console.error('Erro fatal:', err);
    process.exit(1);
});