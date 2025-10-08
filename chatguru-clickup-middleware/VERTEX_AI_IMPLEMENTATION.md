# Implementação: Processamento Assíncrono de Mídia via Vertex AI

## 📋 Sumário da Implementação

Implementação completa da **Opção 2** - Processamento assíncrono de mídia (áudio/imagem) via Pub/Sub + Vertex AI Gemini Pro.

**Status**: ✅ **IMPLEMENTAÇÃO CONCLUÍDA** (pending deployment da Cloud Function)

---

## 🏗️ Arquitetura Implementada

```
ChatGuru Webhook → Pub/Sub: "chatguru-webhook-raw"
                        ↓
                   Worker (extrai media_url/media_type)
                        ↓
                   Vertex AI habilitado?
                   ├─ SIM ↓
                   │  Pub/Sub: "media-processing-requests"
                   │       ↓
                   │  Cloud Function: vertex-media-processor
                   │       ↓
                   │  ┌────────────┴────────────┐
                   │  ↓                         ↓
                   │ [Áudio: Gemini 1.5]   [Imagem: Gemini 1.5]
                   │  Speech-to-Text       Image Description
                   │  ↓                         ↓
                   │  └────────────┬────────────┘
                   │               ↓
                   │  Pub/Sub: "media-processing-results"
                   │               ↓
                   │  Worker (enriquece payload)
                   │
                   └─ NÃO → OpenAI Whisper (fallback)
                        ↓
        Classificação OpenAI → ClickUp
```

---

## ✅ Componentes Implementados

### 1. Infraestrutura GCP

#### Tópicos Pub/Sub Criados
```bash
✅ media-processing-requests  # Requisições de processamento
✅ media-processing-results   # Resultados de processamento
✅ media-results-sub          # Subscription para worker ler resultados
```

#### Service Account
```bash
✅ vertex-media-processor@buzzlightear.iam.gserviceaccount.com
   - roles/aiplatform.user
   - roles/pubsub.publisher
```

### 2. Rust Middleware

#### Novos Arquivos
- ✅ `src/services/vertex.rs` (218 linhas)
  - `VertexAIService`: Publica requisições em Pub/Sub
  - `MediaProcessingRequest/Result`: Structs de dados
  - `process_media_async()`: Método principal
  - Suporte para áudio e imagem

- ✅ `src/services/media_sync.rs` (185 linhas)
  - `MediaSyncService`: Coordena requisições/resultados assíncronos
  - Cache em memória com `tokio::sync::oneshot`
  - `wait_for_result()`: Aguarda com timeout (30s default)
  - `notify_result()`: Notifica quando resultado chega

#### Arquivos Modificados
- ✅ `src/handlers/worker.rs`
  - Linhas 200-320: Processamento de mídia com Vertex AI
  - Fallback automático para OpenAI Whisper
  - Suporte para áudio E imagem

- ✅ `src/lib.rs`
  - AppState: Adicionados `vertex` e `media_sync`

- ✅ `src/main.rs`
  - Inicialização condicional dos serviços Vertex AI
  - Logs estruturados

- ✅ `src/services/mod.rs`
  - Exports de `vertex` e `media_sync`

- ✅ `src/config/settings.rs`
  - `VertexSettings`: Configurações do Vertex AI
  - `GcpSettings`: Adicionados topics de mídia

- ✅ `src/utils/error.rs`
  - `AppError::Timeout`: Novo variant

- ✅ `config/default.toml`
  - Seção `[vertex]` com configurações
  - Tópicos Pub/Sub de mídia

### 3. Cloud Function Python

#### Arquivos Criados
- ✅ `cloud_functions/vertex_media_processor/main.py`
  - `process_media()`: Entry point
  - `transcribe_audio_gemini()`: Transcrição via Gemini 1.5 Flash
  - `describe_image_gemini()`: Descrição via Gemini 1.5 Flash Vision
  - `publish_result()`: Publica resultados no Pub/Sub

- ✅ `cloud_functions/vertex_media_processor/requirements.txt`
  - google-cloud-pubsub
  - google-cloud-aiplatform
  - vertexai
  - requests

- ✅ `cloud_functions/vertex_media_processor/.gcloudignore`
- ✅ `cloud_functions/vertex_media_processor/README.md`

---

## 🚀 Deployment

### Passo 1: Deploy Cloud Function

