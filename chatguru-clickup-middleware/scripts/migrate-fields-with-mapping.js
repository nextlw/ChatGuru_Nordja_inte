const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar mapeamento
const mapping = JSON.parse(fs.readFileSync('migration-mapping.json', 'utf8'));

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

async function migrateList(listId, listName) {
  console.log(`\n📋 ${listName}`);

  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: true });
  if (!tasksData || !tasksData.tasks) {
    console.log('  ⊘ Sem tasks');
    return { updated: 0, skipped: 0, errors: 0 };
  }

  const tasks = tasksData.tasks;
  let updated = 0, skipped = 0, errors = 0;

  for (const task of tasks) {
    if (!task.custom_fields || task.custom_fields.length === 0) {
      skipped++;
      continue;
    }

    // Mapear campos
    const fieldsMap = {};
    let categoriaOldId, subCategoriaOldId, categoriaNovaId, subCategoriaNovaId;

    task.custom_fields.forEach(field => {
      fieldsMap[field.name] = field;

      // Identificar IDs dos campos
      if (field.name === 'Categoria') categoriaOldId = field.id;
      if (field.name === 'Sub Categoria') subCategoriaOldId = field.id;
      if (field.name === 'Categoria*') categoriaNovaId = field.id;
      if (field.name === 'SubCategoria') subCategoriaNovaId = field.id;
    });

    const categoriaOld = fieldsMap['Categoria'];
    const subCategoriaOld = fieldsMap['Sub Categoria'];
    const categoriaNova = fieldsMap['Categoria*'];
    const subCategoriaNova = fieldsMap['SubCategoria'];

    // Verificar se precisa migrar
    if (!categoriaOld || !subCategoriaOld || !categoriaNova || !subCategoriaNova) {
      skipped++;
      continue;
    }

    // Se campos novos já estão preenchidos, pular
    if (categoriaNova.value || subCategoriaNova.value) {
      skipped++;
      continue;
    }

    // Se campos antigos estão vazios, pular
    if (!categoriaOld.value || !subCategoriaOld.value) {
      skipped++;
      continue;
    }

    // Buscar mapeamento
    const categoriaMapping = mapping.categoria_mapping[categoriaOld.value];
    const subCategoriaMapping = mapping.subcategoria_mapping[subCategoriaOld.value];

    if (!categoriaMapping || !subCategoriaMapping) {
      console.log(`  ⚠️  ${task.name.substring(0, 50)}... - Mapeamento não encontrado`);
      skipped++;
      continue;
    }

    // Aplicar migração
    console.log(`  📝 ${task.name.substring(0, 50)}...`);
    console.log(`     ${categoriaMapping.old_name} → ${categoriaMapping.new_name}`);
    console.log(`     ${subCategoriaMapping.old_name} → ${subCategoriaMapping.new_name}`);

    const ok1 = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${categoriaNovaId}`,
      { value: categoriaMapping.new_id }
    );

    const ok2 = await makeRequest(
      'POST',
      `https://api.clickup.com/api/v2/task/${task.id}/field/${subCategoriaNovaId}`,
      { value: subCategoriaMapping.new_id }
    );

    if (ok1 && ok2) {
      updated++;
      console.log(`     ✅ Migrado`);
    } else {
      errors++;
      console.log(`     ❌ Erro`);
    }

    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return { updated, skipped, errors };
}

async function main() {
  console.log('🚀 MIGRAÇÃO INTERPRETATIVA DE CAMPOS PERSONALIZADOS\n');
  console.log('📌 "Categoria" → "Categoria*"');
  console.log('📌 "Sub Categoria" → "SubCategoria"\n');

  const listIdArg = process.argv[2];

  if (listIdArg) {
    const listName = process.argv[3] || 'Lista especificada';
    const result = await migrateList(listIdArg, listName);

    console.log(`\n📊 RESULTADO`);
    console.log(`✅ Migradas: ${result.updated}`);
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

  const logFile = 'migration-log.txt';
  fs.writeFileSync(logFile, `Migração iniciada: ${new Date().toISOString()}\n\n`);

  for (let i = 0; i < lists.length; i++) {
    const list = lists[i];
    console.log(`\n[${i + 1}/${lists.length}] ${list.space} → ${list.name}`);

    const result = await migrateList(list.id, list.name);

    summary.processedLists++;
    summary.totalUpdated += result.updated;
    summary.totalSkipped += result.skipped;
    summary.totalErrors += result.errors;

    const logLine = `[${i + 1}/${lists.length}] ${list.name} | ✅ ${result.updated} | ⊘ ${result.skipped} | ❌ ${result.errors}\n`;
    fs.appendFileSync(logFile, logLine);

    if (result.updated > 0) {
      console.log(`  ✅ ${result.updated} migradas`);
    }

    // Salvar progresso a cada 10 listas
    if ((i + 1) % 10 === 0) {
      fs.writeFileSync('migration-summary.json', JSON.stringify(summary, null, 2));
      console.log(`\n💾 Progresso: ${summary.processedLists} listas | ${summary.totalUpdated} tasks migradas\n`);
    }
  }

  fs.appendFileSync(logFile, `\nMigração concluída: ${new Date().toISOString()}\n`);
  fs.writeFileSync('migration-summary.json', JSON.stringify(summary, null, 2));

  console.log('\n\n📊 RESUMO FINAL');
  console.log('═══════════════════════════════════════');
  console.log(`📋 Listas: ${summary.processedLists}/${summary.totalLists}`);
  console.log(`✅ Tasks migradas: ${summary.totalUpdated}`);
  console.log(`⊘ Tasks ignoradas: ${summary.totalSkipped}`);
  console.log(`❌ Erros: ${summary.totalErrors}`);
  console.log(`\n💾 Log: ${logFile}`);
  console.log(`💾 Resumo: migration-summary.json`);
}

main().catch(console.error);
