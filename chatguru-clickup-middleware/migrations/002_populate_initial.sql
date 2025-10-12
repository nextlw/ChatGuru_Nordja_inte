-- Migration 002: População inicial das tabelas de configuração
-- Data: 2025-10-08
-- IMPORTANTE: Este script popula as tabelas de configuração a partir do ai_prompt.yaml

-- ==============================================================================
-- CONFIGURAÇÕES DE PROMPT
-- ==============================================================================

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

-- ==============================================================================
-- CATEGORIAS
-- ==============================================================================

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

-- ==============================================================================
-- TIPOS DE ATIVIDADE
-- ==============================================================================

INSERT INTO activity_types (name, description, clickup_field_id) VALUES
('Rotineira', 'tarefas recorrentes e do dia a dia', '64f034f3-c5db-46e5-80e5-f515f11e2131'),
('Especifica', 'tarefas pontuais com propósito específico', 'e85a4dc7-82d8-4f63-89ee-462232f50f31'),
('Dedicada', 'tarefas que demandam dedicação especial', '6c810e95-f5e8-4e8f-ba23-808cf555046f')
ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description, clickup_field_id = EXCLUDED.clickup_field_id, updated_at = NOW();

-- ==============================================================================
-- STATUS
-- ==============================================================================

INSERT INTO status_options (name, clickup_field_id, display_order) VALUES
('Executar', '7889796f-033f-450d-97dd-6fee2a44f1b1', 1),
('Aguardando instruções', 'dd9d1b1b-f842-4777-984d-c05ec6b6d8a3', 2),
('Concluido', 'db544ddc-a07d-47a9-8737-40c6be25f7ec', 3)
ON CONFLICT (name) DO UPDATE SET clickup_field_id = EXCLUDED.clickup_field_id, display_order = EXCLUDED.display_order, updated_at = NOW();

-- ==============================================================================
-- REGRAS DE PROMPT (primeiras 15 regras mais importantes)
-- ==============================================================================

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

-- ==============================================================================
-- ATENDENTES PRINCIPAIS (dados iniciais)
-- ==============================================================================

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
