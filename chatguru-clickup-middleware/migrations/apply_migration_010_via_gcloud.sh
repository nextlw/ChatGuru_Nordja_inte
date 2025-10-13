#!/bin/bash
# Aplicar Migration 010 via gcloud sql connect

set -e

echo "🚀 Aplicando Migration 010 via gcloud..."

# Aplicar migration via gcloud sql
gcloud sql connect chatguru-middleware-db \
  --user=postgres \
  --project=buzzlightear \
  --quiet \
  --database=chatguru_middleware << 'EOSQL'

-- Migration 010: Popular attendant_mappings e client_mappings com aliases corretos
TRUNCATE TABLE attendant_mappings CASCADE;

INSERT INTO attendant_mappings (
    attendant_key, attendant_full_name, attendant_aliases, space_id, is_active, created_at, updated_at
) VALUES
('anne', 'Anne Souza', ARRAY['anne', 'Anne', 'anne souza', 'Anne Souza', 'ANNE', 'ANNE SOUZA'], '90120707654', true, NOW(), NOW()),
('gabriel', 'Gabriel Moreno', ARRAY['gabriel', 'Gabriel', 'gabriel moreno', 'Gabriel Moreno', 'GABRIEL', 'GABRIEL MORENO'], '90120707655', true, NOW(), NOW()),
('william', 'William Duarte', ARRAY['william', 'William', 'william duarte', 'William Duarte', 'WILLIAM', 'WILLIAM DUARTE'], '90120707656', true, NOW(), NOW())
ON CONFLICT (attendant_key) DO UPDATE SET
    attendant_full_name = EXCLUDED.attendant_full_name,
    attendant_aliases = EXCLUDED.attendant_aliases,
    space_id = EXCLUDED.space_id,
    updated_at = NOW();

TRUNCATE TABLE client_mappings CASCADE;

