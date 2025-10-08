const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// IDs do campo Categoria em diferentes listas (pode variar)
// Vamos descobrir dinamicamente

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
  // Primeiro, pegar as tasks da lista
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) return { updated: 0, errors: 0 };

  let updated = 0;
  let errors = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields) continue;

    // Procurar campo "Categoria" (pode ser "Categoria" ou "Categoria*")
    const categoriaField = task.custom_fields.find(f =>
      f.name === 'Categoria' || f.name === 'Categoria*'
    );

    if (!categoriaField || !categoriaField.value) continue;

    // Verificar se tem opções
    if (!categoriaField.type_config || !categoriaField.type_config.options) continue;

    // Encontrar a opção atual
    const currentOption = categoriaField.type_config.options.find(o => o.id === categoriaField.value);
    if (!currentOption) continue;

    // Verificar se é "Agendamento" (singular)
    if (currentOption.name !== 'Agendamento') continue;

    // Procurar opção "Agendamentos" (plural)
    const newOption = categoriaField.type_config.options.find(o => o.name === 'Agendamentos');

    if (!newOption) {
      console.log(`    ⚠️  Lista "${listName}" não tem opção "Agendamentos" no campo ${categoriaField.name}`);
      continue;
    }

    console.log(`    📝 ${task.name.substring(0, 60)}...`);
    console.log(`       Campo: ${categoriaField.name}`);
    console.log(`       "Agendamento" → "Agendamentos"`);

    // Atualizar
    const result = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${categoriaField.id}`,
      { value: newOption.id }
    );

    if (result) {
      updated++;
      console.log(`       ✅ Atualizado\n`);
    } else {
      errors++;
      console.log(`       ❌ Erro\n`);
    }

    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return { updated, errors };
}

async function main() {
  console.log('🚀 ATUALIZAÇÃO: "Agendamento" → "Agendamentos"\n');
  console.log('📊 Processando workspace:', WORKSPACE_ID, '\n');

  const logFile = 'agendamento-migration.log';
  fs.writeFileSync(logFile, `Início: ${new Date().toISOString()}\n\n`);

  let totalUpdated = 0;
  let totalErrors = 0;
  let listsProcessed = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  for (const space of spaces) {
    console.log(`🏢 Space: ${space.name}`);

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsProcessed++;
      const result = await updateTasksInList(list.id, list.name);
      totalUpdated += result.updated;
      totalErrors += result.errors;

      if (result.updated > 0) {
        const msg = `  ✅ Lista: ${list.name} → ${result.updated} tasks atualizadas\n`;
        console.log(msg.trim());
        fs.appendFileSync(logFile, msg);
      }
    }

    // Listas em folders
    const folders = await getFolders(space.id);
    for (const folder of folders) {
      const lists = await getLists(folder.id);
      for (const list of lists) {
        listsProcessed++;
        const result = await updateTasksInList(list.id, list.name);
        totalUpdated += result.updated;
        totalErrors += result.errors;

        if (result.updated > 0) {
          const msg = `  ✅ ${folder.name} → ${list.name} → ${result.updated} tasks atualizadas\n`;
          console.log(msg.trim());
          fs.appendFileSync(logFile, msg);
        }
      }
    }
  }

  const summary = `
\n📊 RESUMO FINAL
═══════════════════════════════════════
📋 Listas processadas: ${listsProcessed}
✅ Tasks atualizadas: ${totalUpdated}
❌ Erros: ${totalErrors}
📝 Mudança: "Agendamento" → "Agendamentos"

💾 Log salvo em: ${logFile}
`;

  console.log(summary);
  fs.appendFileSync(logFile, summary);
}

main().catch(error => {
  console.error('❌ Erro fatal:', error);
  process.exit(1);
});
