-- Migration 006: Create system_config table
-- Purpose: Store dynamic configuration flags

CREATE TABLE IF NOT EXISTS system_config (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial configuration
INSERT INTO system_config (key, value, description) VALUES
    ('dynamic_structure_enabled', 'true', 'Enable dynamic folder/list resolution based on client+attendant')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_system_config_key ON system_config(key);

COMMENT ON TABLE system_config IS 'System-wide configuration flags and settings';
COMMENT ON COLUMN system_config.key IS 'Configuration key (unique)';
COMMENT ON COLUMN system_config.value IS 'Configuration value (stored as text)';
