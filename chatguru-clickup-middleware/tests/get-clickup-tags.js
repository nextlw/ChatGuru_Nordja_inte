#!/usr/bin/env node

const axios = require('axios');

// Configurações
const CLICKUP_API_TOKEN = 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657';
const SPACE_ID = '90130086319'; // ID do space Georgia

async function getTags() {
  try {
    console.log('🏷️  Buscando tags do ClickUp...\n');
    
    // Buscar tags do space
    const response = await axios.get(`https://api.clickup.com/api/v2/space/${SPACE_ID}/tag`, {
      headers: {
        'Authorization': CLICKUP_API_TOKEN,
        'Content-Type': 'application/json'
      }
    });

    console.log('Tags disponíveis no Space:');
    console.log('=' .repeat(50));
    
    if (response.data.tags && response.data.tags.length > 0) {
      response.data.tags.forEach((tag, index) => {
        console.log(`\n${index + 1}. Nome: ${tag.name}`);
        if (tag.tag_fg) console.log(`   Cor texto: ${tag.tag_fg}`);
        if (tag.tag_bg) console.log(`   Cor fundo: ${tag.tag_bg}`);
        if (tag.creator) console.log(`   Criador: ${tag.creator}`);
      });
    } else {
      console.log('Nenhuma tag encontrada no space.');
    }

    console.log('\n' + '=' .repeat(50));
    console.log('\nResposta completa:');
    console.log(JSON.stringify(response.data, null, 2));

  } catch (error) {
    console.error('❌ Erro ao buscar tags:');
    if (error.response) {
      console.log('Status:', error.response.status);
      console.log('Erro:', error.response.data);
    } else {
      console.log('Erro:', error.message);
    }
  }
}

// Executar
getTags();