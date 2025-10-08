const axios = require('axios');
const fs = require('fs');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const WORKSPACE_ID = '9013037641';
const TARGET_FIELDS = ['Categoria', 'Sub Categoria', 'Categoria*', 'SubCategoria'];

const delay = ms => new Promise(resolve => setTimeout(resolve, ms));

async function makeRequest(url, params = {}) {
  try {
    const response = await axios.get(url, {
      headers: { 'Authorization': TOKEN, 'Content-Type': 'application/json' },
      params
    });
    await delay(100); // Evita rate limiting
    return response.data;
  } catch (error) {
    console.error(`Erro em ${url}:`, error.response?.status);
    return null;
  }
}

async function getAllData() {
  const results = [];
  let totalTasks = 0;

  console.log('🚀 Iniciando...\n');

  // 1. Buscar spaces
  const spacesData = await makeRequest(`https://api.clickup.com/api/v2/team/${WORKSPACE_ID}/space`);
  if (!spacesData) return;

  const spaces = spacesData.spaces;
  console.log(`✅ ${spaces.length} spaces\n`);

  for (let i = 0; i < spaces.length; i++) {
    const space = spaces[i];
    console.log(`[${i + 1}/${spaces.length}] ${space.name}`);

    // 2. Listas sem folder
    const folderlessData = await makeRequest(`https://api.clickup.com/api/v2/space/${space.id}/list`);
    if (folderlessData && folderlessData.lists) {
      for (const list of folderlessData.lists) {
        const tasksData = await makeRequest(`https://api.clickup.com/api/v2/list/${list.id}/task`, { include_closed: true });
        if (tasksData && tasksData.tasks) {
          tasksData.tasks.forEach(task => {
            totalTasks++;
            if (task.custom_fields && task.custom_fields.length > 0) {
              const relevantFields = task.custom_fields.filter(f => TARGET_FIELDS.includes(f.name));
              if (relevantFields.length > 0) {
                results.push({
                  task_id: task.id,
                  task_name: task.name,
                  task_url: task.url,
                  space: space.name,
                  list: list.name,
                  custom_fields: relevantFields.reduce((acc, f) => {
                    acc[f.name] = { id: f.id, type: f.type, value: f.value };
                    return acc;
                  }, {})
                });
              }
            }
          });
        }
      }
    }

    // 3. Folders
    const foldersData = await makeRequest(`https://api.clickup.com/api/v2/space/${space.id}/folder`);
    if (foldersData && foldersData.folders) {
      for (const folder of foldersData.folders) {
        const listsData = await makeRequest(`https://api.clickup.com/api/v2/folder/${folder.id}/list`);
        if (listsData && listsData.lists) {
          for (const list of listsData.lists) {
            const tasksData = await makeRequest(`https://api.clickup.com/api/v2/list/${list.id}/task`, { include_closed: true });
            if (tasksData && tasksData.tasks) {
              tasksData.tasks.forEach(task => {
                totalTasks++;
                if (task.custom_fields && task.custom_fields.length > 0) {
                  const relevantFields = task.custom_fields.filter(f => TARGET_FIELDS.includes(f.name));
                  if (relevantFields.length > 0) {
                    results.push({
                      task_id: task.id,
                      task_name: task.name,
                      task_url: task.url,
                      space: space.name,
                      list: list.name,
                      custom_fields: relevantFields.reduce((acc, f) => {
                        acc[f.name] = { id: f.id, type: f.type, value: f.value };
                        return acc;
                      }, {})
                    });
                  }
                }
              });
            }
          }
        }
      }
    }

    // Salvar progresso
    fs.writeFileSync('clickup-custom-fields-export.json', JSON.stringify(results, null, 2));
    console.log(`  ➤ ${totalTasks} tasks processadas | ${results.length} com campos relevantes\n`);
  }

  console.log('\n✅ CONCLUÍDO');
  console.log(`📝 Total: ${totalTasks} tasks`);
  console.log(`✨ Com campos: ${results.length} tasks`);
  console.log(`💾 Arquivo: clickup-custom-fields-export.json`);
}

getAllData().catch(console.error);
