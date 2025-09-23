const fetch = require('node-fetch');

// Configurações
const API_KEY = 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f';
const headers = {
  'APIKey': API_KEY,
  'Content-Type': 'application/json'
};

// URLs possíveis para o webhook
const WEBHOOK_URLS = {
  local: 'http://localhost:8080/webhooks/chatguru',
  cloudRun: 'https://chatguru-clickup-middleware-xxxxx-uc.a.run.app/webhooks/chatguru',
  buzzlightear: 'https://buzzlightear-ek3kpvifpq-ue.a.run.app/webhooks/chatguru'
};

async function findNovaApiDialog() {
  console.log('🔍 Buscando diálogo nova_api...\n');
  
  try {
    const response = await fetch('https://s12.chatguru.app/api/v1/dialogs', { headers });
    
    if (!response.ok) {
      console.error('❌ Erro ao buscar diálogos:', response.status);
      return null;
    }
    
    const dialogs = await response.json();
    
    // Procurar pelo diálogo nova_api
    const novaApi = dialogs.find(d => 
      d.name === 'nova_api' || 
      d.name === 'Nova API' ||
      (d.description && d.description.toLowerCase().includes('nova'))
    );
    
    if (!novaApi) {
      console.log('❌ Diálogo nova_api não encontrado!');
      console.log('\nDiálogos disponíveis:');
      dialogs.forEach(d => {
        console.log(`  - ${d.name} (ID: ${d.id})`);
      });
      return null;
    }
    
    console.log('✅ Diálogo encontrado!');
    console.log(`   Nome: ${novaApi.name}`);
    console.log(`   ID: ${novaApi.id}`);
    console.log(`   Status: ${novaApi.active ? 'Ativo' : 'Inativo'}`);
    
    return novaApi;
  } catch (error) {
    console.error('❌ Erro:', error.message);
    return null;
  }
}

async function getDialogDetails(dialogId) {
  console.log('\n📋 Obtendo detalhes do diálogo...\n');
  
  try {
    const response = await fetch(`https://s12.chatguru.app/api/v1/dialogs/${dialogId}`, { headers });
    
    if (!response.ok) {
      console.error('❌ Erro ao obter detalhes:', response.status);
      return null;
    }
    
    const details = await response.json();
    
    // Verificar webhook atual
    const currentWebhook = details.webhook || details.webhookUrl || null;
    
    if (currentWebhook) {
      console.log('🔗 Webhook atual:', currentWebhook);
      
      if (currentWebhook.includes('buzzlightear')) {
        console.log('   ⚠️  Está apontando para o serviço ANTIGO (buzzlightear)');
      } else if (currentWebhook.includes('chatguru-clickup-middleware')) {
        console.log('   ✅ Já está configurado para o serviço NOVO!');
      }
    } else {
      console.log('❌ Webhook NÃO configurado');
    }
    
    // Verificar ações
    if (details.actions && details.actions.length > 0) {
      console.log('\n🎯 Ações configuradas:');
      details.actions.forEach((action, i) => {
        console.log(`   ${i + 1}. ${action.type || action.name || 'Ação'}`);
        if (action.webhook || action.url) {
          console.log(`      URL: ${action.webhook || action.url}`);
        }
      });
    }
    
    return details;
  } catch (error) {
    console.error('❌ Erro:', error.message);
    return null;
  }
}

async function updateDialogWebhook(dialogId, newWebhookUrl) {
  console.log('\n🔧 Atualizando webhook do diálogo...\n');
  console.log('Nova URL:', newWebhookUrl);
  
  try {
    // Preparar o body da requisição
    const updateData = {
      webhook: newWebhookUrl,
      webhookUrl: newWebhookUrl,
      actions: [
        {
          type: 'webhook',
          name: 'Enviar para ClickUp',
          url: newWebhookUrl,
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: {
            annotation: {
              data: '{{annotation_data}}'
            },
            contact: {
              number: '{{chat_number}}',
              name: '{{contact_name}}'
            }
          }
        }
      ]
    };
    
    const response = await fetch(`https://s12.chatguru.app/api/v1/dialogs/${dialogId}`, {
      method: 'PUT',
      headers,
      body: JSON.stringify(updateData)
    });
    
    if (!response.ok) {
      const errorText = await response.text();
      console.error('❌ Erro ao atualizar:', response.status);
      console.error('Resposta:', errorText);
      return false;
    }
    
    const result = await response.json();
    console.log('✅ Webhook atualizado com sucesso!');
    
    return true;
  } catch (error) {
    console.error('❌ Erro:', error.message);
    return false;
  }
}

async function main() {
  console.log('═'.repeat(60));
  console.log('🚀 ATUALIZADOR DE WEBHOOK - DIÁLOGO NOVA_API');
  console.log('═'.repeat(60));
  
  // 1. Encontrar o diálogo
  const dialog = await findNovaApiDialog();
  if (!dialog) {
    console.log('\n❌ Não foi possível continuar sem encontrar o diálogo.');
    return;
  }
  
  // 2. Obter detalhes atuais
  const details = await getDialogDetails(dialog.id);
  if (!details) {
    console.log('\n❌ Não foi possível obter detalhes do diálogo.');
    return;
  }
  
  // 3. Verificar se precisa atualizar
  const currentWebhook = details.webhook || details.webhookUrl || null;
  
  console.log('\n' + '═'.repeat(60));
  console.log('📝 AÇÃO NECESSÁRIA');
  console.log('═'.repeat(60));
  
  if (!currentWebhook || currentWebhook.includes('buzzlightear')) {
    console.log('\n⚠️  O webhook precisa ser atualizado!');
    console.log('\nOpções de URL:');
    console.log('1. Cloud Run (RECOMENDADO após deploy):');
    console.log('   https://chatguru-clickup-middleware-xxxxx-uc.a.run.app/webhooks/chatguru');
    console.log('\n2. Local (para testes):');
    console.log('   http://localhost:8080/webhooks/chatguru');
    console.log('\n3. Serviço antigo (buzzlightear):');
    console.log('   https://buzzlightear-ek3kpvifpq-ue.a.run.app/webhooks/chatguru');
    
    console.log('\n💡 Para atualizar automaticamente, descomente a linha abaixo no código:');
    console.log('   // await updateDialogWebhook(dialog.id, WEBHOOK_URLS.cloudRun);');
    
    // DESCOMENTE A LINHA ABAIXO APÓS OBTER A URL CORRETA DO CLOUD RUN
    // await updateDialogWebhook(dialog.id, 'https://SUA-URL-CLOUD-RUN/webhooks/chatguru');
    
  } else if (currentWebhook.includes('chatguru-clickup-middleware')) {
    console.log('\n✅ Webhook já está configurado corretamente!');
    console.log('   URL atual:', currentWebhook);
  }
  
  console.log('\n' + '═'.repeat(60));
  console.log('📌 PRÓXIMOS PASSOS');
  console.log('═'.repeat(60));
  console.log('\n1. Aguarde o deploy do Cloud Run terminar');
  console.log('2. Obtenha a URL do serviço deployado');
  console.log('3. Execute este script novamente com a URL correta');
  console.log('4. Teste enviando uma mensagem no WhatsApp');
  
  console.log('\n✨ Script finalizado!');
}

// Executar
main().catch(error => {
  console.error('\n❌ Erro fatal:', error);
  process.exit(1);
});