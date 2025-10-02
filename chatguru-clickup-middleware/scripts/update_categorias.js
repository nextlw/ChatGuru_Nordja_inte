const axios = require('axios');

// Configuração
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '901300373349';
const CATEGORIA_FIELD_ID = 'c19b4f95-1ff7-4966-b201-02905d33cec6';

// 12 Categorias EXATAS da tabela API William.html (SEM ADM)
const NOVAS_CATEGORIAS = [
  { name: 'Agendamentos', color: '#f900ea' },
  { name: 'Compras', color: '#02BCD4' },
  { name: 'Documentos', color: '#0079bf' },
  { name: 'Lazer', color: '#f2d600' },
  { name: 'Logística', color: '#2ecd6f' },
  { name: 'Viagens', color: '#61bd4f' },
  { name: 'Plano de Saúde', color: '#eb5a46' },
  { name: 'Agenda', color: '#bf55ec' },
  { name: 'Financeiro', color: '#ffab4a' },
  { name: 'Assuntos Pessoais', color: '#c377e0' },
  { name: 'Atividades Corporativas', color: '#FF7FAB' },
  { name: 'Gestão de Funcionário', color: '#81B1FF' }
];

// Mapeamento para migração de categorias antigas → novas
const CATEGORIA_MIGRATION = {
  'ADM': 'Atividades Corporativas',
  'Agendamento': 'Agendamentos',
  'Atividades Pessoais / Domésticas': 'Assuntos Pessoais',
  'Educação / Academia': 'Assuntos Pessoais',
  'Eventos Corporativos': 'Atividades Corporativas',
  'Gestão de Funcionário Doméstico': 'Gestão de Funcionário',
  'Logistica': 'Logística',
  'Festas / Reuniões / Recepção': 'Lazer',
  'Pagamentos': 'Financeiro',
  'Pesquisas / Orçamentos': 'Atividades Corporativas',
  'Controle Interno': 'Atividades Corporativas',
  'Compra/Venda/Aluguel': 'Assuntos Pessoais',
  'Atividade Baixada/Duplicada': 'Atividades Corporativas'
};

async function atualizarCategorias() {
  console.log('🔧 Iniciando atualização de categorias...\n');

  try {
    // 1. Buscar campo atual
    console.log('📋 1. Buscando configuração atual do campo Categoria...');
    const campoAtual = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      {
        headers: { 'Authorization': TOKEN }
      }
    );

    const categoriaField = campoAtual.data.fields.find(
      f => f.id === CATEGORIA_FIELD_ID
    );

    if (!categoriaField) {
      console.error('❌ Campo Categoria não encontrado!');
      return;
    }

    console.log(`✅ Campo encontrado: ${categoriaField.name}`);
    console.log(`   Opções atuais: ${categoriaField.type_config.options.length}`);
    console.log(`\n📋 Categorias atuais:`);
    categoriaField.type_config.options.forEach((opt, i) => {
      console.log(`   ${i + 1}. ${opt.name}`);
    });

    // 2. Verificar duplicação (garantir que não há duplicatas)
    console.log('\n🔍 2. Verificando duplicação...');
    const nomes = NOVAS_CATEGORIAS.map(c => c.name);
    const nomesUnicos = [...new Set(nomes)];

    if (nomes.length !== nomesUnicos.length) {
      console.error('❌ ERRO: Há categorias duplicadas na lista!');
      return;
    }
    console.log(`✅ Sem duplicação: ${nomesUnicos.length} categorias únicas`);

    // 3. Preparar payload para CRIAR novo campo
    console.log('\n📝 3. Preparando payload para CRIAR novo campo...');
    const options = NOVAS_CATEGORIAS.map((cat, index) => ({
      name: cat.name,
      color: cat.color,
      orderindex: index
    }));

    const payload = {
      name: 'Categoria',
      type: 'drop_down',
      type_config: {
        new_drop_down: true,
        options: options
      }
    };

    console.log(`✅ ${options.length} categorias preparadas:\n`);
    options.forEach((opt, i) => {
      console.log(`   ${i + 1}. ${opt.name} (${opt.color})`);
    });

    // 4. Confirmar antes de criar
    console.log('\n⚠️  LIMITAÇÃO DA API: ClickUp NÃO permite editar dropdown options.');
    console.log('   Solução: Vamos CRIAR um NOVO campo "Categoria" com opções corretas.');
    console.log('   O campo antigo permanecerá, mas você pode ocultá-lo depois.\n');
    console.log('   Pressione Ctrl+C para cancelar ou aguarde 3 segundos...\n');

    await new Promise(resolve => setTimeout(resolve, 3000));

    // 5. Criar novo campo
    console.log('🔄 4. Criando NOVO campo no ClickUp...');
    const response = await axios.post(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      payload,
      {
        headers: {
          'Authorization': TOKEN,
          'Content-Type': 'application/json'
        }
      }
    );

    console.log('✅ Novo campo criado com sucesso!\n');
    console.log(`📋 Detalhes do novo campo:`);
    console.log(`   ID: ${response.data.field.id}`);
    console.log(`   Nome: ${response.data.field.name}`);
    console.log(`   Tipo: ${response.data.field.type}`);

    // 6. Verificar criação
    console.log('\n🔍 5. Verificando novo campo...');
    const verificacao = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      {
        headers: { 'Authorization': TOKEN }
      }
    );

    const novoCampo = verificacao.data.fields.find(
      f => f.id === response.data.field.id
    );

    console.log(`✅ Verificação concluída:`);
    console.log(`   Total de opções: ${novoCampo.type_config.options.length}`);
    console.log(`\n📋 Categorias do novo campo:`);
    novoCampo.type_config.options.forEach((opt, i) => {
      console.log(`   ${i + 1}. ${opt.name} (ID: ${opt.id})`);
    });

    // 7. Mostrar guia de migração
    console.log('\n\n📌 GUIA DE MIGRAÇÃO DE TAREFAS EXISTENTES:');
    console.log('=' .repeat(60));
    console.log('As seguintes categorias antigas devem ser migradas:\n');

    const categoriasPorDestino = {};
    Object.entries(CATEGORIA_MIGRATION).forEach(([antiga, nova]) => {
      if (!categoriasPorDestino[nova]) {
        categoriasPorDestino[nova] = [];
      }
      categoriasPorDestino[nova].push(antiga);
    });

    Object.entries(categoriasPorDestino).forEach(([destino, origens]) => {
      console.log(`\n→ ${destino}:`);
      origens.forEach(origem => {
        console.log(`   - "${origem}"`);
      });
    });

    console.log('\n⚠️  PRÓXIMAS AÇÕES:');
    console.log('   1. Use o novo campo "Categoria" nas próximas tarefas');
    console.log('   2. (Opcional) Execute script de migração das tarefas antigas');
    console.log('   3. (Opcional) Oculte o campo antigo "Categoria" no ClickUp UI\n');

    console.log(`\n✅ NOVO CAMPO ID: ${response.data.field.id}`);
    console.log(`   Use este ID no código Rust para integração\n`);

    return true;

  } catch (error) {
    console.error('❌ Erro ao atualizar categorias:');
    console.error(`   Status: ${error.response?.status}`);
    console.error(`   Erro: ${error.response?.data?.err || error.message}`);
    console.error(`   Código: ${error.response?.data?.ECODE}`);

    if (error.response?.data) {
      console.error('\n📄 Resposta completa:', JSON.stringify(error.response.data, null, 2));
    }

    return false;
  }
}

// Executar
atualizarCategorias();
