const fetch = require('node-fetch');

const API_KEY = 'c5be7758-d3e2-4f07-b039-7e0bdd1e0d7f';
const headers = {
  'APIKey': API_KEY,
  'Content-Type': 'application/json'
};

async function checkDialogs() {
  try {
    console.log('🔍 Buscando todos os diálogos...');
    console.log('');
    
    // Buscar todos os diálogos
    const dialogsUrl = 'https://s12.chatguru.app/api/v1/dialogs';
    const dialogsResponse = await fetch(dialogsUrl, { headers });
    
    if (!dialogsResponse.ok) {
      console.log('❌ Erro ao buscar diálogos:', dialogsResponse.status, dialogsResponse.statusText);
      return;
    }
    
    const dialogs = await dialogsResponse.json();
    
    console.log('📊 Total de diálogos encontrados:', dialogs.length);
    console.log('');
    
    // Listar todos os diálogos
    console.log('📋 Lista de Diálogos:');
    console.log('─'.repeat(60));
    dialogs.forEach((dialog, index) => {
      console.log(`${index + 1}. ${dialog.name || 'Sem nome'}`);
      console.log(`   ID: ${dialog.id}`);
      console.log(`   Status: ${dialog.active ? '✅ Ativo' : '❌ Inativo'}`);
      console.log(`   Descrição: ${dialog.description || 'Sem descrição'}`);
      console.log('');
    });
    
    // Procurar especificamente pelo nova_api
    const novaApiVariations = ['nova_api', 'Nova API', 'nova_API', 'NOVA_API'];
    let novaApi = null;
    
    for (const variation of novaApiVariations) {
      novaApi = dialogs.find(d => 
        d.name === variation || 
        (d.description && d.description.includes(variation))
      );
      if (novaApi) break;
    }
    
    if (!novaApi) {
      console.log('─'.repeat(60));
      console.log('⚠️  DIÁLOGO "nova_api" NÃO ENCONTRADO!');
      console.log('');
      console.log('Possíveis razões:');
      console.log('1. O diálogo ainda não foi criado');
      console.log('2. O diálogo tem outro nome');
      console.log('3. O diálogo está em outro workspace');
      
      // Procurar diálogos que possam ser relacionados
      const possibleDialogs = dialogs.filter(d => 
        d.name && (
          d.name.toLowerCase().includes('api') ||
          d.name.toLowerCase().includes('nova') ||
          d.name.toLowerCase().includes('clickup') ||
          d.name.toLowerCase().includes('task')
        )
      );
      
      if (possibleDialogs.length > 0) {
        console.log('');
        console.log('🔎 Diálogos que podem estar relacionados:');
        possibleDialogs.forEach(d => {
          console.log(`   - ${d.name} (ID: ${d.id})`);
        });
      }
      
      return;
    }
    
    console.log('─'.repeat(60));
    console.log('✅ DIÁLOGO "nova_api" ENCONTRADO!');
    console.log('');
    console.log('📌 Detalhes do Diálogo:');
    console.log(`   Nome: ${novaApi.name}`);
    console.log(`   ID: ${novaApi.id}`);
    console.log(`   Status: ${novaApi.active ? '✅ Ativo' : '❌ Inativo'}`);
    console.log(`   Descrição: ${novaApi.description || 'Sem descrição'}`);
    
    // Buscar detalhes completos
    const detailsUrl = `https://s12.chatguru.app/api/v1/dialogs/${novaApi.id}`;
    const detailsResponse = await fetch(detailsUrl, { headers });
    
    if (detailsResponse.ok) {
      const details = await detailsResponse.json();
      
      // Verificar webhook
      console.log('');
      console.log('🔗 Configuração de Webhook:');
      const webhookUrl = details.webhookUrl || details.webhook || details.url || null;
      
      if (webhookUrl) {
        console.log(`   URL Atual: ${webhookUrl}`);
        
        if (webhookUrl.includes('buzzlightear')) {
          console.log('   ⚠️  ATENÇÃO: Webhook está apontando para buzzlightear!');
          console.log('   📝 Precisa atualizar para o novo serviço Cloud Run');
        } else if (webhookUrl.includes('chatguru-clickup-middleware')) {
          console.log('   ✅ Webhook já está configurado para o novo serviço!');
        } else {
          console.log('   ℹ️  Webhook configurado para outro serviço');
        }
      } else {
        console.log('   ❌ WEBHOOK NÃO CONFIGURADO!');
        console.log('   📝 É necessário configurar o webhook para receber eventos');
      }
      
      // Verificar actions
      if (details.actions && details.actions.length > 0) {
        console.log('');
        console.log('🎯 Ações Configuradas:');
        details.actions.forEach((action, i) => {
          console.log(`   ${i + 1}. ${action.name || action.type || 'Ação'}`);
          if (action.webhook) {
            console.log(`      Webhook: ${action.webhook}`);
          }
        });
      }
      
      // Verificar annotations
      if (details.annotations && details.annotations.length > 0) {
        console.log('');
        console.log('📝 Anotações Configuradas:');
        details.annotations.forEach((ann, i) => {
          console.log(`   ${i + 1}. ${ann.name || ann.key || 'Anotação'}`);
        });
      }
    }
    
    console.log('');
    console.log('═'.repeat(60));
    console.log('📌 RESUMO E PRÓXIMOS PASSOS:');
    console.log('═'.repeat(60));
    
    if (novaApi) {
      console.log('');
      console.log('✅ Diálogo nova_api existe');
      console.log(`   Status: ${novaApi.active ? 'Ativo' : 'Inativo (precisa ativar)'}`);
      
      const webhookUrl = novaApi.webhookUrl || novaApi.webhook || novaApi.url || null;
      if (webhookUrl) {
        if (webhookUrl.includes('buzzlightear')) {
          console.log('');
          console.log('⚠️  Webhook está apontando para o serviço antigo (buzzlightear)');
          console.log('');
          console.log('📝 AÇÕES NECESSÁRIAS:');
          console.log('   1. Aguardar o deploy do Cloud Run terminar');
          console.log('   2. Obter a URL do novo serviço');
          console.log('   3. Atualizar o webhook do diálogo nova_api');
          console.log('   4. Testar com uma mensagem no WhatsApp');
        } else if (webhookUrl.includes('chatguru-clickup-middleware')) {
          console.log('');
          console.log('✅ Webhook já está configurado corretamente!');
          console.log('');
          console.log('📝 PRÓXIMO PASSO:');
          console.log('   - Testar enviando uma mensagem no WhatsApp');
        }
      } else {
        console.log('');
        console.log('❌ Webhook NÃO está configurado');
        console.log('');
        console.log('📝 AÇÕES NECESSÁRIAS:');
        console.log('   1. Aguardar o deploy do Cloud Run terminar');
        console.log('   2. Obter a URL do novo serviço');
        console.log('   3. Configurar o webhook no diálogo nova_api');
        console.log('   4. Testar com uma mensagem no WhatsApp');
      }
    } else {
      console.log('');
      console.log('❌ Diálogo nova_api NÃO existe');
      console.log('');
      console.log('📝 AÇÕES NECESSÁRIAS:');
      console.log('   1. Criar o diálogo nova_api no ChatGuru');
      console.log('   2. Configurar as anotações necessárias');
      console.log('   3. Aguardar o deploy do Cloud Run terminar');
      console.log('   4. Configurar o webhook com a URL do novo serviço');
      console.log('   5. Testar com uma mensagem no WhatsApp');
    }
    
    console.log('');
    console.log('═'.repeat(60));
    
  } catch (error) {
    console.error('❌ Erro ao executar verificação:', error.message);
    if (error.stack) {
      console.error('Stack:', error.stack);
    }
  }
}

// Executar a verificação
console.log('🚀 Iniciando verificação do diálogo nova_api...');
console.log('');
checkDialogs().then(() => {
  console.log('');
  console.log('✨ Verificação concluída!');
}).catch(error => {
  console.log('');
  console.error('❌ Erro fatal:', error);
});