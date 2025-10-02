const axios = require('axios');
const fs = require('fs');

// Configuração
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '901300373349';

// IDs dos novos campos
const FIELD_IDS = {
  categoria: 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a',
  subCategoria: '5333c095-eb40-4a5a-b0c2-76bfba4b1094',
  estrelas: '83afcb8c-2866-498f-9c62-8ea9666b104b'
};

// Mapa completo: SubCategoria → Estrelas (EXATO da tabela HTML)
const SUBCATEGORIA_ESTRELAS = {
  // Agendamentos
  'Consultas': 1,
  'Exames': 1,
  'Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)': 1,
  'Vacinas': 1,
  'Manicure': 1,
  'Cabeleleiro': 1,
  // Compras
  'Shopper': 2,
  'Mercados': 1,
  'Presentes': 1,
  'Petshop': 1,
  'Papelaria': 1,
  'Farmácia': 2,
  'Ingressos': 2,
  'Móveis e Eletros': 2,
  'Itens pessoais e da casa': 2,
  // Documentos
  'CIN': 1,
  'Certificado Digital': 2,
  'Documento de Vacinação (BR/Iternacional)': 1,
  'Seguros Carro/Casa/Viagem (anual)': 2,
  'Assinatura Digital': 1,
  'Contratos/Procurações': 1,
  'Cidadanias': 4,
  'Vistos e Vistos Eletrônicos': 2,
  'Passaporte': 1,
  'CNH': 1,
  'Averbações': 1,
  'Certidões': 1,
  // Lazer
  'Reserva de restaurantes/bares': 1,
  'Planejamento de festas': 4,
  'Pesquisa de passeios/eventos (BR)': 3,
  'Fornecedores no exterior (passeios, fotógrafos)': 2,
  // Logística
  'Corrida de motoboy': 1,
  'Motoboy + Correios e envios internacionais': 1,
  'Lalamove': 1,
  'Corridas com Taxistas': 1,
  'Transporte Urbano (Uber/99)': 1,
  // Viagens
  'Passagens Aéreas': 2,
  'Hospedagens': 2,
  'Compra de Assentos e Bagagens': 1,
  'Passagens de Ônibus': 1,
  'Passagens de Trem': 2,
  'Checkins (Early/Late)': 1,
  'Extravio de Bagagens': 2,
  'Seguro Viagem (Temporário)': 1,
  'Transfer': 2,
  'Programa de Milhagem': 1,
  'Gestão de Contas (CIAs Aereas)': 1,
  'Aluguel de Carro/Ônibus e Vans': 2,
  'Roteiro de Viagens': 3,
  // Plano de Saúde
  'Reembolso Médico': 2,
  'Extrato para IR': 1,
  'Prévia de Reembolso': 1,
  'Contestações': 1,
  'Autorizações': 1,
  'Contratações/Cancelamentos': 2,
  // Agenda
  'Gestão de Agenda': 1,
  'Criação e envio de invites': 1,
  // Financeiro
  'Emissão de NF': 1,
  'Rotina de Pagamentos': 1,
  'Emissão de boletos': 1,
  'Conciliação Bancária': 2,
  'Planilha de Gastos/Pagamentos': 4,
  'Encerramento e Abertura de CNPJ': 2,
  'Imposto de Renda': 1,
  'Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)': 1,
  // Assuntos Pessoais
  'Mudanças': 3,
  'Troca de titularidade': 1,
  'Assuntos do Carro/Moto': 1,
  'Internet e TV por Assinatura': 1,
  'Contas de Consumo': 1,
  'Anúncio de Vendas Online (Itens, eletros. móveis)': 3,
  'Assuntos Escolares e Professores Particulares': 1,
  'Academia e Cursos Livres': 1,
  'Telefone': 1,
  'Assistência Técnica': 1,
  'Consertos na Casa': 1,
  // Atividades Corporativas
  'Financeiro/Contábil': 1,
  'Atendimento ao Cliente': 1,
  'Gestão de Planilhas e Emails': 4,
  'Documentos/Contratos e Assinaturas': 1,
  'Gestão de Agenda (Corporativa)': 1,
  'Recursos Humanos': 1,
  'Gestão de Estoque': 1,
  'Compras/vendas': 1,
  'Fornecedores': 2,
  // Gestão de Funcionário
  'eSocial': 1,
  'Contratações e Desligamentos': 1,
  'DIRF': 1,
  'Férias': 1
};

