# API Behavior Analysis - Legacy System

## Message Processing Flow

### 1. Webhook Reception
- Endpoint: `/webhook`
- Always returns: `{"message": "Success"}` (status 200)
- Processes asynchronously in background

### 2. Message Grouping
The system groups messages from the same contact:
```
INFO:main:Mensagem de Samir Kerbage agrupada recebida: [content]
```

This suggests messages are:
- Collected/queued per contact
- Grouped before processing
- Processed in batches

### 3. AI Processing
Uses OpenAI for:
- Activity classification
- Document analysis (PDFs, images)
- Category classification

Example:
```
INFO:main:Categoria da tarefa de Samir Kerbage atualizada para 'Plano de Saúde' com resposta: {}
```

### 4. State Updates
After processing:
```
INFO:main:Resposta enviada e estado atualizado para Samir Kerbage
INFO:main:Mensagem enviada com sucesso: Tarefa: **Atividade Identificada:** Reembolso Médico
```

### 5. ClickUp Integration
Creates tasks with:
- Title format: "[Category] Contact Name"
- Custom fields for activity type and category
- Tags based on classification

## Important Discovery: NO Direct ChatGuru API Calls

The legacy system does NOT make direct API calls to ChatGuru for sending messages back. Instead:

1. **Internal State Management**: 
   - Updates internal state with annotations
   - Logs success messages

2. **Possible Integration Methods**:
   - ChatGuru may poll the application for updates
   - Webhook responses might carry state information
   - Another service might sync the states

3. **Evidence**:
   - No HTTP POST requests to ChatGuru API in logs
   - No API endpoint calls for message sending
   - Only logging of "Mensagem enviada com sucesso"

## Scheduler Behavior

The `verificar_e_enviar_mensagens` job:
1. Runs every 100 seconds
2. Processes multiple contacts per execution
3. Groups messages per contact
4. Applies business logic (waiting intervals)
5. Updates states and logs success

## Key Fields Used

From webhook:
- `chat_id`: Unique chat identifier
- `phone_id`: Always '62558780e2923cc4705beee1'
- `celular`: Phone number
- `nome`: Contact name
- `texto_mensagem`: Message text
- `campos_personalizados`: Custom fields
- `link_chat`: Chat URL in ChatGuru

## Response Pattern

The system maintains compatibility by:
1. Accepting webhook
2. Returning success immediately
3. Processing in background
4. Updating internal state
5. NOT sending direct API responses to ChatGuru