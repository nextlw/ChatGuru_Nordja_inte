# Documentação: Detecção Inteligente de Duplicatas para Tarefas Recorrentes

## Visão Geral

Este documento descreve a implementação da detecção inteligente de duplicatas que permite a criação de tarefas recorrentes legítimas (como reembolsos médicos mensais e rotinas de pagamento) enquanto continua bloqueando duplicatas verdadeiras.

## Problema Resolvido

**Situação Anterior:**
- O sistema bloqueava tarefas legítimas que se repetem mensalmente
- Tarefas de reembolso médico eram bloqueadas mesmo quando eram de meses diferentes
- Rotinas de pagamento recorrentes não eram criadas
- Usuários precisavam modificar títulos para "enganar" o sistema

**Solução Implementada:**
- Análise contextual completa das tarefas existentes
- Diferenciação entre tarefas recorrentes legítimas e duplicatas verdadeiras
- Consideração de período, beneficiário e tipo de serviço

## Arquitetura da Solução

### 1. Estrutura EnrichedTask

Nova estrutura que enriquece as informações das tarefas com contexto completo:

```rust
pub struct EnrichedTask {
    pub title: String,
    pub created_at: i64,  // timestamp em milissegundos
    pub description_preview: String, // primeiros 200 chars
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub status: String,
    pub custom_fields: HashMap<String, String>,
}
```

**Localização:** `src/models/enriched_task.rs`

### 2. Função get_list_tasks_full

Nova função que busca tarefas completas com todos os dados necessários:

```rust
async fn get_list_tasks_full(
    _client: &clickup_v2::client::ClickUpClient,
    list_id: &str,
) -> Result<Vec<Value>, AppError>
```

**Localização:** `src/handlers/worker.rs`

### 3. Configuração de Tarefas Recorrentes

Configuração no `config/ai_prompt.yaml` que define quais tarefas são recorrentes e suas regras:

```yaml
recurring_task_patterns:
  monthly_allowed:
    - subcategory: "Reembolso Médico"
      duplicate_rules:
        - allow_if: "different_month"
        - allow_if: "different_beneficiary"
        - allow_if: "different_doctor_or_exam"
        - block_if: "same_month_and_same_description"
```

### 4. Regras Aprimoradas de Detecção

Instruções detalhadas para a IA sobre como detectar duplicatas:

```yaml
enhanced_duplicate_detection: |
  ATENÇÃO ESPECIAL - DETECÇÃO INTELIGENTE DE DUPLICATAS:

  1. ANÁLISE CONTEXTUAL OBRIGATÓRIA:
     - NUNCA marque como duplicata baseando-se APENAS em títulos similares
     - SEMPRE analise o contexto temporal e os detalhes específicos
     ...
```

## Fluxo de Processamento

### Passo 1: Buscar Tarefas Existentes
```rust
let existing_tasks_json = get_list_tasks_full(&state.clickup_client, &list_id).await?;
```

### Passo 2: Enriquecer com Contexto
```rust
let enriched_tasks: Vec<EnrichedTask> = existing_tasks_json
    .iter()
    .map(|task| EnrichedTask::from_clickup_task_json(
        task,
        &mappings.category_field_id,
        &mappings.subcategory_field_id,
    ))
    .collect();
```

### Passo 3: Gerar Contexto para IA
O contexto enviado à IA agora inclui:
- Título da tarefa
- Categoria e subcategoria
- Data de criação
- Status
- Preview da descrição

### Passo 4: Análise pela IA
A IA analisa usando as regras configuradas e decide se é duplicata ou não.

## Exemplos de Uso

### Caso 1: Reembolso Médico Mensal ✅ PERMITIDO

**Tarefa Existente:**
```
Título: "Reembolso consulta neurologista Dr. Silva"
Subcategoria: Reembolso Médico
Criada: 05/01/2024
```

**Nova Mensagem:** "Solicitar reembolso da consulta com cardiologista"

**Decisão:** ✅ CRIAR (médico diferente)

### Caso 2: Pagamento Recorrente ✅ PERMITIDO