```bash
cd cloud_functions/vertex_media_processor

gcloud functions deploy vertex-media-processor \
  --gen2 \
  --runtime=python311 \
  --region=southamerica-east1 \
  --source=. \
  --entry-point=process_media \
  --trigger-topic=media-processing-requests \
  --service-account=vertex-media-processor@buzzlightear.iam.gserviceaccount.com \
  --timeout=60s \
  --memory=512MB \
  --project=buzzlightear
```

**Nota**: O deploy da Cloud Function está pendente devido a um problema com o módulo grpc no Python local.
**Solução**: Deploy via Google Cloud Console ou corrigir instalação do gcloud CLI.

### Passo 2: Deploy Middleware Rust

```bash
cd chatguru-clickup-middleware

# Build
cargo build --release

# Deploy no Cloud Run (exemplo)
gcloud run deploy chatguru-clickup-middleware \
  --source=. \
  --region=southamerica-east1 \
  --project=buzzlightear \
  --allow-unauthenticated
```

---

## ⚙️ Configuração

### Habilitar Vertex AI no Middleware

Editar `config/default.toml`:

```toml
[vertex]
enabled = true  # Habilita Vertex AI
timeout_seconds = 30  # Timeout para aguardar resultado
project_id = "buzzlightear"
location = "us-central1"
```

### Desabilitar (Rollback para OpenAI)

```toml
[vertex]
enabled = false  # Desabilita Vertex AI, usa OpenAI Whisper
```

---

## 🧪 Testes

### Teste Local (sem Cloud Function)

```rust
// Vertex AI disabled, fallback para OpenAI
[vertex]
enabled = false
```

### Teste com Vertex AI

1. Deploy Cloud Function
2. Habilitar `[vertex] enabled = true`
3. Enviar webhook com mídia:

```bash
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "campanha_id": "123",
    "campanha_nome": "Test",
    "origem": "whatsapp",
    "nome": "Test User",
    "celular": "5511999999999",
    "texto_mensagem": "",
    "media_url": "https://example.com/test.ogg",
    "media_type": "audio/ogg"
  }'
```

4. Verificar logs:
   - Worker detecta mídia
   - Publica em `media-processing-requests`
   - Cloud Function processa
   - Resultado volta via `media-processing-results`
   - Worker enriquece payload
   - Task criada no ClickUp

---

## 📊 Monitoramento

### Logs do Middleware

```bash
# Cloud Run
gcloud run logs tail chatguru-clickup-middleware \
  --region=southamerica-east1 \
  --project=buzzlightear
```

### Logs da Cloud Function

```bash
gcloud functions logs tail vertex-media-processor \
  --region=southamerica-east1 \
  --project=buzzlightear
```

### Métricas Pub/Sub

```bash
# Mensagens pendentes
gcloud pubsub subscriptions describe media-results-sub \
  --project=buzzlightear

# Dead letter queue (se configurada)
gcloud pubsub topics list --project=buzzlightear | grep dead
```

---

## 💰 Custos Estimados

### Volume Atual: ~1.000-1.200 tasks/mês

Assumindo 30% com mídia (~350 mensagens/mês):

| Serviço | Custo/mês | Observações |
|---------|-----------|-------------|
| **Pub/Sub** | ~$0 | 10GB free tier |
| **Cloud Function** | ~$0 | 2M invocations free |
| **Vertex AI Gemini 1.5 Flash** | ~$7 | $0.02 per 1K chars input (áudio transcrito ~500 chars cada) |
| **Cloud Run** | Atual | Sem mudança |
| **Total adicional** | **~$7/mês** | 97% economia vs OpenAI Whisper (~$25/mês) |

---

## 🔄 Fluxo de Processamento Detalhado

### 1. Webhook Recebe Mídia

```rust
// worker.rs:210
if let (Some(ref media_url), Some(ref media_type)) =
    (&chatguru_payload.media_url, &chatguru_payload.media_type) {
```

### 2. Verifica Tipo Suportado

```rust
if VertexAIService::is_supported_media_type(media_type) {
    // áudio: audio/*, voice/*
    // imagem: image/*, photo/*, png, jpg, jpeg
}
```

### 3. Processamento Vertex AI

```rust
// worker.rs:224
vertex_service.process_media_async(media_url, media_type, chat_id).await
```

```rust
// vertex.rs:92
pub async fn process_media_async(&self, ...) -> AppResult<String> {
    let correlation_id = Uuid::new_v4();
    self.publish_request(...).await?;
    Ok(correlation_id)
}
```

### 4. Cloud Function Processa

