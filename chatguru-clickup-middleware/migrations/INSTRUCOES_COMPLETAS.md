# 🚨 MIGRAÇÃO COMPLETA - SETUP INICIAL DO BANCO DE DADOS

## ⚠️ PROBLEMA CRÍTICO IDENTIFICADO

**As tabelas NÃO EXISTEM no banco de dados!**

Os erros que você recebeu:
```
ERROR: relation "prompt_config" does not exist
ERROR: relation "attendant_mappings" does not exist
ERROR: relation "list_cache" does not exist
```

Isso significa que as migrações `001_create_tables.sql` e `002_populate_initial.sql` **NUNCA foram aplicadas**.

---

## 📋 Solução: Executar TODAS as Migrações

Criamos um script consolidado **[FULL_MIGRATION_ALL.sql](FULL_MIGRATION_ALL.sql)** que contém:
- ✅ Migration 001: Criação de todas as tabelas
- ✅ Migration 002: População inicial de dados
- ✅ Migration 003: Correção dos fallbacks para sistema dinâmico

---

## 🚀 PASSO A PASSO (Via Cloud Console)

### Por que Cloud Console?
Seu computador está usando **IPv6**, mas Cloud SQL não suporta IPv6 via `gcloud sql connect`.
O Cloud Shell já está **dentro da rede do GCP**, então não tem esse problema! ✅

---

### 1️⃣ Acesse o Cloud Console
**URL direta**: https://console.cloud.google.com/sql/instances/chatguru-middleware-db/overview?project=buzzlightear

### 2️⃣ Abra o Cloud Shell
- No topo direito da página, clique no ícone de terminal `>_` ("Activate Cloud Shell")
- Aguarde o Cloud Shell inicializar (leva ~10 segundos)

