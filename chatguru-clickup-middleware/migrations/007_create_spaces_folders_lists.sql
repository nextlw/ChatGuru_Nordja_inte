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
