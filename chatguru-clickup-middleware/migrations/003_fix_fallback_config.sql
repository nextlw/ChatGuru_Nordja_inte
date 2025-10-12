-- Migration 003: Correção dos fallbacks para compatibilidade com sistema dinâmico
-- Data: 2025-10-10
-- Descrição: Remove fallbacks fixos que direcionam para lista específica do Gabriel

-- ==============================================================================
-- ATUALIZAÇÃO DOS FALLBACKS PARA SISTEMA DINÂMICO
-- ==============================================================================

-- Atualizar fallback_folder_id para ser space ID do "Clientes Inativos" ao invés de lista específica
UPDATE prompt_config 
SET value = '90130085983', updated_at = NOW()
WHERE key = 'fallback_folder_id' AND is_active = true;

-- Atualizar fallback_folder_path para refletir o space dinâmico
UPDATE prompt_config 
SET value = 'Clientes Inativos', updated_at = NOW()
WHERE key = 'fallback_folder_path' AND is_active = true;

-- Adicionar configuração para space ID padrão do sistema dinâmico
INSERT INTO prompt_config (key, value, config_type) VALUES
('default_inactive_space_id', '90130085983', 'text'),
('dynamic_structure_enabled', 'true', 'boolean')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- MAPEAMENTO DOS ATENDENTES COM SPACE IDs (dados reais do sistema)
-- ==============================================================================

-- Atualizar mapeamentos de atendentes com space IDs corretos
UPDATE attendant_mappings 
SET space_id = '90130178602', updated_at = NOW()
WHERE attendant_key = 'anne';

UPDATE attendant_mappings 
SET space_id = '90130178610', updated_at = NOW()
WHERE attendant_key = 'bruna';

UPDATE attendant_mappings 
SET space_id = '90130178618', updated_at = NOW()
WHERE attendant_key = 'mariana_cruz';

UPDATE attendant_mappings 
SET space_id = '90130178626', updated_at = NOW()
WHERE attendant_key = 'mariana_medeiros';

UPDATE attendant_mappings 
SET space_id = '90130178634', updated_at = NOW()
WHERE attendant_key = 'gabriel';

-- ==============================================================================
-- LIMPEZA DE CACHE PARA FORÇAR REGENERAÇÃO
-- ==============================================================================

-- Marcar cache de listas como inativo para forçar regeneração com nova estrutura
UPDATE list_cache 
SET is_active = false, last_verified = NOW()
WHERE list_id = '901300373349'; -- Lista específica do Gabriel que estava sendo usada como fallback

-- ==============================================================================
-- LOG DA MIGRAÇÃO
-- ==============================================================================

-- Inserir log da migração
INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_003_applied', NOW()::text, 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();