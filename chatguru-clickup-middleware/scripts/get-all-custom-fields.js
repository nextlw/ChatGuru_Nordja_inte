const axios = require('axios');

// Configuração do ClickUp
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Campos personalizados que estamos procurando
const TARGET_FIELDS = ['Categoria', 'Sub Categoria', 'Categoria*', 'SubCategoria'];

async function getAllSpaces() {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`, {
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      }
    });
    return response.data.spaces;
  } catch (error) {
    console.error('❌ Erro ao buscar spaces:', error.response?.data || error.message);
    return [];
  }
}

async function getFolders(spaceId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/space/${spaceId}/folder`, {
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      }
    });
    return response.data.folders;
  } catch (error) {
    console.error(`❌ Erro ao buscar folders do space ${spaceId}:`, error.response?.data || error.message);
    return [];
  }
}

async function getLists(folderId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/folder/${folderId}/list`, {
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      }
    });
    return response.data.lists;
  } catch (error) {
    console.error(`❌ Erro ao buscar listas do folder ${folderId}:`, error.response?.data || error.message);
    return [];
  }
}

async function getFolderlessLists(spaceId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/space/${spaceId}/list`, {
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      }
    });
    return response.data.lists;
  } catch (error) {
    console.error(`❌ Erro ao buscar listas sem folder do space ${spaceId}:`, error.response?.data || error.message);
    return [];
  }
}

async function getTasks(listId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/list/${listId}/task`, {
      headers: {
        'Authorization': TOKEN,
        'Content-Type': 'application/json'
      },
      params: {
        include_closed: true,
        page: 0
      }
    });
    return response.data.tasks;
  } catch (error) {
    console.error(`❌ Erro ao buscar tasks da lista ${listId}:`, error.response?.data || error.message);
    return [];
  }
}

function extractCustomFields(task, listName) {
  const result = {
    task_id: task.id,
    task_name: task.name,
    task_url: task.url,
    list_name: listName,
    custom_fields: {}
  };

  if (task.custom_fields && task.custom_fields.length > 0) {
    task.custom_fields.forEach(field => {
      if (TARGET_FIELDS.includes(field.name)) {
        result.custom_fields[field.name] = {
          id: field.id,
          type: field.type,
          value: field.value || null,
          type_config: field.type_config || null
        };
      }
    });
  }

  return result;
}

async function main() {
  console.log('🚀 Iniciando extração de campos personalizados do ClickUp...\n');
  console.log(`📊 Workspace ID: ${WORKSPACE_ID}`);
  console.log(`🔍 Campos procurados: ${TARGET_FIELDS.join(', ')}\n`);

  const allResults = [];
  let totalTasks = 0;
  let tasksWithTargetFields = 0;

  // 1. Buscar todos os spaces
  console.log('📂 Buscando spaces...');
  const spaces = await getAllSpaces();
  console.log(`✅ Encontrados ${spaces.length} spaces\n`);

  for (const space of spaces) {
    console.log(`\n🏢 Space: ${space.name} (ID: ${space.id})`);

    // 2. Buscar listas sem folder (folderless)
    console.log('  📋 Buscando listas sem folder...');
    const folderlessLists = await getFolderlessLists(space.id);

    for (const list of folderlessLists) {
      console.log(`    ├─ Lista: ${list.name} (ID: ${list.id})`);
      const tasks = await getTasks(list.id);
      console.log(`    │  └─ ${tasks.length} tasks encontradas`);

      tasks.forEach(task => {
        totalTasks++;
        const taskData = extractCustomFields(task, list.name);

        if (Object.keys(taskData.custom_fields).length > 0) {
          tasksWithTargetFields++;
          allResults.push(taskData);
        }
      });
    }

    // 3. Buscar folders e suas listas
    console.log('  📁 Buscando folders...');
    const folders = await getFolders(space.id);

    for (const folder of folders) {
      console.log(`    ├─ Folder: ${folder.name} (ID: ${folder.id})`);
      const lists = await getLists(folder.id);

      for (const list of lists) {
        console.log(`    │  ├─ Lista: ${list.name} (ID: ${list.id})`);
        const tasks = await getTasks(list.id);
        console.log(`    │  │  └─ ${tasks.length} tasks encontradas`);

        tasks.forEach(task => {
          totalTasks++;
          const taskData = extractCustomFields(task, list.name);

          if (Object.keys(taskData.custom_fields).length > 0) {
            tasksWithTargetFields++;
            allResults.push(taskData);
          }
        });
      }
    }
  }

  // Exibir resultados
  console.log('\n\n📊 RESUMO DA EXTRAÇÃO');
  console.log('═══════════════════════════════════════════════════');
  console.log(`📝 Total de tasks analisadas: ${totalTasks}`);
  console.log(`✅ Tasks com campos personalizados procurados: ${tasksWithTargetFields}`);
  console.log(`🎯 Campos procurados: ${TARGET_FIELDS.join(', ')}\n`);

  if (allResults.length > 0) {
    console.log('📋 DETALHES DAS TASKS COM CAMPOS PERSONALIZADOS:');
    console.log('═══════════════════════════════════════════════════\n');

    allResults.forEach((result, index) => {
      console.log(`\n${index + 1}. Task: ${result.task_name}`);
      console.log(`   🔗 URL: ${result.task_url}`);
      console.log(`   📂 Lista: ${result.list_name}`);
      console.log(`   🆔 ID: ${result.task_id}`);
      console.log(`   📌 Campos Personalizados:`);

      Object.entries(result.custom_fields).forEach(([fieldName, fieldData]) => {
        console.log(`      • ${fieldName}:`);
        console.log(`        - ID: ${fieldData.id}`);
        console.log(`        - Tipo: ${fieldData.type}`);
        console.log(`        - Valor: ${JSON.stringify(fieldData.value)}`);
      });
    });

    // Salvar resultados em JSON
    const fs = require('fs');
    const outputFile = 'clickup-custom-fields-export.json';
    fs.writeFileSync(outputFile, JSON.stringify(allResults, null, 2));
    console.log(`\n\n💾 Resultados salvos em: ${outputFile}`);
  } else {
    console.log('\n⚠️  Nenhuma task encontrada com os campos personalizados procurados.');
  }
}

main().catch(error => {
  console.error('❌ Erro fatal:', error);
  process.exit(1);
});
