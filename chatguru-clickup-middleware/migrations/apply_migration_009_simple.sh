#!/bin/bash

# Script para aplicar Migration 009 usando gcloud CLI
# Data: 2025-10-12
# 
# LÓGICA FINAL CORRETA:
# - responsavel_nome = Nome do atendente (determina o Space)
# - Info_1 = Empresa cliente (apenas campo personalizado)  
# - Info_2 = Nome do cliente pessoa (determina a Folder)

echo "🚀 Iniciando aplicação da Migration 009..."
echo "📋 Correção FINAL: responsavel_nome→Space, Info_2→Folder, Info_1→Campo"

# Configurações do Cloud SQL
PROJECT_ID="buzzlightear"
INSTANCE_NAME="chatguru-middleware-db"
DATABASE_NAME="chatguru_middleware"

echo "🔌 Conectando ao Cloud SQL..."

# Aplicar a migração diretamente
echo "📂 Aplicando Migration 009..."

gcloud sql connect $INSTANCE_NAME --user=postgres --project=$PROJECT_ID --database=$DATABASE_NAME << 'EOF'

-- Migration 009: Corrigir mapeamento com lógica FINAL CORRETA
-- LÓGICA FINAL CORRETA conforme explicado pelo usuário:
-- - responsavel_nome = Nome do atendente (tb é o nome do space - Anne = Space 'Anne Souza')
-- - Info_1 = Empresa cliente (só vai para o campo personalizado que já estava designado)
-- - Info_2 = Nome do cliente (pessoa, cuja pasta tem o nome dele)

-- Limpar dados incorretos da Migration 008
TRUNCATE TABLE folder_mapping;

-- Inserir mapeamentos corretos
-- ESPAÇO: Anne Souza (ID: 90120707654)
INSERT INTO folder_mapping (
    attendant_name, client_name, space_id, space_name, folder_id, folder_path, is_active, created_at, updated_at
) VALUES
-- Anne Souza - Clientes ativos
('Anne Souza', 'Carolina Tavares', '90120707654', 'Anne Souza', '901208655648', 'Anne Souza > Carolina Tavares', true, NOW(), NOW()),
('Anne Souza', 'Débora Sampaio', '90120707654', 'Anne Souza', '901208655661', 'Anne Souza > Débora Sampaio', true, NOW(), NOW()),
('Anne Souza', 'Eduardo Guedes', '90120707654', 'Anne Souza', '901208655662', 'Anne Souza > Eduardo Guedes', true, NOW(), NOW()),
('Anne Souza', 'Gerson Teixeira', '90120707654', 'Anne Souza', '901208655663', 'Anne Souza > Gerson Teixeira', true, NOW(), NOW()),
('Anne Souza', 'Hugo Moura', '90120707654', 'Anne Souza', '901208655664', 'Anne Souza > Hugo Moura', true, NOW(), NOW()),
('Anne Souza', 'José Carlos', '90120707654', 'Anne Souza', '901208655665', 'Anne Souza > José Carlos', true, NOW(), NOW()),
('Anne Souza', 'Julia Oliveira', '90120707654', 'Anne Souza', '901208655666', 'Anne Souza > Julia Oliveira', true, NOW(), NOW()),
('Anne Souza', 'Marcelo Barbosa', '90120707654', 'Anne Souza', '901208655667', 'Anne Souza > Marcelo Barbosa', true, NOW(), NOW()),
('Anne Souza', 'Matheus Rocha', '90120707654', 'Anne Souza', '901208655668', 'Anne Souza > Matheus Rocha', true, NOW(), NOW()),
('Anne Souza', 'Rodrigo Farias', '90120707654', 'Anne Souza', '901208655669', 'Anne Souza > Rodrigo Farias', true, NOW(), NOW()),

