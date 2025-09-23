#!/usr/bin/env node

/**
 * Script para testar o webhook buzzlightear e entender o fluxo de resposta
 */

const https = require('https');

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
console.log(`${colors.cyan}║   TESTE WEBHOOK BUZZLIGHTEAR                   ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Função para fazer requisição HTTPS
function makeRequest(url, method = 'POST', payload = null) {
    return new Promise((resolve, reject) => {
        const urlObj = new URL(url);
        
        const options = {
            hostname: urlObj.hostname,
            port: 443,
            path: urlObj.pathname,
            method: method,
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'ChatGuru-Webhook-Test/1.0'
            },
            timeout: 10000
        };
        
        if (payload) {
            const data = JSON.stringify(payload);
            options.headers['Content-Length'] = Buffer.byteLength(data);
        }
        
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

// Teste 1: Webhook Buzzlightear direto
async function testBuzzlightearWebhook() {
    console.log(`${colors.blue}═══ Teste 1: Webhook Buzzlightear Direto ═══${colors.reset}`);
    console.log('URL: https://buzzlightear.rj.r.appspot.com/webhook');
    console.log('');
    
    const payload = {
        event_type: 'dialog.executed',
        dialog_id: 'nova_api',
        timestamp: new Date().toISOString(),
        chat_number: '5585989530473',
        variables: {
            tarefa: 'Buscar documentações de API',
            tipo: 'Pesquisa',
            categoria: 'Atividades de Pesquisa em geral'
        },
        account: {
            id: '625584ce6fdcb7bda7d94aa8',
            name: 'Nordja'
        },
        phone: {
            id: '6537de23b6d5b0bb0b80421a',
            number: '+558512345678'
        }
    };
    
    console.log('Payload enviado:');
    console.log(JSON.stringify(payload, null, 2));
    console.log('');
    
    try {
        const result = await makeRequest('https://buzzlightear.rj.r.appspot.com/webhook', 'POST', payload);
        
        console.log(`Status: ${result.status}`);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Webhook aceito com sucesso!${colors.reset}`);
        } else {
            console.log(`${colors.yellow}⚠️ Status: ${result.status}${colors.reset}`);
        }
        
        console.log('');
        console.log('Headers de resposta:');
        console.log(JSON.stringify(result.headers, null, 2));
        
        console.log('');
        console.log('Resposta do servidor:');
        if (result.data) {
            try {
                const parsed = JSON.parse(result.data);
                console.log(JSON.stringify(parsed, null, 2));
                
                // Verificar se há instrução de resposta
                if (parsed.response || parsed.message || parsed.annotation) {
                    console.log('');
                    console.log(`${colors.cyan}📝 Possível resposta para o ChatGuru:${colors.reset}`);
                    console.log(parsed.response || parsed.message || parsed.annotation);
                }
            } catch {
                console.log(result.data);
            }
        } else {
            console.log('(Sem corpo de resposta)');
        }
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Teste 2: Simular evento de anotação
async function testAnnotationEvent() {
    console.log(`${colors.blue}═══ Teste 2: Evento de Anotação ═══${colors.reset}`);
    console.log('URL: https://buzzlightear.rj.r.appspot.com/webhook');
    console.log('');
    
    const payload = {
        event_type: 'annotation.added',
        timestamp: new Date().toISOString(),
        chat: {
            number: '5585989530473',
            contact: {
                name: 'Cliente Teste',
                number: '5585989530473'
            }
        },
        annotation: {
            id: `ann_${Date.now()}`,
            text: `Tarefa: Atividade Identificada: Buscar documentações de API
Tipo de Atividade: Específica
Categoria: Atividades de Pesquisa em geral
Sub Categoria: []
Subtarefas: (se aplicável)
- Subtarefa 1
- Subtarefa 2`,
            created_at: new Date().toISOString()
        }
    };
    
    console.log('Payload de anotação:');
    console.log(JSON.stringify(payload, null, 2));
    console.log('');
    
    try {
        const result = await makeRequest('https://buzzlightear.rj.r.appspot.com/webhook', 'POST', payload);
        
        console.log(`Status: ${result.status}`);
        if (result.status === 200 || result.status === 201) {
            console.log(`${colors.green}✅ Anotação processada!${colors.reset}`);
        } else {
            console.log(`${colors.yellow}⚠️ Status: ${result.status}${colors.reset}`);
        }
        
        console.log('');
        console.log('Resposta:');
        if (result.data) {
            try {
                const parsed = JSON.parse(result.data);
                console.log(JSON.stringify(parsed, null, 2));
            } catch {
                console.log(result.data);
            }
        }
    } catch (error) {
        console.log(`${colors.red}❌ Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// Executar testes
async function runTests() {
    await testBuzzlightearWebhook();
    await testAnnotationEvent();
    
    console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║   ANÁLISE DO FLUXO                             ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
    console.log('');
    
    console.log(`${colors.yellow}📋 Fluxo identificado:${colors.reset}`);
    console.log('');
    console.log('1. Usuário envia mensagem no WhatsApp');
    console.log('2. ChatGuru processa e identifica o diálogo "nova_api"');
    console.log('3. ChatGuru executa o diálogo e:');
    console.log('   a) Envia webhook para buzzlightear.rj.r.appspot.com/webhook');
    console.log('   b) Cria uma anotação no chat com os dados processados');
    console.log('');
    console.log(`${colors.green}✅ A anotação "Chatbot anotou:" é criada pelo PRÓPRIO ChatGuru${colors.reset}`);
    console.log('   - Não é o webhook que retorna a anotação');
    console.log('   - O ChatGuru cria a anotação após executar o diálogo');
    console.log('   - O webhook buzzlightear pode processar os dados mas não controla a anotação');
    console.log('');
    console.log(`${colors.blue}💡 Webhook do middleware (seu):${colors.reset}`);
    console.log('   - URL: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru');
    console.log('   - Recebe eventos de anotação e cria tarefas no ClickUp');
    console.log('   - Funciona independente do webhook buzzlightear');
}

// Executar
runTests().catch(console.error);