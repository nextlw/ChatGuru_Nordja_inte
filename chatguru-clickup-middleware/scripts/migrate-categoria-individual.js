const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar mapeamento
const mapping = JSON.parse(fs.readFileSync('migration-mapping.json', 'utf8'));

// ID do campo Categoria antigo
const OLD_CATEGORIA_FIELD_ID = 'c19b4f95-1ff7-4966-b201-02905d33cec6';

// Pegar ID antigo da categoria do argumento
const OLD_VALUE_ID = process.argv[2];

if (!OLD_VALUE_ID) {
  console.log('❌ Uso: node migrate-categoria-individual.js <OLD_VALUE_ID>');
  console.log('\nCategorias disponíveis:');
  Object.entries(mapping.categoria_mapping).forEach(([id, data]) => {
    console.log(`  ${id} → "${data.old_name}" → "${data.new_name}"`);
  });
  process.exit(1);
}

const categoriaMapping = mapping.categoria_mapping[OLD_VALUE_ID];

if (!categoriaMapping) {
  console.log(`❌ ID não encontrado no mapeamento: ${OLD_VALUE_ID}`);
  process.exit(1);
}

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

async function updateTasksInList(listId) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) return 0;

  let updated = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields) continue;

    const categoriaField = task.custom_fields.find(f => f.name === 'Categoria');
    if (!categoriaField || categoriaField.value !== OLD_VALUE_ID) continue;

    console.log(`    📝 ${task.name.substring(0, 60)}...`);
    console.log(`       "${categoriaMapping.old_name}" → "${categoriaMapping.new_name}"`);

    const result = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${OLD_CATEGORIA_FIELD_ID}`,
      { value: categoriaMapping.new_id }
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
  console.log(`🚀 MIGRAÇÃO: "${categoriaMapping.old_name}" → "${categoriaMapping.new_name}"\n`);

  let totalUpdated = 0;
  let listsProcessed = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  for (const space of spaces) {
    console.log(`🏢 Space: ${space.name}`);

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsProcessed++;
      const updated = await updateTasksInList(list.id);
      if (updated > 0) {
        console.log(`  ✅ Lista: ${list.name} → ${updated} tasks atualizadas`);
        totalUpdated += updated;
      }
    }

    // Listas em folders
    const folders = await getFolders(space.id);
    for (const folder of folders) {
      const lists = await getLists(folder.id);
      for (const list of lists) {
        listsProcessed++;
        const updated = await updateTasksInList(list.id);
        if (updated > 0) {
          console.log(`  ✅ ${folder.name} → ${list.name} → ${updated} tasks atualizadas`);
          totalUpdated += updated;
        }
      }
    }
  }

  console.log('\n\n📊 RESUMO');
  console.log('═══════════════════════════════════════');
  console.log(`📋 Listas processadas: ${listsProcessed}`);
  console.log(`✅ Tasks atualizadas: ${totalUpdated}`);
  console.log(`📝 Migração: "${categoriaMapping.old_name}" → "${categoriaMapping.new_name}"`);
}

main().catch(console.error);