**Tarefa Existente:**
```
Título: "Rotina pagamentos janeiro - fornecedores"
Subcategoria: Rotina de Pagamentos
Criada: 10/01/2024
```

**Nova Mensagem:** "Pagar contas de luz e água de janeiro"

**Decisão:** ✅ CRIAR (tipo diferente: fornecedores vs contas consumo)

### Caso 3: Duplicata Verdadeira ❌ BLOQUEADA

**Tarefa Existente:**
```
Título: "Reembolso ressonância João - Clínica XYZ"
Subcategoria: Reembolso Médico
Criada: 15/01/2024
```

**Nova Mensagem:** "Pedir reembolso da ressonância do João na XYZ"

**Decisão:** ❌ DUPLICATA (mesmo exame, mesmo paciente, mesma clínica)

## Tarefas Recorrentes Configuradas

### Mensais (monthly_allowed):
- **Reembolso Médico**: Permite múltiplos se consulta/exame diferente OU mês diferente OU médico diferente
- **Rotina de Pagamentos**: Permite múltiplos se contas diferentes OU período diferente OU fornecedor diferente
- **Emissão de NF**: Permite múltiplas se cliente diferente OU período diferente OU mês diferente
- **Conciliação Bancária**: Permite múltiplas se mês diferente OU conta diferente
- **Imposto de Renda**: Permite múltiplos se ano diferente OU tipo de imposto diferente

### Conforme Necessário (as_needed):
- **Consultas**: Permite múltiplas se médico diferente OU data diferente OU paciente diferente
- **Compras**: Permite múltiplas se itens diferentes OU passou 7 dias

## Regras de Decisão

### Quando PERMITIR Criação:
1. Período/mês diferente
2. Beneficiário/objetivo diferente
3. Evento específico diferente
4. Tipo de serviço diferente

### Quando BLOQUEAR (Duplicata):
1. Mesmo período + mesmo objetivo + mesmo beneficiário
2. Mesmo evento + mesmo mês + mesmo contexto

### Regra de Ouro:
**Se houver DÚVIDA, sempre permita a criação (is_duplicate=false)**

## Monitoramento

Após a implementação, monitorar:
- Taxa de criação de tarefas recorrentes
- Feedback dos usuários sobre duplicatas
- Casos extremos não previstos
- Performance da classificação

## Arquivos Modificados

1. **src/models/enriched_task.rs** (NOVO)
   - Estrutura EnrichedTask
   - Função from_clickup_task_json

2. **src/models/mod.rs**
   - Adicionado módulo enriched_task

3. **src/handlers/worker.rs**
   - Função get_list_tasks_full (NOVA)
   - Modificado worker_process_message para usar EnrichedTask
   - Contexto enriquecido para IA

4. **config/ai_prompt.yaml**
   - Seção recurring_task_patterns (NOVA)
   - Seção enhanced_duplicate_detection (NOVA)
   - Regras de detecção de duplicatas atualizadas

## Testes Recomendados

### Teste 1: Múltiplos Reembolsos
- Criar tarefa: "Reembolso consulta janeiro"
- Tentar criar: "Reembolso exame fevereiro"
- **Esperado:** ✅ Permitido

### Teste 2: Rotina de Pagamentos
- Criar tarefa: "Pagamento fornecedores janeiro"
- Tentar criar: "Pagamento contas consumo janeiro"
- **Esperado:** ✅ Permitido

### Teste 3: Duplicata Real
- Criar tarefa: "Reembolso ressonância João - Clínica XYZ"
- Tentar criar: "Pedir reembolso ressonância João XYZ"
- **Esperado:** ❌ Bloqueado

## Manutenção Futura

Para adicionar novas tarefas recorrentes:

1. Editar `config/ai_prompt.yaml`
2. Adicionar entrada em `recurring_task_patterns.monthly_allowed` ou `as_needed`
3. Definir regras de `duplicate_rules`
4. Atualizar exemplos em `enhanced_duplicate_detection` se necessário

## Suporte

Em caso de dúvidas ou problemas:
1. Verificar logs do sistema
2. Analisar contexto enviado à IA
3. Revisar regras em `ai_prompt.yaml`
4. Consultar exemplos neste documento

