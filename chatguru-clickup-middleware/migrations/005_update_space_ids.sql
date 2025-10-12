-- Migration 005: Atualizar Space IDs dos Atendentes
-- Data: 2025-10-10
-- Descrição: Atualiza space_id de todos os atendentes com valores corretos da API do ClickUp

-- ==============================================================================
-- MAPEAMENTO DE SPACE IDs CORRETOS (consultado via API)
-- ==============================================================================

-- ATENDENTES COM SPACE IDs CORRETOS (validados):
-- Anne Souza       → 90131713706 (API) vs 90130178602 (Banco) ❌ DIFERENTE!
-- Bruna Senhora    → 90132952032 (API) vs 90130178610 (Banco) ❌ DIFERENTE!
-- Mariana Cruz     → 90134505966 (API) vs 90130178618 (Banco) ❌ DIFERENTE!
-- Mariana Medeiros → 90134254183 (API) vs 90130178626 (Banco) ❌ DIFERENTE!

-- ATENDENTES NOVOS COM SPACE IDs (da API):
-- Carlos Ribeiro     → Não tem space próprio (usar Clientes Inativos)
-- Georgia Schreiber  → 90130086319
-- Graziella Leite    → 90134506045
-- Marilia Moura      → Não tem space próprio (usar Clientes Inativos)
-- Natalia Branco     → 901310948326
-- Paloma Lira        → Não tem space próprio (usar Clientes Inativos)
-- Renata Schnoor     → 90131747051
-- Thais Cotts        → 90131747001
-- Velma Fortes       → 90130187145
-- William            → Não tem space próprio (usar Clientes Inativos)

-- ==============================================================================
-- ATUALIZAÇÃO DOS SPACE IDs
-- ==============================================================================

-- Atualizar atendentes existentes com IDs CORRETOS da API
UPDATE attendant_mappings SET space_id = '90131713706', updated_at = NOW() WHERE attendant_key = 'anne';
UPDATE attendant_mappings SET space_id = '90132952032', updated_at = NOW() WHERE attendant_key = 'bruna';
UPDATE attendant_mappings SET space_id = '90134505966', updated_at = NOW() WHERE attendant_key = 'mariana_cruz';
UPDATE attendant_mappings SET space_id = '90134254183', updated_at = NOW() WHERE attendant_key = 'mariana_medeiros';

-- Adicionar space IDs para atendentes novos que TÊM space próprio
UPDATE attendant_mappings SET space_id = '90130086319', updated_at = NOW() WHERE attendant_key = 'georgia';
UPDATE attendant_mappings SET space_id = '90134506045', updated_at = NOW() WHERE attendant_key = 'graziella';
UPDATE attendant_mappings SET space_id = '901310948326', updated_at = NOW() WHERE attendant_key = 'natalia';
UPDATE attendant_mappings SET space_id = '90131747051', updated_at = NOW() WHERE attendant_key = 'renata';
UPDATE attendant_mappings SET space_id = '90131747001', updated_at = NOW() WHERE attendant_key = 'thais';
UPDATE attendant_mappings SET space_id = '90130187145', updated_at = NOW() WHERE attendant_key = 'velma';

-- Atendentes SEM space próprio (Carlos, Marilia, Paloma, William)
-- Configurar space_id como "Clientes Inativos" (90130085983)
UPDATE attendant_mappings SET space_id = '90130085983', updated_at = NOW() WHERE attendant_key = 'carlos';
UPDATE attendant_mappings SET space_id = '90130085983', updated_at = NOW() WHERE attendant_key = 'marilia';
UPDATE attendant_mappings SET space_id = '90130085983', updated_at = NOW() WHERE attendant_key = 'paloma';
UPDATE attendant_mappings SET space_id = '90130085983', updated_at = NOW() WHERE attendant_key = 'william';

-- ==============================================================================
-- ADICIONAR ALIASES FALTANTES
-- ==============================================================================

-- Adicionar aliases baseados nos emails/usernames da API
UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Carlos', 'Carlos Ribeiro', 'Carlos R'],
    updated_at = NOW()
WHERE attendant_key = 'carlos';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Georgia', 'Georgia Schreiber', 'Georgia S'],
    updated_at = NOW()
WHERE attendant_key = 'georgia';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Graziella', 'Graziella Leite', 'Graziella L'],
    updated_at = NOW()
WHERE attendant_key = 'graziella';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Marilia', 'Marília', 'Marilia Moura', 'Marília Moura'],
    updated_at = NOW()
WHERE attendant_key = 'marilia';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Natalia', 'Natália', 'Natalia Branco', 'Natália Branco'],
    updated_at = NOW()
WHERE attendant_key = 'natalia';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Paloma', 'Paloma Lira', 'Paloma L'],
    updated_at = NOW()
WHERE attendant_key = 'paloma';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Renata', 'Renata Schnoor', 'Renata S'],
    updated_at = NOW()
WHERE attendant_key = 'renata';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Thais', 'Thaís', 'Thais Cotts', 'Thaís Cotts'],
    updated_at = NOW()
WHERE attendant_key = 'thais';

UPDATE attendant_mappings
SET attendant_aliases = ARRAY['Velma', 'Velma Fortes', 'Velma F'],
    updated_at = NOW()
WHERE attendant_key = 'velma';

-- ==============================================================================
-- LOG DA MIGRAÇÃO
-- ==============================================================================

INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_005_applied', NOW()::text, 'text'),
('space_ids_updated_from_api', 'true', 'boolean')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- VERIFICAÇÃO
-- ==============================================================================

DO $$
DECLARE
    total_with_space INTEGER;
    total_without_space INTEGER;
BEGIN
    SELECT COUNT(*) INTO total_with_space
    FROM attendant_mappings
    WHERE is_active = true AND space_id IS NOT NULL;

    SELECT COUNT(*) INTO total_without_space
    FROM attendant_mappings
    WHERE is_active = true AND space_id IS NULL;

    RAISE NOTICE '=== RESULTADO DA MIGRAÇÃO 005 ===';
    RAISE NOTICE 'Atendentes COM space próprio: % (esperado: 10)', total_with_space;
    RAISE NOTICE 'Atendentes COM space Clientes Inativos: % (esperado: 4 - Carlos, Marilia, Paloma, William)', total_without_space;

    IF total_with_space = 10 AND total_without_space = 0 THEN
        RAISE NOTICE '✅ Space IDs atualizados corretamente! (10 com space próprio + 4 com Clientes Inativos = 14 total)';
    ELSE
        RAISE WARNING '⚠️ Números inesperados. Verificar manualmente.';
    END IF;
END $$;

-- Listar atendentes com space ID
SELECT
    attendant_key,
    attendant_full_name,
    space_id,
    CASE
        WHEN space_id = '90130085983' THEN '⚠️ CLIENTES INATIVOS'
        WHEN space_id IS NOT NULL THEN '✅ SPACE PRÓPRIO'
        ELSE '❌ SEM SPACE'
    END as status
FROM attendant_mappings
WHERE is_active = true
ORDER BY attendant_key;
