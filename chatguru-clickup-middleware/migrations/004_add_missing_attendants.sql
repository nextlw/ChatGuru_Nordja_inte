-- Migration 004: Adicionar atendentes faltantes
-- Data: 2025-10-10
-- Descrição: Adiciona os 14 atendentes completos do sistema

-- ==============================================================================
-- ATENDENTES COMPLETOS (14 total)
-- ==============================================================================

-- Remover mapeamento hardcoded de Gabriel que não existe
DELETE FROM attendant_mappings WHERE attendant_key = 'gabriel';

-- Inserir/Atualizar todos os 14 atendentes
INSERT INTO attendant_mappings (attendant_key, attendant_full_name, attendant_aliases, space_id) VALUES
-- Atendentes já existentes (atualizar)
('anne', 'Anne', ARRAY['Anne', 'Anne S', 'Anne Souza'], '90130178602'),
('bruna', 'Bruna Senhora', ARRAY['Bruna', 'Bruna S', 'Bruna Senhora'], '90130178610'),
('mariana_cruz', 'Mariana', ARRAY['Mariana', 'Mariana C', 'Mariana Cruz'], '90130178618'),
('mariana_medeiros', 'Mariana Medeiros', ARRAY['Mariana M', 'Mariana Medeiros'], '90130178626'),

-- Novos atendentes (inserir)
('carlos', 'Carlos', ARRAY['Carlos'], NULL),
('georgia', 'Georgia', ARRAY['Georgia'], NULL),
('graziella', 'Graziella', ARRAY['Graziella'], NULL),
('marilia', 'Marilia', ARRAY['Marilia'], NULL),
('natalia', 'Natalia', ARRAY['Natalia'], NULL),
('paloma', 'Paloma', ARRAY['Paloma'], NULL),
('renata', 'Renata', ARRAY['Renata'], NULL),
('thais', 'Thais Cotts', ARRAY['Thais', 'Thais C', 'Thais Cotts'], NULL),
('velma', 'Velma', ARRAY['Velma'], NULL),
('william', 'William', ARRAY['William', 'Will'], NULL)

ON CONFLICT (attendant_key) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    attendant_aliases = EXCLUDED.attendant_aliases,
    space_id = COALESCE(EXCLUDED.space_id, attendant_mappings.space_id),  -- Preservar space_id existente se novo for NULL
    updated_at = NOW();

-- ==============================================================================
-- ATUALIZAÇÃO DE MAPEAMENTOS DE CLIENTES
-- ==============================================================================

-- Atualizar clientes que estavam mapeados para Gabriel (que não existe mais)
-- e redirecionar para "Clientes Inativos"
UPDATE client_mappings
SET folder_path = REPLACE(folder_path, 'Gabriel Benarros', 'Clientes Inativos'),
    updated_at = NOW()
WHERE folder_path LIKE '%Gabriel Benarros%';

-- ==============================================================================
-- LOG DA MIGRAÇÃO
-- ==============================================================================

INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_004_applied', NOW()::text, 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- VERIFICAÇÃO
-- ==============================================================================

-- Contar atendentes
DO $$
DECLARE
    total_attendants INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_attendants FROM attendant_mappings WHERE is_active = true;
    RAISE NOTICE 'Total de atendentes ativos: %', total_attendants;

    IF total_attendants <> 14 THEN
        RAISE WARNING 'Esperado 14 atendentes, encontrado %', total_attendants;
    ELSE
        RAISE NOTICE '✅ 14 atendentes cadastrados corretamente';
    END IF;
END $$;
