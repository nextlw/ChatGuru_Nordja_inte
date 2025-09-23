#!/usr/bin/env node

/**
 * Teste mínimo e direto
 */

const axios = require('axios');

async function testWebhook() {
    const url = 'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru';
    
    const payload = {
        event_type: 'annotation.added',
        id: `test_${Date.now()}`,
        timestamp: new Date().toISOString(),
        data: {
            chat_number: '5585989530473',
            customer_name: 'Teste Cliente',
            annotation: 'Teste de integração via script minimal',
            created_at: new Date().toISOString()
        }
    };
    
    console.log('=== TESTANDO WEBHOOK MIDDLEWARE ===');
    console.log('URL:', url);
    console.log('Payload:', JSON.stringify(payload, null, 2));
    console.log('');
    
    try {
        const response = await axios.post(url, payload, {
            headers: {
                'Content-Type': 'application/json'
            },
            timeout: 10000
        });
        
        console.log('✅ SUCESSO!');
        console.log('Status:', response.status);
        console.log('Resposta:', JSON.stringify(response.data, null, 2));
        
        if (response.data.clickup_task_id) {
            console.log('\n🎯 Task criada/atualizada no ClickUp!');
            console.log('ID da Task:', response.data.clickup_task_id);
            console.log('Ação:', response.data.clickup_task_action);
        }
        
    } catch (error) {
        console.log('❌ ERRO!');
        if (error.response) {
            console.log('Status:', error.response.status);
            console.log('Erro:', error.response.data);
        } else {
            console.log('Erro:', error.message);
        }
    }
}

// Executar teste
testWebhook();