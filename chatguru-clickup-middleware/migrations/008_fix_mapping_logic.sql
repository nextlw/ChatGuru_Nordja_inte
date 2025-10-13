-- MIGRATION 008: Corrigir lógica de mapeamento Info_1 vs Info_2
-- 
-- PROBLEMA IDENTIFICADO:
-- A lógica estava invertida:
-- - Info_1 (responsável) deveria determinar o SPACE
-- - Info_2 (cliente) deveria determinar a FOLDER
--
-- CORREÇÃO:
-- - Info_1 = Attendant (determina Space)
-- - Info_2 = Client (determina Folder)

BEGIN;

-- 1. Atualizar attendant_mappings com aliases corretos
-- Garantir que aliases como "anne" mapeiam para "Anne Souza"

-- Limpar dados existentes incorretos
DELETE FROM attendant_mappings WHERE attendant_key IN ('anne', 'gabriel', 'william');

-- Inserir mappings corretos para attendants (Info_1 determina Space)
INSERT INTO attendant_mappings (
    attendant_key, 
    attendant_full_name, 
    attendant_aliases, 
    space_id, 
    space_name, 
    is_active
) VALUES 
-- Anne Souza (Info_1 = "anne" determina Space)
('anne', 'Anne Souza', ARRAY['anne', 'anne souza', 'annesouza'], '90111015558', 'Anne Souza', true),

-- Gabriel Moreno (Info_1 = "gabriel" determina Space) 
('gabriel', 'Gabriel Moreno', ARRAY['gabriel', 'gabriel moreno', 'gabrielmoreno'], '90110986096', 'Gabriel Moreno', true),

-- William Duarte (Info_1 = "william" determina Space)
('william', 'William Duarte', ARRAY['william', 'william duarte', 'williamduarte'], '90110995734', 'William Duarte', true)

ON CONFLICT (attendant_key) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    attendant_aliases = EXCLUDED.attendant_aliases,
    space_id = EXCLUDED.space_id,
    space_name = EXCLUDED.space_name,
    is_active = EXCLUDED.is_active,
    updated_at = NOW();

-- 2. Atualizar client_mappings com aliases corretos
-- Garantir que Info_2 (cliente) determina corretamente a Folder

-- Limpar alguns exemplos se existirem
DELETE FROM client_mappings WHERE client_key IN ('nexcode', 'carolina', 'carolina tavares');

-- Inserir mappings corretos para clients (Info_2 determina Folder)
INSERT INTO client_mappings (
    client_key,
    client_full_name, 
    client_aliases,
    folder_id,
    folder_path,
    space_id,
    is_active
) VALUES 
-- Nexcode (Info_2 = "nexcode" determina Folder "Anne Souza / Nexcode")
('nexcode', 'Nexcode', ARRAY['nexcode', 'nex code', 'nex'], '901320655648', 'Anne Souza / Nexcode', '90111015558', true),

-- Carolina Tavares (Info_2 = "carolina" determina Folder "Anne Souza / Carolina Tavares")
('carolina', 'Carolina Tavares', ARRAY['carolina', 'carolina tavares', 'carolinatavares'], '901320655649', 'Anne Souza / Carolina Tavares', '90111015558', true)

ON CONFLICT (client_key) DO UPDATE SET
    client_full_name = EXCLUDED.client_full_name,
    client_aliases = EXCLUDED.client_aliases,
    folder_id = EXCLUDED.folder_id,
    folder_path = EXCLUDED.folder_path,
    space_id = EXCLUDED.space_id,
    is_active = EXCLUDED.is_active,
    updated_at = NOW();

-- 3. Verificar os mappings criados
DO $$
BEGIN
    RAISE NOTICE '=== MIGRATION 008 APLICADA COM SUCESSO ===';
    RAISE NOTICE 'LÓGICA CORRIGIDA:';
    RAISE NOTICE '- Info_1 (responsável) → Determina SPACE via attendant_mappings';
    RAISE NOTICE '- Info_2 (cliente) → Determina FOLDER via client_mappings';
    RAISE NOTICE '';
    RAISE NOTICE 'Attendants configurados:';
    RAISE NOTICE '- anne → Anne Souza (Space: 90111015558)';
    RAISE NOTICE '- gabriel → Gabriel Moreno (Space: 90110986096)';
    RAISE NOTICE '- william → William Duarte (Space: 90110995734)';
    RAISE NOTICE '';
    RAISE NOTICE 'Clients configurados:';
    RAISE NOTICE '- nexcode → Folder: Anne Souza / Nexcode (901320655648)';
    RAISE NOTICE '- carolina → Folder: Anne Souza / Carolina Tavares (901320655649)';
END$$;

COMMIT;