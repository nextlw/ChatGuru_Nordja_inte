#!/usr/bin/env node

/**
 * Script para investigar por que TESTE_API cria anotações automaticamente
 * mesmo sem ter ação de anotação configurada
 */

const https = require('https');

const colors = {
    reset: '\x1b[0m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
    magenta: '\x1b[35m'
};

console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
console.log(`${colors.cyan}║   INVESTIGAÇÃO: ANOTAÇÕES AUTOMÁTICAS          ║${colors.reset}`);
console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
console.log('');

// Função para fazer requisição
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
                'User-Agent': 'ChatGuru-Investigation/1.0'
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

// HIPÓTESE 1: O ChatGuru tem configuração automática para criar anotações
async function testeHipotese1() {
    console.log(`${colors.magenta}═══ HIPÓTESE 1: Configuração Automática no ChatGuru ═══${colors.reset}`);
    console.log('');
    console.log('Teoria: O ChatGuru pode ter uma configuração que automaticamente');
    console.log('cria anotações quando um diálogo é executado com sucesso.');
    console.log('');
    
    // Simular execução de TESTE_API
    const payload1 = {
        event_type: 'dialog.executed',
        dialog_id: 'TESTE_API',
        timestamp: new Date().toISOString(),
        chat_number: '5585989530473',
        variables: {
            tarefa: 'Teste automático',
            prioridade: 'Normal'
        }
    };
    
    console.log('Testando TESTE_API:');
    console.log('Payload:', JSON.stringify(payload1, null, 2));
    
    try {
        // Enviar para seu middleware para ver o que acontece
        const result = await makeRequest(
            'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru',
            'POST',
            payload1
        );
        
        console.log(`Status: ${result.status}`);
        console.log('Resposta:', result.data);
    } catch (error) {
        console.log(`${colors.red}Erro: ${error.message}${colors.reset}`);
    }
    
    console.log('');
}

// HIPÓTESE 2: O webhook buzzlightear está configurando a anotação
async function testeHipotese2() {
    console.log(`${colors.magenta}═══ HIPÓTESE 2: Webhook Buzzlightear Configura Anotação ═══${colors.reset}`);
    console.log('');
    console.log('Teoria: O webhook buzzlightear pode estar retornando instruções');
    console.log('para o ChatGuru criar anotações, mesmo sem código explícito.');
    console.log('');
    
    // Testar diretamente o buzzlightear
    const payloads = [
        {
            name: 'TESTE_API',
            data: {
                event_type: 'dialog.executed',
                dialog_id: 'TESTE_API',
                chat_number: '5585989530473',
                variables: { teste: 'TESTE_API' }
            }
        },
        {
            name: 'nova_api',
            data: {
                event_type: 'dialog.executed',
                dialog_id: 'nova_api',
                chat_number: '5585989530473',
                variables: { teste: 'nova_api' }
            }
        }
    ];
    
    for (const test of payloads) {
        console.log(`Testando ${test.name}:`);
        
        try {
            const result = await makeRequest(
                'https://buzzlightear.rj.r.appspot.com/webhook',
                'POST',
                test.data
            );
            
            console.log(`Status: ${result.status}`);
            console.log('Resposta:', result.data);
            
            // Analisar se há diferenças na resposta
            if (result.data.includes('annotation') || result.data.includes('anotacao')) {
                console.log(`${colors.yellow}⚠️ Resposta contém referência a anotação!${colors.reset}`);
            }
        } catch (error) {
            console.log(`${colors.red}Erro: ${error.message}${colors.reset}`);
        }
        
        console.log('');
    }
}

// HIPÓTESE 3: Comportamento diferente baseado no nome do diálogo
async function testeHipotese3() {
    console.log(`${colors.magenta}═══ HIPÓTESE 3: Nome do Diálogo Importa ═══${colors.reset}`);
    console.log('');
    console.log('Teoria: O ChatGuru pode ter comportamento especial para');
    console.log('diálogos com nome "TESTE_API" ou que contenham "API".');
    console.log('');
    
    const dialogNames = [
        'TESTE_API',
        'nova_api',
        'teste_outro',
        'api_test',
        'dialog_normal'
    ];
    
    for (const dialogName of dialogNames) {
        console.log(`Testando diálogo: ${dialogName}`);
        
        const payload = {
            event_type: 'dialog.executed',
            dialog_id: dialogName,
            timestamp: new Date().toISOString(),
            chat_number: '5585989530473',
            variables: {
                teste: `Teste do diálogo ${dialogName}`
            }
        };
        
        try {
            const result = await makeRequest(
                'https://buzzlightear.rj.r.appspot.com/webhook',
                'POST',
                payload
            );
            
            console.log(`  Status: ${result.status}`);
            console.log(`  Resposta: ${result.data}`);
        } catch (error) {
            console.log(`  ${colors.red}Erro: ${error.message}${colors.reset}`);
        }
    }
    
    console.log('');
}

