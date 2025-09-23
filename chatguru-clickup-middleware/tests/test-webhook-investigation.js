#!/usr/bin/env node

/**
 * Script para investigar e comparar webhooks
 * TESTE_API → https://buzzlightear.rj.r.appspot.com/webhook (Google Cloud existente)
 * nova_api → Deve apontar para nossa nova integração
 */

const axios = require('axios');

// URLs dos webhooks
const BUZZLIGHTEAR_WEBHOOK = 'https://buzzlightear.rj.r.appspot.com/webhook';
const NOSSA_INTEGRACAO_LOCAL = 'http://localhost:8080/webhooks/chatguru';
const NOSSA_INTEGRACAO_GCP = 'https://chatguru-clickup-middleware-625297003049.us-central1.run.app/webhooks/chatguru';

// Payload de teste baseado no formato ChatGuru
const testPayload = {
    event_type: 'annotation.added',
    timestamp: new Date().toISOString(),
    account_id: '625584ce6fdcb7bda7d94aa8',
    phone_id: '6537de23b6d5b0bb0b80421a',
    contact: {
        phone: '5511999999999',
        name: 'Cliente Teste'
    },
    annotation: {
        title: 'Pedido de Teste',
        description: 'Cliente: João Silva\nProduto: Pizza Grande\nEndereço: Rua Teste, 123',
        tags: ['pedido', 'teste'],
        data: {
            cliente: 'João Silva',
            produto: 'Pizza Grande',
            endereco: 'Rua Teste, 123'
        }
    }
};

async function testWebhook(name, url) {
    console.log(`\n${'='.repeat(60)}`);
    console.log(`TESTANDO: ${name}`);
    console.log(`URL: ${url}`);
    console.log('='.repeat(60));
    
    try {
        console.log('\n📤 Enviando webhook de teste...');
        console.log('Payload:', JSON.stringify(testPayload, null, 2));
        
        const startTime = Date.now();
        const response = await axios.post(url, testPayload, {
            headers: {
                'Content-Type': 'application/json',
                'User-Agent': 'ChatGuru-Webhook-Test'
            },
            timeout: 15000,
            validateStatus: () => true // Aceita qualquer status
        });
        const responseTime = Date.now() - startTime;
        
        console.log(`\n📥 Resposta recebida em ${responseTime}ms`);
        console.log(`Status: ${response.status} ${response.statusText}`);
        console.log('Headers:', response.headers);
        
        if (response.data) {
            console.log('Body da resposta:', 
                typeof response.data === 'object' 
                    ? JSON.stringify(response.data, null, 2) 
                    : response.data
            );
        }
        
        return {
            success: response.status >= 200 && response.status < 300,
            status: response.status,
            responseTime,
            data: response.data,
            headers: response.headers
        };
        
    } catch (error) {
        console.log('\n❌ Erro ao testar webhook:');
        console.log(`   ${error.message}`);
        
        if (error.code === 'ECONNREFUSED') {
            console.log('   → Serviço não está rodando ou porta fechada');
        } else if (error.code === 'ETIMEDOUT') {
            console.log('   → Timeout - serviço demorou muito para responder');
        } else if (error.response) {
            console.log(`   → Status: ${error.response.status}`);
            console.log(`   → Data: ${JSON.stringify(error.response.data)}`);
        }
        
        return {
            success: false,
            error: error.message,
            code: error.code
        };
    }
}

async function checkGCPEndpoints() {
    console.log('\n' + '='.repeat(60));
    console.log('VERIFICANDO ENDPOINTS NO GOOGLE CLOUD');
    console.log('='.repeat(60));
    
    // Tenta descobrir mais sobre o buzzlightear
    console.log('\n1. Testando buzzlightear (TESTE_API atual):');
    try {
        // Tenta GET para ver se retorna algo
        const getResponse = await axios.get(BUZZLIGHTEAR_WEBHOOK.replace('/webhook', ''), {
            timeout: 5000,
            validateStatus: () => true
        });
        console.log(`   GET /: Status ${getResponse.status}`);
        if (getResponse.data) {
            console.log('   Resposta:', getResponse.data);
        }
    } catch (error) {
        console.log('   GET /: ' + error.message);
    }
    
    // Tenta endpoint de health
    try {
        const healthResponse = await axios.get(BUZZLIGHTEAR_WEBHOOK.replace('/webhook', '/health'), {
            timeout: 5000,
            validateStatus: () => true
        });
        console.log(`   GET /health: Status ${healthResponse.status}`);
    } catch (error) {
        console.log('   GET /health: ' + error.message);
    }
}

