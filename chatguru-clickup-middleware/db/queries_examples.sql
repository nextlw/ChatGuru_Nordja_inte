-- ============================================================================
-- ChatGuru-ClickUp Middleware - SQL Query Examples
-- ============================================================================
-- Useful queries for development, debugging, and analytics
-- ============================================================================

-- ============================================================================
-- BASIC QUERIES - Exploration
-- ============================================================================

-- Ver todas as categorias ordenadas
SELECT
    id,
    name,
    color,
    orderindex
FROM categories
ORDER BY orderindex;

-- Ver todas as subcategorias com dificuldade (stars)
SELECT
    id,
    name,
    stars,
    color,
    orderindex
FROM subcategories
ORDER BY orderindex;

-- Ver relacionamento categoria-subcategoria
SELECT
    c.name AS categoria,
    s.name AS subcategoria,
    s.stars AS dificuldade
FROM categories c
JOIN category_subcategory_mapping csm ON csm.category_id = c.id
JOIN subcategories s ON s.id = csm.subcategory_id
ORDER BY c.orderindex, s.orderindex;

-- ============================================================================
-- ESTRUTURA CLICKUP - Hierarquia Completa
-- ============================================================================

-- Ver toda a estrutura (Team → Space → Folder → List)
SELECT
    t.name AS team,
    s.name AS space_atendente,
    f.name AS folder_cliente,
    l.name AS lista_mensal,
    l.archived AS lista_arquivada
FROM teams t
JOIN spaces s ON s.team_id = t.id
JOIN folders f ON f.space_id = s.id
JOIN lists l ON l.folder_id = f.id
WHERE NOT l.archived
ORDER BY s.name, f.name, l.name DESC;

-- Ver apenas estrutura ativa (não arquivada)
SELECT * FROM v_folder_structure
WHERE NOT space_archived AND NOT folder_archived AND NOT list_archived
ORDER BY space_name, folder_name, list_name;

-- Contar folders por space
SELECT
    s.name AS space,
    COUNT(f.id) AS total_folders,
    COUNT(CASE WHEN NOT f.archived THEN 1 END) AS folders_ativos
FROM spaces s
LEFT JOIN folders f ON f.space_id = s.id
GROUP BY s.id, s.name
ORDER BY total_folders DESC;

-- Contar listas por folder
SELECT
    s.name AS space,
    f.name AS folder,
    COUNT(l.id) AS total_listas,
    COUNT(CASE WHEN NOT l.archived THEN 1 END) AS listas_ativas
FROM folders f
JOIN spaces s ON s.id = f.space_id
LEFT JOIN lists l ON l.folder_id = f.id
GROUP BY s.name, f.id, f.name
ORDER BY s.name, f.name;

-- ============================================================================
-- MAPEAMENTO DINÂMICO - Resolução de Estrutura
-- ============================================================================

-- Ver todos os mapeamentos ativos
SELECT * FROM v_active_mappings
ORDER BY space_name, folder_name;

-- Buscar mapeamento específico (ex: Cliente + Atendente)
SELECT
    fm.client_name,
    fm.attendant_name,
    s.name AS space_name,
    f.name AS folder_name,
    fm.folder_path
FROM folder_mapping fm
JOIN spaces s ON s.id = fm.space_id
JOIN folders f ON f.id = fm.folder_id
WHERE LOWER(fm.client_normalized) = LOWER('gabriel')
AND LOWER(fm.attendant_normalized) = LOWER('renata')
AND fm.is_active = TRUE;

-- Ver todos os aliases de atendentes
SELECT
    alias,
    full_name,
    s.name AS space_name,
    is_active
FROM attendant_aliases aa
LEFT JOIN spaces s ON s.id = aa.space_id
ORDER BY full_name;

-- Buscar alias específico
SELECT
    alias,
    full_name,
    space_id,
    is_active
FROM attendant_aliases
WHERE LOWER(alias) = LOWER('renata');

-- ============================================================================
-- CUSTOM FIELDS - Campos Personalizados
-- ============================================================================

-- Ver todos os custom fields
SELECT
    id,
    name,
    type,
    required,
    date_created
FROM custom_field_types
ORDER BY name;

-- Ver opções de dropdown "Categoria_nova"
SELECT
    c.name AS categoria,
    c.color,
    c.orderindex
FROM categories c
JOIN custom_field_types cft ON cft.id = c.field_id
WHERE cft.name = 'Categoria_nova'
ORDER BY c.orderindex;