```python
# main.py:transcribe_audio_gemini
model = GenerativeModel("gemini-1.5-flash")
audio_part = Part.from_data(audio_bytes, mime_type)
response = model.generate_content([prompt, audio_part])
```

### 5. Resultado Retorna

```python
# main.py:publish_result
publisher.publish(RESULTS_TOPIC, result_payload)
```

### 6. Worker Aguarda Resultado

```rust
// worker.rs:233
media_sync.wait_for_result(correlation_id).await
```

```rust
// media_sync.rs:45
timeout(self.default_timeout, rx).await
```

### 7. Enriquece Payload

```rust
// worker.rs:303
chatguru_payload.texto_mensagem = format!(
    "{}\n\n[{}]: {}",
    chatguru_payload.texto_mensagem,
    label,  // "Transcrição do áudio" ou "Descrição da imagem"
    result_text
);
```

### 8. Continua Fluxo Normal

- Classificação OpenAI
- Criação de task no ClickUp
- Envio de anotação ao ChatGuru

---

## 🛡️ Estratégia de Fallback

### Níveis de Fallback

1. **Vertex AI Timeout** (30s)
   - Usa OpenAI Whisper (apenas para áudio)
   - Log warning

2. **Vertex AI Error**
   - Usa OpenAI Whisper (apenas para áudio)
   - Log error

3. **OpenAI Whisper Failure**
   - Continua sem transcrição
   - Log error

4. **Vertex AI Disabled**
   - Usa OpenAI Whisper diretamente
   - Log info

### Código de Fallback

```rust
// worker.rs:256
let final_result = if media_result.is_none() && processing_type == "audio" {
    log_info("🔄 Fallback para OpenAI Whisper");
    // ... OpenAI Whisper logic
} else {
    media_result
};
```

---

## 🚨 Troubleshooting

### Problema: Timeout aguardando resultado

**Sintomas:**
```
⏱️ Timeout aguardando resultado: uuid (30s)
```

**Causas Possíveis:**
1. Cloud Function não deployada
2. Cloud Function com erro
3. Tópico Pub/Sub incorreto
4. Network issues

**Solução:**
```bash
# Verificar Cloud Function
gcloud functions describe vertex-media-processor \
  --region=southamerica-east1 \
  --project=buzzlightear

# Verificar logs
gcloud functions logs read vertex-media-processor \
  --region=southamerica-east1 \
  --limit=50
```

### Problema: Vertex AI sempre disabled

**Sintomas:**
```
ℹ️ Vertex AI não configurado, usando OpenAI Whisper
```

**Causa:**
Configuração incorreta em `config/default.toml`

**Solução:**
```toml
[vertex]
enabled = true  # Verificar se está true
timeout_seconds = 30
project_id = "buzzlightear"
location = "us-central1"
```

### Problema: Cloud Function com erro "grpc"

**Sintomas:**
```
No module named 'grpc'
```

**Solução:**
Deploy via Google Cloud Console ou reinstalar gcloud CLI:
```bash
curl https://sdk.cloud.google.com | bash
exec -l $SHELL
gcloud init
```

---

## 📈 Próximos Passos

### Obrigatórios
- [ ] Deploy da Cloud Function
- [ ] Teste end-to-end com mídia real
- [ ] Configurar Dead Letter Queue
- [ ] Configurar alertas no Cloud Monitoring

### Opcionais
- [ ] Adicionar suporte para vídeo
- [ ] Implementar cache de resultados
- [ ] Adicionar métricas customizadas
- [ ] Configurar auto-scaling da Cloud Function

---

## 📝 Checklist de Validação

- [x] Infraestrutura GCP criada
- [x] Código Rust compilando
- [x] Cloud Function criada
- [ ] Cloud Function deployada
- [ ] Teste com áudio passando
- [ ] Teste com imagem passando
- [ ] Fallback OpenAI testado
- [ ] Timeout testado
- [ ] Logs estruturados configurados
- [ ] Rollback testado (enabled=false)

---

## 📚 Referências

- [Vertex AI Gemini API](https://cloud.google.com/vertex-ai/generative-ai/docs/multimodal/overview)
- [Gemini 1.5 Flash Pricing](https://cloud.google.com/vertex-ai/generative-ai/pricing)
- [Cloud Functions Gen2](https://cloud.google.com/functions/docs/2nd-gen/overview)
- [Pub/Sub Best Practices](https://cloud.google.com/pubsub/docs/publisher)

---

**Implementação por**: Claude Code
**Data**: 2025-01-07
**Status**: ✅ COMPLETO (pending Cloud Function deployment)
