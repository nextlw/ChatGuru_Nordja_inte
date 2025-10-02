const axios = require('axios');

// Configuração
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '901300373349';

// 90 Subcategorias EXATAS da tabela API William.html
const SUBCATEGORIAS = [
  // Agendamentos (6)
  { name: 'Consultas', color: '#f900ea' },
  { name: 'Exames', color: '#f900ea' },
  { name: 'Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)', color: '#f900ea' },
  { name: 'Vacinas', color: '#f900ea' },
  { name: 'Manicure', color: '#f900ea' },
  { name: 'Cabeleleiro', color: '#f900ea' },

  // Compras (9)
  { name: 'Shopper', color: '#02BCD4' },
  { name: 'Mercados', color: '#02BCD4' },
  { name: 'Presentes', color: '#02BCD4' },
  { name: 'Petshop', color: '#02BCD4' },
  { name: 'Papelaria', color: '#02BCD4' },
  { name: 'Farmácia', color: '#02BCD4' },
  { name: 'Ingressos', color: '#02BCD4' },
  { name: 'Móveis e Eletros', color: '#02BCD4' },
  { name: 'Itens pessoais e da casa', color: '#02BCD4' },

  // Documentos (12)
  { name: 'CIN', color: '#0079bf' },
  { name: 'Certificado Digital', color: '#0079bf' },
  { name: 'Documento de Vacinação (BR/Iternacional)', color: '#0079bf' },
  { name: 'Seguros Carro/Casa/Viagem (anual)', color: '#0079bf' },
  { name: 'Assinatura Digital', color: '#0079bf' },
  { name: 'Contratos/Procurações', color: '#0079bf' },
  { name: 'Cidadanias', color: '#0079bf' },
  { name: 'Vistos e Vistos Eletrônicos', color: '#0079bf' },
  { name: 'Passaporte', color: '#0079bf' },
  { name: 'CNH', color: '#0079bf' },
  { name: 'Averbações', color: '#0079bf' },
  { name: 'Certidões', color: '#0079bf' },

  // Lazer (4)
  { name: 'Reserva de restaurantes/bares', color: '#f2d600' },
  { name: 'Planejamento de festas', color: '#f2d600' },
  { name: 'Pesquisa de passeios/eventos (BR)', color: '#f2d600' },
  { name: 'Fornecedores no exterior (passeios, fotógrafos)', color: '#f2d600' },

  // Logística (5)
  { name: 'Corrida de motoboy', color: '#2ecd6f' },
  { name: 'Motoboy + Correios e envios internacionais', color: '#2ecd6f' },
  { name: 'Lalamove', color: '#2ecd6f' },
  { name: 'Corridas com Taxistas', color: '#2ecd6f' },
  { name: 'Transporte Urbano (Uber/99)', color: '#2ecd6f' },

  // Viagens (13)
  { name: 'Passagens Aéreas', color: '#61bd4f' },
  { name: 'Hospedagens', color: '#61bd4f' },
  { name: 'Compra de Assentos e Bagagens', color: '#61bd4f' },
  { name: 'Passagens de Ônibus', color: '#61bd4f' },
  { name: 'Passagens de Trem', color: '#61bd4f' },
  { name: 'Checkins (Early/Late)', color: '#61bd4f' },
  { name: 'Extravio de Bagagens', color: '#61bd4f' },
  { name: 'Seguro Viagem (Temporário)', color: '#61bd4f' },
  { name: 'Transfer', color: '#61bd4f' },
  { name: 'Programa de Milhagem', color: '#61bd4f' },
  { name: 'Gestão de Contas (CIAs Aereas)', color: '#61bd4f' },
  { name: 'Aluguel de Carro/Ônibus e Vans', color: '#61bd4f' },
  { name: 'Roteiro de Viagens', color: '#61bd4f' },

  // Plano de Saúde (6)
  { name: 'Reembolso Médico', color: '#eb5a46' },
  { name: 'Extrato para IR', color: '#eb5a46' },
  { name: 'Prévia de Reembolso', color: '#eb5a46' },
  { name: 'Contestações', color: '#eb5a46' },
  { name: 'Autorizações', color: '#eb5a46' },
  { name: 'Contratações/Cancelamentos', color: '#eb5a46' },

  // Agenda (2)
  { name: 'Gestão de Agenda', color: '#bf55ec' },
  { name: 'Criação e envio de invites', color: '#bf55ec' },

  // Financeiro (8)
  { name: 'Emissão de NF', color: '#ffab4a' },
  { name: 'Rotina de Pagamentos', color: '#ffab4a' },
  { name: 'Emissão de boletos', color: '#ffab4a' },
  { name: 'Conciliação Bancária', color: '#ffab4a' },
  { name: 'Planilha de Gastos/Pagamentos', color: '#ffab4a' },
  { name: 'Encerramento e Abertura de CNPJ', color: '#ffab4a' },
  { name: 'Imposto de Renda', color: '#ffab4a' },
  { name: 'Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)', color: '#ffab4a' },

  // Assuntos Pessoais (11)
  { name: 'Mudanças', color: '#c377e0' },
  { name: 'Troca de titularidade', color: '#c377e0' },
  { name: 'Assuntos do Carro/Moto', color: '#c377e0' },
  { name: 'Internet e TV por Assinatura', color: '#c377e0' },
  { name: 'Contas de Consumo', color: '#c377e0' },
  { name: 'Anúncio de Vendas Online (Itens, eletros. móveis)', color: '#c377e0' },
  { name: 'Assuntos Escolares e Professores Particulares', color: '#c377e0' },
  { name: 'Academia e Cursos Livres', color: '#c377e0' },
  { name: 'Telefone', color: '#c377e0' },
  { name: 'Assistência Técnica', color: '#c377e0' },
  { name: 'Consertos na Casa', color: '#c377e0' },

  // Atividades Corporativas (9)
  { name: 'Financeiro/Contábil', color: '#FF7FAB' },
  { name: 'Atendimento ao Cliente', color: '#FF7FAB' },
  { name: 'Gestão de Planilhas e Emails', color: '#FF7FAB' },
  { name: 'Documentos/Contratos e Assinaturas', color: '#FF7FAB' },
  { name: 'Gestão de Agenda (Corporativa)', color: '#FF7FAB' },
  { name: 'Recursos Humanos', color: '#FF7FAB' },
  { name: 'Gestão de Estoque', color: '#FF7FAB' },
  { name: 'Compras/vendas', color: '#FF7FAB' },
  { name: 'Fornecedores', color: '#FF7FAB' },

  // Gestão de Funcionário (4)
  { name: 'eSocial', color: '#81B1FF' },
  { name: 'Contratações e Desligamentos', color: '#81B1FF' },
  { name: 'DIRF', color: '#81B1FF' },
  { name: 'Férias', color: '#81B1FF' }
];

