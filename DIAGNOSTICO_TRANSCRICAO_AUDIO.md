# 🔍 DIAGNÓSTICO: Demora nas Transcrições de Áudio

**Data**: 19/11/2024
**Sistema**: ChatGuru-ClickUp Middleware
**Problema**: Transcrições de áudio demoram muito para aparecer no chat

---

## 📊 RESUMO EXECUTIVO

As transcrições de áudio estão funcionando corretamente, mas existe um **delay de até 3 minutos** entre o recebimento do áudio e a criação da tarefa no ClickUp devido ao sistema de filas (MessageQueueService).

### ⏱️ Tempos Atuais

- **Transcrição do áudio**: ~2-5 segundos (OpenAI Whisper)
- **Envio da anotação**: Imediato (quando funciona)
- **Criação da task no ClickUp**: Até 180 segundos (3 minutos)

---

## 🔄 FLUXO ATUAL DO SISTEMA

### 1. Recebimento do Áudio (webhook.rs)

```
Webhook ChatGuru → Download áudio → Transcrição (Whisper) → Anotação
     ↓                                                         ↓
  < 100ms                                                  Imediata*
```

### 2. Sistema de Filas (MessageQueueService)

```
Mensagem → Fila por Chat → Aguarda Condições → Batch → PubSub → Worker → ClickUp
    ↓           ↓                  ↓             ↓        ↓         ↓
Imediato    Agrupada      Até 180s/8 msgs    Imediato  < 1s    ~2-5s
```

---

## 🚨 PROBLEMA IDENTIFICADO

### Gargalo Principal: MessageQueueService

O sistema agrupa mensagens por chat antes de processar, esperando por:

1. **8 mensagens** acumuladas no chat OU
2. **180 segundos** (3 minutos) desde a primeira mensagem OU
3. **Mensagem de fechamento** ("obrigado", "valeu", "tchau") OU
4. **Silêncio de 3 minutos** sem novas mensagens OU
5. **Mudança de tópico** detectada

### Configuração Atual

```rust
const MAX_MESSAGES_PER_CHAT: usize = 8;     // Máximo de mensagens
const MAX_WAIT_SECONDS: u64 = 180;          // 3 minutos
const SCHEDULER_INTERVAL_SECONDS: u64 = 10; // Verifica a cada 10s
```

---

## 🔍 ANÁLISE DETALHADA

### Por que existe esse delay?

1. **Objetivo Original**: Agrupar mensagens relacionadas em uma única tarefa
2. **Problema**: Para áudios únicos ou conversas curtas, o sistema espera desnecessariamente
3. **Scheduler**: Só verifica as filas a cada 10 segundos, adicionando mais delay

### Fluxo de Transcrição

```
1. Áudio recebido → Download (< 1s)
2. Transcrição via Whisper (~2-5s)
3. Tentativa de enviar anotação ao ChatGuru (imediata)
4. Mensagem adicionada à fila ← AQUI ESTÁ O DELAY
5. Espera até 180s para processar
6. Criação da task no ClickUp
```

---

## ✅ SOLUÇÕES PROPOSTAS

### 🚀 Solução 1: Processamento Imediato para Áudios (RECOMENDADA)

**Modificar** `webhook.rs` para processar áudios imediatamente:

```rust
// Em process_media_immediately()
if media_type.contains("audio") {
    // Após transcrever com sucesso
    if let Ok(transcription) = ia_service.transcribe_audio(&audio_bytes, &filename).await {
        // Enviar anotação
        send_annotation_to_chatguru(...).await?;

        // NOVO: Publicar IMEDIATAMENTE no PubSub
        publish_single_message_to_pubsub(&state, &final_payload).await?;

        // Não adicionar à fila
        return Ok(Json(success_response));
    }
}
```

**Vantagens**:

- Transcrições aparecem em ~5-10 segundos
- Mantém agrupamento para mensagens de texto
- Mínima mudança no código

