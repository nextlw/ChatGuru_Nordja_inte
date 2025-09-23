const axios = require('axios');

// CONFIGURAÇÃO CORRIGIDA
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID_CORRETO = '901300373349'; // Lista: 📋 Pagamentos para Clientes

console.log('🔧 APLICANDO CORREÇÕES IDENTIFICADAS...\n');

async function testarSolucaoCompleta() {
  console.log('✅ 1. FORMATO CORRETO: SEM Bearer');
  console.log('✅ 2. LIST ID CORRETO: Lista real do workspace\n');
  
  // Testar criação de tarefa com configuração corrigida
  console.log('🧪 TESTE FINAL: Criar tarefa com configuração correta\n');
  
  const payload = {
    name: 'TESTE - Integração ChatGuru ClickUp RESOLVIDA',
    description: `
✅ PROBLEMA RESOLVIDO!

**Erro OAUTH_019 - CAUSA RAIZ:**
- Formato: Token estava usando "Bearer" incorretamente
- List ID: Estava usando Workspace ID (9013037641) como List ID

**SOLUÇÕES APLICADAS:**
- ✅ Formato correto: Authorization: ${TOKEN.substring(0, 20)}...
- ✅ List ID correto: ${LIST_ID_CORRETO} (Lista válida)

**Data/Hora:** ${new Date().toLocaleString('pt-BR')}
**Origem:** Debug sistemático ChatGuru-ClickUp
    `,
    tags: ['chatguru-bot', 'debug-resolvido', 'integracao'],
    status: 'Open',
    priority: 2
  };
  
  try {
    const response = await axios.post(`https://api.clickup.com/api/v2/list/${LIST_ID_CORRETO}/task`, payload, {
      headers: {
        'Authorization': TOKEN, // SEM Bearer!
        'Content-Type': 'application/json'
      },
      timeout: 10000
    });
    
    console.log('🎉 SUCESSO TOTAL! Integração funcionando!');
    console.log(`📋 Tarefa ID: ${response.data.id}`);
    console.log(`🔗 URL: ${response.data.url}`);
    console.log(`📝 Nome: ${response.data.name}\n`);
    
    console.log('🎯 CONFIGURAÇÃO FINAL PARA IMPLEMENTAR:');
    console.log('========================================');
    console.log(`CLICKUP_API_TOKEN=${TOKEN}`);
    console.log(`CLICKUP_LIST_ID=${LIST_ID_CORRETO}`);
    console.log('');
    console.log('HEADER CORRETO:');
    console.log("'Authorization': TOKEN // SEM Bearer!");
    console.log('');
    
    return true;
    
  } catch (error) {
    console.log('❌ Ainda há problemas:');
    console.log(`Status: ${error.response?.status}`);
    console.log(`Erro: ${error.response?.data?.err || error.message}`);
    console.log(`Código: ${error.response?.data?.ECODE}\n`);
    return false;
  }
}

testarSolucaoCompleta();
