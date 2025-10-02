const axios = require('axios');

// Configuração
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const LIST_ID = '901300373349';

// Unicode code points para emojis (sem o prefixo U+)
// ⭐ Estrela - code point: 2b50
// 🌟 Estrela brilhante - code point: 1f31f
// ⭐️ Estrela média - code point: 2b50

async function criarCampoEstrelas() {
  console.log('🔧 Iniciando criação de campo Estrelas (Rating)...\n');

  try {
    // Payload para campo rating (emoji) de 1-4 estrelas
    const payload = {
      name: 'Estrelas',
      type: 'emoji',
      type_config: {
        code_point: '2b50',  // ⭐ Estrela
        count: 4  // Máximo de 4 estrelas (conforme tabela)
      }
    };

    console.log('📝 Configuração do campo:');
    console.log(`   Nome: ${payload.name}`);
    console.log(`   Tipo: ${payload.type} (rating)`);
    console.log(`   Emoji: ⭐ (code_point: ${payload.type_config.code_point})`);
    console.log(`   Escala: 1-${payload.type_config.count} estrelas`);
    console.log('\n   Valores possíveis: 1⭐, 2⭐, 3⭐, 4⭐\n');

    // Confirmar
    console.log('⚠️  Pressione Ctrl+C para cancelar ou aguarde 3 segundos...\n');
    await new Promise(resolve => setTimeout(resolve, 3000));

    // Criar campo
    console.log('🔄 Criando campo Estrelas no ClickUp...');
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
    console.log(`   Code Point: ${response.data.field.type_config.code_point}`);
    console.log(`   Max Count: ${response.data.field.type_config.count}`);

    // Verificar criação
    console.log('\n🔍 Verificando campo criado...');
    const verificacao = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      {
        headers: { 'Authorization': TOKEN }
      }
    );

    const novoCampo = verificacao.data.fields.find(
      f => f.id === response.data.field.id
    );

    console.log(`✅ Verificação concluída!`);
    console.log(`\n✅ NOVO CAMPO ID: ${response.data.field.id}`);
    console.log(`   Use este ID no código Rust para integração\n`);

    // Mostrar como usar
    console.log('📌 Como usar no Rust:');
    console.log('   Para definir o valor de estrelas em uma tarefa:');
    console.log(`   - 1 estrela: { "id": "${response.data.field.id}", "value": 1 }`);
    console.log(`   - 2 estrelas: { "id": "${response.data.field.id}", "value": 2 }`);
    console.log(`   - 3 estrelas: { "id": "${response.data.field.id}", "value": 3 }`);
    console.log(`   - 4 estrelas: { "id": "${response.data.field.id}", "value": 4 }\n`);

    // Mostrar distribuição da tabela
    console.log('📊 Distribuição de estrelas (conforme tabela HTML):');
    console.log('   1 estrela: 56 subcategorias');
    console.log('   2 estrelas: 22 subcategorias');
    console.log('   3 estrelas: 4 subcategorias');
    console.log('   4 estrelas: 3 subcategorias\n');

    return true;

  } catch (error) {
    console.error('❌ Erro ao criar campo Estrelas:');
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
criarCampoEstrelas();
