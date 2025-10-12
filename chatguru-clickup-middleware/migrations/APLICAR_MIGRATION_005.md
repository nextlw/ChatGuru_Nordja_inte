# Como Aplicar Migration 005

## ✅ Build Completado
Build ID: `09e22bfa-b80e-40aa-a309-c0513a5c2c37` (SUCCESS)

## 📋 Próximo Passo: Aplicar Migration 005

### Via Cloud Console (Recomendado)

1. **Acesse o Cloud Console**:
   ```
   https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear
   ```

2. **Abra o Cloud Shell** (ícone `>_` no topo direito)

3. **Conecte ao banco**:
   ```bash
   gcloud sql connect chatguru-middleware-db --user=postgres --database=chatguru_middleware
   ```
   Senha: `Nextl@2024`

4. **Copie e cole** o conteúdo do arquivo:
   ```
   migrations/005_update_space_ids.sql
   ```

5. **Resultado esperado**:
   ```
   UPDATE 4  -- Anne, Bruna, Mariana Cruz, Mariana Medeiros
   UPDATE 6  -- Georgia, Graziella, Natalia, Renata, Thais, Velma
   UPDATE 4  -- Carlos, Marilia, Paloma, William → Clientes Inativos
   UPDATE 9  -- Aliases

   NOTICE: === RESULTADO DA MIGRAÇÃO 005 ===
   NOTICE: Atendentes COM space próprio: 10 (esperado: 10)
   NOTICE: Atendentes COM space Clientes Inativos: 0 (esperado: 4)
   NOTICE: ✅ Space IDs atualizados corretamente! (10 com space próprio + 4 com Clientes Inativos = 14 total)
   ```

6. **Verificação**:
   Você verá a lista de 14 atendentes com seus respectivos space IDs:
   - 10 com `✅ SPACE PRÓPRIO`
   - 4 com `⚠️ CLIENTES INATIVOS` (Carlos, Marilia, Paloma, William)

---

## 🔄 Mudança Implementada

### Antes (versão antiga):
- Carlos, Marilia, Paloma, William → space_id = NULL

### Depois (versão atualizada):
- Carlos, Marilia, Paloma, William → space_id = `90130085983` (Clientes Inativos)

**Vantagem**:
- Todos os atendentes têm space_id definido no banco
- Lógica mais simples (não precisa tratar NULL)
- Mais explícito onde cada atendente vai criar tasks

---

## 🚀 Depois da Migration 005

### 1. Deploy no Cloud Run
```bash
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --project=buzzlightear
```

### 2. Remover Secret Hardcoded
```bash
gcloud secrets delete clickup-list-id --project=buzzlightear
```

### 3. Teste com Payload Real
Envie um webhook do ChatGuru e monitore os logs:
```bash
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=chatguru-clickup-middleware" \
  --limit=50 --project=buzzlightear
```

---

## 📊 Mapeamento Final (após migration 005)

### Atendentes com Space Próprio (10):
- Anne → `90131713706` (Anne Souza)
- Bruna → `90132952032` (Bruna Senhora)
- Georgia → `90130086319` (Georgia)
- Graziella → `90134506045` (Graziella Leite)
- Mariana Cruz → `90134505966` (Mariana Cruz)
- Mariana Medeiros → `90134254183` (Mariana Medeiros)
- Natalia → `901310948326` (Natália Branco)
- Renata → `90131747051` (Renata Schnoor)
- Thais → `90131747001` (Thaís Cotts)
- Velma → `90130187145` (Velma Fortes)

### Atendentes em Clientes Inativos (4):
- Carlos → `90130085983` (Clientes Inativos)
- Marilia → `90130085983` (Clientes Inativos)
- Paloma → `90130085983` (Clientes Inativos)
- William → `90130085983` (Clientes Inativos)

**Total**: 14 atendentes ✅
