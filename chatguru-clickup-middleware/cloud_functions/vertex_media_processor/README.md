# Vertex AI Media Processor - Cloud Function

Processa áudio e imagens usando Gemini Pro via Vertex AI.

## Arquitetura

- **Trigger**: Pub/Sub topic `media-processing-requests`
- **Output**: Pub/Sub topic `media-processing-results`
- **Model**: Gemini 1.5 Flash (suporta áudio e imagem)

## Deployment

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

## Payload Format

### Input (media-processing-requests)
```json
{
  "correlation_id": "uuid-v4",
  "media_url": "https://example.com/audio.ogg",
  "media_type": "audio/ogg",
  "chat_id": "optional-chat-id",
  "timestamp": "2025-01-01T00:00:00Z"
}
```

### Output (media-processing-results)
```json
{
  "correlation_id": "uuid-v4",
  "result": "Texto transcrito ou descrição da imagem",
  "media_type": "audio" | "image",
  "error": null | "error message"
}
```

## Supported Media Types

### Audio
- audio/ogg
- audio/mpeg (MP3)
- audio/wav
- audio/mp4 (M4A)

### Image
- image/png
- image/jpeg
- image/webp

## Monitoring

```bash
# View logs
gcloud functions logs read vertex-media-processor \
  --region=southamerica-east1 \
  --limit=50 \
  --project=buzzlightear

# Tail logs
gcloud functions logs tail vertex-media-processor \
  --region=southamerica-east1 \
  --project=buzzlightear
```

## Testing

```bash
# Publish test message
gcloud pubsub topics publish media-processing-requests \
  --message='{"correlation_id":"test-123","media_url":"https://example.com/test.ogg","media_type":"audio/ogg","timestamp":"2025-01-01T00:00:00Z"}' \
  --project=buzzlightear
```

## Cost Optimization

- **Gemini 1.5 Flash**: Mais barato e rápido que Pro
- **Timeout**: 60s (suficiente para maioria dos áudios)
- **Memory**: 512MB (balanceado)
- **Region**: us-central1 (Gemini disponível)

## Error Handling

- Download timeout: 30s
- Retries automáticos pelo Pub/Sub (até 7 dias)
- Errors são retornados no campo `error` do result payload
