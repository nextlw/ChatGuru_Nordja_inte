const axios = require('axios');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '901300373349'; // Lista de teste da Anne

// IDs hardcoded no worker.rs
const HARDCODED_IDS = {
  categoria: 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a',
  subCategoria: '5333c095-eb40-4a5a-b0c2-76bfba4b1094',
  estrelas: '83afcb8c-2866-498f-9c62-8ea9666b104b'
};

async function verificarFieldIDs() {
  console.log('🔍 Verificando se IDs dos campos mudaram após renomeação...\n');

  try {
    const response = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      { headers: { 'Authorization': TOKEN } }
    );

    const fields = response.data.fields;

    console.log('📋 Campos encontrados na lista:\n');

    const categoriaNova = fields.find(f => f.name === 'Categoria_nova');
    const subCategoriaNova = fields.find(f => f.name === 'SubCategoria_nova');
    const estrelas = fields.find(f => f.name === 'Estrelas');

    let hasChanges = false;

    // Verificar Categoria_nova
    if (categoriaNova) {
      console.log(`✅ Campo "Categoria_nova":`);
      console.log(`   ID atual: ${categoriaNova.id}`);
      console.log(`   ID hardcoded: ${HARDCODED_IDS.categoria}`);
      if (categoriaNova.id !== HARDCODED_IDS.categoria) {
        console.log(`   ⚠️  ATENÇÃO: ID MUDOU!\n`);
        hasChanges = true;
      } else {
        console.log(`   ✅ ID está correto\n`);
      }
    } else {
      console.log(`❌ Campo "Categoria_nova" não encontrado\n`);
    }

    // Verificar SubCategoria_nova
    if (subCategoriaNova) {
      console.log(`✅ Campo "SubCategoria_nova":`);
      console.log(`   ID atual: ${subCategoriaNova.id}`);
      console.log(`   ID hardcoded: ${HARDCODED_IDS.subCategoria}`);
      if (subCategoriaNova.id !== HARDCODED_IDS.subCategoria) {
        console.log(`   ⚠️  ATENÇÃO: ID MUDOU!\n`);
        hasChanges = true;
      } else {
        console.log(`   ✅ ID está correto\n`);
      }
    } else {
      console.log(`❌ Campo "SubCategoria_nova" não encontrado\n`);
    }

    // Verificar Estrelas
    if (estrelas) {
      console.log(`✅ Campo "Estrelas":`);
      console.log(`   ID atual: ${estrelas.id}`);
      console.log(`   ID hardcoded: ${HARDCODED_IDS.estrelas}`);
      if (estrelas.id !== HARDCODED_IDS.estrelas) {
        console.log(`   ⚠️  ATENÇÃO: ID MUDOU!\n`);
        hasChanges = true;
      } else {
        console.log(`   ✅ ID está correto\n`);
      }
    } else {
      console.log(`❌ Campo "Estrelas" não encontrado\n`);
    }

    // Resumo
    console.log('\n📊 RESUMO:');
    if (hasChanges) {
      console.log('❌ OS IDs MUDARAM! É necessário atualizar worker.rs com os novos IDs.');
      console.log('\n🔧 Atualizações necessárias em worker.rs:');
      if (categoriaNova && categoriaNova.id !== HARDCODED_IDS.categoria) {
        console.log(`   Categoria: "${HARDCODED_IDS.categoria}" → "${categoriaNova.id}"`);
      }
      if (subCategoriaNova && subCategoriaNova.id !== HARDCODED_IDS.subCategoria) {
        console.log(`   SubCategoria: "${HARDCODED_IDS.subCategoria}" → "${subCategoriaNova.id}"`);
      }
      if (estrelas && estrelas.id !== HARDCODED_IDS.estrelas) {
        console.log(`   Estrelas: "${HARDCODED_IDS.estrelas}" → "${estrelas.id}"`);
      }
    } else {
      console.log('✅ Todos os IDs permanecem iguais. A renomeação NÃO afetou a funcionalidade do código.');
    }

  } catch (error) {
    console.error('❌ Erro ao verificar fields:');
    console.error(`Status: ${error.response?.status}`);
    console.error(`Erro: ${error.response?.data?.err || error.message}`);
  }
}

verificarFieldIDs();