// HIPÓTESE 4: Análise de eventos reais
async function testeHipotese4() {
    console.log(`${colors.magenta}═══ HIPÓTESE 4: Análise de Eventos Reais ═══${colors.reset}`);
    console.log('');
    console.log('Analisando o comportamento real quando o usuário interage:');
    console.log('');
    
    console.log(`${colors.yellow}Observações do comportamento real:${colors.reset}`);
    console.log('');
    console.log('1. Quando usuário envia: "Pode buscar documentações de API?"');
    console.log('   - ChatGuru identifica DOIS diálogos: nova_api E TESTE_API');
    console.log('   - Ambos são executados');
    console.log('   - Aparece anotação formatada');
    console.log('');
    console.log('2. A anotação tem formato específico:');
    console.log('   "Tarefa: Atividade Identificada: [texto]"');
    console.log('   "Tipo de Atividade: Específica"');
    console.log('   "Categoria: Atividades de Pesquisa em geral"');
    console.log('');
    
    console.log(`${colors.green}💡 DESCOBERTA PROVÁVEL:${colors.reset}`);
    console.log('');
    console.log('O formato da anotação sugere que:');
    console.log('1. Existe um sistema de NLP/IA no ChatGuru que:');
    console.log('   - Analisa a mensagem do usuário');
    console.log('   - Identifica automaticamente tarefas/atividades');
    console.log('   - Cria anotações estruturadas');
    console.log('');
    console.log('2. TESTE_API pode ser um diálogo especial que:');
    console.log('   - Foi criado para testar essa funcionalidade');
    console.log('   - Tem configuração interna para gerar anotações');
    console.log('   - Pode ter sido configurado via interface web (não via API)');
    console.log('');
}

// Executar todas as hipóteses
async function runInvestigation() {
    await testeHipotese1();
    await testeHipotese2();
    await testeHipotese3();
    await testeHipotese4();
    
    console.log(`${colors.cyan}╔════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.cyan}║   CONCLUSÃO DA INVESTIGAÇÃO                    ║${colors.reset}`);
    console.log(`${colors.cyan}╚════════════════════════════════════════════════╝${colors.reset}`);
    console.log('');
    
    console.log(`${colors.green}✅ EXPLICAÇÃO MAIS PROVÁVEL:${colors.reset}`);
    console.log('');
    console.log('O ChatGuru tem um sistema de IA/NLP que:');
    console.log('');
    console.log('1. Analisa mensagens e identifica intenções de tarefa');
    console.log('2. Quando detecta uma tarefa, aciona MÚLTIPLOS diálogos:');
    console.log('   - nova_api: Para processar via webhook');
    console.log('   - TESTE_API: Diálogo de teste/debug que cria anotação');
    console.log('');
    console.log('3. A anotação é criada pelo TESTE_API porque:');
    console.log('   - Foi configurado na interface web do ChatGuru');
    console.log('   - Tem template de anotação pré-definido');
    console.log('   - Usa variáveis extraídas pelo NLP');
    console.log('');
    console.log(`${colors.yellow}📋 RECOMENDAÇÕES:${colors.reset}`);
    console.log('');
    console.log('1. Acesse o painel do ChatGuru e verifique:');
    console.log('   - Configuração do diálogo TESTE_API');
    console.log('   - Se há ação de anotação configurada nele');
    console.log('   - Se há configuração de NLP/IA ativa');
    console.log('');
    console.log('2. Para nova_api criar anotações também:');
    console.log('   - Adicione ação de anotação no diálogo');
    console.log('   - Use o mesmo template do TESTE_API');
    console.log('   - Ou desative TESTE_API se for apenas teste');
    console.log('');
    console.log('3. Considere:');
    console.log('   - TESTE_API pode ser um diálogo de sistema/debug');
    console.log('   - Pode ter sido criado automaticamente pelo ChatGuru');
    console.log('   - Verifique se pode ser desativado sem prejuízo');
}

// Executar investigação
runInvestigation().catch(console.error);