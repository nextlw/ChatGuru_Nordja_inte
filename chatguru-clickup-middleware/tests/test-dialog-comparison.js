#!/usr/bin/env node

/**
 * Script para testar e comparar os diálogos TESTE_API e nova_api
 * Identifica diferenças no comportamento e ajuda a diagnosticar problemas
 */

const axios = require('axios');

// Configurações do ChatGuru - SUBSTITUA COM SUAS CREDENCIAIS
const CHATGURU_CONFIG = {
    API_KEY: process.env.CHATGURU_API_KEY || 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
    ACCOUNT_ID: process.env.CHATGURU_ACCOUNT_ID || '625584ce6fdcb7bda7d94aa8',
    PHONE_ID: process.env.CHATGURU_PHONE_ID || '6537de23b6d5b0bb0b80421a',
    BASE_URL: 'https://s15.chatguru.app/api/v1'
};

// Configuração do webhook local
const WEBHOOK_URL = process.env.WEBHOOK_URL || 'http://localhost:8080/webhooks/chatguru';

// Número de WhatsApp para teste
const TEST_PHONE = process.env.TEST_PHONE || '5585989530473';

class DialogTester {
    constructor() {
        this.results = {
            TESTE_API: { success: false, response: null, error: null },
            nova_api: { success: false, response: null, error: null }
        };
    }

    async testDialog(dialogId, testData) {
        console.log(`\n${'='.repeat(50)}`.cyan);
        console.log(`Testando Diálogo: ${dialogId}`.yellow);
        console.log('='.repeat(50).cyan);

        try {
            const payload = {
                chat_number: TEST_PHONE,
                dialog_id: dialogId,
                key: CHATGURU_CONFIG.API_KEY,
                account_id: CHATGURU_CONFIG.ACCOUNT_ID,
                phone_id: CHATGURU_CONFIG.PHONE_ID,
                variables: testData
            };

            console.log('📤 Enviando requisição...'.gray);
            console.log('Payload:', JSON.stringify(payload, null, 2).gray);

            const response = await axios.post(
                `${CHATGURU_CONFIG.BASE_URL}/dialog_execute`,
                payload,
                {
                    headers: { 'Content-Type': 'application/json' },
                    timeout: 10000
                }
            );

            this.results[dialogId] = {
                success: true,
                response: response.data,
                error: null
            };

            console.log('✅ Resposta recebida:'.green);
            console.log(JSON.stringify(response.data, null, 2).green);

            return response.data;

        } catch (error) {
            this.results[dialogId] = {
                success: false,
                response: null,
                error: error.message
            };

            console.log('❌ Erro ao executar diálogo:'.red);
            console.log(error.message.red);
            
            if (error.response) {
                console.log('Detalhes do erro:'.red);
                console.log(JSON.stringify(error.response.data, null, 2).red);
            }

            return null;
        }
    }

    async testAddNote(dialogId, noteContent) {
        console.log(`\n📝 Testando adicionar anotação para ${dialogId}...`.cyan);

        try {
            const response = await axios.post(
                `${CHATGURU_CONFIG.BASE_URL}/note_add`,
                {
                    chat_number: TEST_PHONE,
                    note: noteContent,
                    key: CHATGURU_CONFIG.API_KEY,
                    account_id: CHATGURU_CONFIG.ACCOUNT_ID,
                    phone_id: CHATGURU_CONFIG.PHONE_ID
                },
                {
                    headers: { 'Content-Type': 'application/json' },
                    timeout: 10000
                }
            );

            console.log('✅ Anotação adicionada com sucesso'.green);
            return response.data;

        } catch (error) {
            console.log('❌ Erro ao adicionar anotação:'.red, error.message);
            return null;
        }
    }

    async testWebhookDirectly(taskData) {
        console.log('\n🔧 Testando webhook diretamente...'.cyan);

        try {
            const webhookPayload = {
                event_type: 'task_created',
                id: `test_${Date.now()}`,
                timestamp: new Date().toISOString(),
                data: {
                    chat_number: TEST_PHONE,
                    message: taskData.message || 'Teste direto do webhook',
                    custom_fields: {
                        tarefa: taskData.tarefa || 'Tarefa de teste',
                        prioridade: taskData.prioridade || 'Normal',
                        responsavel: taskData.responsavel || 'Sistema'
                    }
                }
            };

            console.log('Payload do webhook:', JSON.stringify(webhookPayload, null, 2).gray);

            const response = await axios.post(
                WEBHOOK_URL,
                webhookPayload,
                {
                    headers: { 'Content-Type': 'application/json' },
                    timeout: 5000
                }
            );

            console.log('✅ Webhook processado com sucesso'.green);
            console.log('Resposta:', JSON.stringify(response.data, null, 2).green);
            return response.data;

        } catch (error) {
            console.log('❌ Erro ao testar webhook:'.red, error.message);
            return null;
        }
    }

