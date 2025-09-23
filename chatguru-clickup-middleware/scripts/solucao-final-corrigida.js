const axios = require('axios');

// CONFIGURAÇÃO CORRIGIDA
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID_CORRETO = '901300373349'; // Lista: 📋 Pagamentos para Clientes

console.log('🔧 APLICANDO CORREÇÃO FINAL - Status válido...\n');

async function testarSolucaoFinal() {
  
  // Payload simplificado sem status específico
  const payload = {
    name: '✅ SUCESSO - Integração ChatGuru ClickUp FUNCIONANDO',
    description: `
🎉 PROBLEMA OAUTH_019 TOTALMENTE RESOLVIDO!

**CAUSA RAIZ IDENTIFICADA:**
- ❌ Formato incorreto: "Bearer" + token
- ❌ List ID incorreto: Usando Workspace ID como List ID

**SOLUÇÕES APLICADAS:**
- ✅ Formato correto: Authorization sem "Bearer"  
- ✅ List ID correto: Lista real do workspace

**Configuração Final:**
- Token: ${TOKEN.substring(0, 20)}...
- List ID: ${LIST_ID_CORRETO}
- Header: 'Authorization': TOKEN (SEM Bearer)

**Data/Hora:** ${new Date().toLocaleString('pt-BR')}
**Status:** Debug concluído com sucesso
    `,
    tags: ['chatguru-bot', 'integracao-sucesso', 'oauth-resolvido'],
    priority: 2
  };
  
  try {
    const response = await axios.post(`https://api.clickup.com/api/v2/list/${LIST_ID_CORRETO}/task`, payload, {
      headers: {
        'Authorization': TOKEN, // SEM Bearer - FORMATO CORRETO!
        'Content-Type': 'application/json'
      },
      timeout: 10000
    });
    
    console.log('🎉 INTEGRAÇÃO CHATGURU-CLICKUP FUNCIONANDO 100%!');
    console.log('===========================================');
    console.log(`📋 Tarefa criada: ${response.data.id}`);
    console.log(`🔗 URL: ${response.data.url}`);
    console.log(`📝 Nome: ${response.data.name}\n`);
    
    console.log('📊 RESUMO DO DEBUG:');
    console.log('===================');
    console.log('❌ Erro original: OAUTH_019 - "Oauth token not found"');
    console.log('🔍 Causa #1: Formato "Bearer" incorreto para tokens pessoais');
    console.log('🔍 Causa #2: List ID era na verdade Workspace ID');
    console.log('✅ Solução: Token SEM Bearer + List ID correto');
    console.log('✅ Status: RESOLVIDO\n');
    
    console.log('🎯 IMPLEMENTAR NO CÓDIGO:');
    console.log('========================');
    console.log('// Arquivo: middleware/server.js - Linha 112');
    console.log('headers: {');
    console.log(`  'Authorization': '${TOKEN}', // SEM Bearer!`);
    console.log('  "Content-Type": "application/json"');
    console.log('}');
    console.log('');
    console.log('// Variáveis de ambiente (.env)');
    console.log(`CLICKUP_API_TOKEN=${TOKEN}`);
    console.log(`CLICKUP_LIST_ID=${LIST_ID_CORRETO} # Lista válida`);
    console.log('');
    
    return true;
    
  } catch (error) {
    console.log('❌ Erro restante:');
    console.log(`Status: ${error.response?.status}`);
    console.log(`Erro: ${error.response?.data?.err || error.message}`);
    console.log(`Código: ${error.response?.data?.ECODE}\n`);
    return false;
  }
}

testarSolucaoFinal();