// Mapeamento de palavras-chave para categorização automática
const KEYWORD_MAPPING = {
  // Logística
  'motoboy': { categoria: 'Logística', subCategoria: 'Corrida de motoboy' },
  'sedex': { categoria: 'Logística', subCategoria: 'Motoboy + Correios e envios internacionais' },
  'correio': { categoria: 'Logística', subCategoria: 'Motoboy + Correios e envios internacionais' },
  'lalamove': { categoria: 'Logística', subCategoria: 'Lalamove' },
  'uber': { categoria: 'Logística', subCategoria: 'Transporte Urbano (Uber/99)' },
  '99': { categoria: 'Logística', subCategoria: 'Transporte Urbano (Uber/99)' },
  'taxista': { categoria: 'Logística', subCategoria: 'Corridas com Taxistas' },
  'entrega': { categoria: 'Logística', subCategoria: 'Corrida de motoboy' },
  'retirada': { categoria: 'Logística', subCategoria: 'Corrida de motoboy' },
  // Plano de Saúde
  'reembolso': { categoria: 'Plano de Saúde', subCategoria: 'Reembolso Médico' },
  'bradesco saúde': { categoria: 'Plano de Saúde', subCategoria: 'Reembolso Médico' },
  'plano de saúde': { categoria: 'Plano de Saúde', subCategoria: 'Reembolso Médico' },
  // Compras
  'mercado': { categoria: 'Compras', subCategoria: 'Mercados' },
  'farmácia': { categoria: 'Compras', subCategoria: 'Farmácia' },
  'presente': { categoria: 'Compras', subCategoria: 'Presentes' },
  'shopper': { categoria: 'Compras', subCategoria: 'Shopper' },
  'papelaria': { categoria: 'Compras', subCategoria: 'Papelaria' },
  'petshop': { categoria: 'Compras', subCategoria: 'Petshop' },
  'ingresso': { categoria: 'Compras', subCategoria: 'Ingressos' },
  // Assuntos Pessoais
  'troca': { categoria: 'Assuntos Pessoais', subCategoria: 'Troca de titularidade' },
  'internet': { categoria: 'Assuntos Pessoais', subCategoria: 'Internet e TV por Assinatura' },
  'telefone': { categoria: 'Assuntos Pessoais', subCategoria: 'Telefone' },
  'conserto': { categoria: 'Assuntos Pessoais', subCategoria: 'Consertos na Casa' },
  'assistência': { categoria: 'Assuntos Pessoais', subCategoria: 'Assistência Técnica' },
  // Financeiro
  'pagamento': { categoria: 'Financeiro', subCategoria: 'Rotina de Pagamentos' },
  'boleto': { categoria: 'Financeiro', subCategoria: 'Emissão de boletos' },
  'nota fiscal': { categoria: 'Financeiro', subCategoria: 'Emissão de NF' },
  // Viagens
  'passagem': { categoria: 'Viagens', subCategoria: 'Passagens Aéreas' },
  'hospedagem': { categoria: 'Viagens', subCategoria: 'Hospedagens' },
  'hotel': { categoria: 'Viagens', subCategoria: 'Hospedagens' },
  'check in': { categoria: 'Viagens', subCategoria: 'Checkins (Early/Late)' },
  'bagagem': { categoria: 'Viagens', subCategoria: 'Extravio de Bagagens' },
  // Agendamentos
  'consulta': { categoria: 'Agendamentos', subCategoria: 'Consultas' },
  'exame': { categoria: 'Agendamentos', subCategoria: 'Exames' },
  'vacina': { categoria: 'Agendamentos', subCategoria: 'Vacinas' },
  'manicure': { categoria: 'Agendamentos', subCategoria: 'Manicure' },
  'cabeleireiro': { categoria: 'Agendamentos', subCategoria: 'Cabeleleiro' },
  // Lazer
  'restaurante': { categoria: 'Lazer', subCategoria: 'Reserva de restaurantes/bares' },
  'reserva': { categoria: 'Lazer', subCategoria: 'Reserva de restaurantes/bares' },
  'festa': { categoria: 'Lazer', subCategoria: 'Planejamento de festas' },
  // Documentos
  'passaporte': { categoria: 'Documentos', subCategoria: 'Passaporte' },
  'cnh': { categoria: 'Documentos', subCategoria: 'CNH' },
  'cidadania': { categoria: 'Documentos', subCategoria: 'Cidadanias' },
  'visto': { categoria: 'Documentos', subCategoria: 'Vistos e Vistos Eletrônicos' },
  'certidão': { categoria: 'Documentos', subCategoria: 'Certidões' },
  'contrato': { categoria: 'Documentos', subCategoria: 'Contratos/Procurações' },
};

