# ChatGuru CLI Tool - Guia Completo

## 📋 Descrição

Ferramenta de linha de comando (CLI) para interagir com a API do ChatGuru, permitindo:
- Executar diálogos (nova_api, TESTE_API)
- Enviar mensagens via WhatsApp
- Adicionar anotações em contatos
- Atualizar campos personalizados
- Testar webhooks
- Gerenciar integração com ClickUp

## 🚀 Instalação e Configuração

### 1. Pré-requisitos
- Node.js instalado (versão 14 ou superior)
- Credenciais do ChatGuru (API Key, Account ID, Phone ID)

### 2. Como executar

```bash
# Executar a ferramenta
node chatguru-cli-tool.js
```

### 3. Primeira configuração

Ao executar pela primeira vez, você deve:
1. Escolher a opção **7** (Configurar credenciais)
2. Inserir suas credenciais do ChatGuru:
   - **API Key**: Obtida no painel do ChatGuru
   - **Account ID**: ID da conta (página "Celulares")
   - **Phone ID**: ID do celular configurado

## 📱 Funcionalidades

### 1. Enviar Mensagem de Teste
Envia uma mensagem direta para um número WhatsApp.

**Dados necessários:**
- Número do WhatsApp (formato: 5511999999999)
- Mensagem a enviar

### 2. Executar Diálogo (nova_api)
Executa o diálogo "nova_api" configurado no ChatGuru.

**Dados necessários:**
- Número do WhatsApp
- Descrição da tarefa
- Prioridade (Alta/Média/Baixa)

### 3. Executar Diálogo (TESTE_API)
Executa o diálogo "TESTE_API" configurado no ChatGuru.

**Dados necessários:**
- Número do WhatsApp  
- Descrição da tarefa
- Prioridade

### 4. Adicionar Anotação
Adiciona uma anotação (note) em um contato.

**Dados necessários:**
- Número do WhatsApp
- Texto da anotação

### 5. Atualizar Campos Personalizados
Atualiza campos customizados de um contato.

**Campos configuráveis:**
- Tarefa
- Prioridade
- Responsável

### 6. Testar Webhook Diretamente
Envia um payload de teste diretamente para o webhook configurado.

