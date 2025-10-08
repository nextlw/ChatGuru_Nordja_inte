const axios = require('axios');
const readline = require('readline');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Mapeamento: campo antigo → campo novo
const FIELD_MAPPING = {
  'Categoria': 'Categoria*',
  'Sub Categoria': 'SubCategoria'
};

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
    console.error(`❌ Erro em ${url}:`, error.response?.data || error.message);
    return null;
  }
}

async function getAllLists() {
  console.log('📂 Buscando todas as listas...\n');

  const allLists = [];

  // Buscar spaces
  const spacesData = await makeRequest('GET', `https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`);
  if (!spacesData) return [];

  for (const space of spacesData.spaces) {
    // Listas sem folder
    const folderlessData = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${space.id}/list`);
    if (folderlessData && folderlessData.lists) {
      folderlessData.lists.forEach(list => {
        allLists.push({
          id: list.id,
          name: list.name,
          space: space.name,
          folder: null
        });
      });
    }

    // Listas em folders
    const foldersData = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${space.id}/folder`);
    if (foldersData && foldersData.folders) {
      for (const folder of foldersData.folders) {
        const listsData = await makeRequest('GET', `https://api.clickup.com/api/v2/folder/${folder.id}/list`);
        if (listsData && listsData.lists) {
          listsData.lists.forEach(list => {
            allLists.push({
              id: list.id,
              name: list.name,
              space: space.name,
              folder: folder.name
            });
          });
        }
      }
    }
  }

  return allLists;
}

async function updateTaskCustomFields(listId, listName) {
  console.log(`\n📋 Processando lista: ${listName}`);
  console.log(`🆔 ID: ${listId}\n`);

  // Buscar tasks da lista
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) {
    console.log('❌ Nenhuma task encontrada');
    return;
  }

  const tasks = tasksData.tasks;
  console.log(`✅ ${tasks.length} tasks encontradas\n`);

  let updated = 0;
  let skipped = 0;
  let errors = 0;

  for (let i = 0; i < tasks.length; i++) {
    const task = tasks[i];
    console.log(`[${i + 1}/${tasks.length}] ${task.name}`);

    if (!task.custom_fields || task.custom_fields.length === 0) {
      console.log('  ⊘ Sem campos personalizados');
      skipped++;
      continue;
    }

    // Encontrar os campos antigos e novos
    const fieldsMap = {};
    task.custom_fields.forEach(field => {
      fieldsMap[field.name] = field;
    });

    let hasUpdates = false;
    const updates = [];

    // Verificar cada mapeamento
    for (const [oldField, newField] of Object.entries(FIELD_MAPPING)) {
      const oldFieldData = fieldsMap[oldField];
      const newFieldData = fieldsMap[newField];

      if (oldFieldData && newFieldData) {
        const oldValue = oldFieldData.value;
        const newValue = newFieldData.value;

        // Só atualiza se o campo antigo tem valor e o novo está vazio
        if (oldValue && !newValue) {
          updates.push({
            oldField,
            newField,
            newFieldId: newFieldData.id,
            value: oldValue
          });
          hasUpdates = true;
        }
      }
    }

    if (!hasUpdates) {
      console.log('  ⊘ Nada para atualizar');
      skipped++;
      continue;
    }

    // Aplicar atualizações
    let taskUpdated = false;
    for (const update of updates) {
      console.log(`  📝 Copiando "${update.oldField}" (${update.value}) → "${update.newField}"`);

      const result = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${update.newFieldId}`,
        { value: update.value }
      );

      if (result) {
        taskUpdated = true;
        console.log(`  ✅ Atualizado: ${update.newField}`);
      } else {
        console.log(`  ❌ Erro ao atualizar: ${update.newField}`);
        errors++;
      }
    }

    if (taskUpdated) {
      updated++;
      console.log(`  ✨ Task atualizada com sucesso`);
    }

    // Delay para evitar rate limiting
    await new Promise(resolve => setTimeout(resolve, 200));
  }

  console.log(`\n📊 RESUMO`);
  console.log(`═══════════════════════════════════`);
  console.log(`✅ Atualizadas: ${updated}`);
  console.log(`⊘ Ignoradas: ${skipped}`);
  console.log(`❌ Erros: ${errors}`);
}

async function main() {
  console.log('🚀 SCRIPT DE ATUALIZAÇÃO DE CAMPOS PERSONALIZADOS\n');
  console.log('📌 Mapeamento:');
  Object.entries(FIELD_MAPPING).forEach(([old, newF]) => {
    console.log(`   "${old}" → "${newF}"`);
  });
  console.log('');

  // Se um LIST_ID foi passado como argumento
  const listIdArg = process.argv[2];

  if (listIdArg) {
    const listNameArg = process.argv[3] || 'Lista especificada';
    await updateTaskCustomFields(listIdArg, listNameArg);
    return;
  }

  // Caso contrário, listar todas as listas
  const lists = await getAllLists();
  console.log(`\n✅ ${lists.length} listas encontradas\n`);

  console.log('📋 LISTAS DISPONÍVEIS:');
  console.log('═══════════════════════════════════════════════════════════════\n');

  lists.forEach((list, index) => {
    const folder = list.folder ? ` / ${list.folder}` : '';
    console.log(`${index + 1}. ${list.name}`);
    console.log(`   🏢 ${list.space}${folder}`);
    console.log(`   🆔 ${list.id}\n`);
  });

  console.log('\n💡 Para processar uma lista específica, rode:');
  console.log('   node update-custom-fields-by-list.js <LIST_ID> "<NOME_DA_LISTA>"');
  console.log('\nExemplo:');
  console.log(`   node update-custom-fields-by-list.js ${lists[0].id} "${lists[0].name}"`);
}

main().catch(console.error);