// Buscar IDs das opções de categoria e subcategoria
async function buscarOpcoesCampos() {
  const response = await axios.get(
    `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
    { headers: { 'Authorization': TOKEN } }
  );

  const categoriaField = response.data.fields.find(f => f.id === FIELD_IDS.categoria);
  const subCategoriaField = response.data.fields.find(f => f.id === FIELD_IDS.subCategoria);

  const categoriaOptions = {};
  const subCategoriaOptions = {};

  categoriaField.type_config.options.forEach(opt => {
    categoriaOptions[opt.name] = opt.id;
  });

  subCategoriaField.type_config.options.forEach(opt => {
    subCategoriaOptions[opt.name] = opt.id;
  });

  return { categoriaOptions, subCategoriaOptions };
}

// Analisar tarefa e sugerir categorização
function analisarTarefa(taskName) {
  const nameLower = taskName.toLowerCase();

  for (const [keyword, mapping] of Object.entries(KEYWORD_MAPPING)) {
    if (nameLower.includes(keyword)) {
      return mapping;
    }
  }

  // Default para tarefas não categorizadas
  return null;
}

// Atualizar custom field de uma tarefa
async function atualizarCampoTarefa(taskId, fieldId, value) {
  try {
    await axios.post(
      `https://api.clickup.com/api/v2/task/${taskId}/field/${fieldId}`,
      { value },
      {
        headers: {
          'Authorization': TOKEN,
          'Content-Type': 'application/json'
        }
      }
    );
    return true;
  } catch (error) {
    console.error(`     Erro: ${error.response?.data?.err || error.message}`);
    return false;
  }
}

async function categorizarTarefas() {
  console.log('🔧 Iniciando categorização automática de tarefas...\n');

  try {
    // 1. Buscar opções dos campos
    console.log('📋 1. Carregando opções de categoria e subcategoria...');
    const { categoriaOptions, subCategoriaOptions } = await buscarOpcoesCampos();
    console.log(`   ✅ ${Object.keys(categoriaOptions).length} categorias`);
    console.log(`   ✅ ${Object.keys(subCategoriaOptions).length} subcategorias\n`);

    // 2. Buscar tarefas pendentes
    console.log('📋 2. Buscando tarefas pendentes...');
    const response = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/task?subtasks=true&include_closed=false`,
      { headers: { 'Authorization': TOKEN } }
    );

    const tasks = response.data.tasks;
    console.log(`   ✅ ${tasks.length} tarefas encontradas\n`);

    // 3. Analisar e categorizar
    console.log('🔍 3. Analisando e categorizando tarefas...\n');

    let categorizadas = 0;
    let naoCategorizadas = 0;
    let erros = 0;

    for (const task of tasks) {
      const analise = analisarTarefa(task.name);

      if (analise) {
        const categoriaId = categoriaOptions[analise.categoria];
        const subCategoriaId = subCategoriaOptions[analise.subCategoria];

        if (!categoriaId || !subCategoriaId) {
          console.log(`   ⚠️  [${task.id}] "${task.name.substring(0, 60)}..."`);
          console.log(`       Categoria/SubCategoria não encontrada: ${analise.categoria} / ${analise.subCategoria}\n`);
          naoCategorizadas++;
          continue;
        }

        console.log(`   📝 [${task.id}] "${task.name.substring(0, 60)}..."`);
        console.log(`       → ${analise.categoria} > ${analise.subCategoria} (${analise.estrelas}⭐)`);

        // Atualizar campos
        const ok1 = await atualizarCampoTarefa(task.id, FIELD_IDS.categoria, categoriaId);
        const ok2 = await atualizarCampoTarefa(task.id, FIELD_IDS.subCategoria, subCategoriaId);
        const ok3 = await atualizarCampoTarefa(task.id, FIELD_IDS.estrelas, analise.estrelas);

        if (ok1 && ok2 && ok3) {
          console.log(`       ✅ Atualizada\n`);
          categorizadas++;
        } else {
          console.log(`       ❌ Erro ao atualizar\n`);
          erros++;
        }

        // Delay para evitar rate limit
        await new Promise(resolve => setTimeout(resolve, 300));
      } else {
        console.log(`   ⚠️  [${task.id}] "${task.name.substring(0, 60)}..."`);
        console.log(`       Sem categorização automática disponível\n`);
        naoCategorizadas++;
      }
    }

    // 4. Resumo
    console.log('\n📊 RESUMO:');
    console.log(`   ✅ Categorizadas: ${categorizadas}`);
    console.log(`   ⚠️  Não categorizadas: ${naoCategorizadas}`);
    console.log(`   ❌ Erros: ${erros}`);
    console.log(`   📋 Total processadas: ${tasks.length}\n`);

    return true;

  } catch (error) {
    console.error('❌ Erro ao categorizar tarefas:');
    console.error(`   Status: ${error.response?.status}`);
    console.error(`   Erro: ${error.response?.data?.err || error.message}`);
    return false;
  }
}

// Executar
categorizarTarefas();
