const axios = require('axios');
const fs = require('fs');

// Configuração do ClickUp
const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Campos personalizados que estamos procurando
const TARGET_FIELDS = ['Categoria', 'Sub Categoria', 'Categoria*', 'SubCategoria'];

async function getAllSpaces() {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`, {
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' }
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
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' }
    });
    return response.data.folders;
  } catch (error) {
    return [];
  }
}

async function getLists(folderId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/folder/${folderId}/list`, {
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' }
    });
    return response.data.lists;
  } catch (error) {
    return [];
  }
}

async function getFolderlessLists(spaceId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/space/${spaceId}/list`, {
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' }
    });
    return response.data.lists;
  } catch (error) {
    return [];
  }
}

async function getTasks(listId) {
  try {
    const response = await axios.get(`https://api.clickup.com/api/v2/list/${listId}/task`, {
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' },
      params: { include_closed: true, page: 0 }
    });
    return response.data.tasks;
  } catch (error) {
    return [];
  }
}

function extractCustomFields(task, listName, spaceName) {
  const result = {
    task_id: task.id,
    task_name: task.name,
    task_url: task.url,
    space_name: spaceName,
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
  console.log('🚀 Iniciando extração otimizada...\n');

  const allResults = [];
  let totalTasks = 0;
  let tasksWithTargetFields = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  for (let i = 0; i < spaces.length; i++) {
    const space = spaces[i];
    console.log(`[${i + 1}/${spaces.length}] ${space.name}`);

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      const tasks = await getTasks(list.id);
      tasks.forEach(task => {
        totalTasks++;
        const taskData = extractCustomFields(task, list.name, space.name);
        if (Object.keys(taskData.custom_fields).length > 0) {
          tasksWithTargetFields++;
          allResults.push(taskData);
        }
      });
    }

    // Folders e listas
    const folders = await getFolders(space.id);
    for (const folder of folders) {
      const lists = await getLists(folder.id);
      for (const list of lists) {
        const tasks = await getTasks(list.id);
        tasks.forEach(task => {
          totalTasks++;
          const taskData = extractCustomFields(task, list.name, space.name);
          if (Object.keys(taskData.custom_fields).length > 0) {
            tasksWithTargetFields++;
            allResults.push(taskData);
          }
        });
      }
    }

    // Salvar incrementalmente
    fs.writeFileSync('clickup-custom-fields-export.json', JSON.stringify(allResults, null, 2));
    console.log(`  ✅ ${totalTasks} tasks | ${tasksWithTargetFields} com campos\n`);
  }

  console.log('\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════');
  console.log(`📝 Total de tasks: ${totalTasks}`);
  console.log(`✅ Tasks com campos: ${tasksWithTargetFields}`);
  console.log(`💾 Arquivo: clickup-custom-fields-export.json`);
}

main().catch(error => {
  console.error('❌ Erro:', error);
  process.exit(1);
});
