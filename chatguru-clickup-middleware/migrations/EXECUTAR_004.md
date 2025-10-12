# Executar Migration 004 - Atendentes Completos

## 🚨 Problema Identificado

**LOG DO ERRO**:
```
WARN: ⚠️ Nenhum atendente encontrado - usando fallback 'Gabriel'
```

**ATENDENTES ENCONTRADOS NO SISTEMA** (14 total):
- Anne
- Bruna Senhora
- Carlos
- Georgia
- Graziella
- Mariana
- Mariana Medeiros
- Marilia
- Natalia
- Paloma
- Renata
- Thais Cotts
- Velma
- William

**ATENDENTES NO BANCO** (apenas 5):
- anne
- bruna
- gabriel ❌ (NÃO EXISTE no sistema real!)
- mariana_cruz
- mariana_medeiros

---

## ✅ Solução: Migration 004

### O Que Faz:
1. **Remove** atendente "gabriel" (não existe no sistema real)
2. **Adiciona** os 10 atendentes faltantes
3. **Atualiza** mapeamentos de clientes que referenciavam Gabriel
4. **Corrige** fallback no código (de "Gabriel" → "Clientes Inativos")

---

## 🚀 Como Executar

### Via Cloud Console (Recomendado)

1. **Acesse**: https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear

2. **Abra Cloud Shell** (ícone `>_` no topo direito)

3. **Conecte ao banco**:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```

4. **Digite a senha**: `Nextl@2024`

5. **Copie e cole** o conteúdo de `004_add_missing_attendants.sql`

---

## 📋 Verificação Pós-Migração

```sql
-- 1. Contar atendentes
SELECT COUNT(*) FROM attendant_mappings WHERE is_active = true;
-- Esperado: 14

-- 2. Listar todos
SELECT attendant_key, attendant_full_name, space_id
FROM attendant_mappings
WHERE is_active = true
ORDER BY attendant_key;

-- 3. Verificar Gabriel foi removido
SELECT COUNT(*) FROM attendant_mappings WHERE attendant_key = 'gabriel';
-- Esperado: 0

-- 4. Verificar migração aplicada
SELECT value FROM prompt_config WHERE key = 'migration_004_applied';
```

---

## 🔄 Mudanças no Código (Já Aplicadas)

### `worker.rs` (linha 513-520)

**Antes**:
```rust
let attendant = attendant_opt.unwrap_or_else(|| {
    log_warning("⚠️ Nenhum atendente encontrado - usando fallback 'Gabriel'");
    "Gabriel".to_string()
});
```

**Depois**:
```rust
let attendant = attendant_opt.unwrap_or_else(|| {
    log_warning("⚠️ Nenhum atendente encontrado - tarefa será criada em 'Clientes Inativos'");
    String::new()  // String vazia aciona fallback para "Clientes Inativos"
});
```

---

## 🎯 Resultado Esperado

### Antes:
- ❌ Atendente não encontrado → Usava "Gabriel" (que não existe)
- ❌ Tentava criar no space de Gabriel (não mapeado)
- ❌ Caía em fallback incorreto

### Depois:
- ✅ Atendente não encontrado → String vazia
- ✅ `estrutura.rs` detecta string vazia como "sem atendente"
- ✅ Cria automaticamente em space "Clientes Inativos" (`90130085983`)
- ✅ 14 atendentes cadastrados e disponíveis para mapeamento

---

## 📊 Próximos Passos

Após aplicar a migration 004:

1. **Redeploy** da aplicação (build já deve estar pronto)
2. **Testar** com payload real
3. **Mapear space_id** para os novos atendentes (quando disponível)

```sql
-- Exemplo de mapeamento futuro:
UPDATE attendant_mappings
SET space_id = 'SPACE_ID_AQUI', updated_at = NOW()
WHERE attendant_key = 'william';
```

---

**Criado por**: Claude Code
**Data**: 2025-10-10
**Urgência**: 🔴 ALTA (corrige erro ativo em produção)
