# Análise: Anotações Automáticas no ChatGuru

## 🔍 Situação Identificada

- `TESTE_API` **NÃO tem** configuração de anotação
- `nova_api` **não funcionou** no teste
- Mas as anotações **aparecem automaticamente** quando mensagens são enviadas

## 🎯 Hipóteses Possíveis

### 1. Sistema de IA/NLP Automático do ChatGuru
**Mais provável!**

O ChatGuru pode ter um sistema de IA próprio que:
- Analisa TODAS as mensagens
- Identifica automaticamente tarefas/atividades
- Cria anotações independente de diálogos

**Evidências:**
- Formato padronizado: "Atividade Identificada:"
- Categorização automática: "Atividades de Pesquisa em geral"
- Aparece mesmo sem configuração explícita

### 2. Configuração Global da Conta/Instância
O ChatGuru pode ter configurações em nível de:
- Conta (account_id)
- Instância (S15)
- Phone/Bot específico

Que ativam anotações automáticas para certas palavras-chave.

### 3. Feature de "Task Mining" ou "Activity Recognition"
Pode ser uma feature do ChatGuru que:
- Detecta intenções de tarefa
- É ativada por padrão em certas contas
- Não aparece na configuração de diálogos individuais

## 📊 Análise do Padrão das Anotações

Sempre seguem o formato:
```
Tarefa: Atividade Identificada: [texto extraído]
Tipo de Atividade: [Específica/Geral]
Categoria: [categoria detectada]
Sub Categoria: []
Subtarefas: (se aplicável)
- Subtarefa 1
- Subtarefa 2
```

## 🧪 Testes para Confirmar

### Teste 1: Mensagens sem Diálogos
Envie mensagens que NÃO acionem nenhum diálogo conhecido:
- "Preciso organizar os documentos"
- "Vou fazer uma pesquisa sobre APIs"
- "Tenho que desenvolver um novo recurso"

**Se criar anotação:** É sistema automático do ChatGuru

### Teste 2: Palavras-chave Específicas
Teste palavras que sugerem tarefas:
- Com "fazer", "criar", "desenvolver", "buscar"
- Sem essas palavras

**Se só funcionar com palavras-chave:** É detecção por padrão

### Teste 3: Desativar TESTE_API
- Desative temporariamente o TESTE_API
- Envie a mesma mensagem
- Veja se ainda cria anotação

**Se continuar criando:** Não é o diálogo que cria

## 🤖 Possível Sistema de IA do ChatGuru

### Como provavelmente funciona:

```
Mensagem do usuário
    ↓
[Análise de NLP/IA]
    ↓
Detectou tarefa/atividade?
    ├─ SIM → Cria anotação automática
    │         ↓
    │    [Extrai entidades]
    │    - Tarefa principal
    │    - Tipo de atividade
    │    - Categoria
    │    - Subtarefas
    │         ↓
    │    Formata e adiciona anotação
    │
    └─ NÃO → Processa normalmente
```

## 💡 Como Verificar no ChatGuru

### 1. Configurações da Conta
Procure por:
- "Anotações Automáticas"
- "Task Mining"
- "Activity Recognition"
- "IA/NLP Settings"
- "Smart Annotations"

### 2. Configurações do Bot/Phone
Verifique se há:
- "Auto-annotate"
- "Task Detection"
- "AI Features"

### 3. Logs/Histórico
Veja se há:
- Registro de "Sistema" criando anotações
- Diferença entre anotações manuais e automáticas

## 🎯 Solução Prática

### Se você QUER manter as anotações automáticas:
1. Aproveite o sistema existente
2. Configure seu middleware para processar essas anotações
3. Use o formato padrão para extrair dados

### Se você QUER controlar as anotações:
1. Procure desativar o sistema automático
2. Configure anotações manuais nos diálogos
3. Use webhooks para criar anotações programaticamente

## 📝 Script de Teste

```javascript
// Teste para confirmar origem das anotações
async function testeOrigemAnotacao() {
    const mensagensTeste = [
        "Olá, bom dia!", // Sem tarefa
        "Preciso fazer um relatório", // Com tarefa
        "Buscar informações sobre preços", // Com tarefa
        "Como está o tempo hoje?", // Sem tarefa
        "Desenvolver nova funcionalidade" // Com tarefa
    ];
    
    for (const msg of mensagensTeste) {
        console.log(`Enviando: "${msg}"`);
        // Enviar via API ou manualmente
        // Verificar se cria anotação
        await sleep(5000);
    }
}
```

## 🔑 Conclusão Provável

O ChatGuru tem um **sistema de IA/NLP nativo** que:
1. **Analisa todas as mensagens** automaticamente
2. **Detecta intenções de tarefa** usando IA
3. **Cria anotações estruturadas** sem necessidade de configuração
4. **Funciona independente** dos diálogos configurados

Isso explicaria por que:
- Não há configuração visível em TESTE_API
- As anotações têm formato consistente
- Aparecem com palavras-chave específicas
- nova_api não precisa funcionar para anotações aparecerem