# Execução Manual da Migração 003

## Problema IPv6
Seu computador está usando IPv6, que o Cloud SQL não suporta diretamente via `gcloud sql connect`.

## Soluções para Executar

### OPÇÃO 1: Via Cloud Console (Mais Simples) ⭐ RECOMENDADO

1. Acesse: https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear

2. Clique em "OPEN CLOUD SHELL" (ícone terminal no topo direito)

3. No Cloud Shell, execute:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```

4. Digite a senha: `Nextl@2024`

5. Copie e cole os comandos SQL abaixo (um por vez ou todos de uma vez):

```sql
-- 1. Atualizar fallback_folder_id para space "Clientes Inativos"
UPDATE prompt_config
SET value = '90130085983', updated_at = NOW()
WHERE key = 'fallback_folder_id' AND is_active = true;

-- 2. Atualizar fallback_folder_path
UPDATE prompt_config
SET value = 'Clientes Inativos', updated_at = NOW()
WHERE key = 'fallback_folder_path' AND is_active = true;

-- 3. Adicionar configurações do sistema dinâmico
INSERT INTO prompt_config (key, value, config_type) VALUES
('default_inactive_space_id', '90130085983', 'text'),
('dynamic_structure_enabled', 'true', 'boolean')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();

-- 4. Mapear atendentes com space IDs corretos
UPDATE attendant_mappings SET space_id = '90130178602', updated_at = NOW() WHERE attendant_key = 'anne';
UPDATE attendant_mappings SET space_id = '90130178610', updated_at = NOW() WHERE attendant_key = 'bruna';
UPDATE attendant_mappings SET space_id = '90130178618', updated_at = NOW() WHERE attendant_key = 'mariana_cruz';
UPDATE attendant_mappings SET space_id = '90130178626', updated_at = NOW() WHERE attendant_key = 'mariana_medeiros';
UPDATE attendant_mappings SET space_id = '90130178634', updated_at = NOW() WHERE attendant_key = 'gabriel';

-- 5. Invalidar cache da lista problemática
UPDATE list_cache
SET is_active = false, last_verified = NOW()
WHERE list_id = '901300373349';

-- 6. Registrar migração aplicada
INSERT INTO prompt_config (key, value, config_type) VALUES
('migration_003_applied', NOW()::text, 'text')
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW();
```

6. Verifique se foi aplicado:
```sql
-- Verificar fallbacks atualizados
SELECT key, value FROM prompt_config WHERE key LIKE 'fallback%' OR key LIKE 'default_inactive%' OR key LIKE 'dynamic_structure%';

-- Verificar atendentes mapeados
SELECT attendant_key, space_id FROM attendant_mappings WHERE is_active = true;

-- Verificar cache invalidado
SELECT list_id, is_active FROM list_cache WHERE list_id = '901300373349';
```

### OPÇÃO 2: Via Terminal Remoto (SSH para máquina com IPv4)

Se você tiver acesso a uma máquina remota com IPv4:
```bash
ssh usuario@maquina-remota
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres --project=buzzlightear
# Cole os comandos SQL acima
```

### OPÇÃO 3: Via VPN/NAT64 (Se disponível)

Configure uma VPN que suporte IPv4 e execute normalmente:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```

## Verificação Final

Após executar, rode estas queries para confirmar:

```sql
-- 1. Verificar fallback correto
SELECT key, value FROM prompt_config WHERE key = 'fallback_folder_id';
-- Esperado: value = '90130085983'

-- 2. Verificar atendentes
SELECT COUNT(*) FROM attendant_mappings WHERE space_id IS NOT NULL AND is_active = true;
-- Esperado: 5 (todos os atendentes)

-- 3. Verificar migração aplicada
SELECT value FROM prompt_config WHERE key = 'migration_003_applied';
-- Esperado: timestamp de hoje
```

## O Que Esta Migração Corrige

✅ **Antes**: Fallback apontava para lista do Gabriel (`901300373349`)
✅ **Depois**: Fallback aponta para space "Clientes Inativos" (`90130085983`)

✅ **Antes**: Atendentes sem space_id mapeado
✅ **Depois**: Cada atendente tem seu space_id correto

✅ **Antes**: Cache direcionava para lista errada
✅ **Depois**: Cache invalidado, sistema dinâmico funciona

## Impacto Esperado

Após aplicar:
- ✅ Clientes mapeados → Vão para space do atendente correto
- ✅ Clientes inativos → Vão para space "Clientes Inativos" com pasta individual
- ✅ Fim do problema → NUNCA mais usará lista do Gabriel como fallback