### 3️⃣ Conecte ao Banco de Dados
No Cloud Shell, execute:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```

**Quando pedir a senha**, digite:
```
Nextl@2024
```

### 4️⃣ Execute a Migração Completa

Você verá o prompt `postgres=>`. Agora escolha uma das opções:

#### **OPÇÃO A: Upload do arquivo (Recomendado)**

1. No Cloud Shell, clique no ícone de **3 pontos verticais** (⋮) no canto superior direito
2. Selecione **"Upload"**
3. Faça upload do arquivo `FULL_MIGRATION_ALL.sql` (está em `chatguru-clickup-middleware/migrations/`)
4. No terminal psql, execute:
```sql
\i FULL_MIGRATION_ALL.sql
```

#### **OPÇÃO B: Copiar e colar (Mais rápido, mas menos confiável)**

1. Abra o arquivo [FULL_MIGRATION_ALL.sql](FULL_MIGRATION_ALL.sql) localmente
2. Copie **TODO o conteúdo** (Cmd+A, Cmd+C)
3. Cole diretamente no terminal do Cloud Shell (onde está o prompt `postgres=>`)
4. Pressione Enter

**IMPORTANTE**: Se colar via terminal, fique atento a possíveis erros de caracteres especiais.

---

## 🔍 Verificação Pós-Migração

Após executar a migração, rode estas queries no psql para confirmar:

### 1. Verificar se as tabelas foram criadas
```sql
\dt
```
**Esperado**: Listar todas as tabelas (client_mappings, attendant_mappings, list_cache, categories, subcategories, etc.)

### 2. Verificar fallback correto
```sql
SELECT key, value FROM prompt_config WHERE key = 'fallback_folder_id';
```
**Esperado**: `value = '90130085983'` (space "Clientes Inativos")

### 3. Verificar atendentes mapeados
```sql
SELECT attendant_key, attendant_full_name, space_id FROM attendant_mappings WHERE is_active = true;
```
**Esperado**: 5 atendentes com space_id preenchido:
- anne → 90130178602
- bruna → 90130178610
- mariana_cruz → 90130178618
- mariana_medeiros → 90130178626
- gabriel → 90130178634

### 4. Verificar categorias
```sql
SELECT COUNT(*) FROM categories WHERE is_active = true;
```
**Esperado**: `12` (doze categorias)

### 5. Verificar migrações aplicadas
```sql
SELECT key, value FROM prompt_config WHERE key LIKE 'migration_%';
```
**Esperado**: 3 registros
- migration_001_applied
- migration_002_applied
- migration_003_applied

### 6. Verificar sistema dinâmico habilitado
```sql
SELECT key, value FROM prompt_config WHERE key = 'dynamic_structure_enabled';
```
**Esperado**: `value = 'true'`

---

## ✅ O Que Esta Migração Resolve

### Migration 001 (Estrutura)
- ✅ Cria todas as tabelas necessárias
- ✅ Define índices para performance
- ✅ Cria triggers para auto-update de timestamps

### Migration 002 (Dados Iniciais)
- ✅ Popula 12 categorias do ClickUp
- ✅ Popula tipos de atividade (Rotineira, Específica, Dedicada)
- ✅ Popula status (Executar, Aguardando instruções, Concluído)
- ✅ Popula regras de prompt para OpenAI
- ✅ Cadastra 5 atendentes principais
- ✅ Define fallbacks temporários (corrigidos na migration 003)

### Migration 003 (Sistema Dinâmico)
- ✅ **ANTES**: Fallback apontava para lista do Gabriel (`901300373349`)
- ✅ **DEPOIS**: Fallback aponta para space "Clientes Inativos" (`90130085983`)
- ✅ Mapeia space_id correto para cada atendente
- ✅ Habilita sistema dinâmico de criação de pastas
- ✅ Invalida cache problemático

---

## 🎯 Impacto Esperado

Após aplicar a migração completa:

### ✅ Para Clientes Mapeados
- Cliente + Atendente → Direciona para **space correto do atendente**
- Cria pasta individual no formato "ClienteName"
- Cria lista mensal no formato "OUTUBRO 2025"

### ✅ Para Clientes Inativos (não mapeados)
- Direciona para space **"Clientes Inativos"** (`90130085983`)
- Cria pasta individual com nome do cliente
- Cria lista mensal no formato "ClientName - OUTUBRO 2025"
- **NUNCA mais usará lista do Gabriel como fallback**

### ✅ Cache Inteligente (3 níveis)
- L1: In-memory (1 hora TTL) → Performance
- L2: Database `list_cache` → Persistência
- L3: ClickUp API (quando necessário) → Fonte da verdade

---

## 🆘 Troubleshooting

### Erro: "relation already exists"
Algumas tabelas já podem existir parcialmente. Isso é OK, o script usa `IF NOT EXISTS`.

### Erro: "duplicate key value violates unique constraint"
Alguns dados já podem estar inseridos. Isso é OK, o script usa `ON CONFLICT DO UPDATE`.

### Erro de conexão IPv6
Se ainda encontrar erro de IPv6, certifique-se de estar usando o **Cloud Shell** (não seu terminal local).

### Cloud Shell desconectou
Se a sessão expirar, reconecte:
```bash
gcloud sql connect chatguru-middleware-db --user=postgres --database=postgres
```

---

## 📞 Próximos Passos

Após a migração bem-sucedida:

1. **Reinicie o Cloud Run** para recarregar configurações:
```bash
gcloud run services update chatguru-clickup-middleware \
  --region southamerica-east1 \
  --project=buzzlightear
```

2. **Teste o webhook** com um payload real
3. **Monitore os logs** para confirmar que está direcionando para estruturas corretas
4. **Valide** que clientes inativos NÃO vão mais para lista do Gabriel

---

## 📁 Arquivos Criados

- ✅ [FULL_MIGRATION_ALL.sql](FULL_MIGRATION_ALL.sql) - Script consolidado (001 + 002 + 003)
- ✅ Este guia completo de instruções

---

**Boa sorte! 🚀**
