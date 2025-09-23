const axios = require('axios');

const TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';

async function listarWorkspacesEListas() {
  console.log('🔍 Buscando workspaces e listas disponíveis...\n');
  
  try {
    // Listar workspaces/teams
    const teamsResponse = await axios.get('https://api.clickup.com/api/v2/team', {
      headers: { 'Authorization': TOKEN },
      timeout: 10000
    });
    
    console.log('🏢 WORKSPACES DISPONÍVEIS:');
    for (const team of teamsResponse.data.teams) {
      console.log(`\n📋 Workspace: ${team.name} (ID: ${team.id})`);
      
      // Listar spaces do workspace
      try {
        const spacesResponse = await axios.get(`https://api.clickup.com/api/v2/team/${team.id}/space`, {
          headers: { 'Authorization': TOKEN },
          timeout: 5000
        });
        
        for (const space of spacesResponse.data.spaces) {
          console.log(`  📂 Space: ${space.name} (ID: ${space.id})`);
          
          // Listar folders do space
          try {
            const foldersResponse = await axios.get(`https://api.clickup.com/api/v2/space/${space.id}/folder`, {
              headers: { 'Authorization': TOKEN },
              timeout: 5000
            });
            
            for (const folder of foldersResponse.data.folders) {
              console.log(`    📁 Folder: ${folder.name} (ID: ${folder.id})`);
              
              // Listar listas do folder
              try {
                const listsResponse = await axios.get(`https://api.clickup.com/api/v2/folder/${folder.id}/list`, {
                  headers: { 'Authorization': TOKEN },
                  timeout: 5000
                });
                
                for (const list of listsResponse.data.lists) {
                  console.log(`      📝 Lista: ${list.name} (ID: ${list.id})`);
                }
              } catch (err) {
                console.log(`      ❌ Erro ao listar listas do folder: ${err.response?.data?.err || err.message}`);
              }
            }
            
            // Listar listas diretas do space (sem folder)
            try {
              const spaceListsResponse = await axios.get(`https://api.clickup.com/api/v2/space/${space.id}/list`, {
                headers: { 'Authorization': TOKEN },
                timeout: 5000
              });
              
              if (spaceListsResponse.data.lists.length > 0) {
                console.log(`    📝 Listas diretas do space:`);
                for (const list of spaceListsResponse.data.lists) {
                  console.log(`      📋 Lista: ${list.name} (ID: ${list.id})`);
                }
              }
            } catch (err) {
              console.log(`    ❌ Erro ao listar listas do space: ${err.response?.data?.err || err.message}`);
            }
            
          } catch (err) {
            console.log(`    ❌ Erro ao listar folders: ${err.response?.data?.err || err.message}`);
          }
        }
      } catch (err) {
        console.log(`  ❌ Erro ao listar spaces: ${err.response?.data?.err || err.message}`);
      }
    }
    
  } catch (error) {
    console.log('❌ Erro ao listar workspaces:');
    console.log(`Erro: ${error.response?.data?.err || error.message}`);
  }
}

listarWorkspacesEListas();
