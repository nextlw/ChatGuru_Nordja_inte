-- ==============================================================================
-- MIGRAÇÃO COMPLETA: 001 + 002 + 003
-- Data: 2025-10-10
-- Descrição: Script consolidado com todas as migrações para setup inicial
-- ==============================================================================

-- ==============================================================================
-- MIGRATION 001: ESTRUTURA DE TABELAS
-- ==============================================================================

-- Tabela de mapeamento de clientes
CREATE TABLE IF NOT EXISTS client_mappings (
    id SERIAL PRIMARY KEY,
    client_key VARCHAR(100) NOT NULL UNIQUE,
    client_aliases TEXT[],
    folder_path VARCHAR(255),
    folder_id VARCHAR(50),
    space_id VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_client_key ON client_mappings(client_key);
CREATE INDEX IF NOT EXISTS idx_folder_id ON client_mappings(folder_id);
CREATE INDEX IF NOT EXISTS idx_client_active ON client_mappings(is_active);

-- Tabela de mapeamento de atendentes
CREATE TABLE IF NOT EXISTS attendant_mappings (
    id SERIAL PRIMARY KEY,
    attendant_key VARCHAR(100) NOT NULL UNIQUE,
    attendant_full_name VARCHAR(255) NOT NULL,
    attendant_aliases TEXT[],
    space_id VARCHAR(50),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_attendant_key ON attendant_mappings(attendant_key);
CREATE INDEX IF NOT EXISTS idx_attendant_full_name ON attendant_mappings(attendant_full_name);
CREATE INDEX IF NOT EXISTS idx_attendant_active ON attendant_mappings(is_active);

-- Tabela de cache de listas mensais
CREATE TABLE IF NOT EXISTS list_cache (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(50) NOT NULL,
    list_id VARCHAR(50) NOT NULL UNIQUE,
    list_name VARCHAR(100) NOT NULL,
    year_month VARCHAR(20) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_verified TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_folder_month ON list_cache(folder_id, year_month);
CREATE INDEX IF NOT EXISTS idx_list_id ON list_cache(list_id);
CREATE INDEX IF NOT EXISTS idx_year_month ON list_cache(year_month);
CREATE INDEX IF NOT EXISTS idx_list_active ON list_cache(is_active);

-- Tabela de categorias
CREATE TABLE IF NOT EXISTS categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    clickup_field_id VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_category_name ON categories(name);
CREATE INDEX IF NOT EXISTS idx_category_active ON categories(is_active);

-- Tabela de subcategorias
CREATE TABLE IF NOT EXISTS subcategories (
    id SERIAL PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    clickup_field_id VARCHAR(50) NOT NULL,
    stars INTEGER NOT NULL DEFAULT 1 CHECK (stars BETWEEN 1 AND 4),
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(category_id, name)
);

CREATE INDEX IF NOT EXISTS idx_subcategory_category ON subcategories(category_id);
CREATE INDEX IF NOT EXISTS idx_subcategory_name ON subcategories(name);
CREATE INDEX IF NOT EXISTS idx_subcategory_active ON subcategories(is_active);

-- Tabela de tipos de atividade
CREATE TABLE IF NOT EXISTS activity_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    clickup_field_id VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activity_type_name ON activity_types(name);
CREATE INDEX IF NOT EXISTS idx_activity_type_active ON activity_types(is_active);

-- Tabela de opções de status
CREATE TABLE IF NOT EXISTS status_options (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    clickup_field_id VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_status_name ON status_options(name);
CREATE INDEX IF NOT EXISTS idx_status_active ON status_options(is_active);

-- Tabela de regras de prompt
CREATE TABLE IF NOT EXISTS prompt_rules (
    id SERIAL PRIMARY KEY,
    rule_text TEXT NOT NULL,
    rule_type VARCHAR(50) DEFAULT 'general',
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rule_type ON prompt_rules(rule_type);
CREATE INDEX IF NOT EXISTS idx_rule_active ON prompt_rules(is_active);

-- Tabela de configurações gerais de prompt
CREATE TABLE IF NOT EXISTS prompt_config (
    id SERIAL PRIMARY KEY,
    key VARCHAR(100) NOT NULL UNIQUE,
    value TEXT NOT NULL,
    config_type VARCHAR(50) DEFAULT 'text',
    is_active BOOLEAN DEFAULT true,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_prompt_config_key ON prompt_config(key);
CREATE INDEX IF NOT EXISTS idx_prompt_config_active ON prompt_config(is_active);

-- Triggers para auto-update do updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_client_mappings_updated_at ON client_mappings;
CREATE TRIGGER update_client_mappings_updated_at BEFORE UPDATE ON client_mappings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_attendant_mappings_updated_at ON attendant_mappings;
CREATE TRIGGER update_attendant_mappings_updated_at BEFORE UPDATE ON attendant_mappings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_categories_updated_at ON categories;
CREATE TRIGGER update_categories_updated_at BEFORE UPDATE ON categories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_subcategories_updated_at ON subcategories;
CREATE TRIGGER update_subcategories_updated_at BEFORE UPDATE ON subcategories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_activity_types_updated_at ON activity_types;
CREATE TRIGGER update_activity_types_updated_at BEFORE UPDATE ON activity_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_status_options_updated_at ON status_options;
CREATE TRIGGER update_status_options_updated_at BEFORE UPDATE ON status_options
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_prompt_config_updated_at ON prompt_config;
CREATE TRIGGER update_prompt_config_updated_at BEFORE UPDATE ON prompt_config
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ==============================================================================
-- MIGRATION 002: POPULAÇÃO INICIAL
-- ==============================================================================

-- Configurações de prompt (com fallbacks TEMPORÁRIOS, corrigidos na migration 003)
INSERT INTO prompt_config (key, value, config_type) VALUES
('system_role', 'Você é um assistente especializado em classificar solicitações e mapear campos para o sistema ClickUp.', 'text'),
('task_description', E'TAREFA:\n1. Classifique se é uma atividade de trabalho válida\n  - Se for atividade, determine os campos apropriados baseado no contexto\n  - Se aplicável, identifique possíveis subtarefas para a atividade', 'text'),
('category_field_id', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'text'),
('subcategory_field_id', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'text'),
('activity_type_field_id', 'f1259ffb-7be8-49ff-92f8-5ff9882888d0', 'text'),
('status_field_id', '6abbfe79-f80b-4b55-9b4b-9bd7f65b6458', 'text'),
('fallback_folder_id', '901300373349', 'text'),
('fallback_folder_path', 'Clientes Inativos / Gabriel Benarros', 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- Categorias
INSERT INTO categories (name, clickup_field_id, display_order) VALUES
('Agendamentos', '4b6cd768-fb58-48a5-a3d3-6993d2026764', 1),
('Compras', '11155a3f-5b4a-46f0-a447-4753bd9c3682', 2),
('Documentos', '60b9e5ad-7135-473c-97b2-c18d99b4a2b1', 3),
('Lazer', 'd12372bc-b2c1-4b15-b444-7edc7e477362', 4),
('Logística', 'e94fdbaa-7442-4579-8f98-3d345a5a862b', 5),
('Viagens', '632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 6),
('Plano de Saúde', 'c99d911f-595b-45c4-bb01-15d627d5a62f', 7),
('Agenda', 'c2ebd410-5ec1-4eb4-b585-d6bb9a9b9ff3', 8),
('Financeiro', '6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 9),
('Assuntos Pessoais', '72c6a009-bce5-41db-870f-c29d7094dbaf', 10),
('Atividades Corporativas', '5baa7715-1dfa-4a36-8452-78d60748e193', 11),
('Gestão de Funcionário', 'b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', 12)
ON CONFLICT (name) DO UPDATE SET clickup_field_id = EXCLUDED.clickup_field_id, display_order = EXCLUDED.display_order, updated_at = NOW();

-- Tipos de atividade
INSERT INTO activity_types (name, description, clickup_field_id) VALUES
('Rotineira', 'tarefas recorrentes e do dia a dia', '64f034f3-c5db-46e5-80e5-f515f11e2131'),
('Especifica', 'tarefas pontuais com propósito específico', 'e85a4dc7-82d8-4f63-89ee-462232f50f31'),
('Dedicada', 'tarefas que demandam dedicação especial', '6c810e95-f5e8-4e8f-ba23-808cf555046f')
ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description, clickup_field_id = EXCLUDED.clickup_field_id, updated_at = NOW();

-- Status
INSERT INTO status_options (name, clickup_field_id, display_order) VALUES
('Executar', '7889796f-033f-450d-97dd-6fee2a44f1b1', 1),
('Aguardando instruções', 'dd9d1b1b-f842-4777-984d-c05ec6b6d8a3', 2),
('Concluido', 'db544ddc-a07d-47a9-8737-40c6be25f7ec', 3)
ON CONFLICT (name) DO UPDATE SET clickup_field_id = EXCLUDED.clickup_field_id, display_order = EXCLUDED.display_order, updated_at = NOW();

-- Regras de prompt
INSERT INTO prompt_rules (rule_text, rule_type, display_order) VALUES
('CRÍTICO: Use SOMENTE as categorias listadas em CATEGORIAS DISPONÍVEIS NO CLICKUP. NUNCA crie categorias novas ou diferentes', 'validation', 1),
('A subcategoria SEMPRE deve ser relacionada à categoria principal escolhida', 'validation', 2),
('Para agendamentos (consultas, exames, veterinário, etc): escolha a categoria Agendamentos', 'category_specific', 3),
('Para pedidos de compra (mercado, presentes, farmácia, etc): escolha a categoria Compras', 'category_specific', 4),
('Para documentos (passaporte, CNH, certidões, etc): escolha a categoria Documentos', 'category_specific', 5),
('Para lazer (restaurantes, festas, eventos): escolha a categoria Lazer', 'category_specific', 6),
('Para entregas/transporte (motoboy, uber, correios): escolha a categoria Logística', 'category_specific', 7),
('Para viagens (passagens, hospedagens, transfer): escolha a categoria Viagens', 'category_specific', 8),
('Para plano de saúde (reembolsos, autorizações): escolha a categoria Plano de Saúde', 'category_specific', 9),
('Para agenda (gestão de agenda, invites): escolha a categoria Agenda', 'category_specific', 10),
('Para financeiro (NF, pagamentos, IR): escolha a categoria Financeiro', 'category_specific', 11),
('Para assuntos pessoais (mudanças, carro, casa): escolha a categoria Assuntos Pessoais', 'category_specific', 12),
('Para atividades corporativas (RH, estoque, planilhas): escolha a categoria Atividades Corporativas', 'category_specific', 13),
('Para gestão de funcionários (eSocial, DIRF, férias): escolha a categoria Gestão de Funcionário', 'category_specific', 14),
('Se não houver certeza sobre a atividade, classifique como false', 'general', 98),
('Sempre escolha valores EXATOS das listas fornecidas, não invente opções', 'validation', 99)
ON CONFLICT DO NOTHING;

-- Atendentes principais (sem space_id ainda, será preenchido na migration 003)
INSERT INTO attendant_mappings (attendant_key, attendant_full_name, attendant_aliases) VALUES
('anne', 'Anne Souza', ARRAY['Anne', 'Anne S', 'Anne Souza']),
('bruna', 'Bruna Senhora', ARRAY['Bruna', 'Bruna S', 'Bruna Senhora']),
('mariana_cruz', 'Mariana Cruz', ARRAY['Mariana C', 'Mariana Cruz']),
('mariana_medeiros', 'Mariana Medeiros', ARRAY['Mariana M', 'Mariana Medeiros']),
('gabriel', 'Gabriel Benarros', ARRAY['Gabriel', 'Gabriel B', 'Gabriel Benarros'])
ON CONFLICT (attendant_key) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    attendant_aliases = EXCLUDED.attendant_aliases,
    updated_at = NOW();

-- ==============================================================================
-- MIGRATION 003: CORREÇÃO DOS FALLBACKS E SISTEMA DINÂMICO
-- ==============================================================================

-- Atualizar fallback_folder_id para space "Clientes Inativos"
UPDATE prompt_config
SET value = '90130085983', updated_at = NOW()
WHERE key = 'fallback_folder_id' AND is_active = true;

-- Atualizar fallback_folder_path
UPDATE prompt_config
SET value = 'Clientes Inativos', updated_at = NOW()
WHERE key = 'fallback_folder_path' AND is_active = true;

-- Adicionar configurações do sistema dinâmico
INSERT INTO prompt_config (key, value, config_type) VALUES
('default_inactive_space_id', '90130085983', 'text'),
('dynamic_structure_enabled', 'true', 'boolean')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- Mapear atendentes com space IDs corretos
UPDATE attendant_mappings SET space_id = '90130178602', updated_at = NOW() WHERE attendant_key = 'anne';
UPDATE attendant_mappings SET space_id = '90130178610', updated_at = NOW() WHERE attendant_key = 'bruna';
UPDATE attendant_mappings SET space_id = '90130178618', updated_at = NOW() WHERE attendant_key = 'mariana_cruz';
UPDATE attendant_mappings SET space_id = '90130178626', updated_at = NOW() WHERE attendant_key = 'mariana_medeiros';
UPDATE attendant_mappings SET space_id = '90130178634', updated_at = NOW() WHERE attendant_key = 'gabriel';

-- Invalidar cache da lista problemática (se existir)
UPDATE list_cache
SET is_active = false, last_verified = NOW()
WHERE list_id = '901300373349';

-- Registrar migração aplicada
INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_001_applied', NOW()::text, 'text'),
('migration_002_applied', NOW()::text, 'text'),
('migration_003_applied', NOW()::text, 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- MIGRATION 007: TABELAS PARA SPACES, FOLDERS E LISTS DO CLICKUP
-- Data: 2025-10-11
-- Descrição: Cria estrutura para armazenar a hierarquia completa do ClickUp
-- ==============================================================================

-- Tabela de Spaces
CREATE TABLE IF NOT EXISTS spaces (
    id SERIAL PRIMARY KEY,
    space_id VARCHAR(50) NOT NULL UNIQUE,
    space_name VARCHAR(255) NOT NULL,
    team_id VARCHAR(50) NOT NULL,
    is_private BOOLEAN DEFAULT false,
    is_archived BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    raw_data JSONB,
    synced_at TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_spaces_space_id ON spaces(space_id);
CREATE INDEX IF NOT EXISTS idx_spaces_team_id ON spaces(team_id);
CREATE INDEX IF NOT EXISTS idx_spaces_active ON spaces(is_active);
CREATE INDEX IF NOT EXISTS idx_spaces_synced_at ON spaces(synced_at);

-- Tabela de Folders
CREATE TABLE IF NOT EXISTS folders (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(50) NOT NULL UNIQUE,
    folder_name VARCHAR(255) NOT NULL,
    space_id VARCHAR(50) NOT NULL,
    is_hidden BOOLEAN DEFAULT false,
    is_archived BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    task_count INTEGER DEFAULT 0,
    raw_data JSONB,
    synced_at TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    FOREIGN KEY (space_id) REFERENCES spaces(space_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_folders_folder_id ON folders(folder_id);
CREATE INDEX IF NOT EXISTS idx_folders_space_id ON folders(space_id);
CREATE INDEX IF NOT EXISTS idx_folders_active ON folders(is_active);
CREATE INDEX IF NOT EXISTS idx_folders_name ON folders(folder_name);
CREATE INDEX IF NOT EXISTS idx_folders_synced_at ON folders(synced_at);

-- Tabela de Lists
CREATE TABLE IF NOT EXISTS lists (
    id SERIAL PRIMARY KEY,
    list_id VARCHAR(50) NOT NULL UNIQUE,
    list_name VARCHAR(255) NOT NULL,
    folder_id VARCHAR(50),
    space_id VARCHAR(50) NOT NULL,
    is_folderless BOOLEAN DEFAULT false,
    is_archived BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    task_count INTEGER DEFAULT 0,
    raw_data JSONB,
    synced_at TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    FOREIGN KEY (space_id) REFERENCES spaces(space_id) ON DELETE CASCADE,
    FOREIGN KEY (folder_id) REFERENCES folders(folder_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_lists_list_id ON lists(list_id);
CREATE INDEX IF NOT EXISTS idx_lists_folder_id ON lists(folder_id);
CREATE INDEX IF NOT EXISTS idx_lists_space_id ON lists(space_id);
CREATE INDEX IF NOT EXISTS idx_lists_active ON lists(is_active);
CREATE INDEX IF NOT EXISTS idx_lists_name ON lists(list_name);
CREATE INDEX IF NOT EXISTS idx_lists_folderless ON lists(is_folderless);
CREATE INDEX IF NOT EXISTS idx_lists_synced_at ON lists(synced_at);

-- View para consulta hierárquica completa
CREATE OR REPLACE VIEW hierarchy_view AS
SELECT
    s.space_id,
    s.space_name,
    f.folder_id,
    f.folder_name,
    l.list_id,
    l.list_name,
    l.is_folderless,
    l.task_count,
    s.synced_at as space_synced_at,
    f.synced_at as folder_synced_at,
    l.synced_at as list_synced_at
FROM spaces s
LEFT JOIN folders f ON s.space_id = f.space_id AND f.is_active = true
LEFT JOIN lists l ON f.folder_id = l.folder_id AND l.is_active = true
WHERE s.is_active = true
ORDER BY s.space_name, f.folder_name, l.list_name;

-- Função para atualizar timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers para atualizar updated_at
DROP TRIGGER IF EXISTS update_spaces_updated_at ON spaces;
CREATE TRIGGER update_spaces_updated_at
    BEFORE UPDATE ON spaces
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_folders_updated_at ON folders;
CREATE TRIGGER update_folders_updated_at
    BEFORE UPDATE ON folders
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_lists_updated_at ON lists;
CREATE TRIGGER update_lists_updated_at
    BEFORE UPDATE ON lists
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Registrar migração 007
INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_007_applied', NOW()::text, 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- MIGRATION 008: CORREÇÃO DA LÓGICA DE MAPEAMENTO Info_1 vs Info_2
-- Data: 2025-10-12
-- Descrição: Corrige a interpretação dos campos Info_1 e Info_2 do ChatGuru
-- LÓGICA CORRIGIDA: Info_1 = Responsável (Space), Info_2 = Cliente (Folder)
-- ==============================================================================

-- Limpar dados existentes que podem estar com lógica invertida
TRUNCATE TABLE client_mappings RESTART IDENTITY CASCADE;
TRUNCATE TABLE attendant_mappings RESTART IDENTITY CASCADE;

-- LÓGICA CORRIGIDA: Info_1 determina o SPACE (responsável/atendente)
-- Recriar attendant_mappings com lógica correta: Info_1 → SPACE
INSERT INTO attendant_mappings (attendant_key, attendant_full_name, attendant_aliases, space_id, is_active) VALUES
('anne', 'Anne Souza', ARRAY['anne', 'Anne', 'Anne S', 'Anne Souza'], '90130178602', true),
('gabriel', 'Gabriel Moreno', ARRAY['gabriel', 'Gabriel', 'Gabriel M', 'Gabriel Moreno'], '90130178634', true),
('william', 'William Duarte', ARRAY['william', 'William', 'William D', 'William Duarte'], '90130178610', true)
ON CONFLICT (attendant_key) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    attendant_aliases = EXCLUDED.attendant_aliases,
    space_id = EXCLUDED.space_id,
    is_active = EXCLUDED.is_active,
    updated_at = NOW();

-- LÓGICA CORRIGIDA: Info_2 determina a FOLDER (cliente)
-- Recriar client_mappings com lógica correta: Info_2 → FOLDER
INSERT INTO client_mappings (client_key, client_full_name, client_aliases, folder_path, folder_id, space_id, is_active) VALUES
('nexcode', 'Nexcode', ARRAY['nexcode', 'Nexcode', 'NEXCODE'], 'Anne Souza / Nexcode', '901320655648', '90130178602', true),
('carolina', 'Carolina Torres', ARRAY['carolina', 'Carolina', 'Carolina T', 'Carolina Torres'], 'Gabriel Moreno / Carolina Torres', '901300373349', '90130178634', true)
ON CONFLICT (client_key) DO UPDATE SET
    client_full_name = EXCLUDED.client_full_name,
    client_aliases = EXCLUDED.client_aliases,
    folder_path = EXCLUDED.folder_path,
    folder_id = EXCLUDED.folder_id,
    space_id = EXCLUDED.space_id,
    is_active = EXCLUDED.is_active,
    updated_at = NOW();

-- Registrar migração 008
INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_008_applied', NOW()::text, 'text'),
('logic_correction_info1_space', 'Info_1 (responsável) determina SPACE', 'text'),
('logic_correction_info2_folder', 'Info_2 (cliente) determina FOLDER', 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- ==============================================================================
-- FIM DA MIGRAÇÃO COMPLETA
-- ==============================================================================
