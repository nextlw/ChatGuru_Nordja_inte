const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar mapeamento
const mapping = JSON.parse(fs.readFileSync('migration-mapping.json', 'utf8'));

// IDs dos campos ANTIGOS (que vamos atualizar)
const OLD_FIELD_IDS = {
  categoria: 'c19b4f95-1ff7-4966-b201-02905d33cec6',
  subCategoria: '330d635b-b0be-4a4a-960c-3ff974d597c3'
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
    console.error(`    ❌ Erro: ${error.response?.data?.err || error.message}`);
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
        allLists.push({ id: list.id, name: list.name, space: space.name });
      });
    }

    const foldersData = await makeRequest('GET', `https://api.clickup.com/api/v2/space/${space.id}/folder`);
    if (foldersData && foldersData.folders) {
      for (const folder of foldersData.folders) {
        const listsData = await makeRequest('GET', `https://api.clickup.com/api/v2/folder/${folder.id}/list`);
        if (listsData && listsData.lists) {
          listsData.lists.forEach(list => {
            allLists.push({ id: list.id, name: list.name, space: space.name });
          });
        }
      }
    }
  }

  return allLists;
}

async function updateListTasks(listId, listName, targetCategoriaValue = null) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) {
    return { updated: 0, skipped: 0, errors: 0 };
  }

  const tasks = tasksData.tasks;
  let updated = 0, skipped = 0, errors = 0;

  for (const task of tasks) {
    if (!task.custom_fields || task.custom_fields.length === 0) {
      skipped++;
      continue;
    }

    // Encontrar campo Categoria antigo
    const categoriaField = task.custom_fields.find(f => f.name === 'Categoria');
    const subCategoriaField = task.custom_fields.find(f => f.name === 'Sub Categoria');

    if (!categoriaField || !categoriaField.value) {
      skipped++;
      continue;
    }

    // Se targetCategoriaValue foi especificado, filtrar
    if (targetCategoriaValue && categoriaField.value !== targetCategoriaValue) {
      skipped++;
      continue;
    }

    // Buscar mapeamento para Categoria
    const categoriaMapping = mapping.categoria_mapping[categoriaField.value];
    if (!categoriaMapping) {
      skipped++;
      continue;
    }

    // Buscar mapeamento para SubCategoria (se existir)
    let subCategoriaMapping = null;
    if (subCategoriaField && subCategoriaField.value) {
      subCategoriaMapping = mapping.subcategoria_mapping[subCategoriaField.value];
    }

    console.log(`  📝 ${task.name.substring(0, 60)}...`);
    console.log(`     Categoria: "${categoriaMapping.old_name}" → "${categoriaMapping.new_name}"`);

    // Atualizar Categoria
    const ok1 = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${OLD_FIELD_IDS.categoria}`,
      { value: categoriaMapping.new_id }
    );

    // Atualizar SubCategoria se houver mapeamento
    let ok2 = true;
    if (subCategoriaMapping) {
      console.log(`     SubCategoria: "${subCategoriaMapping.old_name}" → "${subCategoriaMapping.new_name}"`);
      ok2 = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${OLD_FIELD_IDS.subCategoria}`,
        { value: subCategoriaMapping.new_id }
      );
    }

    if (ok1 && ok2) {
      updated++;
      console.log(`     ✅ Atualizado\n`);
    } else {
      errors++;
      console.log(`     ❌ Erro\n`);
    }

    await new Promise(resolve => setTimeout(resolve, 250));
  }

  return { updated, skipped, errors };
}

async function main() {
  console.log('🚀 ATUALIZAÇÃO DE VALORES DOS CAMPOS ANTIGOS\n');
  console.log('📌 Atualiza valores do campo "Categoria" (antigo)');
  console.log('📌 Atualiza valores do campo "Sub Categoria" (antigo)\n');

  // Se especificar lista e categoria específica
  const listIdArg = process.argv[2];
  const categoriaValue = process.argv[3]; // ID do valor antigo da categoria (ex: a4a4e85c-4eb5-44f9-9175-f98594da5c70 para ADM)

  if (listIdArg) {
    console.log(`📋 Processando lista: ${listIdArg}\n`);

    if (categoriaValue) {
      const catMapping = mapping.categoria_mapping[categoriaValue];
      if (catMapping) {
        console.log(`🎯 Filtrando apenas: "${catMapping.old_name}" → "${catMapping.new_name}"\n`);
      }
    }

    const result = await updateListTasks(listIdArg, 'Lista especificada', categoriaValue);

    console.log(`\n📊 RESULTADO`);
    console.log(`✅ Atualizadas: ${result.updated}`);
    console.log(`⊘ Ignoradas: ${result.skipped}`);
    console.log(`❌ Erros: ${result.errors}`);
    return;
  }

  // Processar todas as listas
  console.log('📂 Carregando listas...');
  const lists = await getAllLists();
  console.log(`✅ ${lists.length} listas\n`);

  const summary = {
    totalLists: lists.length,
    processedLists: 0,
    totalUpdated: 0,
    totalSkipped: 0,
    totalErrors: 0
  };

  const logFile = 'update-old-fields-log.txt';
  fs.writeFileSync(logFile, `Atualização iniciada: ${new Date().toISOString()}\n\n`);

  for (let i = 0; i < lists.length; i++) {
    const list = lists[i];
    console.log(`\n[${i + 1}/${lists.length}] ${list.space} → ${list.name}`);

    const result = await updateListTasks(list.id, list.name, null);

    summary.processedLists++;
    summary.totalUpdated += result.updated;
    summary.totalSkipped += result.skipped;
    summary.totalErrors += result.errors;

    const logLine = `[${i + 1}/${lists.length}] ${list.name} | ✅ ${result.updated} | ⊘ ${result.skipped} | ❌ ${result.errors}\n`;
    fs.appendFileSync(logFile, logLine);

    if (result.updated > 0) {
      console.log(`  ✅ ${result.updated} atualizadas`);
    }

    // Salvar progresso a cada 10 listas
    if ((i + 1) % 10 === 0) {
      fs.writeFileSync('update-old-fields-summary.json', JSON.stringify(summary, null, 2));
      console.log(`\n💾 Progresso: ${summary.processedLists} listas | ${summary.totalUpdated} tasks atualizadas\n`);
    }
  }

  fs.appendFileSync(logFile, `\nAtualização concluída: ${new Date().toISOString()}\n`);
  fs.writeFileSync('update-old-fields-summary.json', JSON.stringify(summary, null, 2));

  console.log('\n\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════');
  console.log(`📋 Listas: ${summary.processedLists}/${summary.totalLists}`);
  console.log(`✅ Tasks atualizadas: ${summary.totalUpdated}`);
  console.log(`⊘ Tasks ignoradas: ${summary.totalSkipped}`);
  console.log(`❌ Erros: ${summary.totalErrors}`);
  console.log(`\n💾 Log: ${logFile}`);
  console.log(`💾 Resumo: update-old-fields-summary.json`);
}

main().catch(console.error);
