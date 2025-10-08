const axios = require('axios');
const fs = require('fs');
const yaml = require('js-yaml');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar mapeamentos
const migrationMapping = JSON.parse(fs.readFileSync('migration-mapping.json', 'utf8'));
const config = yaml.load(fs.readFileSync('../config/ai_prompt.yaml', 'utf8'));

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

function findStarsForSubcategory(subcategoryId) {
  // Procurar estrelas na configuração YAML
  for (const [categoria, subcategorias] of Object.entries(config.subcategory_mappings)) {
    const subcat = subcategorias.find(s => s.id === subcategoryId);
    if (subcat) {
      return subcat.stars;
    }
  }
  return null;
}

async function processTasksInList(listId, listName) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, {
    include_closed: true,
    page: 0
  });

  if (!tasksData || !tasksData.tasks) return { updated: 0, skipped: 0, errors: 0 };

  let updated = 0;
  let skipped = 0;
  let errors = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields || task.custom_fields.length === 0) {
      skipped++;
      continue;
    }

    // Encontrar campos ANTIGOS
    const categoriaOld = task.custom_fields.find(f => f.name === 'Categoria');
    const subCategoriaOld = task.custom_fields.find(f => f.name === 'Sub Categoria');

    // Encontrar campos NOVOS
    const categoriaNova = task.custom_fields.find(f => f.name === 'Categoria_nova');
    const subCategoriaNova = task.custom_fields.find(f => f.name === 'SubCategoria_nova');
    const estrelas = task.custom_fields.find(f => f.name === 'Estrelas');

    // Se não tem campos antigos com valor ou campos novos não existem, pular
    if (!categoriaOld || !categoriaOld.value || !categoriaNova) {
      skipped++;
      continue;
    }

    // Se campos novos já estão preenchidos, pular
    if (categoriaNova.value && subCategoriaNova?.value && estrelas?.value) {
      skipped++;
      continue;
    }

    // Buscar mapeamento da categoria antiga
    const categoriaMapping = migrationMapping.categoria_mapping[categoriaOld.value];
    if (!categoriaMapping) {
      skipped++;
      continue;
    }

    console.log(`    📝 ${task.name.substring(0, 60)}...`);
    console.log(`       Categoria: "${categoriaMapping.old_name}" → "${categoriaMapping.new_name}"`);

    let taskSuccess = true;

    // Atualizar Categoria_nova
    if (!categoriaNova.value) {
      const result = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${categoriaNova.id}`,
        { value: categoriaMapping.new_id }
      );
      if (!result) {
        taskSuccess = false;
        errors++;
      }
    }

    // Atualizar SubCategoria_nova se houver
    if (subCategoriaOld && subCategoriaOld.value && subCategoriaNova && !subCategoriaNova.value && taskSuccess) {
      const subCategoriaMapping = migrationMapping.subcategoria_mapping[subCategoriaOld.value];

      if (subCategoriaMapping) {
        console.log(`       SubCategoria: "${subCategoriaMapping.old_name}" → "${subCategoriaMapping.new_name}"`);

        const result = await makeRequest(
          'POST',
          `https://api.clickup.com/api/v2/task/${task.id}/field/${subCategoriaNova.id}`,
          { value: subCategoriaMapping.new_id }
        );

        if (!result) {
          taskSuccess = false;
          errors++;
        } else {
          // Atualizar Estrelas baseado na subcategoria
          if (estrelas && !estrelas.value) {
            const stars = findStarsForSubcategory(subCategoriaMapping.new_id);
            if (stars) {
              console.log(`       Estrelas: ${stars}⭐`);

              const result2 = await makeRequest(
                'POST',
                `https://api.clickup.com/api/v2/task/${task.id}/field/${estrelas.id}`,
                { value: stars }
              );

              if (!result2) {
                taskSuccess = false;
                errors++;
              }
            }
          }
        }
      }
    }

    if (taskSuccess) {
      updated++;
      console.log(`       ✅ Migrado com sucesso\n`);
    } else {
      console.log(`       ❌ Erro ao migrar\n`);
    }

    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return { updated, skipped, errors };
}

async function main() {
  console.log('🚀 MIGRAÇÃO DOS CAMPOS ANTIGOS PARA NOVOS\n');
  console.log('📊 Usando mapeamento de migration-mapping.json\n');
  console.log('📝 Categoria → Categoria_nova');
  console.log('📝 Sub Categoria → SubCategoria_nova');
  console.log('📝 Estrelas baseadas na subcategoria\n');

  const logFile = 'migration-to-new-fields.log';
  fs.writeFileSync(logFile, `Início: ${new Date().toISOString()}\n\n`);

  let totalUpdated = 0;
  let totalSkipped = 0;
  let totalErrors = 0;
  let listsProcessed = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces encontrados\n`);

  for (const space of spaces) {
    console.log(`\n🏢 Space: ${space.name}`);

    // Listas sem folder
    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsProcessed++;
      const result = await processTasksInList(list.id, list.name);
      totalUpdated += result.updated;
      totalSkipped += result.skipped;
      totalErrors += result.errors;

      if (result.updated > 0) {
        const msg = `  ✅ Lista: ${list.name} → ${result.updated} tasks migradas\n`;
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
        const result = await processTasksInList(list.id, list.name);
        totalUpdated += result.updated;
        totalSkipped += result.skipped;
        totalErrors += result.errors;

        if (result.updated > 0) {
          const msg = `  ✅ ${folder.name} → ${list.name} → ${result.updated} tasks migradas\n`;
          console.log(msg.trim());
          fs.appendFileSync(logFile, msg);
        }

        // Progresso a cada 50 listas
        if (listsProcessed % 50 === 0) {
          console.log(`\n💾 Progresso: ${listsProcessed} listas | ${totalUpdated} tasks migradas\n`);
          fs.writeFileSync('migration-progress.json', JSON.stringify({
            lists_processed: listsProcessed,
            tasks_updated: totalUpdated,
            tasks_skipped: totalSkipped,
            errors: totalErrors
          }, null, 2));
        }
      }
    }
  }

  const summary = `
\n📊 RESUMO FINAL
═══════════════════════════════════════
📋 Listas processadas: ${listsProcessed}
✅ Tasks migradas: ${totalUpdated}
⊘ Tasks ignoradas: ${totalSkipped}
❌ Erros: ${totalErrors}

💾 Log: ${logFile}
💾 Progresso: migration-progress.json
`;

  console.log(summary);
  fs.appendFileSync(logFile, summary);
}

main().catch(error => {
  console.error('❌ Erro fatal:', error);
  process.exit(1);
});
