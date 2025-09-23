const axios = require('axios');

async function testSimpleWebhook() {
    console.log('\n📌 Teste Simples de Webhook\n');
    
    // Payload exatamente como ChatGuru envia (baseado no que vemos no buzzlightear)
    const payload = {
        "event_type": "annotation.added",
        "timestamp": new Date().toISOString(),
        "account_id": "625584ce6fdcb7bda7d94aa8",
        "phone_id": "6537de23b6d5b0bb0b80421a",
        "contact": {
            "phone": "5511999999999",
            "name": "Cliente Teste"
        },
        "annotation": {
            "title": "Pedido de Teste",
            "description": "Cliente: João Silva\nProduto: Pizza Grande\nEndereço: Rua Teste, 123",
            "tags": ["pedido", "teste"],
            "data": {
                "cliente": "João Silva",
                "produto": "Pizza Grande",
                "endereco": "Rua Teste, 123"
            }
        }
    };

    // Testar contra produção que está funcionando
    const url = 'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru';
    
    console.log('📤 Enviando para produção GCP...');
    console.log('URL:', url);
    console.log('Payload:', JSON.stringify(payload, null, 2));

    try {
        const response = await axios.post(url, payload);
        console.log('✅ Sucesso!');
        console.log('Status:', response.status);
        console.log('Resposta:', response.data);
    } catch (error) {
        if (error.response) {
            console.log('❌ Erro:', error.response.status);
            console.log('Mensagem:', error.response.data);
        } else {
            console.log('❌ Erro de conexão:', error.message);
        }
    }
}

testSimpleWebhook();