-- Ver opções de dropdown "SubCategoria_nova"
SELECT
    s.name AS subcategoria,
    s.stars AS dificuldade,
    s.color,
    s.orderindex
FROM subcategories s
JOIN custom_field_types cft ON cft.id = s.field_id
WHERE cft.name = 'SubCategoria_nova'
ORDER BY s.orderindex;

-- Buscar subcategorias de uma categoria específica
SELECT
    s.name AS subcategoria,
    s.stars AS dificuldade
FROM categories c
JOIN category_subcategory_mapping csm ON csm.category_id = c.id
JOIN subcategories s ON s.id = csm.subcategory_id
WHERE c.name = 'Agendamentos'
ORDER BY s.orderindex;

-- Ver subcategorias mais complexas (4 stars)
SELECT
    c.name AS categoria,
    s.name AS subcategoria,
    s.stars AS dificuldade
FROM subcategories s
JOIN category_subcategory_mapping csm ON csm.subcategory_id = s.id
JOIN categories c ON c.id = csm.category_id
WHERE s.stars = 4
ORDER BY c.name, s.name;

-- ============================================================================
-- CLIENTES - Cliente Solicitante
-- ============================================================================

-- Ver todos os clientes
SELECT
    id,
    name
FROM client_requesters
ORDER BY name;

-- Buscar cliente por nome (case-insensitive)
SELECT * FROM client_requesters
WHERE LOWER(name) LIKE LOWER('%gabriel%');

-- Buscar cliente exato
SELECT * FROM client_requesters
WHERE name = 'Gabriel Benarros';

-- Contar clientes
SELECT COUNT(*) AS total_clientes FROM client_requesters;

-- ============================================================================
-- CACHE - List Cache (3-tier)
-- ============================================================================

-- Ver cache ativo de listas
SELECT
    lc.folder_id,
    f.name AS folder_name,
    lc.list_id,
    lc.full_name AS lista,
    lc.last_verified,
    lc.is_active
FROM list_cache lc
JOIN folders f ON f.id = lc.folder_id
WHERE lc.is_active = TRUE
ORDER BY lc.last_verified DESC;

-- Ver cache expirado (> 1 hora)
SELECT
    lc.folder_id,
    f.name AS folder_name,
    lc.full_name AS lista,
    lc.last_verified,
    NOW() - lc.last_verified AS idade
FROM list_cache lc
JOIN folders f ON f.id = lc.folder_id
WHERE lc.is_active = TRUE
AND lc.last_verified < NOW() - INTERVAL '1 hour'
ORDER BY lc.last_verified;

-- Invalidar cache antigo
UPDATE list_cache
SET is_active = FALSE
WHERE last_verified < NOW() - INTERVAL '1 hour';

-- Limpar cache completamente
UPDATE list_cache SET is_active = FALSE;

-- Ver cache por folder
SELECT
    lc.full_name AS lista,
    lc.month,
    lc.year,
    lc.last_verified,
    lc.is_active
FROM list_cache lc
WHERE lc.folder_id = '90130246641'
ORDER BY lc.year DESC, lc.month DESC;

-- ============================================================================
-- TASKS - Task Cache (Opcional)
-- ============================================================================

-- Ver tasks criadas recentemente
SELECT
    tc.task_id,
    tc.task_name,
    tc.status,
    l.name AS lista,
    f.name AS folder,
    tc.created_at
FROM task_cache tc
JOIN lists l ON l.id = tc.list_id
JOIN folders f ON f.id = l.folder_id
ORDER BY tc.created_at DESC
LIMIT 50;

-- Contar tasks por status
SELECT
    status,
    COUNT(*) AS total
FROM task_cache
GROUP BY status
ORDER BY total DESC;

-- Contar tasks por lista
SELECT
    l.name AS lista,
    f.name AS folder,
    COUNT(tc.id) AS total_tasks
FROM task_cache tc
JOIN lists l ON l.id = tc.list_id
JOIN folders f ON f.id = l.folder_id
GROUP BY l.id, l.name, f.name
ORDER BY total_tasks DESC;

-- ============================================================================
-- ANALYTICS - Estatísticas e Métricas
-- ============================================================================

-- Resumo geral do sistema
SELECT
    'teams' AS tabela,
    COUNT(*) AS total