async function main() {
    console.log('\n╔════════════════════════════════════════════════╗');
    console.log('║     INVESTIGAÇÃO DE WEBHOOKS CHATGURU         ║');
    console.log('╚════════════════════════════════════════════════╝');
    
    console.log('\n📋 CONTEXTO:');
    console.log('   TESTE_API usa: https://buzzlightear.rj.r.appspot.com/webhook');
    console.log('   nova_api deveria usar nossa nova integração');
    console.log('\n🎯 OBJETIVO:');
    console.log('   Entender o que o buzzlightear faz e como configurar nova_api');
    
    // Primeiro vamos investigar os endpoints GCP
    await checkGCPEndpoints();
    
    // Testa cada webhook
    const results = {};
    
    results.buzzlightear = await testWebhook(
        'Buzzlightear (usado pelo TESTE_API)', 
        BUZZLIGHTEAR_WEBHOOK
    );
    
    results.nossaLocal = await testWebhook(
        'Nossa Integração (Local)', 
        NOSSA_INTEGRACAO_LOCAL
    );
    
    results.nossaGCP = await testWebhook(
        'Nossa Integração (GCP)', 
        NOSSA_INTEGRACAO_GCP
    );
    
    // Análise comparativa
    console.log('\n' + '='.repeat(60));
    console.log('📊 ANÁLISE COMPARATIVA');
    console.log('='.repeat(60));
    
    console.log('\n1. STATUS DOS WEBHOOKS:');
    console.log(`   Buzzlightear:     ${results.buzzlightear.success ? '✅ OK' : '❌ FALHOU'} (Status: ${results.buzzlightear.status || 'N/A'})`);
    console.log(`   Nossa Local:      ${results.nossaLocal.success ? '✅ OK' : '❌ FALHOU'} (Status: ${results.nossaLocal.status || 'N/A'})`);
    console.log(`   Nossa GCP:        ${results.nossaGCP.success ? '✅ OK' : '❌ FALHOU'} (Status: ${results.nossaGCP.status || 'N/A'})`);
    
    console.log('\n2. COMPORTAMENTO OBSERVADO:');
    
    if (results.buzzlightear.success) {
        console.log('\n   Buzzlightear (TESTE_API):');
        console.log('   - Aceita webhooks do ChatGuru');
        console.log('   - Está rodando no Google App Engine');
        console.log(`   - Tempo de resposta: ${results.buzzlightear.responseTime}ms`);
        if (results.buzzlightear.data) {
            console.log('   - Retorno:', JSON.stringify(results.buzzlightear.data).substring(0, 100) + '...');
        }
    }
    
    if (results.nossaLocal.success || results.nossaGCP.success) {
        console.log('\n   Nossa Integração:');
        if (results.nossaLocal.success) {
            console.log('   - ✅ Local funcionando');
        }
        if (results.nossaGCP.success) {
            console.log('   - ✅ GCP funcionando');
        }
    }
    
    console.log('\n' + '='.repeat(60));
    console.log('🔍 PRÓXIMOS PASSOS');
    console.log('='.repeat(60));
    
    console.log('\n1. VERIFICAR NO PAINEL DO CHATGURU:');
    console.log('   - Acessar editor de diálogos');
    console.log('   - Abrir diálogo "nova_api"');
    console.log('   - Procurar ação de webhook');
    console.log('   - Verificar/alterar URL configurada');
    
    console.log('\n2. URL RECOMENDADA PARA nova_api:');
    if (results.nossaGCP.success) {
        console.log(`   ✅ Usar: ${NOSSA_INTEGRACAO_GCP}`);
        console.log('   (Nossa integração está funcionando no GCP)');
    } else if (results.nossaLocal.success) {
        console.log(`   ⚠️  Para testes locais: ${NOSSA_INTEGRACAO_LOCAL}`);
        console.log('   (Mas precisa configurar ngrok ou deploy no GCP para produção)');
    } else {
        console.log('   ❌ Nossa integração não está acessível');
        console.log('   Verificar se o serviço está rodando corretamente');
    }
    
    console.log('\n3. DIFERENÇA PRINCIPAL:');
    console.log('   - TESTE_API → buzzlightear (integração antiga/diferente)');
    console.log('   - nova_api → deve usar nossa nova integração com ClickUp');
    
    console.log('\n✅ Investigação concluída!\n');
}

// Executa o script
main().catch(error => {
    console.error('\n❌ Erro fatal:', error.message);
    process.exit(1);
});