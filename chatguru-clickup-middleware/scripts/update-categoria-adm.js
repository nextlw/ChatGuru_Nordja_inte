const axios = require('axios');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Mapeamento específico: ADM → Atividades Corporativas
const OLD_VALUE_ID = 'a4a4e85c-4eb5-44f9-9175-f98594da5c70'; // ADM
const NEW_VALUE_ID = '5baa7715-1dfa-4a36-8452-78d60748e193'; // Atividades Corporativas

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
    return null;
  }
}

async function getAllSpaces() {
  const data = await makeRequest('GET', `https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`);
  return data ? data.spaces : [];
}

async function getFolders(spaceId) {
  const data = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${spaceId}/folder`);
  return data ? data.folders : [];
}

async function getLists(folderId) {
  const data = await makeRequest('GET', `https://api.clickup.com/api/v2/folder/${folderId}/list`);
  return data ? data.lists : [];
}

async function getFolderlessLists(spaceId) {
  const data = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${spaceId}/list`);
  return data ? data.lists : [];
}

async function updateTasksInList(listId, listName) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) return 0;

  let updated = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields) continue;

    const categoriaField = task.custom_fields.find(f => f.name === 'Categoria');
    if (!categoriaField || categoriaField.value !== OLD_VALUE_ID) continue;

    console.log(`    📝 ${task.name.substring(0, 60)}...`);
    console.log(`       ADM → Atividades Corporativas`);

    const result = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${categoriaField.id}`,
      { value: NEW_VALUE_ID }
    );

    if (result) {
      updated++;
      console.log(`       ✅ Atualizado\n`);
    } else {
      console.log(`       ❌ Erro\n`);
    }

    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return updated;
}

async function main() {
  console.log('🚀 ATUALIZAÇÃO: ADM → Atividades Corporativas\n');
  console.log(`📊 Workspace: ${WORKSPACE_ID}\n`);

  let totalUpdated = 0;
  let listsProcessed = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  for (const space of spaces) {
    console.log(`\n🏢 Space: ${space.name}`);

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsProcessed++;
      console.log(`  ├─ Lista: ${list.name}`);
      const updated = await updateTasksInList(list.id, list.name);
      if (updated > 0) {
        console.log(`     ✅ ${updated} tasks atualizadas`);
        totalUpdated += updated;
      }
    }

    // Listas em folders
    const folders = await getFolders(space.id);
    for (const folder of folders) {
      console.log(`  ├─ Folder: ${folder.name}`);
      const lists = await getLists(folder.id);
      for (const list of lists) {
        listsProcessed++;
        console.log(`     ├─ Lista: ${list.name}`);
        const updated = await updateTasksInList(list.id, list.name);
        if (updated > 0) {
          console.log(`        ✅ ${updated} tasks atualizadas`);
          totalUpdated += updated;
        }
      }
    }
  }

  console.log('\n\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════');
  console.log(`📋 Listas processadas: ${listsProcessed}`);
  console.log(`✅ Tasks atualizadas: ${totalUpdated}`);
  console.log(`📝 Alteração: "ADM" → "Atividades Corporativas"`);
}

main().catch(console.error);
