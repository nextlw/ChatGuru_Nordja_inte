# ChatGuru-ClickUp Integration Middleware

## Descrição
Middleware de integração entre ChatGuru (chatbot WhatsApp) e ClickUp (gerenciamento de tarefas) para automatizar a criação e atualização de tarefas baseadas em conversas do WhatsApp.

## Funcionalidades

### ✅ Implementadas
- Recebimento de webhooks do ChatGuru
- Criação automática de tarefas no ClickUp
- Prevenção de duplicatas (detecta tarefas existentes pelo título)
- Atualização de tarefas existentes com histórico em comentários
- Health check e endpoints de status
- Logging estruturado com tracing
- Configuração via arquivo TOML e variáveis de ambiente

### 🔧 Correções Recentes (23/09/2025)
1. **Status da Tarefa**: Corrigido de "to do" para "pendente" (status válido da lista)
2. **Custom Fields**: Removidos temporariamente (requerem UUIDs válidos)
3. **Detecção de Duplicatas**: Funcionando corretamente baseada no título da tarefa

## Arquitetura

```
chatguru-clickup-middleware/
├── src/
│   ├── main.rs              # Entry point da aplicação
│   ├── handlers/
│   │   └── webhook.rs        # Handler para webhooks do ChatGuru
│   ├── services/
│   │   ├── clickup.rs        # Integração com API do ClickUp
│   │   ├── pubsub.rs         # Google Pub/Sub (desativado)
│   │   └── secret_manager.rs # Gerenciamento de secrets
│   ├── models/
│   │   └── chatguru_events.rs # Estrutura de dados do ChatGuru
│   ├── config/
│   │   └── mod.rs            # Configuração da aplicação
│   └── utils/
│       ├── mod.rs            # Utilitários gerais
│       └── logging.rs        # Sistema de logging
├── config/
│   └── default.toml          # Configuração padrão
├── Cargo.toml               # Dependências Rust
└── Dockerfile               # Container para deploy

```

## Configuração

### Variáveis de Ambiente Necessárias

```bash
# ClickUp API
CLICKUP_API_TOKEN=pk_...  # Token de API do ClickUp
CLICKUP_LIST_ID=901300373349  # ID da lista no ClickUp

# Servidor
PORT=8080  # Porta do servidor (padrão: 8080)
RUST_LOG=info  # Nível de log (trace, debug, info, warn, error)

# Opcional - ChatGuru
CHATGURU_WEBHOOK_SECRET=your_secret  # Para validação de assinatura
```

### Arquivo de Configuração (config/default.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080

[clickup]
# Token e List ID são obtidos via variáveis de ambiente

[chatguru]
# webhook_secret é opcional
```

## Instalação e Execução

### Pré-requisitos
- Rust 1.70+ instalado
- Docker (opcional, para containerização)
- Node.js 14+ (para scripts de teste)

### Desenvolvimento Local

```bash
# Clonar o repositório
git clone [seu-repositorio]
cd ChatGuru_Nordja_inte

# Entrar no diretório do middleware
cd chatguru-clickup-middleware

# Instalar dependências e compilar
cargo build

# Configurar variáveis de ambiente
export CLICKUP_API_TOKEN="pk_seu_token_aqui"
export CLICKUP_LIST_ID="901300373349"

# Executar em modo desenvolvimento
cargo run

# Ou em modo release (otimizado)
cargo build --release
./target/release/chatguru-clickup-middleware
```

### Docker

```bash
# Build da imagem
docker build -t chatguru-clickup-middleware .

# Executar container
docker run -p 8080:8080 \
  -e CLICKUP_API_TOKEN="pk_seu_token" \
  -e CLICKUP_LIST_ID="901300373349" \
  chatguru-clickup-middleware
```

## Endpoints da API

### Health & Status

```bash
# Health check
GET /health
Response: {"service":"suri-clickup-middleware","status":"healthy","version":"0.1.0"}

# Readiness check
GET /ready
Response: {"ready":true}

# Status completo
GET /status
Response: {"service":"suri-clickup-middleware","status":"operational","uptime":123}
```

### Webhook ChatGuru

```bash
POST /webhooks/chatguru
Content-Type: application/json