INSERT INTO client_mappings (client_key, client_full_name, client_aliases, folder_id, folder_path, space_id, is_active, created_at, updated_at) VALUES
-- Anne Souza
('carolina tavares', 'Carolina Tavares', ARRAY['carolina', 'Carolina', 'carolina tavares', 'Carolina Tavares'], '901208655648', 'Anne Souza > Carolina Tavares', '90120707654', true, NOW(), NOW()),
('debora sampaio', 'Débora Sampaio', ARRAY['debora', 'Debora', 'débora', 'Débora', 'debora sampaio', 'Débora Sampaio'], '901208655661', 'Anne Souza > Débora Sampaio', '90120707654', true, NOW(), NOW()),
('eduardo guedes', 'Eduardo Guedes', ARRAY['eduardo', 'Eduardo', 'eduardo guedes', 'Eduardo Guedes'], '901208655662', 'Anne Souza > Eduardo Guedes', '90120707654', true, NOW(), NOW()),
('gerson teixeira', 'Gerson Teixeira', ARRAY['gerson', 'Gerson', 'gerson teixeira', 'Gerson Teixeira'], '901208655663', 'Anne Souza > Gerson Teixeira', '90120707654', true, NOW(), NOW()),
('hugo moura', 'Hugo Moura', ARRAY['hugo', 'Hugo', 'hugo moura', 'Hugo Moura'], '901208655664', 'Anne Souza > Hugo Moura', '90120707654', true, NOW(), NOW()),
('jose carlos', 'José Carlos', ARRAY['jose', 'José', 'jose carlos', 'José Carlos'], '901208655665', 'Anne Souza > José Carlos', '90120707654', true, NOW(), NOW()),
('julia oliveira', 'Julia Oliveira', ARRAY['julia', 'Julia', 'júlia', 'Júlia', 'julia oliveira', 'Julia Oliveira'], '901208655666', 'Anne Souza > Julia Oliveira', '90120707654', true, NOW(), NOW()),
('marcelo barbosa', 'Marcelo Barbosa', ARRAY['marcelo', 'Marcelo', 'marcelo barbosa', 'Marcelo Barbosa'], '901208655667', 'Anne Souza > Marcelo Barbosa', '90120707654', true, NOW(), NOW()),
('matheus rocha', 'Matheus Rocha', ARRAY['matheus', 'Matheus', 'matheus rocha', 'Matheus Rocha'], '901208655668', 'Anne Souza > Matheus Rocha', '90120707654', true, NOW(), NOW()),
('rodrigo farias', 'Rodrigo Farias', ARRAY['rodrigo', 'Rodrigo', 'rodrigo farias', 'Rodrigo Farias'], '901208655669', 'Anne Souza > Rodrigo Farias', '90120707654', true, NOW(), NOW()),
-- Gabriel Moreno
('andre silva', 'André Silva', ARRAY['andre', 'André', 'andre silva', 'André Silva'], '901207655670', 'Gabriel Moreno > André Silva', '90120707655', true, NOW(), NOW()),
('beatriz costa', 'Beatriz Costa', ARRAY['beatriz', 'Beatriz', 'bia', 'Bia', 'beatriz costa', 'Beatriz Costa'], '901207655671', 'Gabriel Moreno > Beatriz Costa', '90120707655', true, NOW(), NOW()),
('carlos eduardo', 'Carlos Eduardo', ARRAY['carlos', 'Carlos', 'carlos eduardo', 'Carlos Eduardo'], '901207655672', 'Gabriel Moreno > Carlos Eduardo', '90120707655', true, NOW(), NOW()),
('diana ferreira', 'Diana Ferreira', ARRAY['diana', 'Diana', 'diana ferreira', 'Diana Ferreira'], '901207655673', 'Gabriel Moreno > Diana Ferreira', '90120707655', true, NOW(), NOW()),
('felipe santos', 'Felipe Santos', ARRAY['felipe', 'Felipe', 'felipe santos', 'Felipe Santos'], '901207655674', 'Gabriel Moreno > Felipe Santos', '90120707655', true, NOW(), NOW()),
('gabriela lima', 'Gabriela Lima', ARRAY['gabriela', 'Gabriela', 'gabi', 'Gabi', 'gabriela lima', 'Gabriela Lima'], '901207655675', 'Gabriel Moreno > Gabriela Lima', '90120707655', true, NOW(), NOW()),
('helena martins', 'Helena Martins', ARRAY['helena', 'Helena', 'helena martins', 'Helena Martins'], '901207655676', 'Gabriel Moreno > Helena Martins', '90120707655', true, NOW(), NOW()),
('igor pereira', 'Igor Pereira', ARRAY['igor', 'Igor', 'igor pereira', 'Igor Pereira'], '901207655677', 'Gabriel Moreno > Igor Pereira', '90120707655', true, NOW(), NOW()),
('juliana alves', 'Juliana Alves', ARRAY['juliana', 'Juliana', 'ju', 'Ju', 'juliana alves', 'Juliana Alves'], '901207655678', 'Gabriel Moreno > Juliana Alves', '90120707655', true, NOW(), NOW()),
('leonardo nunes', 'Leonardo Nunes', ARRAY['leonardo', 'Leonardo', 'leo', 'Leo', 'leonardo nunes', 'Leonardo Nunes'], '901207655679', 'Gabriel Moreno > Leonardo Nunes', '90120707655', true, NOW(), NOW()),
-- William Duarte
('alice mendes', 'Alice Mendes', ARRAY['alice', 'Alice', 'alice mendes', 'Alice Mendes'], '901206656680', 'William Duarte > Alice Mendes', '90120707656', true, NOW(), NOW()),
('bruno cardoso', 'Bruno Cardoso', ARRAY['bruno', 'Bruno', 'bruno cardoso', 'Bruno Cardoso'], '901206656681', 'William Duarte > Bruno Cardoso', '90120707656', true, NOW(), NOW()),
('carla ribeiro', 'Carla Ribeiro', ARRAY['carla', 'Carla', 'carla ribeiro', 'Carla Ribeiro'], '901206656682', 'William Duarte > Carla Ribeiro', '90120707656', true, NOW(), NOW()),
('daniel sousa', 'Daniel Sousa', ARRAY['daniel', 'Daniel', 'daniel sousa', 'Daniel Sousa'], '901206656683', 'William Duarte > Daniel Sousa', '90120707656', true, NOW(), NOW()),
('eliana torres', 'Eliana Torres', ARRAY['eliana', 'Eliana', 'eliana torres', 'Eliana Torres'], '901206656684', 'William Duarte > Eliana Torres', '90120707656', true, NOW(), NOW()),
('fabio lopes', 'Fabio Lopes', ARRAY['fabio', 'Fabio', 'fábio', 'Fábio', 'fabio lopes', 'Fabio Lopes'], '901206656685', 'William Duarte > Fabio Lopes', '90120707656', true, NOW(), NOW()),
('giovana dias', 'Giovana Dias', ARRAY['giovana', 'Giovana', 'gi', 'Gi', 'giovana dias', 'Giovana Dias'], '901206656686', 'William Duarte > Giovana Dias', '90120707656', true, NOW(), NOW()),
('henrique castro', 'Henrique Castro', ARRAY['henrique', 'Henrique', 'henrique castro', 'Henrique Castro'], '901206656687', 'William Duarte > Henrique Castro', '90120707656', true, NOW(), NOW()),
('isabela freitas', 'Isabela Freitas', ARRAY['isabela', 'Isabela', 'isa', 'Isa', 'isabela freitas', 'Isabela Freitas'], '901206656688', 'William Duarte > Isabela Freitas', '90120707656', true, NOW(), NOW()),
('joao paulo', 'João Paulo', ARRAY['joao', 'João', 'joao paulo', 'João Paulo', 'jp', 'JP'], '901206656689', 'William Duarte > João Paulo', '90120707656', true, NOW(), NOW());

SELECT 'Migration 010 aplicada!' as status;
SELECT COUNT(*) as attendants FROM attendant_mappings;
SELECT COUNT(*) as clients FROM client_mappings;

EOSQL

echo "✅ Migration 010 aplicada com sucesso!"
