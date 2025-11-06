# Legacy System Reference

This folder contains analysis and documentation from the legacy App Engine system for future reference.

## System Information

- **Project ID**: buzzlightear
- **Service**: default
- **Version**: 20240812t221240
- **Runtime**: Python (App Engine Standard)
- **Region**: southamerica-east1
- **Container**: us.gcr.io/buzzlightear/appengine/default.20240812t221240

## Key Discoveries

### 1. Webhook Processing

- Receives webhooks at `/webhook` endpoint
- Always returns `{"message": "Success"}` with status 200
- Processes in background using async tasks

### 2. AI Classification

- Uses OpenAI GPT-3.5-turbo for activity classification
- API Key: [REDACTED - stored in environment variable]
- Classifies messages as activities or non-activities

### 3. Message Handling

- Does NOT directly call ChatGuru API to send messages
- Logs "Mensagem enviada com sucesso: [annotation]"
- Logs "Resposta enviada e estado atualizado para [nome]"
- Uses internal state management

### 4. Scheduler System

- Uses APScheduler with job "verificar_e_enviar_mensagens"
- Runs every 100 seconds (1:40)
- Processes pending messages for multiple contacts
- Adds/removes jobs dynamically based on contact
- s

### 5. Webhook Payload Structure

```python
{
    'campanha_id': '',
    'campanha_nome': '',
    'origem': '',
    'email': '558589530473',
    'nome': 'Não Disponível',
    'tags': [],
    'texto_mensagem': 'teste de anotação',
    'campos_personalizados': {'Info_2': 'William'},
    'bot_context': {},
    'responsavel_nome': '',
    'responsavel_email': '',
    'link_chat': 'https://s15.chatguru.app/chats#68c487a0289aaf655f315141',
    'celular': '558589530473',
    'phone_id': '62558780e2923cc4705beee1',
    'chat_id': '68c487a0289aaf655f315141',
    'chat_created': '2025-09-12 20:50:40.442000',
    'datetime_post': '2025-09-24 10:44:07.171806',
    'tipo_mensagem': 'chat'
}
```

## Important Notes

- The legacy system does NOT make direct API calls to ChatGuru
- It uses a state-based approach with scheduled jobs
- The `phone_id` is constant: '62558780e2923cc4705beee1'
- The `account_id` is: '625584ce6fdcb7bda7d94aa8'
