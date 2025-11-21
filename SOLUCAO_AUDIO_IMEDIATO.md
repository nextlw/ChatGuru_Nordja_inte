# 🚀 SOLUÇÃO: Processamento Imediato de Áudios

**Objetivo**: Fazer transcrições de áudio aparecerem em ~10 segundos ao invés de até 3 minutos

---

## 📝 MUDANÇAS NECESSÁRIAS

### 1. Adicionar função em `src/main.rs`

Adicione esta função após a função `publish_batch_to_pubsub`:

```rust
/// Publica uma única mensagem diretamente no Pub/Sub (para áudios)
async fn publish_single_message_to_pubsub(
    settings: &Settings,
    payload: &Value
) -> Result<(), Box<dyn std::error::Error>> {
    use google_cloud_pubsub::client::{Client, ClientConfig};

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config).await?;

    let topic_name = settings.gcp.chatguru_topic
        .as_ref()
        .ok_or("CHATGURU_TOPIC não configurado")?;

    let topic = client.topic(topic_name);

    if !topic.exists(None).await? {
        return Err(format!("Topic '{}' não existe", topic_name).into());
    }

    let publisher = topic.new_publisher(None);

    // Serializar payload
    let message_data = serde_json::to_string(payload)?;

    // Criar mensagem Pub/Sub
    let msg = google_cloud_pubsub::publisher::message::MessageBuilder::new()
        .data(message_data.into_bytes())
        .attributes([
            ("source", "webhook"),
            ("type", "audio_transcription"),
            ("priority", "high")
        ])
        .build();

    // Publicar com retry
    const MAX_RETRIES: u32 = 3;

    for attempt in 1..=MAX_RETRIES {
        match publisher.publish(msg.clone()).await.get().await {
            Ok(_) => {
                tracing::info!(
                    "✅ Áudio publicado no Pub/Sub - Topic: {} | Attempt: {}",
                    topic_name, attempt
                );
                return Ok(());
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    let backoff_ms = attempt * 100;
                    tracing::warn!(
                        "⚠️ Erro ao publicar áudio (attempt {}/{}): {}. Retrying in {}ms...",
                        attempt, MAX_RETRIES, e, backoff_ms
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms as u64)).await;
                } else {
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}
```

### 2. Modificar `src/handlers/webhook.rs`

Localize a função `process_media_immediately` (linha ~43) e modifique o retorno para incluir um flag indicando se é áudio:

```rust
// No início da função process_media_immediately, mude o tipo de retorno:
async fn process_media_immediately(
    state: &Arc<AppState>,
    payload: &mut Value
) -> Option<(Value, bool)> { // Retorna tupla com (payload, is_audio)
```

Depois, dentro do bloco de processamento de áudio (após linha ~111), modifique:

```rust
// Onde estava:
// Some((transcription, media_type.clone()))

// Mude para:
return Some((synthetic_payload, true)); // true indica que é áudio
```

E para imagem e PDF:

```rust
// Onde estava:
// Some((description, media_type.clone()))

// Mude para:
return Some((synthetic_payload, false)); // false indica que NÃO é áudio
```

### 3. Modificar o handler principal em `webhook.rs`

Na função `webhook_process` (linha ~250), após o processamento de mídia:

```rust
// Modifique o match de process_media_immediately (linha ~331)
match process_media_immediately(&state, &mut final_payload).await {
    Some((synthetic_payload, is_audio)) => {
        log_info(&format!(
            "✅ MÍDIA PROCESSADA - RequestID: {} | ChatID: {} | Is Audio: {}",
            request_id, chat_id, is_audio
        ));
        final_payload = synthetic_payload;

        // NOVO: Se é áudio, publicar imediatamente
        if is_audio {
            log_info(&format!(
                "🎤 ÁUDIO DETECTADO - Publicando imediatamente no PubSub | ChatID: {}",
                chat_id
            ));

            match publish_single_message_to_pubsub(&state.settings, &final_payload).await {
                Ok(_) => {
                    let processing_time = start_time.elapsed().as_millis() as u64;

                    log_info(&format!(
                        "✅ ÁUDIO PUBLICADO DIRETAMENTE - RequestID: {} | ChatID: {} | Time: {}ms",
                        request_id, chat_id, processing_time
                    ));

                    // Retornar sucesso SEM adicionar à fila
                    return Ok(Json(json!({
                        "message": "Audio processed and published immediately",
                        "request_id": request_id,
                        "chat_id": chat_id,
                        "processing_time_ms": processing_time,
                        "audio_fast_track": true
                    })));
                }
                Err(e) => {
                    log_error(&format!(
                        "❌ Erro ao publicar áudio no PubSub: {} | Continuando com fila normal",
                        e
                    ));
                    // Se falhar, continua com o fluxo normal (fila)
                }
            }
        }
    }
    None => {
        // ... código existente ...
    }
}
```

---

## 🧪 COMO TESTAR

### 1. Compile e execute localmente:

```bash
cargo build --release
cargo run
```

### 2. Teste com áudio:

```bash
# Use o script de teste existente
./test-audio-local.sh
```

### 3. Monitore os logs:

```bash
# Em outro terminal
tail -f output_test.log | grep -E "(ÁUDIO|audio|Transcr)"
```

### 4. Verifique o tempo:

- Deve ver "ÁUDIO PUBLICADO DIRETAMENTE" em ~5-10s
- A task deve aparecer no ClickUp em ~15s total

---

## 📊 LOGS ESPERADOS

### Sucesso:

```
🎤 Processando áudio...
✅ Áudio transcrito: 125 caracteres
✅ Anotação enviada ao ChatGuru com sucesso
🎤 ÁUDIO DETECTADO - Publicando imediatamente no PubSub
✅ ÁUDIO PUBLICADO DIRETAMENTE - Time: 5234ms
```

### Fallback (se PubSub falhar):

```
🎤 ÁUDIO DETECTADO - Publicando imediatamente no PubSub
❌ Erro ao publicar áudio no PubSub: timeout | Continuando com fila normal
📬 ADICIONANDO À FILA - Queue size: 1
```

---

## 🚨 ROLLBACK

Se houver problemas, simplesmente:

1. Remova o bloco `if is_audio`
2. Volte o tipo de retorno de `process_media_immediately` para `Option<Value>`
3. Remova a função `publish_single_message_to_pubsub`

---

## 🔍 MONITORAMENTO

### Cloud Console:

1. **Cloud Run Logs**: Filtrar por "ÁUDIO PUBLICADO DIRETAMENTE"
2. **Pub/Sub Metrics**: Verificar aumento em mensagens com attribute `type=audio_transcription`
3. **ClickUp**: Tempo entre envio do áudio e criação da task

### Métricas-chave:

- **Antes**: ~180s (3 min) máximo
- **Depois**: ~15s médio
- **Melhoria**: 90-95% redução

---

## ⚡ OTIMIZAÇÕES FUTURAS

1. **Cache de transcrições**: Evitar retranscrever áudios duplicados
2. **Processamento paralelo**: Transcrever enquanto envia ao PubSub
3. **Prioridade no Worker**: Tasks de áudio processadas primeiro
4. **Webhook dedicado**: `/webhook/audio` para processamento otimizado

---

## 📌 NOTAS IMPORTANTES

1. **Custo PubSub**: Cada áudio gera uma mensagem adicional (estimar impacto)
2. **Rate Limits**: Monitorar limites do ClickUp (100 req/min)
3. **Failover**: Se PubSub falhar, sistema volta ao comportamento normal (fila)
4. **Logs**: Todos os áudios terão log "ÁUDIO PUBLICADO DIRETAMENTE" para auditoria
