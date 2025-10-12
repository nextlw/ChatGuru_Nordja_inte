-- Migration: Estrutura de tabelas para resolução dinâmica de pastas/listas e configuração de prompts
-- Data: 2025-10-08

-- ==============================================================================
-- TABELAS DE ESTRUTURA DO CLICKUP
-- ==============================================================================

-- Tabela de mapeamento de clientes
CREATE TABLE IF NOT EXISTS client_mappings (
    id SERIAL PRIMARY KEY,
    client_key VARCHAR(100) NOT NULL UNIQUE, -- Nome normalizado do cliente
    client_aliases TEXT[], -- Array com variações do nome
    folder_path VARCHAR(255), -- Caminho completo: "Anne Souza / Carolina Tavares"
    folder_id VARCHAR(50), -- ID da pasta no ClickUp
    space_id VARCHAR(50), -- ID do space no ClickUp
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_client_key ON client_mappings(client_key);
CREATE INDEX idx_folder_id ON client_mappings(folder_id);
CREATE INDEX idx_client_active ON client_mappings(is_active);

-- Tabela de mapeamento de atendentes
CREATE TABLE IF NOT EXISTS attendant_mappings (
    id SERIAL PRIMARY KEY,
    attendant_key VARCHAR(100) NOT NULL UNIQUE, -- Nome normalizado "Anne"
    attendant_full_name VARCHAR(255) NOT NULL, -- Nome completo "Anne Souza"
    attendant_aliases TEXT[], -- Variações ["Anne", "Anne S", "Anne Souza"]
    space_id VARCHAR(50), -- ID do Space no ClickUp
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_attendant_key ON attendant_mappings(attendant_key);
CREATE INDEX idx_attendant_full_name ON attendant_mappings(attendant_full_name);
CREATE INDEX idx_attendant_active ON attendant_mappings(is_active);

-- Tabela de cache de listas mensais
CREATE TABLE IF NOT EXISTS list_cache (
    id SERIAL PRIMARY KEY,
    folder_id VARCHAR(50) NOT NULL,
    list_id VARCHAR(50) NOT NULL UNIQUE,
    list_name VARCHAR(100) NOT NULL, -- "OUTUBRO 2025"
    year_month VARCHAR(20) NOT NULL, -- "2025-10" para busca
    is_active BOOLEAN DEFAULT true,
    last_verified TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_folder_month ON list_cache(folder_id, year_month);
CREATE INDEX idx_list_id ON list_cache(list_id);
CREATE INDEX idx_year_month ON list_cache(year_month);
CREATE INDEX idx_list_active ON list_cache(is_active);

-- ==============================================================================
-- TABELAS DE CONFIGURAÇÃO DE PROMPTS AI (ai_prompt.yaml → DB)
-- ==============================================================================

-- Tabela de categorias
CREATE TABLE IF NOT EXISTS categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    clickup_field_id VARCHAR(50) NOT NULL, -- UUID do campo no ClickUp
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_category_name ON categories(name);
CREATE INDEX idx_category_active ON categories(is_active);

-- Tabela de subcategorias
CREATE TABLE IF NOT EXISTS subcategories (
    id SERIAL PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    clickup_field_id VARCHAR(50) NOT NULL, -- UUID da subcategoria no ClickUp
    stars INTEGER NOT NULL DEFAULT 1 CHECK (stars BETWEEN 1 AND 4),
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(category_id, name)
);

CREATE INDEX idx_subcategory_category ON subcategories(category_id);
CREATE INDEX idx_subcategory_name ON subcategories(name);
CREATE INDEX idx_subcategory_active ON subcategories(is_active);

-- Tabela de tipos de atividade
CREATE TABLE IF NOT EXISTS activity_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    clickup_field_id VARCHAR(50) NOT NULL, -- UUID do tipo no ClickUp
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_activity_type_name ON activity_types(name);
CREATE INDEX idx_activity_type_active ON activity_types(is_active);

-- Tabela de opções de status
CREATE TABLE IF NOT EXISTS status_options (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    clickup_field_id VARCHAR(50) NOT NULL, -- UUID do status no ClickUp
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_status_name ON status_options(name);
CREATE INDEX idx_status_active ON status_options(is_active);

-- Tabela de regras de prompt
CREATE TABLE IF NOT EXISTS prompt_rules (
    id SERIAL PRIMARY KEY,
    rule_text TEXT NOT NULL,
    rule_type VARCHAR(50) DEFAULT 'general', -- 'general', 'category_specific', 'validation'
    is_active BOOLEAN DEFAULT true,
    display_order INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_rule_type ON prompt_rules(rule_type);
CREATE INDEX idx_rule_active ON prompt_rules(is_active);

-- Tabela de configurações gerais de prompt
CREATE TABLE IF NOT EXISTS prompt_config (
    id SERIAL PRIMARY KEY,
    key VARCHAR(100) NOT NULL UNIQUE, -- 'system_role', 'task_description', etc
    value TEXT NOT NULL,
    config_type VARCHAR(50) DEFAULT 'text', -- 'text', 'template', 'json'
    is_active BOOLEAN DEFAULT true,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_prompt_config_key ON prompt_config(key);
CREATE INDEX idx_prompt_config_active ON prompt_config(is_active);

-- ==============================================================================
-- TRIGGERS PARA AUTO-UPDATE DO updated_at
-- ==============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_client_mappings_updated_at BEFORE UPDATE ON client_mappings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_attendant_mappings_updated_at BEFORE UPDATE ON attendant_mappings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_categories_updated_at BEFORE UPDATE ON categories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subcategories_updated_at BEFORE UPDATE ON subcategories
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_activity_types_updated_at BEFORE UPDATE ON activity_types
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_status_options_updated_at BEFORE UPDATE ON status_options
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_prompt_config_updated_at BEFORE UPDATE ON prompt_config
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
