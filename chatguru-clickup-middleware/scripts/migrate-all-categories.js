const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar mapeamento
const mapping = JSON.parse(fs.readFileSync('migration-mapping.json', 'utf8'));

// ID do campo Categoria antigo
const OLD_CATEGORIA_FIELD_ID = 'c19b4f95-1ff7-4966-b201-02905d33cec6';
const OLD_SUBCATEGORIA_FIELD_ID = '330d635b-b0be-4a4a-960c-3ff974d597c3';

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
  if (!tasksData || !tasksData.tasks) return { updated: 0, errors: 0 };

  let updated = 0;
  let errors = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields) continue;

    const categoriaField = task.custom_fields.find(f => f.name === 'Categoria');
    const subCategoriaField = task.custom_fields.find(f => f.name === 'Sub Categoria');

    let taskUpdated = false;

    // Atualizar Categoria se houver mapeamento
    if (categoriaField && categoriaField.value) {
      const categoriaMapping = mapping.categoria_mapping[categoriaField.value];

      if (categoriaMapping) {
        const result = await makeRequest(
          'POST',
          `https://api.clickup.com/api/v2/task/${task.id}/field/${OLD_CATEGORIA_FIELD_ID}`,
          { value: categoriaMapping.new_id }
        );

        if (result) {
          taskUpdated = true;
          logUpdate(task.name, 'Categoria', categoriaMapping.old_name, categoriaMapping.new_name);
        } else {
          errors++;
        }
      }
    }

    // Atualizar SubCategoria se houver mapeamento
    if (subCategoriaField && subCategoriaField.value) {
      const subCategoriaMapping = mapping.subcategoria_mapping[subCategoriaField.value];

      if (subCategoriaMapping) {
        const result = await makeRequest(
          'POST',
          `https://api.clickup.com/api/v2/task/${task.id}/field/${OLD_SUBCATEGORIA_FIELD_ID}`,
          { value: subCategoriaMapping.new_id }
        );

        if (result) {
          taskUpdated = true;
          logUpdate(task.name, 'SubCategoria', subCategoriaMapping.old_name, subCategoriaMapping.new_name);
        } else {
          errors++;
        }
      }
    }

    if (taskUpdated) {
      updated++;
    }

    await new Promise(resolve => setTimeout(resolve, 150));
  }

  return { updated, errors };
}

function logUpdate(taskName, fieldType, oldValue, newValue) {
  const timestamp = new Date().toISOString();
  const logLine = `[${timestamp}] ${fieldType}: "${oldValue}" → "${newValue}" | Task: ${taskName.substring(0, 80)}\n`;
  fs.appendFileSync('migration-detailed.log', logLine);
}

async function main() {
  const startTime = new Date();

  console.log('🚀 MIGRAÇÃO COMPLETA DE CATEGORIAS E SUBCATEGORIAS');
  console.log(`📅 Início: ${startTime.toISOString()}\n`);

  // Inicializar arquivos de log
  fs.writeFileSync('migration-detailed.log', `Migração iniciada: ${startTime.toISOString()}\n\n`);
  fs.writeFileSync('migration-progress.log', `Migração iniciada: ${startTime.toISOString()}\n\n`);

  const summary = {
    start_time: startTime.toISOString(),
    total_lists: 0,
    processed_lists: 0,
    total_updated: 0,
    total_errors: 0,
    categoria_mappings: Object.keys(mapping.categoria_mapping).length,
    subcategoria_mappings: Object.keys(mapping.subcategoria_mapping).length
  };

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  let listsCount = 0;

  for (const space of spaces) {
    const progressLine = `\n🏢 Space: ${space.name}\n`;
    fs.appendFileSync('migration-progress.log', progressLine);
    console.log(progressLine.trim());

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsCount++;
      const result = await updateTasksInList(list.id);
      summary.processed_lists++;
      summary.total_updated += result.updated;
      summary.total_errors += result.errors;

      if (result.updated > 0) {
        const msg = `  ✅ Lista: ${list.name} | ${result.updated} tasks atualizadas\n`;
        fs.appendFileSync('migration-progress.log', msg);
        console.log(msg.trim());
      }

      // Salvar progresso a cada 20 listas
      if (listsCount % 20 === 0) {
        summary.total_lists = listsCount;
        fs.writeFileSync('migration-summary.json', JSON.stringify(summary, null, 2));
        console.log(`💾 Progresso: ${listsCount} listas | ${summary.total_updated} tasks atualizadas\n`);
      }
    }

    // Listas em folders
    const folders = await getFolders(space.id);
    for (const folder of folders) {
      const lists = await getLists(folder.id);
      for (const list of lists) {
        listsCount++;
        const result = await updateTasksInList(list.id);
        summary.processed_lists++;
        summary.total_updated += result.updated;
        summary.total_errors += result.errors;

        if (result.updated > 0) {
          const msg = `  ✅ ${folder.name} → ${list.name} | ${result.updated} tasks atualizadas\n`;
          fs.appendFileSync('migration-progress.log', msg);
          console.log(msg.trim());
        }

        // Salvar progresso a cada 20 listas
        if (listsCount % 20 === 0) {
          summary.total_lists = listsCount;
          fs.writeFileSync('migration-summary.json', JSON.stringify(summary, null, 2));
          console.log(`💾 Progresso: ${listsCount} listas | ${summary.total_updated} tasks atualizadas\n`);
        }
      }
    }
  }

  const endTime = new Date();
  const durationMinutes = ((endTime - startTime) / 1000 / 60).toFixed(2);

  summary.end_time = endTime.toISOString();
  summary.duration_minutes = durationMinutes;
  summary.total_lists = listsCount;

  fs.writeFileSync('migration-summary.json', JSON.stringify(summary, null, 2));

  const finalReport = `
\n\n📊 RESUMO FINAL DA MIGRAÇÃO
═══════════════════════════════════════════════════════
📅 Início: ${summary.start_time}
📅 Fim: ${summary.end_time}
⏱️  Duração: ${durationMinutes} minutos

📋 Listas processadas: ${summary.processed_lists}
✅ Tasks atualizadas: ${summary.total_updated}
❌ Erros: ${summary.total_errors}

📊 Mapeamentos aplicados:
   - Categorias: ${summary.categoria_mappings}
   - SubCategorias: ${summary.subcategoria_mappings}

💾 Arquivos gerados:
   - migration-summary.json (resumo completo)
   - migration-progress.log (progresso por lista)
   - migration-detailed.log (detalhes de cada atualização)
`;

  fs.appendFileSync('migration-progress.log', finalReport);
  console.log(finalReport);
}

main().catch(error => {
  console.error('❌ Erro fatal:', error);
  fs.appendFileSync('migration-progress.log', `\n\n❌ ERRO FATAL: ${error.message}\n`);
  process.exit(1);
});