async function criarSubCategorias() {
  console.log('🔧 Iniciando criação de campo SubCategoria...\n');

  try {
    // 1. Verificar duplicação
    console.log('🔍 1. Verificando duplicação...');
    const nomes = SUBCATEGORIAS.map(c => c.name);
    const nomesUnicos = [...new Set(nomes)];

    if (nomes.length !== nomesUnicos.length) {
      const duplicatas = nomes.filter((item, index) => nomes.indexOf(item) !== index);
      console.error('❌ ERRO: Há subcategorias duplicadas!');
      console.error(`   Duplicatas: ${[...new Set(duplicatas)].join(', ')}`);
      return;
    }
    console.log(`✅ Sem duplicação: ${nomesUnicos.length} subcategorias únicas`);

    // 2. Preparar payload
    console.log('\n📝 2. Preparando payload para criar campo SubCategoria...');
    const options = SUBCATEGORIAS.map((sub, index) => ({
      name: sub.name,
      color: sub.color,
      orderindex: index
    }));

    const payload = {
      name: 'SubCategoria',
      type: 'drop_down',
      type_config: {
        new_drop_down: true,
        options: options
      }
    };

    console.log(`✅ ${options.length} subcategorias preparadas`);

    // 3. Mostrar resumo por categoria
    console.log('\n📊 Resumo por categoria:');
    const porCategoria = {
      'Agendamentos': 6,
      'Compras': 9,
      'Documentos': 12,
      'Lazer': 4,
      'Logística': 5,
      'Viagens': 13,
      'Plano de Saúde': 6,
      'Agenda': 2,
      'Financeiro': 8,
      'Assuntos Pessoais': 11,
      'Atividades Corporativas': 9,
      'Gestão de Funcionário': 4
    };

    Object.entries(porCategoria).forEach(([cat, count]) => {
      console.log(`   ${cat}: ${count} subcategorias`);
    });
    console.log(`   TOTAL: ${Object.values(porCategoria).reduce((a, b) => a + b, 0)}`);

    // 4. Confirmar
    console.log('\n⚠️  Pressione Ctrl+C para cancelar ou aguarde 3 segundos...\n');
    await new Promise(resolve => setTimeout(resolve, 3000));

    // 5. Criar campo
    console.log('🔄 3. Criando campo SubCategoria no ClickUp...');
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

    console.log('✅ Campo criado com sucesso!\n');
    console.log(`📋 Detalhes do campo:`);
    console.log(`   ID: ${response.data.field.id}`);
    console.log(`   Nome: ${response.data.field.name}`);
    console.log(`   Tipo: ${response.data.field.type}`);
    console.log(`   Total opções: ${response.data.field.type_config.options.length}`);

    // 6. Verificar criação
    console.log('\n🔍 4. Verificando campo criado...');
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
    console.log(`   Total de opções no campo: ${novoCampo.type_config.options.length}`);

    console.log(`\n✅ NOVO CAMPO ID: ${response.data.field.id}`);
    console.log(`   Use este ID no código Rust para integração\n`);

    // Salvar IDs das opções em arquivo para referência
    console.log('📄 Salvando IDs das subcategorias...\n');
    const subcategoriasComIds = novoCampo.type_config.options.map(opt => ({
      name: opt.name,
      id: opt.id,
      color: opt.color
    }));

    console.log('Primeiras 10 subcategorias:');
    subcategoriasComIds.slice(0, 10).forEach((sub, i) => {
      console.log(`   ${i + 1}. ${sub.name} (${sub.id})`);
    });
    console.log(`   ... e mais ${subcategoriasComIds.length - 10} subcategorias`);

    return true;

  } catch (error) {
    console.error('❌ Erro ao criar subcategorias:');
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
criarSubCategorias();