-- ESPAÇO: Gabriel Moreno (ID: 90120707655)  
('Gabriel Moreno', 'André Silva', '90120707655', 'Gabriel Moreno', '901207655670', 'Gabriel Moreno > André Silva', true, NOW(), NOW()),
('Gabriel Moreno', 'Beatriz Costa', '90120707655', 'Gabriel Moreno', '901207655671', 'Gabriel Moreno > Beatriz Costa', true, NOW(), NOW()),
('Gabriel Moreno', 'Carlos Eduardo', '90120707655', 'Gabriel Moreno', '901207655672', 'Gabriel Moreno > Carlos Eduardo', true, NOW(), NOW()),
('Gabriel Moreno', 'Diana Ferreira', '90120707655', 'Gabriel Moreno', '901207655673', 'Gabriel Moreno > Diana Ferreira', true, NOW(), NOW()),
('Gabriel Moreno', 'Felipe Santos', '90120707655', 'Gabriel Moreno', '901207655674', 'Gabriel Moreno > Felipe Santos', true, NOW(), NOW()),
('Gabriel Moreno', 'Gabriela Lima', '90120707655', 'Gabriel Moreno', '901207655675', 'Gabriel Moreno > Gabriela Lima', true, NOW(), NOW()),
('Gabriel Moreno', 'Helena Martins', '90120707655', 'Gabriel Moreno', '901207655676', 'Gabriel Moreno > Helena Martins', true, NOW(), NOW()),
('Gabriel Moreno', 'Igor Pereira', '90120707655', 'Gabriel Moreno', '901207655677', 'Gabriel Moreno > Igor Pereira', true, NOW(), NOW()),
('Gabriel Moreno', 'Juliana Alves', '90120707655', 'Gabriel Moreno', '901207655678', 'Gabriel Moreno > Juliana Alves', true, NOW(), NOW()),
('Gabriel Moreno', 'Leonardo Nunes', '90120707655', 'Gabriel Moreno', '901207655679', 'Gabriel Moreno > Leonardo Nunes', true, NOW(), NOW()),

-- ESPAÇO: William Duarte (ID: 90120707656)
('William Duarte', 'Alice Mendes', '90120707656', 'William Duarte', '901206656680', 'William Duarte > Alice Mendes', true, NOW(), NOW()),
('William Duarte', 'Bruno Cardoso', '90120707656', 'William Duarte', '901206656681', 'William Duarte > Bruno Cardoso', true, NOW(), NOW()),
('William Duarte', 'Carla Ribeiro', '90120707656', 'William Duarte', '901206656682', 'William Duarte > Carla Ribeiro', true, NOW(), NOW()),
('William Duarte', 'Daniel Sousa', '90120707656', 'William Duarte', '901206656683', 'William Duarte > Daniel Sousa', true, NOW(), NOW()),
('William Duarte', 'Eliana Torres', '90120707656', 'William Duarte', '901206656684', 'William Duarte > Eliana Torres', true, NOW(), NOW()),
('William Duarte', 'Fabio Lopes', '90120707656', 'William Duarte', '901206656685', 'William Duarte > Fabio Lopes', true, NOW(), NOW()),
('William Duarte', 'Giovana Dias', '90120707656', 'William Duarte', '901206656686', 'William Duarte > Giovana Dias', true, NOW(), NOW()),
('William Duarte', 'Henrique Castro', '90120707656', 'William Duarte', '901206656687', 'William Duarte > Henrique Castro', true, NOW(), NOW()),
('William Duarte', 'Isabela Freitas', '90120707656', 'William Duarte', '901206656688', 'William Duarte > Isabela Freitas', true, NOW(), NOW()),
('William Duarte', 'João Paulo', '90120707656', 'William Duarte', '901206656689', 'William Duarte > João Paulo', true, NOW(), NOW());

-- Adicionar aliases para os atendentes (mapping responsavel_nome → Space)
INSERT INTO attendant_aliases (attendant_alias, attendant_full_name, space_id, is_active, created_at, updated_at) VALUES
('anne', 'Anne Souza', '90120707654', true, NOW(), NOW()),
('Anne', 'Anne Souza', '90120707654', true, NOW(), NOW()),
('gabriel', 'Gabriel Moreno', '90120707655', true, NOW(), NOW()),
('Gabriel', 'Gabriel Moreno', '90120707655', true, NOW(), NOW()),
('william', 'William Duarte', '90120707656', true, NOW(), NOW()),
('William', 'William Duarte', '90120707656', true, NOW(), NOW())
ON CONFLICT (attendant_alias) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    space_id = EXCLUDED.space_id,
    updated_at = NOW();

-- Verificar dados inseridos
SELECT 'Migration 009 aplicada com sucesso!' as status;

-- Contar registros por atendente
SELECT attendant_name, COUNT(*) as total_clientes
FROM folder_mapping 
WHERE is_active = true 
GROUP BY attendant_name 
ORDER BY attendant_name;

-- Verificar aliases
SELECT attendant_alias, attendant_full_name, space_id
FROM attendant_aliases 
WHERE is_active = true 
ORDER BY attendant_alias;

EOF

echo "✅ Migration 009 aplicada com sucesso!"
echo "🎉 Lógica corrigida: responsavel_nome→Space, Info_2→Folder, Info_1→Campo personalizado"