-- ============================================================================
-- ChatGuru-ClickUp Middleware Database Schema
-- ============================================================================
-- Version: 1.0
-- Database: PostgreSQL 14+
-- Description: Normalized schema for ClickUp structure mapping and data cache
-- ============================================================================

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- CORE ENTITIES
-- ============================================================================

-- Teams (Root level - Nordja)
CREATE TABLE teams (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Spaces (Level 1 - Attendants: Renata, Anne, etc.)
CREATE TABLE spaces (
    id VARCHAR(50) PRIMARY KEY,
    team_id VARCHAR(50) NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(team_id, name)
);

-- Folders (Level 2 - Clients: Gabriel Benarros, Fernanda Munhoz, etc.)
CREATE TABLE folders (
    id VARCHAR(50) PRIMARY KEY,
    space_id VARCHAR(50) NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(space_id, name)
);

-- Lists (Level 3 - Monthly: "OUTUBRO 2025", "NOVEMBRO 2025", etc.)
CREATE TABLE lists (
    id VARCHAR(50) PRIMARY KEY,
    folder_id VARCHAR(50) NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    archived BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(folder_id, name)
);

-- ============================================================================
-- CUSTOM FIELDS DEFINITIONS
-- ============================================================================

-- Custom Field Types (metadata)
CREATE TABLE custom_field_types (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    type VARCHAR(50) NOT NULL, -- drop_down, text, number, date, etc.
    description TEXT,
    required BOOLEAN DEFAULT FALSE,
    hide_from_guests BOOLEAN DEFAULT FALSE,
    date_created BIGINT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Categories (Categoria_nova field)
CREATE TABLE categories (
    id UUID PRIMARY KEY,
    field_id UUID NOT NULL REFERENCES custom_field_types(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(10),
    orderindex INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(field_id, name)
);

-- Subcategories (SubCategoria_nova field)
CREATE TABLE subcategories (
    id UUID PRIMARY KEY,
    field_id UUID NOT NULL REFERENCES custom_field_types(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(10),
    orderindex INTEGER,
    stars INTEGER DEFAULT 1, -- Difficulty/complexity level from ai_prompt.yaml
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(field_id, name)
);

-- Category-Subcategory Relationship (many-to-many)
CREATE TABLE category_subcategory_mapping (
    id SERIAL PRIMARY KEY,
    category_id UUID NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    subcategory_id UUID NOT NULL REFERENCES subcategories(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(category_id, subcategory_id)
);

-- Activity Types (Tipo de Atividade: Rotineira, Especifica, Dedicada)
CREATE TABLE activity_types (
    id UUID PRIMARY KEY,
    field_id UUID NOT NULL REFERENCES custom_field_types(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(field_id, name)
);

-- Status Options (Status Back Office: Executar, Aguardando instruções, Concluído)
CREATE TABLE status_options (
    id UUID PRIMARY KEY,
    field_id UUID NOT NULL REFERENCES custom_field_types(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(field_id, name)
);

-- Cliente Solicitante (Clients list)
CREATE TABLE client_requesters (
    id UUID PRIMARY KEY,
    field_id UUID NOT NULL REFERENCES custom_field_types(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MAPPING TABLES (Dynamic Structure Resolution)
-- ============================================================================

-- Attendant Aliases (responsavel_nome → space_name mapping)
CREATE TABLE attendant_aliases (
    id SERIAL PRIMARY KEY,
    alias VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255) NOT NULL,
    space_id VARCHAR(50) REFERENCES spaces(id) ON DELETE SET NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Client-Attendant-Folder Mapping (info_2 + responsavel_nome → folder)
CREATE TABLE folder_mapping (
    id SERIAL PRIMARY KEY,
    client_name VARCHAR(255) NOT NULL,
    client_normalized VARCHAR(255) NOT NULL, -- Lowercase, trimmed
    attendant_name VARCHAR(255) NOT NULL,
    attendant_normalized VARCHAR(255) NOT NULL, -- Lowercase, trimmed
    space_id VARCHAR(50) NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
    folder_id VARCHAR(50) NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    folder_path TEXT, -- Full path for debugging: Space/Folder
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(client_normalized, attendant_normalized)
);

-- ============================================================================
-- CACHE TABLES (3-tier cache: memory → DB → API)
-- ============================================================================

-- List Cache (folder_id + month/year → list_id)
CREATE TABLE list_cache (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(50) NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    list_id VARCHAR(50) NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    list_name VARCHAR(255) NOT NULL,
    month VARCHAR(20) NOT NULL, -- "OUTUBRO", "NOVEMBRO"
    year INTEGER NOT NULL,
    full_name VARCHAR(255) NOT NULL, -- "OUTUBRO 2025"
    is_active BOOLEAN DEFAULT TRUE,
    last_verified TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(folder_id, full_name)
);

-- Task Cache (optional - for tracking created tasks)
CREATE TABLE task_cache (
    id SERIAL PRIMARY KEY,
    task_id VARCHAR(50) NOT NULL UNIQUE,
    list_id VARCHAR(50) NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    task_name VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- SYSTEM CONFIGURATION
-- ============================================================================

-- System Config (key-value store for app settings)
CREATE TABLE system_config (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) NOT NULL UNIQUE,
    value TEXT,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- Spaces indexes
CREATE INDEX idx_spaces_team_id ON spaces(team_id);
CREATE INDEX idx_spaces_name ON spaces(name);
CREATE INDEX idx_spaces_archived ON spaces(archived);

-- Folders indexes
CREATE INDEX idx_folders_space_id ON folders(space_id);
CREATE INDEX idx_folders_name ON folders(name);
CREATE INDEX idx_folders_archived ON folders(archived);

-- Lists indexes
CREATE INDEX idx_lists_folder_id ON lists(folder_id);
CREATE INDEX idx_lists_name ON lists(name);
CREATE INDEX idx_lists_archived ON lists(archived);

-- Categories indexes
CREATE INDEX idx_categories_field_id ON categories(field_id);
CREATE INDEX idx_categories_name ON categories(name);
CREATE INDEX idx_categories_orderindex ON categories(orderindex);

-- Subcategories indexes
CREATE INDEX idx_subcategories_field_id ON subcategories(field_id);
CREATE INDEX idx_subcategories_name ON subcategories(name);
CREATE INDEX idx_subcategories_orderindex ON subcategories(orderindex);
CREATE INDEX idx_subcategories_stars ON subcategories(stars);

-- Category-Subcategory mapping indexes
CREATE INDEX idx_cat_subcat_category_id ON category_subcategory_mapping(category_id);
CREATE INDEX idx_cat_subcat_subcategory_id ON category_subcategory_mapping(subcategory_id);

-- Activity Types indexes
CREATE INDEX idx_activity_types_field_id ON activity_types(field_id);
CREATE INDEX idx_activity_types_name ON activity_types(name);

-- Status Options indexes
CREATE INDEX idx_status_options_field_id ON status_options(field_id);
CREATE INDEX idx_status_options_name ON status_options(name);

-- Client Requesters indexes
CREATE INDEX idx_client_requesters_field_id ON client_requesters(field_id);
CREATE INDEX idx_client_requesters_name ON client_requesters(name);

-- Attendant Aliases indexes
CREATE INDEX idx_attendant_aliases_alias ON attendant_aliases(alias);
CREATE INDEX idx_attendant_aliases_full_name ON attendant_aliases(full_name);
CREATE INDEX idx_attendant_aliases_space_id ON attendant_aliases(space_id);
CREATE INDEX idx_attendant_aliases_is_active ON attendant_aliases(is_active);

-- Folder Mapping indexes
CREATE INDEX idx_folder_mapping_client_normalized ON folder_mapping(client_normalized);
CREATE INDEX idx_folder_mapping_attendant_normalized ON folder_mapping(attendant_normalized);
CREATE INDEX idx_folder_mapping_space_id ON folder_mapping(space_id);
CREATE INDEX idx_folder_mapping_folder_id ON folder_mapping(folder_id);
CREATE INDEX idx_folder_mapping_is_active ON folder_mapping(is_active);
CREATE INDEX idx_folder_mapping_client_attendant ON folder_mapping(client_normalized, attendant_normalized);

-- List Cache indexes
CREATE INDEX idx_list_cache_folder_id ON list_cache(folder_id);
CREATE INDEX idx_list_cache_list_id ON list_cache(list_id);
CREATE INDEX idx_list_cache_month_year ON list_cache(month, year);
CREATE INDEX idx_list_cache_is_active ON list_cache(is_active);
CREATE INDEX idx_list_cache_last_verified ON list_cache(last_verified);

-- Task Cache indexes
CREATE INDEX idx_task_cache_task_id ON task_cache(task_id);
CREATE INDEX idx_task_cache_list_id ON task_cache(list_id);
CREATE INDEX idx_task_cache_created_at ON task_cache(created_at);

-- System Config indexes
CREATE INDEX idx_system_config_key ON system_config(key);

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to all tables with updated_at
CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_spaces_updated_at BEFORE UPDATE ON spaces FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_folders_updated_at BEFORE UPDATE ON folders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_lists_updated_at BEFORE UPDATE ON lists FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_custom_field_types_updated_at BEFORE UPDATE ON custom_field_types FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_categories_updated_at BEFORE UPDATE ON categories FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_subcategories_updated_at BEFORE UPDATE ON subcategories FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_activity_types_updated_at BEFORE UPDATE ON activity_types FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_status_options_updated_at BEFORE UPDATE ON status_options FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_client_requesters_updated_at BEFORE UPDATE ON client_requesters FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_attendant_aliases_updated_at BEFORE UPDATE ON attendant_aliases FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_folder_mapping_updated_at BEFORE UPDATE ON folder_mapping FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_list_cache_updated_at BEFORE UPDATE ON list_cache FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_task_cache_updated_at BEFORE UPDATE ON task_cache FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_system_config_updated_at BEFORE UPDATE ON system_config FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- VIEWS FOR COMMON QUERIES
-- ============================================================================

-- View: Complete folder structure
CREATE OR REPLACE VIEW v_folder_structure AS
SELECT
    t.id AS team_id,
    t.name AS team_name,
    s.id AS space_id,
    s.name AS space_name,
    f.id AS folder_id,
    f.name AS folder_name,
    l.id AS list_id,
    l.name AS list_name,
    s.archived AS space_archived,
    f.archived AS folder_archived,
    l.archived AS list_archived
FROM teams t
JOIN spaces s ON s.team_id = t.id
JOIN folders f ON f.space_id = s.id
JOIN lists l ON l.folder_id = f.id
ORDER BY t.name, s.name, f.name, l.name;

-- View: Active mappings
CREATE OR REPLACE VIEW v_active_mappings AS
SELECT
    fm.id,
    fm.client_name,
    fm.attendant_name,
    s.name AS space_name,
    f.name AS folder_name,
    fm.folder_path,
    fm.is_active
FROM folder_mapping fm
JOIN spaces s ON s.id = fm.space_id
JOIN folders f ON f.id = fm.folder_id
WHERE fm.is_active = TRUE
ORDER BY s.name, f.name;

-- View: Category-Subcategory relationships
CREATE OR REPLACE VIEW v_category_subcategory AS
SELECT
    c.id AS category_id,
    c.name AS category_name,
    c.color AS category_color,
    s.id AS subcategory_id,
    s.name AS subcategory_name,
    s.color AS subcategory_color,
    s.stars AS subcategory_stars
FROM categories c
JOIN category_subcategory_mapping csm ON csm.category_id = c.id
JOIN subcategories s ON s.id = csm.subcategory_id
ORDER BY c.orderindex, s.orderindex;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE teams IS 'Root level - Nordja team';
COMMENT ON TABLE spaces IS 'Level 1 - Attendants (Renata Schnoor, Anne Souza, etc.)';
COMMENT ON TABLE folders IS 'Level 2 - Clients (Gabriel Benarros, Fernanda Munhoz, etc.)';
COMMENT ON TABLE lists IS 'Level 3 - Monthly lists (OUTUBRO 2025, NOVEMBRO 2025, etc.)';
COMMENT ON TABLE custom_field_types IS 'Custom field definitions from ClickUp';
COMMENT ON TABLE categories IS 'Categoria_nova options from ClickUp';
COMMENT ON TABLE subcategories IS 'SubCategoria_nova options from ClickUp';
COMMENT ON TABLE category_subcategory_mapping IS 'Many-to-many relationship between categories and subcategories';
COMMENT ON TABLE activity_types IS 'Tipo de Atividade options (Rotineira, Especifica, Dedicada)';
COMMENT ON TABLE status_options IS 'Status Back Office options (Executar, Aguardando instruções, Concluído)';
COMMENT ON TABLE client_requesters IS 'Cliente Solicitante dropdown options';
COMMENT ON TABLE attendant_aliases IS 'Maps responsavel_nome aliases to space_name';
COMMENT ON TABLE folder_mapping IS 'Maps Client + Attendant to Folder/List (core mapping table)';
COMMENT ON TABLE list_cache IS 'Cache for monthly lists (3-tier cache: memory → DB → API)';
COMMENT ON TABLE task_cache IS 'Optional cache for tracking created tasks';
COMMENT ON TABLE system_config IS 'Key-value store for application settings';

-- ============================================================================
-- END OF SCHEMA
-- ============================================================================
