#!/usr/bin/env node

/**
 * Script para buscar categorias e subcategorias do ClickUp
 * e gerar JSON estruturado para atualizar o projeto
 */

const axios = require('axios');
const fs = require('fs');
const path = require('path');

// Configurações
const CONFIG = {
    CLICKUP_TOKEN: process.env.CLICKUP_API_TOKEN || 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657',
    CLICKUP_LIST_ID: process.env.CLICKUP_LIST_ID || '901300373349',
};

/**
 * Busca campos customizados do ClickUp
 */
async function fetchClickUpFields() {
    try {
        console.log('🔍 Buscando campos do ClickUp...');

        const response = await axios.get(
            `https://api.clickup.com/api/v2/list/${CONFIG.CLICKUP_LIST_ID}/field`,
            {
                headers: {
                    'Authorization': CONFIG.CLICKUP_TOKEN
                }
            }
        );

        console.log(`✅ Encontrados ${response.data.fields.length} campos customizados\n`);
        return response.data.fields;
    } catch (error) {
        console.error('❌ Erro ao buscar campos do ClickUp:', error.message);
        throw error;
    }
}

/**
 * Extrai categoria pai do nome da subcategoria
 * Formato esperado: "Nome da Subcategoria [Categoria Pai]"
 */
function extractParentCategory(subcatName) {
    const match = subcatName.match(/\[(.+?)\]$/);
    return match ? match[1] : null;
}

/**
 * Extrai estrelas do nome ou cor da subcategoria
 */
function extractStars(option) {
    // Mapeamento de cores para estrelas (baseado no padrão que você configurou)
    const colorToStars = {
        '#00C8FF': 1, // Azul claro
        '#7B68EE': 2, // Roxo
        '#FF6B00': 3, // Laranja
        '#FF0080': 4  // Rosa
    };

    // Primeiro tenta extrair do nome
    const starsMatch = option.name.match(/(\d+)\s*estrela/i);
    if (starsMatch) {
        return parseInt(starsMatch[1]);
    }

    // Se não encontrou no nome, usa a cor
    if (option.color && colorToStars[option.color]) {
        return colorToStars[option.color];
    }

    // Padrão: 1 estrela
    return 1;
}

/**
 * Processa os campos e gera estrutura de dados
 */
function processFields(fields) {
    const result = {
        last_updated: new Date().toISOString(),
        clickup_list_id: CONFIG.CLICKUP_LIST_ID,
        field_ids: {},
        categories: [],
        subcategories_map: {}
    };

    // Encontrar campos relevantes
    const categoryField = fields.find(f => f.name === 'Categoria*');
    const subcategoryField = fields.find(f => f.name === 'SubCategoria');
    const activityTypeField = fields.find(f => f.name === 'Tipo de Atividade');
    const statusField = fields.find(f => f.name === 'Status Back Office');

    if (!categoryField || !subcategoryField) {
        throw new Error('Campos "Categoria*" ou "SubCategoria" não encontrados!');
    }

    // Armazenar IDs dos campos
    result.field_ids = {
        category_field_id: categoryField.id,
        subcategory_field_id: subcategoryField.id,
        activity_type_field_id: activityTypeField?.id || '',
        status_field_id: statusField?.id || ''
    };

    // Processar categorias
    if (categoryField.type_config && categoryField.type_config.options) {
        result.categories = categoryField.type_config.options.map(opt => ({
            id: opt.id,
            name: opt.name,
            orderindex: opt.orderindex
        }));
    }

    // Processar subcategorias e agrupar por categoria pai
    if (subcategoryField.type_config && subcategoryField.type_config.options) {
        subcategoryField.type_config.options.forEach(opt => {
            const parentCategory = extractParentCategory(opt.name);
            const subcatName = opt.name.replace(/\s*\[.+?\]\s*$/, '').trim(); // Remove a tag [Categoria]
            const stars = extractStars(opt);

            if (parentCategory) {
                if (!result.subcategories_map[parentCategory]) {
                    result.subcategories_map[parentCategory] = [];
                }
                result.subcategories_map[parentCategory].push({
                    id: opt.id,
                    name: subcatName,
                    full_name: opt.name,
                    stars: stars,
                    color: opt.color || null,
                    orderindex: opt.orderindex
                });
            } else {
                // Subcategoria sem categoria pai definida
                if (!result.subcategories_map['_outros']) {
                    result.subcategories_map['_outros'] = [];
                }
                result.subcategories_map['_outros'].push({
                    id: opt.id,
                    name: opt.name,
                    full_name: opt.name,
                    stars: stars,
                    color: opt.color || null,
                    orderindex: opt.orderindex
                });
            }
        });
    }

    // Processar tipos de atividade
    if (activityTypeField && activityTypeField.type_config && activityTypeField.type_config.options) {
        result.activity_types = activityTypeField.type_config.options.map(opt => ({
            id: opt.id,
            name: opt.name
        }));
    }

    // Processar status
    if (statusField && statusField.type_config && statusField.type_config.options) {
        result.status_options = statusField.type_config.options.map(opt => ({
            id: opt.id,
            name: opt.name
        }));
    }

    return result;
}

/**
 * Função principal
 */
async function main() {
    console.log('🚀 Buscando dados atualizados do ClickUp...\n');

    try {
        // Buscar campos
        const fields = await fetchClickUpFields();

        // Processar campos
        const data = processFields(fields);

        // Exibir resultado
        console.log('📊 Dados processados:\n');
        console.log(`📋 Categorias (${data.categories.length}):`);
        data.categories.forEach(cat => {
            const subcount = data.subcategories_map[cat.name]?.length || 0;
            console.log(`  - ${cat.name} (ID: ${cat.id}) - ${subcount} subcategorias`);
        });

        console.log(`\n📋 Subcategorias por categoria:`);
        Object.keys(data.subcategories_map).forEach(catName => {
            const subs = data.subcategories_map[catName];
            console.log(`\n  ${catName} (${subs.length}):`);
            subs.forEach(sub => {
                console.log(`    - ${sub.name} (${sub.stars} estrela${sub.stars > 1 ? 's' : ''})`);
            });
        });

        // Salvar JSON
        const outputFile = path.join(__dirname, '..', 'config', 'clickup_categories.json');
        fs.writeFileSync(outputFile, JSON.stringify(data, null, 2), 'utf8');
        console.log(`\n✅ Dados salvos em: ${outputFile}`);

        // Exibir JSON completo
        console.log(`\n📄 JSON completo:`);
        console.log(JSON.stringify(data, null, 2));

        process.exit(0);
    } catch (error) {
        console.error('\n💥 Erro:', error.message);
        process.exit(1);
    }
}

// Executar
if (require.main === module) {
    main();
}

module.exports = { fetchClickUpFields, processFields };