FROM teams
UNION ALL
SELECT 'spaces', COUNT(*) FROM spaces
UNION ALL
SELECT 'folders', COUNT(*) FROM folders
UNION ALL
SELECT 'lists', COUNT(*) FROM lists
UNION ALL
SELECT 'categories', COUNT(*) FROM categories
UNION ALL
SELECT 'subcategories', COUNT(*) FROM subcategories
UNION ALL
SELECT 'activity_types', COUNT(*) FROM activity_types
UNION ALL
SELECT 'status_options', COUNT(*) FROM status_options
UNION ALL
SELECT 'client_requesters', COUNT(*) FROM client_requesters
UNION ALL
SELECT 'attendant_aliases', COUNT(*) FROM attendant_aliases
UNION ALL
SELECT 'folder_mappings', COUNT(*) FROM folder_mapping
UNION ALL
SELECT 'list_cache', COUNT(*) FROM list_cache
UNION ALL
SELECT 'task_cache', COUNT(*) FROM task_cache
ORDER BY tabela;

-- Distribuição de subcategorias por dificuldade
SELECT
    stars AS dificuldade,
    COUNT(*) AS total_subcategorias,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM subcategories), 2) AS percentual
FROM subcategories
GROUP BY stars
ORDER BY stars;

-- Categorias com mais subcategorias
SELECT
    c.name AS categoria,
    COUNT(csm.subcategory_id) AS total_subcategorias
FROM categories c
LEFT JOIN category_subcategory_mapping csm ON csm.category_id = c.id
GROUP BY c.id, c.name
ORDER BY total_subcategorias DESC;

-- Spaces com mais folders
SELECT
    s.name AS space,
    COUNT(f.id) AS total_folders
FROM spaces s
LEFT JOIN folders f ON f.space_id = s.id
GROUP BY s.id, s.name
ORDER BY total_folders DESC;

-- Folders com mais listas
SELECT
    s.name AS space,
    f.name AS folder,
    COUNT(l.id) AS total_listas
FROM folders f
JOIN spaces s ON s.id = f.space_id
LEFT JOIN lists l ON l.folder_id = f.id
GROUP BY s.name, f.id, f.name
ORDER BY total_listas DESC;

-- ============================================================================
-- SYSTEM CONFIG - Configurações
-- ============================================================================

-- Ver todas as configurações
SELECT
    key,
    value,
    description
FROM system_config
ORDER BY key;

-- Ver configuração específica
SELECT value
FROM system_config
WHERE key = 'cache_ttl_seconds';

-- Atualizar configuração
UPDATE system_config
SET value = '7200'
WHERE key = 'cache_ttl_seconds';

-- ============================================================================
-- MAINTENANCE - Manutenção e Limpeza
-- ============================================================================

-- Verificar tamanho do banco de dados
SELECT
    pg_database.datname AS database_name,
    pg_size_pretty(pg_database_size(pg_database.datname)) AS size
FROM pg_database
WHERE datname = current_database();

-- Verificar tamanho das tabelas
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    pg_size_pretty(pg_indexes_size(schemaname||'.'||tablename)) AS indexes_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Verificar índices
SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_indexes
JOIN pg_class ON pg_class.relname = indexname
WHERE schemaname = 'public'
ORDER BY pg_relation_size(indexrelid) DESC;

-- VACUUM ANALYZE (otimizar todas as tabelas)
VACUUM ANALYZE;

-- VACUUM FULL (compactar banco - requer lock exclusivo)
-- CUIDADO: Bloqueia tabelas durante execução
-- VACUUM FULL;

-- ============================================================================
-- DEBUGGING - Troubleshooting
-- ============================================================================

-- Ver últimas atualizações em cada tabela
SELECT 'teams' AS tabela, MAX(updated_at) AS ultima_atualizacao FROM teams
UNION ALL
SELECT 'spaces', MAX(updated_at) FROM spaces
UNION ALL
SELECT 'folders', MAX(updated_at) FROM folders
UNION ALL
SELECT 'lists', MAX(updated_at) FROM lists
UNION ALL
SELECT 'categories', MAX(updated_at) FROM categories
UNION ALL
SELECT 'subcategories', MAX(updated_at) FROM subcategories
ORDER BY tabela;

-- Ver registros criados recentemente (últimas 24h)
SELECT
    'spaces' AS tabela,
    COUNT(*) AS novos_registros
FROM spaces
WHERE created_at > NOW() - INTERVAL '24 hours'
UNION ALL
SELECT 'folders', COUNT(*) FROM folders
WHERE created_at > NOW() - INTERVAL '24 hours'
UNION ALL
SELECT 'lists', COUNT(*) FROM lists
WHERE created_at > NOW() - INTERVAL '24 hours'
ORDER BY tabela;

-- Ver mapeamentos duplicados (não deveria existir)
SELECT
    client_normalized,
    attendant_normalized,
    COUNT(*) AS total