**Configurações:**
- URL do webhook (default: http://localhost:8080/webhooks/chatguru)
- Payload automático de teste

### 7. Configurar Credenciais
Permite atualizar as credenciais da API do ChatGuru.

**Credenciais necessárias:**
- API Key
- Account ID
- Phone ID

### 8. Verificar Status da Integração
Verifica o status de:
- Credenciais configuradas
- Middleware local (se está rodando)
- URLs da integração

## 🔧 Estrutura da API

### Base URL
```
https://app.zap.guru/api/v1
```

### Endpoints Utilizados

#### Enviar Mensagem
```http
POST /message_send
{
  "chat_number": "5511999999999",
  "message": "Sua mensagem",
  "key": "API_KEY",
  "account_id": "ACCOUNT_ID",
  "phone_id": "PHONE_ID"
}
```

#### Executar Diálogo
```http
POST /dialog_execute
{
  "chat_number": "5511999999999",
  "dialog_id": "nova_api",
  "variables": {
    "tarefa": "Descrição da tarefa",
    "prioridade": "Alta"
  },
  "key": "API_KEY",
  "account_id": "ACCOUNT_ID",
  "phone_id": "PHONE_ID"
}
```

#### Adicionar Anotação
```http
POST /note_add
{
  "chat_number": "5511999999999",
  "note": "Texto da anotação",
  "key": "API_KEY",
  "account_id": "ACCOUNT_ID",
  "phone_id": "PHONE_ID"
}
```

#### Atualizar Campos Personalizados
```http
POST /chat_update_custom_fields
{
  "chat_number": "5511999999999",
  "custom_fields": {
    "tarefa": "Tarefa",
    "prioridade": "Alta",
    "responsavel": "João"
  },
  "key": "API_KEY",
  "account_id": "ACCOUNT_ID",
  "phone_id": "PHONE_ID"
}
```

## 🔗 Integração com ClickUp

### Fluxo da Integração

1. **Usuário envia mensagem** no WhatsApp
2. **ChatGuru processa** através do diálogo configurado
3. **Webhook é disparado** para o middleware
4. **Middleware cria tarefa** no ClickUp

### URLs do Webhook

#### Local (desenvolvimento)
```
http://localhost:8080/webhooks/chatguru
```

#### Cloud (produção)
```
https://buzzlightear.rj.r.appspot.com/webhooks/chatguru
```

### Formato do Webhook

```json
{
  "event_type": "task_created",
  "id": "unique_event_id",
  "timestamp": "2024-01-20T10:00:00Z",
  "data": {
    "chat_number": "5511999999999",
    "message": "Descrição da tarefa",
    "custom_fields": {
      "tarefa": "Conteúdo da tarefa",
      "prioridade": "Alta",
      "responsavel": "Sistema"
    }
  }
}
```

## 📊 Diagnóstico de Problemas

### Credenciais não funcionam

1. Verifique no painel do ChatGuru:
   - Acesse a página "Celulares"
   - Copie o Account ID e Phone ID corretos
   - Verifique se a API está ativada

2. Teste com curl:
```bash
curl -X POST https://app.zap.guru/api/v1/message_send \
  -H "Content-Type: application/json" \
  -d '{
    "chat_number": "5511999999999",
    "message": "Teste",
    "key": "sua_api_key",
    "account_id": "seu_account_id",
    "phone_id": "seu_phone_id"
  }'
```

### Webhook não recebe dados

1. **Verifique o middleware:**
```bash
# Verificar se está rodando
curl http://localhost:8080/health

# Ver logs
tail -f chatguru-clickup-middleware/logs/*.log
```

2. **No ChatGuru, verifique o diálogo:**
   - Entre no editor de diálogos
   - Abra o diálogo (nova_api ou TESTE_API)
   - Verifique se tem ação de webhook configurada
   - Confirme a URL do webhook

### Diálogo não executa

1. **Verifique se o diálogo existe:**
   - No painel do ChatGuru
   - Editor de diálogos
   - Procure por "nova_api" ou "TESTE_API"

2. **Verifique se está ativo:**
   - Status deve estar como "Ativo"
   - Sem erros de configuração

## 🛠️ Desenvolvimento

### Estrutura do Código

```javascript
// Configuração principal
const CONFIG = {
    API_BASE_URL: 'https://app.zap.guru/api/v1',
    API_KEY: 'sua_api_key',
    ACCOUNT_ID: 'seu_account_id',
    PHONE_ID: 'seu_phone_id'
};

// Função para requisições HTTP
function makeRequest(endpoint, method, data) {
    // Implementação
}

// Funções do menu
async function sendMessage() { }
async function executeDialog(dialogId) { }
async function addNote() { }
async function updateCustomFields() { }
async function testWebhook() { }
async function configureCredentials() { }
async function checkStatus() { }
```

### Adicionar Nova Funcionalidade

1. Crie uma nova função async:
```javascript
async function novaFuncionalidade() {
    console.log(`\n${colors.yellow}📌 Nova Funcionalidade${colors.reset}`);
    // Implementação
}
```

2. Adicione ao menu em `showMenu()`:
```javascript
console.log(`${colors.green}9.${colors.reset} Nova Funcionalidade`);
```

3. Adicione ao switch em `main()`:
```javascript
case '9':
    await novaFuncionalidade();
    break;
```

## 📚 Referências

- [Documentação API ChatGuru v1](https://oldwiki.chatguru.com.br/api/api-documentacao-v1)
- [Painel ChatGuru](https://app.zap.guru)
- [ClickUp API](https://clickup.com/api)

## 🤝 Suporte

- **ChatGuru**: support@chatguru.com.br
- **Documentação**: https://oldwiki.chatguru.com.br
- **Status do Sistema**: https://status.chatguru.com.br

## 📝 Notas Importantes

1. **Limites de API**: Verifique os limites de requisições da sua conta
2. **Segurança**: Nunca compartilhe suas credenciais (API Key, Account ID, Phone ID)
3. **Webhooks**: Certifique-se de que o middleware está rodando antes de executar diálogos
4. **Formato de Números**: Sempre use o formato internacional (5511999999999)

## 🚦 Status dos Componentes

| Componente | Status | Descrição |
|------------|--------|-----------|
| API ChatGuru | ✅ Ativo | https://app.zap.guru/api/v1 |
| Middleware Local | ⚠️ Verificar | http://localhost:8080 |
| Webhook Cloud | ✅ Ativo | https://buzzlightear.rj.r.appspot.com |
| ClickUp API | ✅ Ativo | List ID: 901300373349 |

## 📈 Próximas Melhorias

- [ ] Salvar credenciais em arquivo de configuração
- [ ] Adicionar mais opções de diálogos
- [ ] Suporte a múltiplos webhooks
- [ ] Interface gráfica web
- [ ] Logs detalhados de execução
- [ ] Testes automatizados
- [ ] Suporte a múltiplas contas

---

**Versão**: 1.0.0  
**Última atualização**: Janeiro 2025  
**Autor**: Sistema de Integração ChatGuru-ClickUp