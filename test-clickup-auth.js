const axios = require('axios');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '9013037641';

async function testarAutenticacao() {
  console.log('🔍 Testando formatos de autenticação ClickUp...\n');
  
  // Teste 1: SEM Bearer (documentação oficial)
  console.log('🧪 TESTE 1: Formato SEM Bearer');
  try {
    const response1 = await axios.get('https://api.clickup.com/api/v2/user', {
      headers: { 'Authorization': TOKEN },
      timeout: 5000
    });
    console.log('✅ SUCESSO! Token válido sem Bearer');
    console.log(`👤 Usuário: ${response1.data.user.username}\n`);
    return TOKEN; // Retorna formato correto
  } catch (error) {
    console.log('❌ FALHOU sem Bearer');
    console.log(`Erro: ${error.response?.data?.err || error.message}\n`);
  }
  
  // Teste 2: COM Bearer (implementação atual)
  console.log('🧪 TESTE 2: Formato COM Bearer');
  try {
    const response2 = await axios.get('https://api.clickup.com/api/v2/user', {
      headers: { 'Authorization': `Bearer ${TOKEN}` },
      timeout: 5000
    });
    console.log('✅ SUCESSO! Token válido com Bearer');
    console.log(`👤 Usuário: ${response2.data.user.username}\n`);
    return `Bearer ${TOKEN}`; // Retorna formato correto
  } catch (error) {
    console.log('❌ FALHOU com Bearer');
    console.log(`Erro: ${error.response?.data?.err || error.message}\n`);
  }
  
  return null;
}

async function testarListID(authHeader) {
  console.log('🧪 TESTE 3: Validar List ID');
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/list/${LIST_ID}`, {
      headers: { 'Authorization': authHeader },
      timeout: 5000
    });
    console.log('✅ List ID válido!');
    console.log(`📋 Lista: ${response.data.name}`);
    console.log(`🏢 Space: ${response.data.space.name}\n`);
    return true;
  } catch (error) {
    console.log('❌ List ID inválido');
    console.log(`Erro: ${error.response?.data?.err || error.message}\n`);
    return false;
  }
}

// Executar testes
(async () => {
  const authHeader = await testarAutenticacao();
  if (authHeader) {
    await testarListID(authHeader);
  } else {
    console.log('🚨 DIAGNÓSTICO: Token inválido ou problema de conectividade');
  }
})();