    async compareResults() {
        console.log('\n' + '='.repeat(60));
        console.log('COMPARAÇÃO DOS RESULTADOS');
        console.log('='.repeat(60));

        const testeApi = this.results.TESTE_API;
        const novaApi = this.results.nova_api;

        // Comparar sucesso
        console.log('\n📊 Status de Execução:');
        console.log(`  TESTE_API: ${testeApi.success ? '✅ Sucesso'.green : '❌ Falhou'.red}`);
        console.log(`  nova_api:  ${novaApi.success ? '✅ Sucesso'.green : '❌ Falhou'.red}`);

        // Analisar diferenças
        if (testeApi.success && !novaApi.success) {
            console.log('\n⚠️  PROBLEMA IDENTIFICADO:');
            console.log('  O diálogo TESTE_API funciona, mas nova_api falha.');
            console.log('  Possíveis causas:');
            console.log('  1. nova_api não existe ou está desativado');
            console.log('  2. nova_api não tem permissões adequadas');
            console.log('  3. Erro na configuração do diálogo nova_api');
        } else if (testeApi.success && novaApi.success) {
            console.log('\n✅ Ambos os diálogos executaram com sucesso!');
            console.log('  Verifique se as ações de webhook estão configuradas corretamente em ambos.');
        }

        // Comparar respostas
        if (testeApi.response && novaApi.response) {
            console.log('\n🔍 Diferenças nas respostas:');
            const keys = new Set([
                ...Object.keys(testeApi.response || {}),
                ...Object.keys(novaApi.response || {})
            ]);

            for (const key of keys) {
                const val1 = JSON.stringify(testeApi.response[key]);
                const val2 = JSON.stringify(novaApi.response[key]);
                if (val1 !== val2) {
                    console.log(`  ${key}:`);
                    console.log(`    TESTE_API: ${val1}`);
                    console.log(`    nova_api:  ${val2}`);
                }
            }
        }
    }

    async runFullTest() {
        console.log('\n🚀 INICIANDO TESTE COMPLETO DOS DIÁLOGOS');
        console.log('='.repeat(60));

        // Dados de teste
        const testData = {
            tarefa: `Teste comparativo - ${new Date().toLocaleString('pt-BR')}`,
            prioridade: 'Alta',
            responsavel: 'Sistema de Teste',
            descricao: 'Testando diferenças entre TESTE_API e nova_api'
        };

        // 1. Testar TESTE_API
        await this.testDialog('TESTE_API', testData);
        await this.sleep(2000); // Aguardar 2 segundos

        // 2. Testar nova_api
        await this.testDialog('nova_api', testData);
        await this.sleep(2000);

        // 3. Testar webhook diretamente
        await this.testWebhookDirectly(testData);
        await this.sleep(1000);

        // 4. Testar adição de nota
        await this.testAddNote('TESTE_API', `Nota de teste para TESTE_API - ${Date.now()}`);
        await this.sleep(1000);
        await this.testAddNote('nova_api', `Nota de teste para nova_api - ${Date.now()}`);

        // 5. Comparar resultados
        await this.compareResults();

        // 6. Recomendações
        this.showRecommendations();
    }

    showRecommendations() {
        console.log('\n' + '='.repeat(60));
        console.log('💡 RECOMENDAÇÕES');
        console.log('='.repeat(60));

        console.log('\n1. VERIFICAR NO PAINEL DO CHATGURU:');
        console.log('   - Entre no editor de diálogos');
        console.log('   - Compare as configurações de TESTE_API e nova_api');
        console.log('   - Verifique se nova_api tem ação de webhook configurada');

        console.log('\n2. CONFIGURAÇÃO DO WEBHOOK NO DIÁLOGO:');
        console.log('   - URL: ' + WEBHOOK_URL);
        console.log('   - Método: POST');
        console.log('   - Content-Type: application/json');
        console.log('   - Body: JSON com estrutura correta');

        console.log('\n3. VERIFICAR LOGS DO MIDDLEWARE:');
        console.log('   - Local: tail -f logs/*.log');
        console.log('   - GCP: gcloud run logs read --service chatguru-clickup-middleware --tail');

        console.log('\n4. SE nova_api NÃO FUNCIONA:');
        console.log('   - Copie as ações do TESTE_API');
        console.log('   - Cole no nova_api');
        console.log('   - Ajuste as variáveis se necessário');
        console.log('   - Salve e publique o diálogo');
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Executar teste
async function main() {
    console.clear();
    console.log('╔════════════════════════════════════════════════╗');
    console.log('║   TESTE COMPARATIVO DE DIÁLOGOS CHATGURU      ║');
    console.log('╚════════════════════════════════════════════════╝');

    // Verificar configurações
    if (CHATGURU_CONFIG.API_KEY === 'sua_api_key_aqui') {
        console.log('\n❌ ERRO: Configure suas credenciais do ChatGuru!');
        console.log('   Edite este arquivo ou defina as variáveis de ambiente:');
        console.log('   - CHATGURU_API_KEY');
        console.log('   - CHATGURU_ACCOUNT_ID');
        console.log('   - CHATGURU_PHONE_ID');
        console.log('   - TEST_PHONE (número WhatsApp para teste)');
        process.exit(1);
    }

    const tester = new DialogTester();
    
    try {
        await tester.runFullTest();
    } catch (error) {
        console.error('\n❌ Erro durante os testes:', error.message);
    }

    console.log('\n✅ Teste concluído!');
}

// Instalar dependências se necessário
const checkDependencies = () => {
    try {
        require('axios');
        require('colors');
    } catch (e) {
        console.log('📦 Instalando dependências...');
        require('child_process').execSync('npm install axios colors', { stdio: 'inherit' });
    }
};

checkDependencies();
main().catch(console.error);