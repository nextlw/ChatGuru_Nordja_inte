# Documentação Completa: ChatGuru-ClickUp Middleware

## Visão Geral

Middleware de integração entre ChatGuru (chatbot WhatsApp) e ClickUp (gerenciamento de tarefas) desenvolvido em Rust com framework Axum. O sistema recebe webhooks do ChatGuru e automaticamente cria ou atualiza tarefas no ClickUp.

## URLs de Produção

### Ambiente de Produção (Google Cloud Run)
```
URL Base: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app
Projeto GCP: buzzlightear
Região: southamerica-east1 (São Paulo)
```

### Ambiente Local (Desenvolvimento)
```
URL Base: http://localhost:8080
```

## Endpoints Implementados

### 1. Health Check
**GET /health**

Verifica se o serviço está operacional.

**Response (200 OK):**
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "service": "chatguru-clickup-middleware"
}
```

### 2. Readiness Check
**GET /ready**

Verifica se o serviço está pronto para receber requisições.

**Response (200 OK):**
```json
{
  "ready": true,
  "clickup_connected": true,
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### 3. Status da Aplicação
**GET /status**

Retorna informações detalhadas sobre o status do middleware.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "uptime_seconds": 3600,
  "clickup": {
    "connected": true,
    "list_id": "901300373349",
    "last_check": "2024-01-15T10:30:00Z"
  },
  "environment": "production",
  "version": "1.0.0"
}
```

### 4. Webhook Principal - Receber Eventos ChatGuru
**POST /webhooks/chatguru**

Endpoint principal que recebe eventos do ChatGuru e cria/atualiza tarefas no ClickUp.

**Headers Opcionais:**
```
X-ChatGuru-Signature: sha256=<hmac_signature>
Content-Type: application/json
```

**Request Body (Payload ChatGuru):**
```json
{
  "id": "evt_123456789",
  "event_type": "new_lead",
  "timestamp": "2024-01-15T10:30:00.000Z",
  "data": {
    "lead_name": "João Silva",
    "phone": "+5511999999999",
    "email": "joao@example.com",
    "project_name": "Apartamento 2 Quartos",
    "task_title": "Lead interessado em apartamento - URGENTE",
    "annotation": "Cliente quer agendar visita esta semana",
    "amount": 250000.00,
    "status": "new",
    "custom_data": {
      "source": "WhatsApp",
      "campaign": "Janeiro2024"
    }
  }
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "event_id": "evt_123456789",
  "event_type": "new_lead",
  "processed_at": "2024-01-15T10:30:01.000Z",
  "clickup_task_processed": true,
  "clickup_task_action": "created",
  "clickup_task_id": "86947v4hj",
  "message": "Event processed successfully - task created",
  "pubsub_enabled": false
}
```

**Response Error (400 Bad Request):**
```json
{
  "error": "Validation Error",
  "message": "Event type cannot be empty",
  "details": {
    "field": "event_type",
    "reason": "required"
  }
}
```

**Response Error (500 Internal Server Error):**
```json
{
  "error": "Internal Error",
  "message": "ClickUp integration failed",
  "details": {
    "clickup_error": "Status 'Open' not found"
  }
}
```

### 5. Listar Tarefas do ClickUp
**GET /clickup/tasks**

Lista todas as tarefas da lista configurada no ClickUp.

**Query Parameters:**
- `archived`: boolean (incluir tarefas arquivadas, default: false)
- `page`: number (página para paginação, default: 0)

**Response (200 OK):**
```json
{
  "tasks": [
    {
      "id": "86947v4hj",
      "name": "🎯 João Silva - Apartamento 2 Quartos",
      "status": {
        "status": "pendente",
        "color": "#d3d3d3",
        "type": "open"
      },
      "date_created": "1641919400000",
      "date_updated": "1641919500000",
      "creator": {
        "id": 183,
        "username": "Sistema ChatGuru",
        "email": "system@chatguru.com"
      },
      "assignees": [],
      "description": "Lead: João Silva\nTelefone: +5511999999999\nProjeto: Apartamento 2 Quartos\n\nFonte: WhatsApp ChatGuru",
      "tags": [],
      "priority": null,
      "due_date": null,
      "start_date": null,
      "time_estimate": null,
      "custom_fields": []
    }
  ]
}
```

### 6. Informações da Lista ClickUp
**GET /clickup/list**

Retorna informações detalhadas sobre a lista configurada no ClickUp.

**Response (200 OK):**
```json
{
  "id": "901300373349",
  "name": "Leads WhatsApp",
  "orderindex": 0,
  "content": "",
  "status": {
    "status": "green",
    "color": "#68d391",
    "hide_label": false
  },
  "priority": {
    "priority": "normal",
    "color": "#a0aec0"
  },
  "assignee": null,
  "task_count": 156,
  "due_date": null,
  "start_date": null,
  "folder": {
    "id": "90130000000",
    "name": "Vendas",
    "hidden": false,
    "access": true
  },
  "space": {
    "id": "90130000000",
    "name": "Nordja CRM",
    "access": true
  },
  "statuses": [
    {
      "id": "p90130373349_IaHXtOaW",
      "status": "pendente",
      "orderindex": 0,
      "color": "#d3d3d3",
      "type": "open"
    },
    {
      "id": "p90130373349_VlUNDPLj",
      "status": "em andamento",
      "orderindex": 1,
      "color": "#3b82f6",
      "type": "custom"
    },
    {
      "id": "p90130373349_HKFJQenb",
      "status": "concluído",
      "orderindex": 2,
      "color": "#10b981",
      "type": "closed"
    }
  ]
}
```

### 7. Testar Conexão com ClickUp
**GET /clickup/test**

Testa a conexão com a API do ClickUp e valida as credenciais.

**Response (200 OK):**
```json
{
  "success": true,
  "message": "ClickUp connection successful",
  "list": {
    "id": "901300373349",
    "name": "Leads WhatsApp"
  },
  "user": {
    "id": 106092691,
    "username": "nordja_integration",
    "email": "integration@nordja.com"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Tipos de Eventos Suportados

### Eventos de Lead
- **new_lead**: Novo lead capturado
- **lead_qualified**: Lead qualificado
- **lead_converted**: Lead convertido em cliente

### Eventos de Pagamento
- **payment_created**: Pagamento criado
- **payment_completed**: Pagamento concluído com sucesso
- **payment_failed**: Falha no pagamento
- **pix_received**: PIX recebido

### Eventos de Cliente
- **customer_created**: Novo cliente cadastrado
- **customer_updated**: Dados do cliente atualizados

### Eventos Fiscais
- **invoice_generated**: Nota fiscal gerada
- **invoice_cancelled**: Nota fiscal cancelada

### Eventos de Agendamento
- **appointment_scheduled**: Agendamento realizado
- **appointment_cancelled**: Agendamento cancelado

### Eventos de Status
- **status_change**: Mudança de status
- **stage_change**: Mudança de etapa no funil

### Eventos de Mensagem
- **message_received**: Mensagem recebida
- **mensagem_recebida**: Mensagem recebida (pt-BR)

## Sistema de Geração de Títulos

### Hierarquia de Prioridades

#### 1. Prioridade Máxima: Anotações do ChatGuru
Campos verificados em ordem:
- `task_title`: Título específico configurado no ChatGuru
- `annotation`: Anotação em inglês
- `anotacao`: Anotação em português

Se qualquer um desses campos existir, será usado como título da tarefa.

#### 2. Prioridade Secundária: Campos de Título Genérico
Campos verificados:
- `title`: Título em inglês
- `titulo`: Título em português

#### 3. Prioridade Terciária: Geração Baseada no Tipo de Evento
Quando não há anotações ou títulos explícitos, o sistema gera um título baseado no tipo de evento.

### Títulos Gerados por Tipo de Evento

| Tipo de Evento | Formato do Título | Exemplo |
|----------------|-------------------|---------|
| `new_lead` | 🎯 {lead_name} - {project_name} | 🎯 João Silva - Apartamento 2 Quartos |
| `payment_created` | 💰 Novo Pagamento - R$ {amount} | 💰 Novo Pagamento - R$ 500.00 |
| `payment_completed` | ✅ Pagamento Concluído - R$ {amount} | ✅ Pagamento Concluído - R$ 500.00 |
| `payment_failed` | ❌ Falha no Pagamento - {reason} | ❌ Falha no Pagamento - Cartão recusado |
| `pix_received` | ⚡ PIX Recebido - R$ {amount} | ⚡ PIX Recebido - R$ 250.00 |
| `customer_created` | 👤 Novo Cliente - {name} | 👤 Novo Cliente - Maria Santos |
| `invoice_generated` | 📄 Nota Fiscal Gerada - {invoice_number} | 📄 Nota Fiscal Gerada - NF-2024-001 |
| `appointment_scheduled` | 📅 {appointment_type} - {lead_name} | 📅 Visita ao imóvel - Pedro Costa |
| `status_change` | 🔄 {lead_name} - {new_status} | 🔄 Ana Silva - Em negociação |
| `message_received` | 💬 Mensagem de {sender_name} | 💬 Mensagem de Carlos Lima |
| Evento padrão | 🔔 Evento ChatGuru - {event_type} | 🔔 Evento ChatGuru - custom_event |

## Validações de Webhook

### Campos Obrigatórios
1. **id**: Identificador único do evento (não pode ser vazio)
2. **event_type**: Tipo do evento (não pode ser vazio)
3. **timestamp**: Data/hora do evento em formato RFC3339
4. **data**: Objeto com dados do evento (não pode ser null ou vazio)

### Validações de Timestamp
- Formato obrigatório: RFC3339 (ex: `2024-01-15T10:30:00.000Z`)
- Idade máxima: 5 minutos (eventos mais antigos são rejeitados)

### Assinatura HMAC
Se configurado o `webhook_secret`, o sistema valida a assinatura HMAC-SHA256:
- Header: `X-ChatGuru-Signature`
- Formato: `sha256=<hmac_hex>`

## Configurações e Variáveis de Ambiente

### Variáveis Obrigatórias
```bash
# Token da API do ClickUp
CLICKUP_API_TOKEN=pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657

# ID da lista no ClickUp onde as tarefas serão criadas
CLICKUP_LIST_ID=901300373349

# Nível de log (debug, info, warn, error)
RUST_LOG=info

# Porta do servidor (padrão: 8080)
PORT=8080
```

### Variáveis Opcionais
```bash
# Secret para validação HMAC do webhook
CHATGURU_WEBHOOK_SECRET=seu_secret_aqui

# Configurações do Google Cloud (se usar PubSub)
GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json
GOOGLE_CLOUD_PROJECT=seu-projeto-gcp
```

## Exemplos de Testes com cURL

### Teste 1: Evento com Anotação do ChatGuru
```bash
curl -X POST https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "id": "evt_123456789",
    "event_type": "new_lead",
    "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")'",
    "data": {
      "task_title": "URGENTE - Cliente VIP quer fechar hoje",
      "lead_name": "João Silva",
      "phone": "+5511999999999",
      "project_name": "Cobertura Duplex"
    }
  }'
```

### Teste 2: Evento de Pagamento
```bash
curl -X POST https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "id": "evt_payment_001",
    "event_type": "payment_created",
    "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%S.000Z")'",
    "data": {
      "amount": 1500.00,
      "customer_name": "Maria Santos",
      "payment_method": "pix"
    }
  }'
```

### Teste 3: Verificar Lista do ClickUp
```bash
curl -X GET https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/clickup/list
```

### Teste 4: Listar Tarefas
```bash
curl -X GET https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/clickup/tasks?archived=false&page=0
```

### Teste 5: Health Check
```bash
curl -X GET https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/health
```

## Estrutura do Payload ClickUp (Criação de Tarefa)

Quando o middleware cria uma tarefa no ClickUp, envia o seguinte payload:

```json
{
  "name": "[Campanha de Vendas] João Silva",
  "description": "**Dados do Contato**\n\n- Nome: João Silva\n- Email: joao@example.com\n- Celular: +5511999999999\n- Campanha: Campanha de Vendas\n- Origem: ChatGuru\n\n**Mensagem**\nGostaria de saber mais sobre o produto\n\n**Link do Chat**\nhttps://app.chatguru.com/chat/123\n\n**Campos Personalizados**\n- Empresa: EMPRESA S.A\n- CNPJ: 24.111.111/0001-01\n- Valor: 1567.87\n\n**Responsável**: Maria Santos (maria@empresa.com)",
  "status": "pendente",
  "priority": 3,
  "tags": ["🤖 Zap.Guru", "✅ Fechado e Ganho", "Origem: Instagram"]
}
```

### Prevenção de Duplicatas
- O sistema busca tarefas existentes pelo título `[Campanha] Nome`
- Se encontrar: Atualiza a tarefa e adiciona comentário com histórico
- Se não encontrar: Cria nova tarefa

## Tratamento de Erros

### Erros de Validação (400)
- Campo obrigatório ausente
- Formato de timestamp inválido
- Assinatura HMAC inválida
- Evento muito antigo (> 5 minutos)
- Data vazio ou null

### Erros do ClickUp (500)
- Status inválido para a lista
- Token de API inválido
- Lista não encontrada
- Limite de rate da API excedido

### Respostas de Erro
```json
{
  "error": "Validation Error",
  "message": "Event type cannot be empty",
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "req_abc123"
}
```

## Logs e Depuração

### Habilitar Logs Detalhados
```bash
RUST_LOG=debug cargo run
```

### Tipos de Log
- **INFO**: Requisições recebidas, tarefas criadas
- **WARN**: Eventos rejeitados por validação
- **ERROR**: Falhas de integração com ClickUp
- **DEBUG**: Detalhes de processamento, payloads completos

### Exemplo de Logs
```
[INFO] Request received: /webhooks/chatguru [POST]
[INFO] Processing ChatGuru event: new_lead (evt_123456789)
[INFO] Task title generated: 🎯 João Silva - Apartamento 2 Quartos
[INFO] ClickUp task created - ID: 86947v4hj
[INFO] Request processed: /webhooks/chatguru [200] (125ms)
```

## Deploy e Manutenção

### Deploy no Google Cloud Run
```bash
# Via script automático
cd chatguru-clickup-middleware
./quick-deploy.sh

# Ou manualmente
gcloud run deploy chatguru-clickup-middleware \
  --source . \
  --region southamerica-east1 \
  --allow-unauthenticated \
  --project buzzlightear \
  --set-env-vars "CLICKUP_API_TOKEN=pk_...,CLICKUP_LIST_ID=901300373349,RUST_LOG=info" \
  --memory 512Mi \
  --cpu 1
```

### Monitoramento
- **Cloud Run Console**: Métricas de requisições, latência, erros
- **Cloud Logging**: Logs estruturados em tempo real
- **ClickUp**: Verificar tarefas criadas na lista configurada

### Testes de Integração
```bash
# Executar suite completa de testes
cd chatguru-clickup-middleware
npm test

# Teste rápido local
node test-quick.js

# Teste contra produção
node tests/integration_test.js
```

## Arquivos Importantes do Projeto

| Arquivo | Descrição |
|---------|-----------|
| `src/main.rs` | Entry point, configuração de rotas |
| `src/handlers/webhook.rs` | Handler do webhook, validações |
| `src/services/clickup.rs` | Integração com API ClickUp, geração de títulos |
| `src/models/chatguru_events.rs` | Estruturas de dados dos eventos |
| `config/default.toml` | Configurações padrão |
| `tests/integration_test.js` | Suite de testes de integração |
| `SOLUCAO_ERRO_500.md` | Documentação da correção do erro 500 |

## Notas de Implementação

### Status do ClickUp
A lista `901300373349` possui os seguintes status válidos:
- **pendente** (tipo: open, cor: #d33d44) - Status padrão para novas tarefas
- **aguardando pagamento** (tipo: custom, cor: #f8ae00)
- **para reembolso de cliente** (tipo: closed, cor: #008844)
- **quitado - nada a fazer** (tipo: done, cor: #b660e0)

⚠️ **Importante**: Não usar "Open" ou "to do" como status - usar "pendente" 

### Custom Fields
⚠️ **Temporariamente desabilitados** (23/09/2025)
- Motivo: Requerem UUIDs válidos dos campos
- Erro anterior: "Custom field id must be a valid UUID"
- Para habilitar:
  1. Criar custom fields no ClickUp
  2. Obter os UUIDs dos campos
  3. Mapear campos do ChatGuru para UUIDs
  4. Adicionar ao payload de criação de tarefa

### Taxa de Sucesso dos Testes
- **100%** após correções de 23/09/2025
- Correções aplicadas:
  - Status mudado de "to do" para "pendente"
  - Custom fields removidos temporariamente
  - Detecção de duplicatas funcionando corretamente

## Atualizações Recentes (23/09/2025)

### Correções Implementadas
1. **Status da Tarefa**: Corrigido de "to do" para "pendente" (status válido da lista)
2. **Custom Fields**: Removidos temporariamente (requerem UUIDs válidos)
3. **Detecção de Duplicatas**: Funcionando corretamente baseada no título da tarefa
4. **Estrutura de Dados**: Adaptada para formato atual do ChatGuru

### Testes Realizados
- ✅ Criação de nova tarefa (João Silva)
- ✅ Detecção e atualização de tarefa existente (mesmo nome)
- ✅ Criação de segunda tarefa (Maria Oliveira)
- ✅ Histórico preservado em comentários

## Conclusão

Este middleware fornece uma integração robusta e confiável entre ChatGuru e ClickUp, com:
- ✅ Validação completa de webhooks
- ✅ Geração inteligente de títulos
- ✅ Prevenção de duplicatas
- ✅ Histórico de atualizações
- ✅ Tratamento de erros robusto
- ✅ Deploy automatizado no GCP
- ✅ Logs estruturados
- ✅ Suite de testes 100% funcional

Para suporte ou melhorias, consulte o repositório do projeto.
