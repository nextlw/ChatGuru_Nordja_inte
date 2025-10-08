const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

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
    return null;
  }
}

async function getAllLists() {
  const allLists = [];
  const spacesData = await makeRequest('GET', `https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`);
  if (!spacesData) return [];

  for (const space of spacesData.spaces) {
    const folderlessData = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${space.id}/list`);
    if (folderlessData && folderlessData.lists) {
      folderlessData.lists.forEach(list => {
        allLists.push({ id: list.id, name: list.name, space: space.name, folder: null });
      });
    }

    const foldersData = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${space.id}/folder`);
    if (foldersData && foldersData.folders) {
      for (const folder of foldersData.folders) {
        const listsData = await makeRequest('GET', `https://api.clickup.com/api/v2/folder/${folder.id}/list`);
        if (listsData && listsData.lists) {
          listsData.lists.forEach(list => {
            allLists.push({ id: list.id, name: list.name, space: space.name, folder: folder.name });
          });
        }
      }
    }
  }

  return allLists;
}

async function updateTaskCustomFields(listId, listName) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) return { updated: 0, skipped: 0, errors: 0 };

  const tasks = tasksData.tasks;
  let updated = 0, skipped = 0, errors = 0;

  for (const task of tasks) {
    if (!task.custom_fields || task.custom_fields.length === 0) {
      skipped++;
      continue;
    }

    const fieldsMap = {};
    task.custom_fields.forEach(field => {
      fieldsMap[field.name] = field;
    });

    let hasUpdates = false;
    const updates = [];

    for (const [oldField, newField] of Object.entries(FIELD_MAPPING)) {
      const oldFieldData = fieldsMap[oldField];
      const newFieldData = fieldsMap[newField];

      if (oldFieldData && newFieldData && oldFieldData.value && !newFieldData.value) {
        updates.push({
          oldField,
          newField,
          newFieldId: newFieldData.id,
          value: oldFieldData.value
        });
        hasUpdates = true;
      }
    }

    if (!hasUpdates) {
      skipped++;
      continue;
    }

    let taskUpdated = false;
    for (const update of updates) {
      const result = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${update.newFieldId}`,
        { value: update.value }
      );

      if (result) {
        taskUpdated = true;
      } else {
        errors++;
      }
    }

    if (taskUpdated) updated++;

    await new Promise(resolve => setTimeout(resolve, 150));
  }

  return { updated, skipped, errors };
}

async function main() {
  console.log('🚀 ATUALIZAÇÃO EM MASSA DE CAMPOS PERSONALIZADOS\n');
  console.log('📌 Mapeamento: "Categoria" → "Categoria*" | "Sub Categoria" → "SubCategoria"\n');

  console.log('📂 Carregando listas...');
  const lists = await getAllLists();
  console.log(`✅ ${lists.length} listas encontradas\n`);

  const summary = {
    totalLists: lists.length,
    processedLists: 0,
    totalUpdated: 0,
    totalSkipped: 0,
    totalErrors: 0
  };

  const logFile = 'update-progress.log';
  fs.writeFileSync(logFile, `Início: ${new Date().toISOString()}\n\n`);

  for (let i = 0; i < lists.length; i++) {
    const list = lists[i];
    const progress = `[${i + 1}/${lists.length}]`;
    const folder = list.folder ? ` / ${list.folder}` : '';

    console.log(`${progress} ${list.space}${folder} → ${list.name}`);

    const result = await updateTaskCustomFields(list.id, list.name);

    summary.processedLists++;
    summary.totalUpdated += result.updated;
    summary.totalSkipped += result.skipped;
    summary.totalErrors += result.errors;

    const logLine = `${progress} ${list.name} | ✅ ${result.updated} | ⊘ ${result.skipped} | ❌ ${result.errors}\n`;
    fs.appendFileSync(logFile, logLine);

    if (result.updated > 0) {
      console.log(`  ✅ ${result.updated} atualizadas | ⊘ ${result.skipped} ignoradas | ❌ ${result.errors} erros`);
    } else {
      console.log(`  ⊘ Nada para atualizar`);
    }

    // Salvar progresso a cada 10 listas
    if ((i + 1) % 10 === 0) {
      fs.writeFileSync('update-summary.json', JSON.stringify(summary, null, 2));
      console.log(`\n💾 Progresso salvo: ${summary.processedLists} listas processadas\n`);
    }
  }

  fs.appendFileSync(logFile, `\nFim: ${new Date().toISOString()}\n`);
  fs.writeFileSync('update-summary.json', JSON.stringify(summary, null, 2));

  console.log('\n\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════════════');
  console.log(`📋 Listas processadas: ${summary.processedLists}/${summary.totalLists}`);
  console.log(`✅ Tasks atualizadas: ${summary.totalUpdated}`);
  console.log(`⊘ Tasks ignoradas: ${summary.totalSkipped}`);
  console.log(`❌ Erros: ${summary.totalErrors}`);
  console.log(`\n💾 Logs salvos em: ${logFile}`);
  console.log(`💾 Resumo salvo em: update-summary.json`);
}

main().catch(console.error);
