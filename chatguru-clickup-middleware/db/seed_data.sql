-- ============================================================================
-- ChatGuru-ClickUp Middleware Seed Data
-- ============================================================================
-- Version: 1.0
-- Description: Initial data population from ClickUp JSON/YAML files
-- ============================================================================

-- ============================================================================
-- TEAMS
-- ============================================================================

INSERT INTO teams (id, name) VALUES
('9013037641', 'Nordja')
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- ============================================================================
-- CUSTOM FIELD TYPES
-- ============================================================================

INSERT INTO custom_field_types (id, name, type, required, hide_from_guests, date_created) VALUES
('eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Categoria_nova', 'drop_down', TRUE, FALSE, 1759359175500),
('5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'SubCategoria_nova', 'drop_down', TRUE, FALSE, 1759359516277),
('f1259ffb-7be8-49ff-92f8-5ff9882888d0', 'Tipo de Atividade', 'drop_down', FALSE, FALSE, NULL),
('6abbfe79-f80b-4b55-9b4b-9bd7f65b6458', 'Status Back Office', 'drop_down', FALSE, FALSE, NULL),
('83afcb8c-2866-498f-9c62-8ea9666b104b', 'Stars', 'number', FALSE, FALSE, NULL),
('0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Cliente Solicitante', 'drop_down', FALSE, FALSE, NULL)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    type = EXCLUDED.type,
    required = EXCLUDED.required;

-- ============================================================================
-- CATEGORIES (Categoria_nova)
-- ============================================================================

INSERT INTO categories (id, field_id, name, color, orderindex) VALUES
('4b6cd768-fb58-48a5-a3d3-6993d2026764', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Agendamentos', '#f900ea', 0),
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Compras', '#02BCD4', 1),
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Documentos', '#0079bf', 2),
('d12372bc-b2c1-4b15-b444-7edc7e477362', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Lazer', '#f2d600', 3),
('e94fdbaa-7442-4579-8f98-3d345a5a862b', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Logística', '#2ecd6f', 4),
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Viagens', '#61bd4f', 5),
('c99d911f-595b-45c4-bb01-15d627d5a62f', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Plano de Saúde', '#eb5a46', 6),
('c2ebd410-5ec1-4eb4-b585-d6bb9a9b9ff3', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Agenda', '#bf55ec', 7),
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Financeiro', '#ffab4a', 8),
('5baa7715-1dfa-4a36-8452-78d60748e193', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Atividades Corporativas', '#FF7FAB', 9),
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Assuntos Pessoais', '#c377e0', 10),
('b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', 'eac5bbd3-4ff6-41ac-aa93-0a13a5a2c75a', 'Gestão de Funcionário', '#81B1FF', 11)
ON CONFLICT (field_id, name) DO UPDATE SET
    color = EXCLUDED.color,
    orderindex = EXCLUDED.orderindex;

-- ============================================================================
-- SUBCATEGORIES (SubCategoria_nova)
-- ============================================================================

INSERT INTO subcategories (id, field_id, name, color, orderindex, stars) VALUES
-- Agendamentos
('bff3cb1c-a75c-42e3-8249-18dbe32296f7', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Consultas', '#f900ea', 0, 1),
('279d40c2-7856-4c74-9190-0b344172ce04', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Exames', '#f900ea', 1, 1),
('dfa4f25f-e67c-42e7-a654-a29bba148635', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Veterinário/Petshop (Consultas/Exames/Banhos/Tosas)', '#f900ea', 2, 1),
('9f6b7d48-e72f-41a5-9998-8fd67e7c0348', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Vacinas', '#f900ea', 3, 1),
('edf4e00d-bc90-4731-9390-77ee25d8f212', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Manicure', '#f900ea', 4, 1),
('8eb7dd7c-a67d-4ff6-9ca7-1134687610b6', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Cabeleleiro', '#f900ea', 5, 1),
-- Compras
('ae9c9144-0676-46ec-b9aa-dcae536d2d82', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Shopper', '#02BCD4', 6, 2),
('6d77b238-5e18-45c8-bce1-502347793853', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Mercados', '#02BCD4', 7, 1),
('ff7ebf1d-e28b-4759-9601-480b67815ca4', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Presentes', '#02BCD4', 8, 1),
('b9fbb46e-95f2-4273-8c93-442dfaaff42f', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Petshop', '#02BCD4', 9, 1),
('101a73fe-9ae7-4e66-bddf-e736e6538d37', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Papelaria', '#02BCD4', 10, 1),
('a8944553-7f2a-41d9-a2ca-dc2c4f734ac8', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Farmácia', '#02BCD4', 11, 2),
('ac4019ae-ed29-4e10-b342-203029295d54', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Ingressos', '#02BCD4', 12, 2),
('576fe8f0-8aab-4dca-b11b-94c9f7f7a67c', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Móveis e Eletros', '#02BCD4', 13, 2),
('e161df73-4977-4642-a116-6b21802092a7', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Itens pessoais e da casa', '#02BCD4', 14, 2),
-- Documentos
('3be771dc-7efe-44de-bc66-8ab833537e75', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'CIN', '#0079bf', 15, 1),
('1ad5c593-6b54-4673-83f7-a239d981fe72', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Certificado Digital', '#0079bf', 16, 2),
('12980d4b-c4bd-4ed7-b36d-060422d023da', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Documento de Vacinação (BR/Iternacional)', '#0079bf', 17, 1),
('6745d1ec-2083-4883-bdb0-00fccd1015f0', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Seguros Carro/Casa/Viagem (anual)', '#0079bf', 18, 2),
('94d4ef32-e631-42ca-87e1-aafb5ddbdcf8', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Assinatura Digital', '#0079bf', 19, 1),
('afc11da3-774f-4560-a92d-6d41c1b5eb60', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Contratos/Procurações', '#0079bf', 20, 1),
('8fd5c543-fd90-4c07-be5e-a470f25c41a0', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Cidadanias', '#0079bf', 21, 4),
('b9a5bac8-e0ad-46f4-9309-9b2ee5acdcf4', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Vistos e Vistos Eletrônicos', '#0079bf', 22, 2),
('04cd634f-28fa-41bb-9f66-dae43c3438ca', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Passaporte', '#0079bf', 23, 1),
('a39b0909-862b-4ead-a20a-26a1846a2ba1', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'CNH', '#0079bf', 24, 1),
('50447800-2557-4653-8874-79bc40473624', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Averbações', '#0079bf', 25, 1),
('3c90b607-8bab-40f6-b96d-75f2888ad9aa', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Certidões', '#0079bf', 26, 1),
-- Lazer
('99e3c2b5-e89f-4095-bcf5-7cdc27a36052', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Reserva de restaurantes/bares', '#f2d600', 27, 1),
('75714f82-0366-429e-9131-4e91e925cf61', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Planejamento de festas', '#f2d600', 28, 4),
('2329a8b6-3d57-4136-b62a-4668cf88a7b6', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Pesquisa de passeios/eventos (BR)', '#f2d600', 29, 3),
('7ae92403-be05-4a57-890c-b2e6d5a8a3be', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Fornecedores no exterior (passeios, fotógrafos)', '#f2d600', 30, 2),
-- Logística
('bbb7859d-5f9a-46ac-a1dc-31a2f50c3bf8', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Corrida de motoboy', '#2ecd6f', 31, 1),
('e536f830-09f9-4643-9c96-a368f7751621', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Motoboy + Correios e envios internacionais', '#2ecd6f', 32, 1),
('0f5385b3-f4bf-4fda-8024-cb68bcfb8081', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Lalamove', '#2ecd6f', 33, 1),
('88e88662-8417-4289-8984-26fd9eef47ef', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Corridas com Taxistas', '#2ecd6f', 34, 1),
('1aea6ab4-24c4-405c-9165-8439fe648e51', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Transporte Urbano (Uber/99)', '#2ecd6f', 35, 1),
-- Viagens
('790f6daf-a1f8-4dcb-a2fb-9b13a635af7b', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Passagens Aéreas', '#61bd4f', 36, 2),
('a7b1efc0-21fb-43be-a1ce-093469a605a0', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Hospedagens', '#61bd4f', 37, 2),
('8e1e0e3f-c8d7-4d45-bee0-2fd7872fa1ce', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Compra de Assentos e Bagagens', '#61bd4f', 38, 1),
('43196e32-64bf-4e18-931b-71d1f55bc1f2', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Passagens de Ônibus', '#61bd4f', 39, 1),
('c16c703c-a0bb-4b16-8b41-a0648d41b267', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Passagens de Trem', '#61bd4f', 40, 2),
('d40e5677-26b9-4191-885a-f23e6c19e905', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Checkins (Early/Late)', '#61bd4f', 41, 1),
('812ae904-2273-4793-8228-eba85228b685', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Extravio de Bagagens', '#61bd4f', 42, 2),
('775b4b9b-f9d4-4cec-bba9-df233acd1f96', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Seguro Viagem (Temporário)', '#61bd4f', 43, 1),
('ba110e55-104c-45f7-9302-bd8e14113c37', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Transfer', '#61bd4f', 44, 2),
('23d3e4cd-7d6d-4c0e-be26-2a771e0e8734', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Programa de Milhagem', '#61bd4f', 45, 1),
('960167db-afb7-48dc-b8e8-12d8f695611a', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Gestão de Contas (CIAs Aereas)', '#61bd4f', 46, 1),
('78df793d-3a57-4f33-baa2-1ec35e93a39c', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Aluguel de Carro/Ônibus e Vans', '#61bd4f', 47, 2),
('b4b361fc-6d60-40b5-9c41-74f54d8d1ec7', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Roteiro de Viagens', '#61bd4f', 48, 3),
-- Plano de Saúde
('76b0f31e-f8a1-4fcb-baca-7fc14bde2b30', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Reembolso Médico', '#eb5a46', 49, 2),
('54373456-c0c6-4a7b-87fb-2332831e2d98', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Extrato para IR', '#eb5a46', 50, 1),
('8ae20f10-16ea-4294-8e3b-cb83c64f7cdb', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Prévia de Reembolso', '#eb5a46', 51, 1),
('af6f081a-49a9-414b-82c4-ddee43fbfb2e', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Contestações', '#eb5a46', 52, 1),
('5c805f34-4479-4ec3-8822-5dffe5489124', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Autorizações', '#eb5a46', 53, 1),
('2fdb5cc9-18a7-451d-be47-c184e537855e', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Contratações/Cancelamentos', '#eb5a46', 54, 2),
-- Agenda
('45f0fa60-db1c-4343-93b0-111f1db221e1', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Gestão de Agenda', '#bf55ec', 55, 1),
('4a7728a4-1754-4325-9dc9-400a7209da9e', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Criação e envio de invites', '#bf55ec', 56, 1),
-- Financeiro
('530e1f11-b1a1-4505-bce5-76bc873dbdb4', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Emissão de NF', '#ffab4a', 57, 1),
('1a84d897-b53c-4713-b526-60eb5c965aab', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Rotina de Pagamentos', '#ffab4a', 58, 1),
('b88bc587-6b13-47ec-a393-793ab335404e', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Emissão de boletos', '#ffab4a', 59, 1),
('f01ad326-41e7-4d1e-914e-9457eabf9e30', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Conciliação Bancária', '#ffab4a', 60, 2),
('af184e5d-bc21-4715-b792-1310e41cef12', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Planilha de Gastos/Pagamentos', '#ffab4a', 61, 4),
('88eb8b12-d65b-464a-923a-7d656ea95a23', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Encerramento e Abertura de CNPJ', '#ffab4a', 62, 2),
('db04f3c8-7d9c-4adb-8a19-c033c189a0f9', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Imposto de Renda', '#ffab4a', 63, 1),
('c2e61adb-92c8-4562-a9b2-8629e2f4a493', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Emissão de Guias de Imposto (DARF, DAS, DIRF, GPS)', '#ffab4a', 64, 1),
-- Assuntos Pessoais
('b77afa16-4a74-4a09-abe6-ccb7338d5b8d', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Mudanças', '#c377e0', 65, 3),
('2b9ecf23-5873-4a26-8b4d-36f9f7498421', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Troca de titularidade', '#c377e0', 66, 1),
('c29d6435-7ed8-4324-af46-ff45eab5fa97', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Assuntos do Carro/Moto', '#c377e0', 67, 1),
('3613c025-d6bb-48a1-9cf4-e1b73f837d61', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Internet e TV por Assinatura', '#c377e0', 68, 1),
('dd76b9a0-18e6-41cc-ade3-23add30748a7', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Contas de Consumo', '#c377e0', 69, 1),
('c09ff878-ff56-401a-b51a-7a06d4af2015', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Anúncio de Vendas Online (Itens, eletros. móveis)', '#c377e0', 70, 3),
('18d4ff39-f15b-493a-9082-057b0b33ba8d', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Assuntos Escolares e Professores Particulares', '#c377e0', 71, 1),
('62a15b40-5462-4886-8fa3-bc8ace5008bc', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Academia e Cursos Livres', '#c377e0', 72, 1),
('bff82ef1-6193-4f13-a363-b6bcfa7694e2', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Telefone', '#c377e0', 73, 1),
('4ff47999-34af-4e6b-8c68-11974bc6c56d', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Assistência Técnica', '#c377e0', 74, 1),
('e1262c25-ace2-4c19-bff3-bc56662f98a6', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Consertos na Casa', '#c377e0', 75, 1),
-- Atividades Corporativas
('9de0acb5-2451-4531-adb5-08e0e929996e', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Financeiro/Contábil', '#FF7FAB', 76, 1),
('5f8c3193-31da-46fc-ba67-0b67bb87fea0', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Atendimento ao Cliente', '#FF7FAB', 77, 1),
('744ec6fb-7d28-43a1-a02d-fca5148993e9', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Gestão de Planilhas e Emails', '#FF7FAB', 78, 4),
('43d79a01-7c60-4902-a308-6a708c8ff6b0', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Documentos/Contratos e Assinaturas', '#FF7FAB', 79, 1),
('f06a6de8-dd51-4df5-895c-d7b3495c1f5c', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Gestão de Agenda (Corporativa)', '#FF7FAB', 80, 1),
('e301567c-e1d4-4095-a709-835b26538e0f', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Recursos Humanos', '#FF7FAB', 81, 1),
('82e3ba06-5125-4357-9b0e-3096a8ca6820', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Gestão de Estoque', '#FF7FAB', 82, 1),
('3c72d198-f18b-4747-8b39-302209e7b11d', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Compras/vendas', '#FF7FAB', 83, 1),
('e42cab5b-6c54-4b70-8616-1ae96bbe0568', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Fornecedores', '#FF7FAB', 84, 2),
-- Gestão de Funcionário
('26b11372-6bbc-4a36-aa5c-2b0cde733413', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'eSocial', '#81B1FF', 85, 1),
('3f14982e-5047-48df-95f3-671fd55bd1da', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Contratações e Desligamentos', '#81B1FF', 86, 1),
('168420f6-f02e-4d80-af97-840411973461', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'DIRF', '#81B1FF', 87, 1),
('3de88e9f-09f2-4a9e-84d9-eff5f987b344', '5333c095-eb40-4a5a-b0c2-76bfba4b1094', 'Férias', '#81B1FF', 88, 1)
ON CONFLICT (field_id, name) DO UPDATE SET
    color = EXCLUDED.color,
    orderindex = EXCLUDED.orderindex,
    stars = EXCLUDED.stars;

-- ============================================================================
-- CATEGORY-SUBCATEGORY MAPPING (Based on ai_prompt.yaml)
-- ============================================================================

INSERT INTO category_subcategory_mapping (category_id, subcategory_id) VALUES
-- Agendamentos
('4b6cd768-fb58-48a5-a3d3-6993d2026764', 'bff3cb1c-a75c-42e3-8249-18dbe32296f7'), -- Consultas
('4b6cd768-fb58-48a5-a3d3-6993d2026764', '279d40c2-7856-4c74-9190-0b344172ce04'), -- Exames
('4b6cd768-fb58-48a5-a3d3-6993d2026764', 'dfa4f25f-e67c-42e7-a654-a29bba148635'), -- Veterinário
('4b6cd768-fb58-48a5-a3d3-6993d2026764', '9f6b7d48-e72f-41a5-9998-8fd67e7c0348'), -- Vacinas
('4b6cd768-fb58-48a5-a3d3-6993d2026764', 'edf4e00d-bc90-4731-9390-77ee25d8f212'), -- Manicure
('4b6cd768-fb58-48a5-a3d3-6993d2026764', '8eb7dd7c-a67d-4ff6-9ca7-1134687610b6'), -- Cabeleleiro
-- Compras
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'ae9c9144-0676-46ec-b9aa-dcae536d2d82'), -- Shopper
('11155a3f-5b4a-46f0-a447-4753bd9c3682', '6d77b238-5e18-45c8-bce1-502347793853'), -- Mercados
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'ff7ebf1d-e28b-4759-9601-480b67815ca4'), -- Presentes
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'b9fbb46e-95f2-4273-8c93-442dfaaff42f'), -- Petshop
('11155a3f-5b4a-46f0-a447-4753bd9c3682', '101a73fe-9ae7-4e66-bddf-e736e6538d37'), -- Papelaria
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'a8944553-7f2a-41d9-a2ca-dc2c4f734ac8'), -- Farmácia
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'ac4019ae-ed29-4e10-b342-203029295d54'), -- Ingressos
('11155a3f-5b4a-46f0-a447-4753bd9c3682', '576fe8f0-8aab-4dca-b11b-94c9f7f7a67c'), -- Móveis e Eletros
('11155a3f-5b4a-46f0-a447-4753bd9c3682', 'e161df73-4977-4642-a116-6b21802092a7'), -- Itens pessoais
-- Documentos
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '3be771dc-7efe-44de-bc66-8ab833537e75'), -- CIN
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '1ad5c593-6b54-4673-83f7-a239d981fe72'), -- Certificado Digital
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '12980d4b-c4bd-4ed7-b36d-060422d023da'), -- Documento de Vacinação
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '6745d1ec-2083-4883-bdb0-00fccd1015f0'), -- Seguros
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '94d4ef32-e631-42ca-87e1-aafb5ddbdcf8'), -- Assinatura Digital
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', 'afc11da3-774f-4560-a92d-6d41c1b5eb60'), -- Contratos/Procurações
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '8fd5c543-fd90-4c07-be5e-a470f25c41a0'), -- Cidadanias
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', 'b9a5bac8-e0ad-46f4-9309-9b2ee5acdcf4'), -- Vistos
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '04cd634f-28fa-41bb-9f66-dae43c3438ca'), -- Passaporte
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', 'a39b0909-862b-4ead-a20a-26a1846a2ba1'), -- CNH
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '50447800-2557-4653-8874-79bc40473624'), -- Averbações
('60b9e5ad-7135-473c-97b2-c18d99b4a2b1', '3c90b607-8bab-40f6-b96d-75f2888ad9aa'), -- Certidões
-- Lazer
('d12372bc-b2c1-4b15-b444-7edc7e477362', '99e3c2b5-e89f-4095-bcf5-7cdc27a36052'), -- Reserva restaurantes
('d12372bc-b2c1-4b15-b444-7edc7e477362', '75714f82-0366-429e-9131-4e91e925cf61'), -- Planejamento festas
('d12372bc-b2c1-4b15-b444-7edc7e477362', '2329a8b6-3d57-4136-b62a-4668cf88a7b6'), -- Pesquisa passeios BR
('d12372bc-b2c1-4b15-b444-7edc7e477362', '7ae92403-be05-4a57-890c-b2e6d5a8a3be'), -- Fornecedores exterior
-- Logística
('e94fdbaa-7442-4579-8f98-3d345a5a862b', 'bbb7859d-5f9a-46ac-a1dc-31a2f50c3bf8'), -- Corrida motoboy
('e94fdbaa-7442-4579-8f98-3d345a5a862b', 'e536f830-09f9-4643-9c96-a368f7751621'), -- Motoboy + Correios
('e94fdbaa-7442-4579-8f98-3d345a5a862b', '0f5385b3-f4bf-4fda-8024-cb68bcfb8081'), -- Lalamove
('e94fdbaa-7442-4579-8f98-3d345a5a862b', '88e88662-8417-4289-8984-26fd9eef47ef'), -- Taxistas
('e94fdbaa-7442-4579-8f98-3d345a5a862b', '1aea6ab4-24c4-405c-9165-8439fe648e51'), -- Uber/99
-- Viagens
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '790f6daf-a1f8-4dcb-a2fb-9b13a635af7b'), -- Passagens Aéreas
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'a7b1efc0-21fb-43be-a1ce-093469a605a0'), -- Hospedagens
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '8e1e0e3f-c8d7-4d45-bee0-2fd7872fa1ce'), -- Assentos/Bagagens
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '43196e32-64bf-4e18-931b-71d1f55bc1f2'), -- Passagens Ônibus
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'c16c703c-a0bb-4b16-8b41-a0648d41b267'), -- Passagens Trem
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'd40e5677-26b9-4191-885a-f23e6c19e905'), -- Checkins
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '812ae904-2273-4793-8228-eba85228b685'), -- Extravio Bagagens
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '775b4b9b-f9d4-4cec-bba9-df233acd1f96'), -- Seguro Viagem
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'ba110e55-104c-45f7-9302-bd8e14113c37'), -- Transfer
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '23d3e4cd-7d6d-4c0e-be26-2a771e0e8734'), -- Programa Milhagem
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '960167db-afb7-48dc-b8e8-12d8f695611a'), -- Gestão Contas
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', '78df793d-3a57-4f33-baa2-1ec35e93a39c'), -- Aluguel Carro
('632bf51e-dc85-44de-8dc7-7d5cd2cdcd5e', 'b4b361fc-6d60-40b5-9c41-74f54d8d1ec7'), -- Roteiro Viagens
-- Plano de Saúde
('c99d911f-595b-45c4-bb01-15d627d5a62f', '76b0f31e-f8a1-4fcb-baca-7fc14bde2b30'), -- Reembolso Médico
('c99d911f-595b-45c4-bb01-15d627d5a62f', '54373456-c0c6-4a7b-87fb-2332831e2d98'), -- Extrato IR
('c99d911f-595b-45c4-bb01-15d627d5a62f', '8ae20f10-16ea-4294-8e3b-cb83c64f7cdb'), -- Prévia Reembolso
('c99d911f-595b-45c4-bb01-15d627d5a62f', 'af6f081a-49a9-414b-82c4-ddee43fbfb2e'), -- Contestações
('c99d911f-595b-45c4-bb01-15d627d5a62f', '5c805f34-4479-4ec3-8822-5dffe5489124'), -- Autorizações
('c99d911f-595b-45c4-bb01-15d627d5a62f', '2fdb5cc9-18a7-451d-be47-c184e537855e'), -- Contratações/Cancelamentos
-- Agenda
('c2ebd410-5ec1-4eb4-b585-d6bb9a9b9ff3', '45f0fa60-db1c-4343-93b0-111f1db221e1'), -- Gestão de Agenda
('c2ebd410-5ec1-4eb4-b585-d6bb9a9b9ff3', '4a7728a4-1754-4325-9dc9-400a7209da9e'), -- Criação invites
-- Financeiro
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', '530e1f11-b1a1-4505-bce5-76bc873dbdb4'), -- Emissão NF
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', '1a84d897-b53c-4713-b526-60eb5c965aab'), -- Rotina Pagamentos
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'b88bc587-6b13-47ec-a393-793ab335404e'), -- Emissão boletos
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'f01ad326-41e7-4d1e-914e-9457eabf9e30'), -- Conciliação Bancária
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'af184e5d-bc21-4715-b792-1310e41cef12'), -- Planilha Gastos
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', '88eb8b12-d65b-464a-923a-7d656ea95a23'), -- Abertura CNPJ
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'db04f3c8-7d9c-4adb-8a19-c033c189a0f9'), -- Imposto Renda
('6c7b5c2c-1d11-4198-8a54-6a748ef750a8', 'c2e61adb-92c8-4562-a9b2-8629e2f4a493'), -- Emissão Guias
-- Assuntos Pessoais
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'b77afa16-4a74-4a09-abe6-ccb7338d5b8d'), -- Mudanças
('72c6a009-bce5-41db-870f-c29d7094dbaf', '2b9ecf23-5873-4a26-8b4d-36f9f7498421'), -- Troca titularidade
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'c29d6435-7ed8-4324-af46-ff45eab5fa97'), -- Assuntos Carro
('72c6a009-bce5-41db-870f-c29d7094dbaf', '3613c025-d6bb-48a1-9cf4-e1b73f837d61'), -- Internet/TV
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'dd76b9a0-18e6-41cc-ade3-23add30748a7'), -- Contas Consumo
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'c09ff878-ff56-401a-b51a-7a06d4af2015'), -- Anúncio Vendas
('72c6a009-bce5-41db-870f-c29d7094dbaf', '18d4ff39-f15b-493a-9082-057b0b33ba8d'), -- Assuntos Escolares
('72c6a009-bce5-41db-870f-c29d7094dbaf', '62a15b40-5462-4886-8fa3-bc8ace5008bc'), -- Academia
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'bff82ef1-6193-4f13-a363-b6bcfa7694e2'), -- Telefone
('72c6a009-bce5-41db-870f-c29d7094dbaf', '4ff47999-34af-4e6b-8c68-11974bc6c56d'), -- Assistência Técnica
('72c6a009-bce5-41db-870f-c29d7094dbaf', 'e1262c25-ace2-4c19-bff3-bc56662f98a6'), -- Consertos Casa
-- Atividades Corporativas
('5baa7715-1dfa-4a36-8452-78d60748e193', '9de0acb5-2451-4531-adb5-08e0e929996e'), -- Financeiro/Contábil
('5baa7715-1dfa-4a36-8452-78d60748e193', '5f8c3193-31da-46fc-ba67-0b67bb87fea0'), -- Atendimento Cliente
('5baa7715-1dfa-4a36-8452-78d60748e193', '744ec6fb-7d28-43a1-a02d-fca5148993e9'), -- Gestão Planilhas
('5baa7715-1dfa-4a36-8452-78d60748e193', '43d79a01-7c60-4902-a308-6a708c8ff6b0'), -- Documentos/Contratos
('5baa7715-1dfa-4a36-8452-78d60748e193', 'f06a6de8-dd51-4df5-895c-d7b3495c1f5c'), -- Gestão Agenda Corp
('5baa7715-1dfa-4a36-8452-78d60748e193', 'e301567c-e1d4-4095-a709-835b26538e0f'), -- RH
('5baa7715-1dfa-4a36-8452-78d60748e193', '82e3ba06-5125-4357-9b0e-3096a8ca6820'), -- Gestão Estoque
('5baa7715-1dfa-4a36-8452-78d60748e193', '3c72d198-f18b-4747-8b39-302209e7b11d'), -- Compras/vendas
('5baa7715-1dfa-4a36-8452-78d60748e193', 'e42cab5b-6c54-4b70-8616-1ae96bbe0568'), -- Fornecedores
-- Gestão de Funcionário
('b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', '26b11372-6bbc-4a36-aa5c-2b0cde733413'), -- eSocial
('b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', '3f14982e-5047-48df-95f3-671fd55bd1da'), -- Contratações
('b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', '168420f6-f02e-4d80-af97-840411973461'), -- DIRF
('b0118e0d-1ae9-4275-bda1-c7651eb8c7d0', '3de88e9f-09f2-4a9e-84d9-eff5f987b344')  -- Férias
ON CONFLICT (category_id, subcategory_id) DO NOTHING;

-- ============================================================================
-- ACTIVITY TYPES (From ai_prompt.yaml)
-- ============================================================================

INSERT INTO activity_types (id, field_id, name, description) VALUES
('64f034f3-c5db-46e5-80e5-f515f11e2131', 'f1259ffb-7be8-49ff-92f8-5ff9882888d0', 'Rotineira', 'tarefas recorrentes e do dia a dia'),
('e85a4dc7-82d8-4f63-89ee-462232f50f31', 'f1259ffb-7be8-49ff-92f8-5ff9882888d0', 'Especifica', 'tarefas pontuais com propósito específico'),
('6c810e95-f5e8-4e8f-ba23-808cf555046f', 'f1259ffb-7be8-49ff-92f8-5ff9882888d0', 'Dedicada', 'tarefas que demandam dedicação especial')
ON CONFLICT (field_id, name) DO UPDATE SET description = EXCLUDED.description;

-- ============================================================================
-- STATUS OPTIONS (From ai_prompt.yaml)
-- ============================================================================

INSERT INTO status_options (id, field_id, name) VALUES
('7889796f-033f-450d-97dd-6fee2a44f1b1', '6abbfe79-f80b-4b55-9b4b-9bd7f65b6458', 'Executar'),
('dd9d1b1b-f842-4777-984d-c05ec6b6d8a3', '6abbfe79-f80b-4b55-9b4b-9bd7f65b6458', 'Aguardando instruções'),
('db544ddc-a07d-47a9-8737-40c6be25f7ec', '6abbfe79-f80b-4b55-9b4b-9bd7f65b6458', 'Concluido')
ON CONFLICT (field_id, name) DO NOTHING;

-- ============================================================================
-- CLIENT REQUESTERS (Cliente Solicitante from cliente_solicitante_mappings.yaml)
-- ============================================================================

INSERT INTO client_requesters (id, field_id, name) VALUES
('de78687a-b713-4313-ba72-704174cda7da', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Adriana Tavares'),
('7d5aa97f-a27d-49c9-b5ee-556b588a73f2', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Agencia Fluida (Bia Guedes)'),
('5b206f2a-cd59-4b9d-b785-1521b62c894e', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Alejandra Aguilera'),
('bcc5fda2-1bab-419a-ad94-c6ee32707707', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Alessandra Cardim'),
('600227bb-3d52-405d-b5a8-68f0de3aa94a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Alex Ikonomopoulos'),
('8d5d53b9-b74f-4bbc-a8ad-f6ad2c178942', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Aline Bak'),
('67cfc4a6-8992-4247-b2c1-e1a415968da4', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Aline Hochman'),
('7d260e98-b23f-4f6f-b1ee-bed76e4720fe', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Amadeu e Rebeka'),
('a68db787-eb0c-43a3-a649-ce03cad23df0', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Amure Pinho'),
('2b1d97dd-a2d4-4c90-90c2-c8392af03b00', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Ana Flavia Carrilo'),
('4dafba51-6b09-40eb-a2af-0e17f3eeb868', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Ana Flavia Tavares'),
('4ff97f3a-d6b6-4ec0-b25b-0704e33a02fe', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Anabela Cunha'),
('e8d59699-9cb8-4c98-bb59-38179b53034f', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Anarella'),
('1e9f79c0-7b94-4da7-91c2-2945a3ef29c4', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Andrew Reiter'),
('0bf8e67f-a673-4645-b724-21e51cd1f832', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Arcelia / Jorel '),
('71fbe4d2-4585-48ae-a1e2-b25c497d511c', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Arnaldo e Claudia'),
('4a485872-0be5-4870-a500-427f012c8ee2', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Breno / Leticia'),
('0e9cfa68-dddb-4d9a-a117-b21e740344ab', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Caio Dalfonso'),
('9beb4800-ee7a-4e61-b700-ec9ab9a6184a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Carlos Secron'),
('2e569229-8f89-4b79-a1fe-54c5e6aaae12', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Carolina Oliveira'),
('5615ac61-6956-4c86-a311-d6591c73a0b3', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Carolina Tavares'),
('4f24b242-9c79-4b57-8e72-76b09b04358c', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Claudia Leicand'),
('41609996-132c-42f4-b090-8fedace8c09e', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Claudio Franco / Gabriela'),
('da2e8dc1-c519-4d39-9970-8fd17df5d152', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Dadá Ribeiro'),
('7d672947-a234-4d7a-b5bf-e02fe48c2452', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Daniel Rodrigues'),
('81bce028-b080-4a15-bc44-72195fe51403', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Daniela Sitzer'),
('c066fdc9-fb76-48a0-8a23-b23d335d9256', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'David Politanski'),
('1e191518-3508-425b-b3bc-7cc7ff2f969e', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Eduardo / Isabella Vasconcellos'),
('81385f03-aa3e-4e40-b6af-cdb426f73aea', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Edwilson Coutinho'),
('b26c2bbe-8f1d-4eb7-8f96-62123a51f40b', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Eloa Figaro'),
('00e5ad55-8c3a-4ea8-a21c-ad9c08b66e55', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Emylia Chamorro'),
('688bdab9-1c7e-435e-81cd-d255cf7cf53a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Fabio Tieopolo'),
('c6642cf2-6723-45fe-ae6d-974ce1c5815a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Fabricio Ramos'),
('153ff4fd-0919-410f-b445-384667fc0bae', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Fernanda Cruz'),
('8b34a94b-8e12-4e11-b6ba-0c50d185664d', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Fernanda Munhoz'),
('9276e8a1-a9ad-4c10-8a00-392726646ccf', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Gabriel Benarros'),
('687c1796-573a-425f-ae58-f27b52537cd9', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Gabriela Avian'),
('4c88769e-4a81-4122-b52a-0875362df30e', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Guilherme Lopes'),
('0d631381-9347-43d0-a94a-02deb34b938b', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Henrique Caldeira'),
('c5c5fa4e-aef7-4506-87c5-fe7e42da1056', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Hernandes Sousa'),
('37f134c9-b3f5-4947-aa8d-9e18b3c42aa8', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Hugo / NSA Global'),
('ca5e4f88-f1d8-4880-a915-39fa848f9ede', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Humberto Siuves'),
('49284e0c-843e-42e9-ac6f-50a3338d189a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Hurá BIttencourt'),
('f2f968b2-f47d-41a4-a1d5-ccd50eb53124', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Ian Gallina'),
('cd95ab7a-4dc1-4664-bccb-6fdecabd9f20', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Iara Biderman'),
('508df88a-baab-4da7-9df9-f7d1ce0b4385', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Igor Marchesini'),
('649e8429-f80c-4ee5-bdda-fb145dc43ed7', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Ille Cosmeticos'),
('2d1c1c37-2609-46ba-a52e-b72761ed1a14', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Jack Sarvary/Elena Mariscal'),
('90a87a69-67ee-4636-95a7-02dab3ed2e44', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Jessica Coral'),
('1a6ccf83-3247-4b96-ad06-f4cb71ef4c24', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'José Muritiba'),
('02159429-0da8-4cde-900c-6420f3085d48', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Julia Cambiaghi e Guilherme Bittencourt'),
('979c47eb-d7fb-4f24-9f94-ed795e4af360', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Kiko Augusto (Rei do Pitaco)'),
('2210f404-fff4-407a-8f31-6f63f690544b', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Laynara Coelho'),
('48852090-1c35-417c-95d7-7fd4d5973dc4', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Leo Laranjeira'),
('575b5f67-b2f3-4643-ac34-59ebd5923b83', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Lilian Palacios'),
('b4867ab9-6f1e-45dd-8abe-a46420bafdd8', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Lucas Riani'),
('ef01b0d7-c086-4ab7-8dec-77b4bbe689ab', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Luciana Stracieri'),
('682250ea-fa76-482c-8486-a4cc8a4bfdab', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Luft Shoes'),
('e117d4bd-8f13-43b2-b41d-e8ab11137e58', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Marcelo Caldas e Maria Gabriela'),
('788a961d-6ec6-4f0a-a231-5b52a1568853', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Marcos Bissi'),
('ecdcbf97-11bd-4728-889b-997e7c2fecfa', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Margareth Darezzo'),
('4739b67f-faeb-47cd-b607-2d33d07f6e82', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Mariana Paixão'),
('d67a370b-ea91-4db8-a08e-d4a1db2463dd', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Mariana e David'),
('f8a40a4e-a80f-4ddc-add1-7bdb9ec9de48', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Matheus Benarrós'),
('87191322-d57d-4d4f-b567-5604e16a34eb', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Mauricio Carvalho'),
('0958417a-6b3c-46a2-b6d7-0354c015680c', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Namari'),
('54dd8cb0-5fde-4081-ab88-f2213e7d0aee', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Nicholas Vasconcellos'),
('b942c518-5174-4833-9af6-25045228f00f', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Nino Vashakidze'),
('00c03954-0762-47e2-b9d3-8e2e61481904', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Nordja'),
('98e14439-d7c0-4ff7-9b93-31eaff0e695a', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Padaria dos Bebês'),
('30894690-ca70-4112-8c87-0c98929b9ffb', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Paula Mejia Giraldo'),
('57790c7d-c166-4ae4-8771-7d375bb800ee', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Peter Fernandez'),
('6d87e92c-8482-46e2-9bfc-825e9674eee6', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Rafael Marcondes (Rei do Pitaco)'),
('abeb7e51-2ca7-4322-9a91-4a3b7f4ebd85', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Raphaela Spielberg'),
('546f6bb6-a322-4b03-8514-9c3cdef2c0b2', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Renarto Casseb'),
('26c43b1b-143a-4719-9aca-280769a946aa', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Renata Michaelis'),
('fda6a369-93b7-4492-95d3-c389d1cfa130', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Ricardo Sitzer'),
('142d17ef-ce0b-426f-b9ab-ea38edd87b7f', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Roberto Cerdeira'),
('a9de95c5-9e58-43d8-bea0-f1fea046ac1d', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Rodolfo e Claudia'),
('5bfe93c7-073d-4260-8fe6-a02e0aa35e54', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Samir Kerbage'),
('50d69abb-01e9-4de5-8a7d-0e3b6ef765e7', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Sandra / Grupo Casa'),
('aefb6314-37fe-4e3d-9be6-b275b979e58b', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Silvia Villas Boas'),
('a8d6b4de-3d1b-4e62-af2e-aa1470eaa526', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Stephanie Zalcman'),
('7ad2e6e3-4016-4aeb-86dc-bce5448909f1', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Suco Seguros'),
('a799f7a4-c618-4392-af77-7b803252a92d', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Thiago Martins (Rei do Pitaco)'),
('ca8796c9-b0af-456a-a61b-629467c86400', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Tivita'),
('bcee668b-a4f9-49dd-947c-681667564ba6', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Vicente Carrari'),
('45ac5079-6af3-4445-987d-0d24a32ce64c', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Victor Carbone'),
('9ec4b6f3-3cf9-4b61-b7a0-1a9fe614283b', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'William Duarte'),
('b0dc12f8-8e88-4f28-a2c1-0cf71f67eea9', '0ed63eec-1c50-4190-91c1-59b4b17557f6', 'Yoanna ')
ON CONFLICT (name) DO UPDATE SET field_id = EXCLUDED.field_id;

-- ============================================================================
-- ATTENDANT ALIASES (From ai_prompt.yaml aliases_responsaveis)
-- ============================================================================

-- Note: Space IDs will need to be populated after spaces are synced from ClickUp
INSERT INTO attendant_aliases (alias, full_name, is_active) VALUES
('renata', 'Renata Schnoor', TRUE),
('anne', 'Anne Souza', TRUE),
('bruna', 'Bruna Senhora', TRUE),
('mariana_medeiros', 'Mariana Medeiros', TRUE),
('mariana_cruz', 'Mariana Cruz', TRUE),
('georgia', 'Georgia', TRUE),
('velma', 'Velma Fortes', TRUE),
('thais', 'Thaís Cotts', TRUE),
('natalia', 'Natália Branco', TRUE),
('intl', 'Intl Affairs', TRUE),
('affairs', 'Intl Affairs', TRUE)
ON CONFLICT (alias) DO UPDATE SET
    full_name = EXCLUDED.full_name,
    is_active = EXCLUDED.is_active;

-- ============================================================================
-- SYSTEM CONFIG
-- ============================================================================

INSERT INTO system_config (key, value, description) VALUES
('default_space_inactive', 'Clientes Inativos', 'Default space for unmapped clients'),
('cache_ttl_seconds', '3600', 'Cache TTL in seconds (1 hour)'),
('webhook_timeout_ms', '100', 'Webhook ACK timeout in milliseconds'),
('worker_timeout_seconds', '30', 'Worker processing timeout in seconds'),
('schema_version', '1.0', 'Database schema version')
ON CONFLICT (key) DO UPDATE SET
    value = EXCLUDED.value,
    description = EXCLUDED.description;

-- ============================================================================
-- END OF SEED DATA
-- ============================================================================

-- Display summary
SELECT 'Seed data inserted successfully!' AS status;
SELECT COUNT(*) AS total_categories FROM categories;
SELECT COUNT(*) AS total_subcategories FROM subcategories;
SELECT COUNT(*) AS total_mappings FROM category_subcategory_mapping;
SELECT COUNT(*) AS total_client_requesters FROM client_requesters;
SELECT COUNT(*) AS total_attendant_aliases FROM attendant_aliases;
