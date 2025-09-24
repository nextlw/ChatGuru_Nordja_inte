# Scheduler Analysis - Legacy System

## APScheduler Configuration

### Job Details
- **Job Name**: `verificar_e_enviar_mensagens`
- **Trigger Type**: interval
- **Interval**: 100 seconds (0:01:40)
- **Timezone**: -03:00 (Brazil/São Paulo)

### Execution Pattern
```
INFO:apscheduler.executors.default:Job "verificar_e_enviar_mensagens (trigger: interval[0:01:40], next run at: 2025-09-24 07:47:27 -03)" executed successfully
```

### Job Behavior

1. **Dynamic Job Management**:
   - Jobs are added per contact/chat
   - Jobs are removed when completed
   - Multiple contacts processed in single execution

2. **Processing Logic**:
   - Executes for multiple contacts simultaneously
   - Logs: "Executando verificar_e_enviar_mensagens para [lista de contatos]"
   - Logs: "Aguardando mais mensagens ou intervalo para [contato]"
   - Logs: "Fim de verificar_e_enviar_mensagens para [contato]"

3. **Contact List Example**:
   ```
   Consultório Juliana Plens – Psiquiatra Renata Michaelis, 
   Lele (Cuidadora da Daniela Sitzer), 
   Hurá Bittencourt, 
   Andrew Reider, 
   Nino (Voila), 
   Samir Kerbage,
   Fabricio Ramos,
   ... (60+ contacts)
   ```

### Implementation Notes

The scheduler appears to:
1. Check for pending messages for each contact
2. Process messages that meet certain criteria (time interval?)
3. Send annotations back through some mechanism
4. Remove job when contact has no more pending messages

### Code Location
- Main scheduler file: `/app/utils/task_manager.py`
- Function name: `verificar_e_enviar_mensagens`
- Error location: Line 130 (iterating over conversas dictionary)
- Bug: RuntimeError: dictionary changed size during iteration

### Global State
- Uses a `conversas` dictionary to store chat states
- Each chat_id maps to contact info and messages
- Modified concurrently by webhook handler and scheduler

### Log Patterns

```python
# Job addition
INFO:apscheduler.scheduler:Added job "verificar_e_enviar_mensagens" to job store "default"

# Job execution
INFO:apscheduler.executors.default:Running job "verificar_e_enviar_mensagens (trigger: interval[0:01:40], next run at: 2025-09-24 07:45:47 -03)" 

# Job completion
INFO:apscheduler.executors.default:Job "verificar_e_enviar_mensagens (trigger: interval[0:01:40], next run at: 2025-09-24 07:47:27 -03)" executed successfully

# Job removal
INFO:apscheduler.scheduler:Removed job verificar_e_enviar_mensagens
```

### State Messages

- "Aguardando mais mensagens ou intervalo para [nome]" - Waiting for more messages or interval
- "Mensagem enviada com sucesso: [annotation]" - Message sent successfully
- "Resposta enviada e estado atualizado para [nome]" - Response sent and state updated

This suggests a queue-like system where messages are batched and sent periodically.