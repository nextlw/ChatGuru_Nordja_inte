const axios = require('axios');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Opções para o novo campo dropdown "Categoria_nova"
const CATEGORIAS_OPTIONS = [
  { name: 'Agendamentos', color: '#FF6900' },
  { name: 'Compras', color: '#FCB900' },
  { name: 'Documentos', color: '#7BDCB5' },
  { name: 'Lazer', color: '#00D084' },
  { name: 'Logística', color: '#8ED1FC' },
  { name: 'Viagens', color: '#0693E3' },
  { name: 'Plano de Saúde', color: '#ABB8C3' },
  { name: 'Agenda', color: '#EB144C' },
  { name: 'Financeiro', color: '#F78DA7' },
  { name: 'Assuntos Pessoais', color: '#9900EF' },
  { name: 'Atividades Corporativas', color: '#4A90E2' },
  { name: 'Gestão de Funcionário', color: '#50E3C2' }
];

async function makeRequest(method, url, data = null) {
  try {
    const config = {
      method,
      url,
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      }
    };
    if (data) config.data = data;

    const response = await axios(config);
    return response.data;
  } catch (error) {
    console.error(`Erro em ${url}:`, error.response?.data || error.message);
    return null;
  }
}

async function getAllSpaces() {
  const data = await makeRequest('GET', `https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`);
  return data ? data.spaces : [];
}

async function createFieldInSpace(spaceId, spaceName) {
  console.log(`\n📋 Space: ${spaceName}`);
  console.log(`   ID: ${spaceId}`);

  const fieldData = {
    name: 'Categoria_nova',
    type: 'drop_down',
    type_config: {
      default: 0,
      placeholder: 'Selecione uma categoria',
      options: CATEGORIAS_OPTIONS.map((opt, index) => ({
        name: opt.name,
        color: opt.color,
        orderindex: index
      }))
    }
  };

  const result = await makeRequest(
    'POST',
    `https://api.clickup.com/api/v2/space/${spaceId}/field`,
    fieldData
  );

  if (result) {
    console.log(`   ✅ Campo "Categoria_nova" criado com sucesso!`);
    console.log(`   🆔 Field ID: ${result.id}`);
    return true;
  } else {
    console.log(`   ❌ Erro ao criar campo`);
    return false;
  }
}

async function main() {
  console.log('🚀 CRIAÇÃO DO CAMPO "Categoria_nova" EM TODOS OS SPACES\n');
  console.log(`📊 Workspace: ${WORKSPACE_ID}`);
  console.log(`📝 Opções do campo: ${CATEGORIAS_OPTIONS.map(o => o.name).join(', ')}\n`);

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  let created = 0;
  let errors = 0;

  for (const space of spaces) {
    const success = await createFieldInSpace(space.id, space.name);
    if (success) {
      created++;
    } else {
      errors++;
    }

    // Delay para evitar rate limiting
    await new Promise(resolve => setTimeout(resolve, 1000));
  }

  console.log('\n\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════');
  console.log(`📋 Spaces processados: ${spaces.length}`);
  console.log(`✅ Campos criados: ${created}`);
  console.log(`❌ Erros: ${errors}`);
}

main().catch(error => {
  console.error('❌ Erro fatal:', error);
  process.exit(1);
});
