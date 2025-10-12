# Migration 005: Atualizar Space IDs dos Atendentes

**Data**: 2025-10-10
**Prioridade**: 🔴 CRÍTICA
**Dependência**: Migration 004 deve estar aplicada

---

## 🚨 Problema Identificado

### IDs do Banco vs IDs Reais da API

| Atendente | Banco (ERRADO) | API (CORRETO) | Status |
|-----------|----------------|---------------|--------|
| Anne | `90130178602` | `90131713706` | ❌ DIFERENTE |
| Bruna | `90130178610` | `90132952032` | ❌ DIFERENTE |
| Mariana Cruz | `90130178618` | `90134505966` | ❌ DIFERENTE |
| Mariana Medeiros | `90130178626` | `90134254183` | ❌ DIFERENTE |

**Impacto**: Tasks estão sendo criadas nos spaces ERRADOS!

---

## ✅ Mapeamento Completo (14 Atendentes)

### Com Space Próprio (10 atendentes):

| Atendente | Space ID | Space Name |
|-----------|----------|------------|
| Anne | `90131713706` | Anne Souza |
| Bruna | `90132952032` | Bruna Senhora |
| Georgia | `90130086319` | Georgia |
| Graziella | `90134506045` | Graziella Leite |
| Mariana Cruz | `90134505966` | Mariana Cruz |
| Mariana Medeiros | `90134254183` | Mariana Medeiros |
| Natalia | `901310948326` | Natália Branco |
| Renata | `90131747051` | Renata Schnoor |
| Thais | `90131747001` | Thaís Cotts |
| Velma | `90130187145` | Velma Fortes |

### Sem Space Próprio (4 atendentes):
Estes usarão "Clientes Inativos" (`90130085983`) como fallback:

| Atendente | Status |
|-----------|--------|
| Carlos | ⚠️ NULL → Usará Clientes Inativos |
| Marilia | ⚠️ NULL → Usará Clientes Inativos |
| Paloma | ⚠️ NULL → Usará Clientes Inativos |
| William | ⚠️ NULL → Usará Clientes Inativos |

---

## 🚀 Como Executar

### Via Cloud Console (Recomendado)

1. **Acesse**: https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear

2. **Abra Cloud Shell** (ícone `>_`)

3. **Conecte ao banco**:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```
Senha: `Nextl@2024`

4. **Copie e cole** o conteúdo de `005_update_space_ids.sql`

5. **Resultado esperado**:
```
=== RESULTADO DA MIGRAÇÃO 005 ===
Atendentes COM space próprio: 10 (esperado: 10)
Atendentes SEM space próprio: 4 (esperado: 4 - Carlos, Marilia, Paloma, William)
✅ Space IDs atualizados corretamente!

attendant_key     | attendant_full_name | space_id      | status
------------------+---------------------+---------------+--------------------------------
anne              | Anne                | 90131713706   | ✅ TEM SPACE
bruna             | Bruna Senhora       | 90132952032   | ✅ TEM SPACE
carlos            | Carlos              |               | ⚠️ SEM SPACE (usará Clientes Inativos)
georgia           | Georgia             | 90130086319   | ✅ TEM SPACE
graziella         | Graziella           | 90134506045   | ✅ TEM SPACE
mariana_cruz      | Mariana             | 90134505966   | ✅ TEM SPACE
mariana_medeiros  | Mariana Medeiros    | 90134254183   | ✅ TEM SPACE
marilia           | Marilia             |               | ⚠️ SEM SPACE (usará Clientes Inativos)
natalia           | Natalia             | 901310948326  | ✅ TEM SPACE
paloma            | Paloma              |               | ⚠️ SEM SPACE (usará Clientes Inativos)
renata            | Renata              | 90131747051   | ✅ TEM SPACE
thais             | Thais Cotts         | 90131747001   | ✅ TEM SPACE
velma             | Velma               | 90130187145   | ✅ TEM SPACE
william           | William             |               | ⚠️ SEM SPACE (usará Clientes Inativos)
```

---

## 📋 Verificação Manual (Opcional)

Se quiser verificar antes de aplicar:

```sql
-- Ver IDs atuais (ERRADOS)
SELECT attendant_key, attendant_full_name, space_id
FROM attendant_mappings
WHERE attendant_key IN ('anne', 'bruna', 'mariana_cruz', 'mariana_medeiros')
AND is_active = true;
```

---

## 🎯 Impacto Esperado

### Antes da Migração 005:
```
Webhook com responsavel_nome="Anne"
→ Busca no banco: space_id = '90130178602' ❌ ERRADO
→ Tenta criar task no space errado
→ ❌ FALHA ou cria no lugar errado
```

### Depois da Migração 005:
```
Webhook com responsavel_nome="Anne"
→ Busca no banco: space_id = '90131713706' ✅ CORRETO
→ Cria pasta no space correto de Anne
→ ✅ Task criada no local correto
```

---

## 🔍 Como Foi Descoberto

1. Consultei a API do ClickUp:
```bash
curl -X GET "https://api.clickup.com/api/v2/team/9013037641/space?archived=false" \
  -H "Authorization: TOKEN"
```

2. Comparei com IDs do banco

3. Identifiquei discrepâncias

---

## 📊 Spaces Disponíveis no ClickUp

Todos os spaces do workspace:

| Space Name | Space ID | Uso |
|------------|----------|-----|
| Clientes Inativos | 90130085983 | Fallback |
| Anne Souza | 90131713706 | ✅ Anne |
| Bruna Senhora | 90132952032 | ✅ Bruna |
| Georgia | 90130086319 | ✅ Georgia |
| Graziella Leite | 90134506045 | ✅ Graziella |
| Mariana Cruz | 90134505966 | ✅ Mariana Cruz |
| Mariana Medeiros | 90134254183 | ✅ Mariana Medeiros |
| Natália Branco | 901310948326 | ✅ Natalia |
| Renata Schnoor | 90131747051 | ✅ Renata |
| Thaís Cotts | 90131747001 | ✅ Thais |
| Velma Fortes | 90130187145 | ✅ Velma |
| Clientes Esporádicos | 90131779539 | Disponível |
| Intl Affairs | 90130096830 | Disponível |
| Base de Conhecimento | 90130160441 | Disponível |

---

## ⚠️ Atenção

- **Georgia**: Está na lista de atendentes mas não estava no banco original
- **Carlos, Marilia, Paloma, William**: Não têm space próprio (usarão fallback)
- Se algum atendente criar um space novo futuramente, basta adicionar uma nova migration

---

## 🔄 Ordem de Execução

1. ✅ Migration 003 (fallback correto)
2. ✅ Migration 004 (adiciona 14 atendentes)
3. 🔄 **Migration 005 (corrige space IDs)** ← VOCÊ ESTÁ AQUI
4. ⏳ Deploy nova versão do código
5. ⏳ Remover secret `clickup-list-id`

---

**Criado por**: Claude Code
**Data**: 2025-10-10 13:00 UTC
**Fonte**: API do ClickUp consultada em tempo real
