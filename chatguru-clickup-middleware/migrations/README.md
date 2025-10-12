# Migrations - ChatGuru ClickUp Middleware

## Aplicar Migrações

### Migração 003: Correção dos Fallbacks (CRÍTICA)

Esta migração corrige o problema fundamental onde o sistema estava usando fallbacks incorretos que direcionavam todas as tarefas para a lista específica do Gabriel (`901300373349`) ao invés de usar a estrutura dinâmica.

**Para aplicar (Solução para IPv6):**

```bash
# OPÇÃO 1: Usar Cloud SQL Proxy (Recomendado para IPv6)
# Instalar proxy se não tiver
gcloud components install cloud_sql_proxy

# Conectar via proxy
gcloud beta sql connect chatguru-middleware-db --user=postgres

# OPÇÃO 2: Usar psql direto via IP público
# Primeiro obter o IP da instância
gcloud sql instances describe chatguru-middleware-db --format="value(ipAddresses[0].ipAddress)"

# Conectar diretamente (substitua IP_PUBLICO pelo IP obtido)
psql "host=IP_PUBLICO dbname=postgres user=postgres sslmode=require"

# OPÇÃO 3: Executar via Cloud Shell (se IPv6 persistir)
gcloud cloud-shell ssh --command="gcloud sql connect chatguru-middleware-db --user=postgres"

# Após conectar, executar a migração
\i chatguru-clickup-middleware/migrations/003_fix_fallback_config.sql

# Verificar se foi aplicada
SELECT key, value FROM prompt_config WHERE key = 'migration_003_applied';
```

**Alternativa: Executar via script remoto**

```bash
# Upload da migração para uma máquina VM no GCP e executar de lá
gcloud compute scp chatguru-clickup-middleware/migrations/003_fix_fallback_config.sql VM_NAME:~/
gcloud compute ssh VM_NAME --command="psql 'host=CLOUD_SQL_IP dbname=postgres user=postgres sslmode=require' -f ~/003_fix_fallback_config.sql"
```

**O que esta migração faz:**

1. **Corrige fallback_folder_id**: De lista específica (`901300373349`) para space ID do "Clientes Inativos" (`90130085983`)
2. **Atualiza mapeamentos**: Configura space IDs corretos para cada atendente
3. **Limpa cache**: Remove cache da lista problemática para forçar regeneração
4. **Habilita dinamismo**: Confirma que `dynamic_structure_enabled = true`

**Impacto:**
- ✅ Tarefas de clientes inativos criarão pastas individuais no space "Clientes Inativos"
- ✅ Tarefas de clientes mapeados criarão estrutura no space do atendente correto
- ✅ Fim do direcionamento incorreto para a lista do Gabriel

## Ordem de Aplicação

1. `001_create_tables.sql` - Estrutura inicial
2. `002_populate_initial.sql` - Dados iniciais (com fallbacks incorretos)
3. `003_fix_fallback_config.sql` - **APLICAR IMEDIATAMENTE** - Corrige fallbacks

## Verificação Pós-Migração

```sql
-- Verificar fallbacks corretos
SELECT key, value FROM prompt_config 
WHERE key IN ('fallback_folder_id', 'fallback_folder_path', 'dynamic_structure_enabled');

-- Verificar mapeamentos de atendentes
SELECT attendant_key, space_id FROM attendant_mappings WHERE is_active = true;

-- Verificar se cache problemático foi removido
SELECT * FROM list_cache WHERE list_id = '901300373349';
