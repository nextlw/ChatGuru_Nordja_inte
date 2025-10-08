const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';

// Lista que tem os campos (pode mudar conforme necessário)
const LIST_ID = process.argv[2] || '901300373349';

async function getFieldMappings() {
  console.log(`🔍 Buscando campos personalizados da lista ${LIST_ID}...\n`);

  try {
    const response = await axios.get(
      `https://api.clickup.com/api/v2/list/${LIST_ID}/field`,
      { headers: { 'Authorization': TOKEN } }
    );

    const fields = response.data.fields;

    // Encontrar campos relevantes
    const categoria = fields.find(f => f.name === 'Categoria');
    const subCategoria = fields.find(f => f.name === 'Sub Categoria');
    const categoriaNova = fields.find(f => f.name === 'Categoria*');
    const subCategoriaNova = fields.find(f => f.name === 'SubCategoria');

    console.log('📋 CAMPOS ENCONTRADOS:\n');

    if (categoria) {
      console.log('✅ "Categoria" (antigo)');
      console.log(`   ID: ${categoria.id}`);
      console.log(`   Tipo: ${categoria.type}`);
      if (categoria.type_config?.options) {
        console.log(`   Opções (${categoria.type_config.options.length}):`);
        categoria.type_config.options.forEach(opt => {
          console.log(`      - "${opt.name}" → ${opt.id}`);
        });
      }
      console.log('');
    }

    if (subCategoria) {
      console.log('✅ "Sub Categoria" (antigo)');
      console.log(`   ID: ${subCategoria.id}`);
      console.log(`   Tipo: ${subCategoria.type}`);
      if (subCategoria.type_config?.options) {
        console.log(`   Opções (${subCategoria.type_config.options.length}):`);
        subCategoria.type_config.options.forEach(opt => {
          console.log(`      - "${opt.name}" → ${opt.id}`);
        });
      }
      console.log('');
    }

    if (categoriaNova) {
      console.log('✅ "Categoria*" (novo)');
      console.log(`   ID: ${categoriaNova.id}`);
      console.log(`   Tipo: ${categoriaNova.type}`);
      if (categoriaNova.type_config?.options) {
        console.log(`   Opções (${categoriaNova.type_config.options.length}):`);
        categoriaNova.type_config.options.forEach(opt => {
          console.log(`      - "${opt.name}" → ${opt.id}`);
        });
      }
      console.log('');
    }

    if (subCategoriaNova) {
      console.log('✅ "SubCategoria" (novo)');
      console.log(`   ID: ${subCategoriaNova.id}`);
      console.log(`   Tipo: ${subCategoriaNova.type}`);
      if (subCategoriaNova.type_config?.options) {
        console.log(`   Opções (${subCategoriaNova.type_config.options.length}):`);
        subCategoriaNova.type_config.options.forEach(opt => {
          console.log(`      - "${opt.name}" → ${opt.id}`);
        });
      }
      console.log('');
    }

    // Gerar mapeamento de opções
    console.log('\n📊 MAPEAMENTO DE OPÇÕES (Antigo → Novo):\n');

    if (categoria && categoriaNova) {
      console.log('CATEGORIA:');
      const oldOptions = categoria.type_config?.options || [];
      const newOptions = categoriaNova.type_config?.options || [];

      const mapping = {};
      oldOptions.forEach(oldOpt => {
        const newOpt = newOptions.find(n => n.name === oldOpt.name);
        if (newOpt) {
          mapping[oldOpt.id] = newOpt.id;
          console.log(`  "${oldOpt.name}": ${oldOpt.id} → ${newOpt.id}`);
        } else {
          console.log(`  ⚠️  "${oldOpt.name}": ${oldOpt.id} → NÃO ENCONTRADO`);
        }
      });

      console.log('\n');
    }

    if (subCategoria && subCategoriaNova) {
      console.log('SUB CATEGORIA:');
      const oldOptions = subCategoria.type_config?.options || [];
      const newOptions = subCategoriaNova.type_config?.options || [];

      const mapping = {};
      oldOptions.forEach(oldOpt => {
        const newOpt = newOptions.find(n => n.name === oldOpt.name);
        if (newOpt) {
          mapping[oldOpt.id] = newOpt.id;
          console.log(`  "${oldOpt.name}": ${oldOpt.id} → ${newOpt.id}`);
        } else {
          console.log(`  ⚠️  "${oldOpt.name}": ${oldOpt.id} → NÃO ENCONTRADO`);
        }
      });
    }

    // Salvar em JSON
    const output = {
      list_id: LIST_ID,
      fields: {
        categoria_old: categoria ? {
          id: categoria.id,
          name: categoria.name,
          type: categoria.type,
          options: categoria.type_config?.options || []
        } : null,
        sub_categoria_old: subCategoria ? {
          id: subCategoria.id,
          name: subCategoria.name,
          type: subCategoria.type,
          options: subCategoria.type_config?.options || []
        } : null,
        categoria_new: categoriaNova ? {
          id: categoriaNova.id,
          name: categoriaNova.name,
          type: categoriaNova.type,
          options: categoriaNova.type_config?.options || []
        } : null,
        sub_categoria_new: subCategoriaNova ? {
          id: subCategoriaNova.id,
          name: subCategoriaNova.name,
          type: subCategoriaNova.type,
          options: subCategoriaNova.type_config?.options || []
        } : null
      }
    };

    fs.writeFileSync('field-mappings.json', JSON.stringify(output, null, 2));
    console.log('\n\n💾 Dados salvos em: field-mappings.json');

  } catch (error) {
    console.error('❌ Erro:', error.response?.data || error.message);
  }
}

getFieldMappings();