FROM folder_mapping
GROUP BY client_normalized, attendant_normalized
HAVING COUNT(*) > 1;

-- Ver aliases duplicados (não deveria existir)
SELECT
    alias,
    COUNT(*) AS total
FROM attendant_aliases
GROUP BY alias
HAVING COUNT(*) > 1;

-- Verificar integridade referencial
SELECT
    conname AS constraint_name,
    conrelid::regclass AS table_name,
    confrelid::regclass AS referenced_table
FROM pg_constraint
WHERE contype = 'f'
AND connamespace = 'public'::regnamespace
ORDER BY table_name, constraint_name;

-- ============================================================================
-- ADVANCED QUERIES - Análises Avançadas
-- ============================================================================

-- Análise de categorias por cor
SELECT
    color,
    COUNT(DISTINCT c.id) AS categorias,
    COUNT(DISTINCT s.id) AS subcategorias
FROM categories c
LEFT JOIN category_subcategory_mapping csm ON csm.category_id = c.id
LEFT JOIN subcategories s ON s.id = csm.subcategory_id
GROUP BY color
ORDER BY categorias DESC;

-- Subcategorias sem categoria mapeada (problema!)
SELECT
    s.name AS subcategoria
FROM subcategories s
LEFT JOIN category_subcategory_mapping csm ON csm.subcategory_id = s.id
WHERE csm.id IS NULL;

-- Categorias sem subcategorias
SELECT
    c.name AS categoria
FROM categories c
LEFT JOIN category_subcategory_mapping csm ON csm.category_id = c.id
WHERE csm.id IS NULL;

-- Folders órfãos (sem space)
SELECT
    f.id,
    f.name
FROM folders f
LEFT JOIN spaces s ON s.id = f.space_id
WHERE s.id IS NULL;

-- Listas órfãs (sem folder)
SELECT
    l.id,
    l.name
FROM lists l
LEFT JOIN folders f ON f.id = l.folder_id
WHERE f.id IS NULL;

-- Mapeamentos com space/folder inválido
SELECT
    fm.id,
    fm.client_name,
    fm.attendant_name,
    fm.space_id,
    fm.folder_id
FROM folder_mapping fm
LEFT JOIN spaces s ON s.id = fm.space_id
LEFT JOIN folders f ON f.id = fm.folder_id
WHERE s.id IS NULL OR f.id IS NULL;

-- ============================================================================
-- PERFORMANCE MONITORING - Monitoramento
-- ============================================================================

-- Cache hit ratio (deve ser > 90%)
SELECT
    sum(heap_blks_read) AS heap_read,
    sum(heap_blks_hit) AS heap_hit,
    ROUND(
        sum(heap_blks_hit) / NULLIF(sum(heap_blks_hit) + sum(heap_blks_read), 0) * 100,
        2
    ) AS cache_hit_ratio_percent
FROM pg_statio_user_tables;

-- Tabelas mais acessadas
SELECT
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch
FROM pg_stat_user_tables
ORDER BY seq_scan + idx_scan DESC
LIMIT 10;

-- Índices não utilizados
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes
WHERE idx_scan = 0
ORDER BY schemaname, tablename;

-- Conexões ativas
SELECT
    datname AS database,
    COUNT(*) AS connections
FROM pg_stat_activity
GROUP BY datname
ORDER BY connections DESC;

-- ============================================================================
-- DATA EXPORT - Exportação de Dados
-- ============================================================================

-- Exportar estrutura completa para análise
COPY (
    SELECT
        t.name AS team,
        s.name AS space,
        f.name AS folder,
        l.name AS lista
    FROM teams t
    JOIN spaces s ON s.team_id = t.id
    JOIN folders f ON f.space_id = s.id
    JOIN lists l ON l.folder_id = f.id
    WHERE NOT l.archived
    ORDER BY s.name, f.name, l.name
) TO '/tmp/estrutura_clickup.csv' WITH CSV HEADER;

-- Exportar categorias e subcategorias
COPY (
    SELECT
        c.name AS categoria,
        s.name AS subcategoria,
        s.stars AS dificuldade,
        c.color AS cor_categoria,
        s.color AS cor_subcategoria
    FROM categories c
    JOIN category_subcategory_mapping csm ON csm.category_id = c.id
    JOIN subcategories s ON s.id = csm.subcategory_id
    ORDER BY c.orderindex, s.orderindex
) TO '/tmp/categorias_subcategorias.csv' WITH CSV HEADER;

-- ============================================================================
-- END OF QUERY EXAMPLES
-- ============================================================================
