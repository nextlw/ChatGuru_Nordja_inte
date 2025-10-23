# 📬 Message Queue System - Fila de Mensagens por Chat

## 🎯 Objetivo

Agrupar mensagens do **mesmo chat** antes de processar, exatamente como o sistema legado fazia.

---

## 🔧 Como Funciona

### **Regra de Processamento**

Cada chat acumula mensagens até:
- ✅ **5 mensagens** acumuladas, OU
- ✅ **100 segundos** desde a primeira mensagem

**O que vier primeiro dispara o processamento!**

### **Scheduler**

- Verifica filas **a cada 10 segundos**
- Processa automaticamente chats que atingiram o timeout (100s)
- Filas são processadas de forma **assíncrona** (não bloqueia)

---

## 📊 Fluxo Completo

```
┌─────────────────────────────────────────────────────────────┐
│  CHAT A (5 mensagens em 30 segundos)                       │
└─────────────────────────────────────────────────────────────┘
Webhook → msg1 → Fila (1/5) ⏳ Aguardando...
Webhook → msg2 → Fila (2/5) ⏳ Aguardando...
Webhook → msg3 → Fila (3/5) ⏳ Aguardando...
Webhook → msg4 → Fila (4/5) ⏳ Aguardando...
Webhook → msg5 → Fila (5/5) ✅ PROCESSA AGORA! (5 mensagens)
                      ↓
              Pub/Sub (batch de 5)
                      ↓
                   Worker
                      ↓
              OpenAI + ClickUp

┌─────────────────────────────────────────────────────────────┐
│  CHAT B (2 mensagens, aguarda timeout)                     │
└─────────────────────────────────────────────────────────────┘
Webhook → msg1 → Fila (1/5) ⏳ Aguardando...
  ... 50 segundos ...
Webhook → msg2 → Fila (2/5) ⏳ Aguardando...
  ... 50 segundos ... (total 100s)
Scheduler → ✅ PROCESSA AGORA! (timeout 100s)
                      ↓
              Pub/Sub (batch de 2)
                      ↓
                   Worker
                      ↓
              OpenAI + ClickUp
```

---

## 📂 Arquivos Criados/Modificados

### **Novos Arquivos**

1. **`src/services/message_queue.rs`**
   - Serviço principal de fila
   - Scheduler automático
   - Testes unitários incluídos

### **Arquivos Modificados**

1. **`src/services/mod.rs`**
   - Adicionado módulo `message_queue`
   - Export `MessageQueueService`

2. **`src/lib.rs`**
   - Adicionado `message_queue` ao `AppState`

3. **`src/main.rs`**
   - Inicializa `MessageQueueService`
   - Inicia scheduler automático

4. **`src/handlers/webhook.rs`**
   - Extrai `chat_id` do payload
   - Adiciona mensagem à fila
   - Envia batch para Pub/Sub quando pronto

---

## 🔍 Logs Esperados

### **Mensagem Adicionada à Fila**
```
📬 Chat 'chat123': 3 mensagens na fila (aguardando 5 ou 100s)
⏳ Chat 'chat123': Aguardando mais mensagens...
```

### **Fila Pronta (5 mensagens)**
```
✅ Chat 'chat123': Pronto para processar (5 mensagens atingidas) - 5 mensagens acumuladas
🚀 Chat 'chat123': Fila pronta com 5 mensagens - enviando para processamento
📤 Batch de 5 mensagens publicado no tópico 'chatguru-webhook-raw' (chat: chat123)
```

### **Fila Pronta (timeout 100s)**
```
⏰ Chat 'chat456': Timeout atingido (100s) - 2 mensagens aguardando
🚀 Processando batch do chat 'chat456': 2 mensagens acumuladas
📤 Batch de 2 mensagens publicado no tópico 'chatguru-webhook-raw' (chat: chat456)
```

### **Scheduler Iniciado**
```
🕐 Scheduler iniciado: verifica filas a cada 10s
✅ Message Queue Scheduler iniciado (5 msgs ou 100s por chat)
```

---

## 🎯 Benefícios

### **1. Contexto Completo**
- Agrupa múltiplas mensagens do usuário
- Permite processar com contexto completo
- Campos personalizados podem ser agregados

### **2. Reduz Custo de API**
- Menos chamadas para OpenAI
- Menos tarefas criadas no ClickUp
- Agrupa conversas relacionadas

### **3. Compatível com Legado**
- Sistema legado tinha scheduler de 100s
- Comportamento idêntico ao App Engine Python

### **4. Performance**
- Processamento assíncrono
- Não bloqueia webhook
- ACK < 100ms mantido

---

## 📋 Próximos Passos

### **TODO: Implementar Agregação de Campos**

No arquivo `src/services/message_queue.rs`, função `process_batch`:

```rust
async fn process_batch(
    chat_id: String,
    messages: Vec<QueuedMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Pegar última mensagem como principal
    let last_msg = messages.last().unwrap();
    
    // 2. Agregar campos_personalizados de todas as mensagens
    let mut aggregated_campos = HashMap::new();
    for msg in &messages {
        if let Some(campos) = msg.payload.get("campos_personalizados") {
            // Mesclar campos
            if let Some(obj) = campos.as_object() {
                for (k, v) in obj {
                    aggregated_campos.insert(k.clone(), v.clone());
                }
            }
        }
    }
    
    // 3. Criar payload final com contexto completo
    let mut final_payload = last_msg.payload.clone();
    final_payload["campos_personalizados"] = json!(aggregated_campos);
    final_payload["message_count"] = json!(messages.len());
    
    // 4. Enviar para processamento
    // ...
}
```

---

## 🧪 Testes

### **Teste 1: 5 Mensagens Rápidas**
```bash
# Enviar 5 mensagens do mesmo chat rapidamente
for i in {1..5}; do
  curl -X POST http://localhost:8080/webhooks/chatguru \
    -H "Content-Type: application/json" \
    -d '{"chat_id":"test123","nome":"Teste","texto_mensagem":"Mensagem '$i'"}'
  sleep 1
done
```

**Resultado Esperado**: Processa após a 5ª mensagem

### **Teste 2: Timeout de 100 Segundos**
```bash
# Enviar 2 mensagens e aguardar 100s
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{"chat_id":"test456","nome":"Teste","texto_mensagem":"Msg 1"}'

sleep 50

curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{"chat_id":"test456","nome":"Teste","texto_mensagem":"Msg 2"}'

# Aguardar 50s + scheduler
# Deve processar após 100s total
```

**Resultado Esperado**: Processa após 100s (2 mensagens)

---

## 📊 Monitoramento

### **Endpoint de Debug** (TODO)

Adicionar endpoint para ver estatísticas:

```rust
// GET /queue/stats
{
  "chat_123": 3,  // 3 mensagens aguardando
  "chat_456": 1,  // 1 mensagem aguardando
  "total_chats": 2,
  "total_messages": 4
}
```

---

**Data**: 23 de Outubro de 2025  
**Autor**: Sistema de Fila de Mensagens  
**Status**: ✅ Implementado e Compilado

