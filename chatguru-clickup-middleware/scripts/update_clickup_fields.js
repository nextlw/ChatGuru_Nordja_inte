#!/usr/bin/env node

/**
 * Script para atualizar o arquivo estático de campos do ClickUp
 * Executa periodicamente para manter os campos atualizados
 */

const axios = require('axios');
const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

// Configurações
const CONFIG = {
    CLICKUP_TOKEN: process.env.CLICKUP_API_TOKEN || 'pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657',
    CLICKUP_LIST_ID: process.env.CLICKUP_LIST_ID || '901300373349',
    OUTPUT_FILE: path.join(__dirname, '..', 'config', 'clickup_fields_static.yaml'),
    BACKUP_FILE: path.join(__dirname, '..', 'config', 'clickup_fields_static.backup.yaml'),
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
        
        console.log(`✅ Encontrados ${response.data.fields.length} campos customizados`);
        return response.data.fields;
    } catch (error) {
        console.error('❌ Erro ao buscar campos do ClickUp:', error.message);
        throw error;
    }
}

/**
 * Processa os campos e extrai as informações relevantes
 */
function processFields(fields) {
    const processed = {
        last_updated: new Date().toISOString(),
        clickup_list_id: CONFIG.CLICKUP_LIST_ID,
        // Apenas campos relacionados a tarefas para o fallback da IA
        task_fields: {},
        task_dropdown_options: {}
    };
    
    // Campos relacionados a tarefas que devem ser incluídos no fallback
    const taskRelatedFields = [
        'Categoria',
        'Tipo de Atividade', 
        'Sub Categoria',
        'Status Back Office'
    ];
    
    for (const field of fields) {
        // Incluir apenas campos relacionados a tarefas para o fallback da IA
        if (taskRelatedFields.includes(field.name)) {
            // Guardar informação básica do campo
            processed.task_fields[field.name] = {
                id: field.id,
                type: field.type,
                name: field.name
            };
            
            // Se for dropdown, guardar as opções
            if (field.type === 'drop_down' && field.type_config && field.type_config.options) {
                processed.task_dropdown_options[field.name] = field.type_config.options.map(opt => ({
                    id: opt.id,
                    name: opt.name
                }));
            }
        }
    }
    
    // Extrair informações específicas para o prompt da IA (apenas tarefas)
    processed.categories = processed.task_dropdown_options['Categoria'] 
        ? processed.task_dropdown_options['Categoria'].map(opt => opt.name)
        : [];
    
    processed.activity_types = processed.task_dropdown_options['Tipo de Atividade']
        ? processed.task_dropdown_options['Tipo de Atividade'].map(opt => ({
            name: opt.name,
            description: getActivityTypeDescription(opt.name)
        }))
        : [];
    
    processed.status_options = processed.task_dropdown_options['Status Back Office']
        ? processed.task_dropdown_options['Status Back Office'].map(opt => opt.name)
        : [];
    
    processed.subcategories = processed.task_dropdown_options['Sub Categoria']
        ? processed.task_dropdown_options['Sub Categoria'].map(opt => opt.name)
        : [];
    
    // Mapeamentos de IDs para campos de tarefas
    processed.field_ids = {
        category_field_id: processed.task_fields['Categoria']?.id || '',
        subcategory_field_id: processed.task_fields['Sub Categoria']?.id || '',
        activity_type_field_id: processed.task_fields['Tipo de Atividade']?.id || '',
        status_field_id: processed.task_fields['Status Back Office']?.id || ''
    };
    
    return processed;
}

/**
 * Retorna a descrição do tipo de atividade
 */
function getActivityTypeDescription(typeName) {
    const descriptions = {
        'Rotineira': 'tarefas recorrentes e do dia a dia',
        'Especifica': 'tarefas pontuais com propósito específico',
        'Dedicada': 'tarefas que demandam dedicação especial'
    };
    return descriptions[typeName] || 'atividade';
}

/**
 * Salva os dados em arquivo YAML
 */
async function saveToYaml(data, filepath) {
    try {
        // Fazer backup se o arquivo existir
        if (fs.existsSync(filepath)) {
            console.log('📦 Fazendo backup do arquivo anterior...');
            fs.copyFileSync(filepath, CONFIG.BACKUP_FILE);
        }
        
        const yamlContent = yaml.dump(data, {
            indent: 2,
            lineWidth: -1,
            noRefs: true,
            sortKeys: false
        });
        
        fs.writeFileSync(filepath, yamlContent, 'utf8');
        console.log(`✅ Arquivo salvo em: ${filepath}`);
        
        // Log estatísticas
        console.log('\n📊 Estatísticas (campos de tarefas para fallback da IA):');
        console.log(`  - Categorias: ${data.categories.length}`);
        console.log(`  - Tipos de Atividade: ${data.activity_types.length}`);
        console.log(`  - Status Options: ${data.status_options.length}`);
        console.log(`  - Subcategorias: ${data.subcategories.length}`);
        console.log(`  - Total de campos de tarefas: ${Object.keys(data.task_fields).length}`);
        console.log('\n💡 Este arquivo contém apenas campos de tarefas para uso como fallback da IA');
        
    } catch (error) {
        console.error('❌ Erro ao salvar arquivo YAML:', error.message);
        throw error;
    }
}

/**
 * Função principal
 */
async function main() {
    console.log('🚀 Iniciando atualização dos campos do ClickUp...\n');
    
    try {
        // Buscar campos
        const fields = await fetchClickUpFields();
        
        // Processar campos
        const processedData = processFields(fields);
        
        // Salvar em YAML
        await saveToYaml(processedData, CONFIG.OUTPUT_FILE);
        
        console.log('\n✨ Atualização concluída com sucesso!');
        console.log(`⏰ Última atualização: ${processedData.last_updated}`);
        
        // Retornar 0 para sucesso
        process.exit(0);
    } catch (error) {
        console.error('\n💥 Erro durante a atualização:', error);
        
        // Tentar usar backup se existir
        if (fs.existsSync(CONFIG.BACKUP_FILE)) {
            console.log('🔄 Restaurando backup...');
            fs.copyFileSync(CONFIG.BACKUP_FILE, CONFIG.OUTPUT_FILE);
        }
        
        // Retornar 1 para erro
        process.exit(1);
    }
}

// Executar se for chamado diretamente
if (require.main === module) {
    main();
}

module.exports = { fetchClickUpFields, processFields };