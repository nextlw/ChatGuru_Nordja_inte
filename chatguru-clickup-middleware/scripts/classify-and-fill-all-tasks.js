const axios = require('axios');
const fs = require('fs');
const yaml = require('js-yaml');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';

// Carregar configuração YAML
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

function classifyTask(taskName) {
  const nameLower = taskName.toLowerCase();

  // Procurar por palavras-chave para classificar
  for (const [categoria, subcategorias] of Object.entries(config.subcategory_mappings)) {
    for (const subcat of subcategorias) {
      const keywords = subcat.name.toLowerCase().split(/[\/\(\)]/);

      for (const keyword of keywords) {
        const cleanKeyword = keyword.trim();
        if (cleanKeyword.length > 3 && nameLower.includes(cleanKeyword)) {
          return {
            categoria: categoria,
            categoriaId: config.category_mappings[categoria].id,
            subCategoria: subcat.name,
            subCategoriaId: subcat.id,
            estrelas: subcat.stars
          };
        }
      }
    }
  }

  return null;
}

async function updateTaskFields(listId) {
  const tasksData = await makeRequest('GET', `https://api.clickup.com/api/v2/list/${listId}/task`, { include_closed: false });
  if (!tasksData || !tasksData.tasks) return { updated: 0, skipped: 0 };

  let updated = 0;
  let skipped = 0;

  for (const task of tasksData.tasks) {
    if (!task.custom_fields) {
      skipped++;
      continue;
    }

    // Verificar se já tem os campos preenchidos
    const categoriaNovaField = task.custom_fields.find(f => f.name === 'Categoria_nova');
    const subCategoriaNovaField = task.custom_fields.find(f => f.name === 'SubCategoria_nova');
    const estrelasField = task.custom_fields.find(f => f.name === 'Estrelas');

    if (!categoriaNovaField || !subCategoriaNovaField || !estrelasField) {
      skipped++;
      continue;
    }

    // Se já estão preenchidos, pular
    if (categoriaNovaField.value && subCategoriaNovaField.value && estrelasField.value) {
      skipped++;
      continue;
    }

    // Classificar a task
    const classification = classifyTask(task.name);
    if (!classification) {
      skipped++;
      continue;
    }

    console.log(`    📝 ${task.name.substring(0, 60)}...`);
    console.log(`       ${classification.categoria} > ${classification.subCategoria} (${classification.estrelas}⭐)`);

    // Atualizar campos
    let success = true;

    if (!categoriaNovaField.value) {
      const result1 = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${categoriaNovaField.id}`,
        { value: classification.categoriaId }
      );
      if (!result1) success = false;
    }

    if (!subCategoriaNovaField.value && success) {
      const result2 = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${subCategoriaNovaField.id}`,
        { value: classification.subCategoriaId }
      );
      if (!result2) success = false;
    }

    if (!estrelasField.value && success) {
      const result3 = await makeRequest(
        'POST',
        `https://api.clickup.com/api/v2/task/${task.id}/field/${estrelasField.id}`,
        { value: classification.estrelas }
      );
      if (!result3) success = false;
    }

    if (success) {
      updated++;
      console.log(`       ✅ Atualizado\n`);
    } else {
      console.log(`       ❌ Erro\n`);
    }

    await new Promise(resolve => setTimeout(resolve, 200));
  }

  return { updated, skipped };
}

async function main() {
  console.log('🚀 CLASSIFICAÇÃO AUTOMÁTICA DE TODAS AS TASKS\n');
  console.log('📊 Usando hierarquia do ai_prompt.yaml\n');

  const logFile = 'classification-log.txt';
  fs.writeFileSync(logFile, `Início: ${new Date().toISOString()}\n\n`);

  let totalUpdated = 0;
  let totalSkipped = 0;
  let listsProcessed = 0;

  const spaces = await getAllSpaces();
  console.log(`✅ ${spaces.length} spaces\n`);

  for (const space of spaces) {
    console.log(`🏢 ${space.name}`);

    const folderlessLists = await getFolderlessLists(space.id);
    for (const list of folderlessLists) {
      listsProcessed++;
      const result = await updateTaskFields(list.id);
      totalUpdated += result.updated;
      totalSkipped += result.skipped;

      if (result.updated > 0) {
        const msg = `  ✅ ${list.name} → ${result.updated} tasks\n`;
        console.log(msg.trim());
        fs.appendFileSync(logFile, msg);
      }
    }

    const folders = await getFolders(space.id);
    for (const folder of folders) {
      const lists = await getLists(folder.id);
      for (const list of lists) {
        listsProcessed++;
        const result = await updateTaskFields(list.id);
        totalUpdated += result.updated;
        totalSkipped += result.skipped;

        if (result.updated > 0) {
          const msg = `  ✅ ${folder.name} → ${list.name} → ${result.updated} tasks\n`;
          console.log(msg.trim());
          fs.appendFileSync(logFile, msg);
        }

        // Salvar progresso a cada 50 listas
        if (listsProcessed % 50 === 0) {
          console.log(`\n💾 Progresso: ${listsProcessed} listas | ${totalUpdated} tasks classificadas\n`);
        }
      }
    }
  }

  const summary = `
\n📊 RESUMO FINAL
═══════════════════════════════════════
📋 Listas: ${listsProcessed}
✅ Tasks classificadas: ${totalUpdated}
⊘ Tasks ignoradas: ${totalSkipped}

💾 Log: ${logFile}
`;

  console.log(summary);
  fs.appendFileSync(logFile, summary);
}

main().catch(error => {
  console.error('❌ Erro:', error);
  process.exit(1);
});
