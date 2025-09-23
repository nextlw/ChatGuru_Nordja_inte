# 🤖 ChatGuru Dialog CLI

Uma ferramenta interativa de linha de comando para gerenciar diálogos do ChatGuru via API.

## 📋 Funcionalidades

- ✅ Listar todos os diálogos
- 🔍 Ver detalhes de diálogos específicos
- ➕ Criar novos diálogos
- ✏️ Atualizar diálogos existentes
- 🗑️ Deletar diálogos
- 🔗 Configurar webhooks
- 🧪 Testar webhooks
- 📱 Executar diálogos para contatos WhatsApp
- ⚙️ Gerenciar configurações da API

## 🚀 Como Usar

### Instalação

1. Certifique-se de ter Node.js instalado:
```bash
node --version
```

2. Torne o arquivo executável:
```bash
chmod +x chatguru-dialog-cli.js
```

### Executar a Ferramenta

```bash
# Método 1: Direto com Node.js
node chatguru-dialog-cli.js

# Método 2: Como executável (depois de chmod +x)
./chatguru-dialog-cli.js
```

## 🎯 Exemplos de Uso

### 1. Listar Diálogos

```
Opção: 1
```

Mostra todos os diálogos disponíveis com seus IDs, nomes e status.

### 2. Criar Novo Diálogo com Webhook

```
Opção: 3
Nome do diálogo: Nova API de Tarefas
Descrição: Cria tarefas no ClickUp via WhatsApp
URL do Webhook: https://chatguru-clickup-middleware.run.app/webhooks/chatguru
```

### 3. Configurar Webhook em Diálogo Existente

```
Opção: 6
ID do diálogo: nova_api
URL do webhook: https://sua-url.com/webhook
Método HTTP: POST
Adicionar autenticação? s
Tipo de auth: 1
Valor da autenticação: seu-token-aqui
```

### 4. Testar Webhook

```
Opção: 7
URL do webhook: https://chatguru-clickup-middleware.run.app/webhooks/chatguru
```

Envia dados de teste para verificar se o webhook está funcionando.

### 5. Executar Diálogo para Contato

```
Opção: 8
ID do diálogo: nova_api
Número WhatsApp: 5511999999999
Adicionar variáveis? s
Nome da variável: tarefa
Valor da variável: Criar relatório mensal
Nome da variável: prioridade
Valor da variável: Alta
```

## 📝 Formato de Dados

### Webhook Enviado pelo ChatGuru

```json
{
  "annotation": {
    "data": {
      "tarefa": "Descrição da tarefa",
      "prioridade": "Alta",
      "responsavel": "João",
      "custom_field": "valor"
    }
  },
  "contact": {
    "number": "5511999999999",
    "name": "Nome do Contato"
  },
  "message": {
    "text": "Texto da mensagem",
    "type": "text",
    "timestamp": "2024-01-20T10:00:00Z"
  }
}
```

## 🔧 Configuração da API

### Credenciais Padrão

As credenciais já estão configuradas no arquivo. Para alterá-las:

1. Use a opção 9 no menu principal
2. Ou edite diretamente no arquivo:

```javascript
const CONFIG = {
    API_BASE_URL: 'https://s15.chatguru.app/api/v1',
    API_KEY: 'SUA_API_KEY_AQUI',
    ACCOUNT_ID: 'SEU_ACCOUNT_ID',
    PHONE_IDS: {
        'main': 'PHONE_ID_PRINCIPAL',
        'secondary': 'PHONE_ID_SECUNDARIO'
    }
};
```

### Como Obter as Credenciais

1. Acesse o painel do ChatGuru
2. Vá em **Configurações** → **Celulares**
3. Encontre a seção **INFORMAÇÕES DA API**
4. Copie:
   - **API Key**
   - **Account ID**
   - **Phone IDs**

## 🎨 Interface Colorida

A ferramenta usa cores para melhorar a experiência:

- 🔵 **Azul**: URLs e links
- 🟢 **Verde**: Operações bem-sucedidas
- 🔴 **Vermelho**: Erros e opção de sair
- 🟡 **Amarelo**: Avisos e processamento
- 🔷 **Ciano**: Títulos e IDs

## 🛠️ Troubleshooting

### Erro de Autenticação

```
❌ Erro ao buscar diálogos: 401 Unauthorized
```

**Solução**: Verifique se a API Key está correta (opção 9)

### Diálogo Não Encontrado

```
❌ Erro ao buscar detalhes: 404 Not Found
```

**Solução**: Verifique o ID do diálogo com a opção 1

### Webhook Não Funcionando

1. Use a opção 7 para testar o webhook
2. Verifique se a URL está acessível publicamente
3. Confirme que o formato dos dados está correto

## 📚 Endpoints da API ChatGuru

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| GET | `/dialogs` | Lista todos os diálogos |
| GET | `/dialogs/{id}` | Detalhes de um diálogo |
| POST | `/dialogs` | Criar novo diálogo |
| PUT | `/dialogs/{id}` | Atualizar diálogo |
| DELETE | `/dialogs/{id}` | Deletar diálogo |
| POST | `/dialog_execute` | Executar diálogo |
| POST | `/message_send` | Enviar mensagem |

## 🚨 Segurança

- **Não compartilhe suas credenciais da API**
- Use variáveis de ambiente para credenciais sensíveis em produção
- Configure webhooks com HTTPS sempre que possível
- Adicione autenticação nos webhooks quando necessário

## 💡 Dicas

1. **Teste sempre os webhooks** antes de usar em produção
2. **Mantenha um backup** dos IDs dos diálogos importantes
3. **Use nomes descritivos** para os diálogos
4. **Configure logs** no seu servidor de webhook para debug

## 📞 Suporte

Para problemas com a API do ChatGuru:
- Documentação: https://wiki.chatguru.com.br
- Status: https://status.chatguru.app
- Suporte: support@chatguru.com.br

---

*Desenvolvido para facilitar a integração com ChatGuru API v1*