---

### 🎯 Solução 2: Reduzir Timeouts

**Modificar** `mensageria/src/lib.rs`:

```rust
// Para desenvolvimento/teste rápido
const MAX_MESSAGES_PER_CHAT: usize = 3;      // Era 8
const MAX_WAIT_SECONDS: u64 = 30;            // Era 180 (3 min)
const SCHEDULER_INTERVAL_SECONDS: u64 = 5;   // Era 10
```

**Vantagens**:

- Mais rápido para todos os tipos de mensagem
- Fácil de implementar

**Desvantagens**:

- Pode criar múltiplas tasks para uma conversa
- Mais chamadas à API do ClickUp

---

### 🔧 Solução 3: Detecção Inteligente de Áudio

**Adicionar** em `SmartContextManager`:

```rust
// Nova regra: se é áudio transcrito, processar imediatamente
if messages.iter().any(|m| is_transcribed_audio(m)) {
    return ContextDecision::ProcessNow {
        reason: "Áudio transcrito detectado - processamento imediato".to_string()
    };
}
```

**Vantagens**:

- Usa a arquitetura existente
- Processa áudios rapidamente
- Mantém agrupamento para texto

---

### 📱 Solução 4: Webhook Duplo

Criar dois endpoints:

- `/webhook/batch` - Para mensagens de texto (comportamento atual)
- `/webhook/instant` - Para mídias (processamento imediato)

**Vantagens**:

- Controle total sobre tipos de processamento
- Pode otimizar cada fluxo independentemente

---

## 🛠️ IMPLEMENTAÇÃO RÁPIDA (Solução 1)

### 1. Criar função para publicar mensagem única

```rust
// Em src/main.rs
async fn publish_single_message_to_pubsub(
    state: &Arc<AppState>,
    payload: &Value
) -> Result<(), Box<dyn std::error::Error>> {
    // Usar o mesmo código de publish_batch_to_pubsub
    // mas para uma única mensagem
}
```

### 2. Modificar webhook.rs

```rust
// Linha ~110, após enviar anotação com sucesso
if media_type.contains("audio") && annotation_sent_successfully {
    // Publicar diretamente no PubSub
    if let Err(e) = publish_single_message_to_pubsub(&state, &final_payload).await {
        log_error(&format!("Erro ao publicar áudio no PubSub: {}", e));
    }

    // Retornar sem adicionar à fila
    return Ok(Json(success_response));
}
```

---

## 📊 MÉTRICAS ESPERADAS

### Antes (Atual)

- Transcrição: 5s
- Espera na fila: até 180s
- **Total: até 185s (~3 minutos)**

### Depois (Com Solução 1)

- Transcrição: 5s
- PubSub + Worker: 2-3s
- **Total: ~8-10s**

### Melhoria: **95% de redução no tempo de resposta**

---

## ⚠️ CONSIDERAÇÕES

### 1. Anotações no ChatGuru

- Atualmente há tentativa de enviar anotação imediata
- Verificar logs para confirmar se estão funcionando
- Se não, investigar autenticação/permissões

### 2. Rate Limits

- Processamento imediato pode aumentar chamadas ao ClickUp
- Monitorar limites da API
- Considerar cache se necessário

### 3. Custos

- Mais mensagens no PubSub = maior custo
- Estimar impacto baseado no volume atual

---

## 🎯 PRÓXIMOS PASSOS

1. **Imediato**: Implementar Solução 1 (processamento imediato para áudios)
2. **Teste**: Validar com áudios reais
3. **Monitorar**: Acompanhar métricas e logs
4. **Otimizar**: Ajustar timeouts se necessário

---

## 📞 CONTATO

Para dúvidas ou suporte na implementação, verificar:

- Logs: Cloud Run → chatguru-clickup-middleware
- Métricas: Cloud Monitoring → PubSub topics
- Código: `/src/handlers/webhook.rs` e `/mensageria/src/lib.rs`