Body:
{
  "campanha_id": "123456",
  "campanha_nome": "Campanha de Vendas",
  "nome": "João Silva",
  "email": "cliente@email.com",
  "celular": "5562999999999",
  "texto_mensagem": "Mensagem do cliente",
  "tags": ["tag1", "tag2"],
  "link_chat": "https://app.chatguru.com/chat/123"
}

Response:
{
  "success": true,
  "clickup_task_id": "abc123",
  "clickup_task_action": "created|updated",
  "message": "Event processed successfully"
}
```

### ClickUp Integration

```bash
# Testar conexão com ClickUp
GET /clickup/test
Response: {"success":true,"user":{...}}

# Listar informações da lista
GET /clickup/list
Response: {"list":{...},"statuses":[...]}

# Listar tarefas
GET /clickup/tasks
Response: {"tasks":[...],"count":10}
```

## Testes

### Scripts de Teste Disponíveis

```bash
# Teste de webhook - João Silva
node test-webhook-chatguru.js

# Teste de webhook - Maria Oliveira (dados diferentes)
node test-webhook-chatguru-2.js

# Teste de autenticação ClickUp
node test-clickup-auth.js

# Debug OAuth ClickUp
node debug-clickup-oauth.js
```

### Exemplo de Resposta de Sucesso

```json
{
  "campanha_id": "123456",
  "campanha_nome": "Campanha de Vendas",
  "clickup_task_action": "created",
  "clickup_task_id": "86ac1t2ej",
  "clickup_task_processed": true,
  "message": "Event processed successfully - task created",
  "nome_contato": "João Silva",
  "processed_at": "2025-09-23T21:28:38.378097+00:00",
  "success": true
}
```

## Fluxo de Integração

1. **WhatsApp → ChatGuru**: Cliente envia mensagem
2. **ChatGuru → Middleware**: Webhook com dados do contato
3. **Middleware Processing**:
   - Valida dados recebidos
   - Busca tarefa existente pelo título `[Campanha] Nome`
   - Se existir: Atualiza e adiciona comentário com histórico
   - Se não existir: Cria nova tarefa
4. **Middleware → ClickUp**: API call para criar/atualizar tarefa
5. **Response**: Confirmação com ID da tarefa

## Status das Tarefas no ClickUp

A lista configurada (901300373349) possui os seguintes status válidos:
- `pendente` (padrão para novas tarefas)
- `aguardando pagamento`
- `para reembolso de cliente`
- `quitado - nada a fazer`

## Troubleshooting

### Erro: "Status not found"
- **Causa**: Status inválido para a lista
- **Solução**: Usar "pendente" ou outro status válido da lista

### Erro: "Custom field id must be a valid UUID"
- **Causa**: Tentativa de usar custom fields sem UUIDs
- **Solução**: Custom fields temporariamente removidos

### Erro: "ClickUp integration failed"
- **Possíveis causas**:
  - Token de API inválido
  - List ID incorreto
  - Falta de permissões na lista
- **Verificar**: Variáveis de ambiente e permissões no ClickUp

## Deploy

### Deploy Automático (Recomendado)

```bash
# Entrar no diretório do middleware
cd chatguru-clickup-middleware

# Executar o script de deploy
./deploy.sh

# Escolher opção 1 (Deploy direto do código)
```

### Deploy Manual (Alternativa)

```bash
# Deploy direto do código fonte
gcloud run deploy chatguru-clickup-middleware \
  --source . \
  --region southamerica-east1 \
  --allow-unauthenticated \
  --project buzzlightear \
  --set-env-vars "CLICKUP_API_TOKEN=pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657,CLICKUP_LIST_ID=901300373349,RUST_LOG=info"
```

**Importante**: NÃO incluir PORT nas variáveis de ambiente (Cloud Run define automaticamente)

## Monitoramento

- Logs estruturados com `tracing`
- Níveis: TRACE, DEBUG, INFO, WARN, ERROR
- Health checks para monitoramento externo
- Métricas de tempo de processamento

## Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## Licença

Proprietary - Nordja Company

## Contato

Para suporte e questões técnicas, entre em contato com a equipe de desenvolvimento.