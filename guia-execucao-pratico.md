# 🚀 Guia de Execução Prático - Integração ChatGuru × ClickUp

## ✅ Checklist de Implementação

### 📋 Fase 1: Configuração Inicial (30 min)

**1.1 Obter Credenciais do ClickUp**
- [ ] Acessar ClickUp → Settings → Apps
- [ ] Gerar Personal API Token (ou configurar OAuth)
- [ ] Anotar o token: `pk_xxxxxx...`
- [ ] Identificar o List ID onde criar tarefas
- [ ] Testar token via Postman/Insomnia

**1.2 Preparar Ambiente de Desenvolvimento**
```bash
mkdir chatguru-clickup-nordja
cd chatguru-clickup-nordja
npm init -y
npm install express axios cors dotenv nodemon
```

**1.3 Configurar Variáveis de Ambiente**
```bash
# Criar arquivo .env
CLICKUP_API_TOKEN=pk_seu_token_aqui
CLICKUP_LIST_ID=123456789
MIDDLEWARE_TOKEN=nordja_secure_token_2024
PORT=3000
```

### 📋 Fase 2: Deploy do Middleware (45 min)

**2.1 Criar Servidor**
- [ ] Copiar código do `server.js` da documentação
- [ ] Configurar endpoints `/chatguru/create-task` e `/health`
- [ ] Implementar autenticação por token
- [ ] Adicionar logs de debug

**2.2 Testar Localmente**
```bash
# Iniciar servidor
npm run dev

# Testar endpoints
curl http://localhost:3000/health
curl -H "Authorization: Bearer nordja_secure_token_2024" \
     http://localhost:3000/test-clickup
```

**2.3 Deploy em Produção**
```bash
# Opção 1: Heroku
heroku create chatguru-clickup-nordja
git push heroku main

# Opção 2: VPS próprio
pm2 start server.js --name chatguru-clickup-integration
```

### 📋 Fase 3: Configuração do Flow ChatGuru (60 min)

**3.1 Criar Flow Principal**
- [ ] Acessar painel ChatGuru → Flows
- [ ] Criar novo flow: "Criar Pedido ClickUp"
- [ ] Configurar gatilho PLN com frases de treino

**3.2 Frases de Treino PLN**
```
- "Preciso fazer um pedido"
- "Quero solicitar um serviço"
- "Gostaria de encomendar"
- "Preciso de um orçamento"
- "Quero fazer uma solicitação"
- "Tenho uma demanda"
- "Preciso contratar"
- "Quero um projeto"
```

**3.3 Sequência de Captura**
1. **Boas-vindas**: "Vou te ajudar a registrar seu pedido! 😊"
2. **Nome**: "Qual é o seu nome completo?"
3. **Descrição**: "Descreva o que você precisa:"
4. **Prioridade**: Botões [Alta/Média/Baixa]
5. **Prazo**: "Qual o prazo desejado?"
6. **Contato**: "Confirme seu melhor contato:"
7. **Confirmação**: Resumo dos dados
8. **Requisição HTTP**: Envio para middleware

**3.4 Configurar Requisição HTTP**
- **URL**: `https://seu-middleware.com/chatguru/create-task`
- **Método**: POST
- **Headers**: 
  ```json
  {
    "Content-Type": "application/json",
    "Authorization": "Bearer nordja_secure_token_2024"
  }
  ```
- **Body**:
  ```json
  {
    "cliente": "{{nome_cliente}}",
    "descricao": "{{descricao_pedido}}",
    "prioridade": "{{prioridade_selecionada}}",
    "prazo": "{{prazo_desejado}}",
    "contato": "{{contato_cliente}}"
  }
  ```

### 📋 Fase 4: Testes End-to-End (30 min)

**4.1 Teste de Conectividade**
- [ ] Verificar se middleware está online
- [ ] Testar endpoint de saúde
- [ ] Validar conexão com ClickUp API

**4.2 Teste de Fluxo Completo**
- [ ] Iniciar conversa no WhatsApp
- [ ] Disparar frase de treino: "Preciso fazer um pedido"
- [ ] Completar todo o fluxo de captura
- [ ] Verificar criação da tarefa no ClickUp
- [ ] Confirmar dados corretos na tarefa

**4.3 Teste de Cenários de Erro**
- [ ] Token inválido
- [ ] Campos obrigatórios vazios  
- [ ] ClickUp indisponível
- [ ] Timeout na requisição

## 🔧 Comandos Úteis para Debug

### Logs do Middleware
```bash
# Ver logs em tempo real
pm2 logs chatguru-clickup-integration

# Reiniciar se necessário
pm2 restart chatguru-clickup-integration
```

### Teste Manual da API ClickUp
```bash
# Testar criação de tarefa diretamente
curl -X POST "https://api.clickup.com/api/v2/list/SEU_LIST_ID/task" \
  -H "Authorization: Bearer SEU_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Teste - Cliente Exemplo",
    "description": "Teste de integração",
    "tags": ["teste", "chatguru-bot"]
  }'
```

### Teste do Middleware
```bash
# Simular requisição da ChatGuru
curl -X POST "https://seu-middleware.com/chatguru/create-task" \
  -H "Authorization: Bearer nordja_secure_token_2024" \
  -H "Content-Type: application/json" \
  -d '{
    "cliente": "João Silva",
    "descricao": "Preciso de um site institucional",
    "prioridade": "Alta",
    "prazo": "2 semanas",
    "contato": "joao@email.com"
  }'
```

## 📊 Monitoramento e Métricas

### KPIs para Acompanhar

**Técnicos:**
- Taxa de sucesso da integração (> 95%)
- Tempo médio de resposta (< 2 segundos)
- Disponibilidade do middleware (> 99%)
- Erros por dia (< 5)

**Negócio:**
- Pedidos criados via bot por dia
- Taxa de conversão do fluxo
- Tempo médio para atendimento
- Satisfação do cliente

### Dashboard Básico
```javascript
// Adicionar ao middleware para métricas
let stats = {
  totalRequests: 0,
  successfulTasks: 0,
  errors: 0,
  avgResponseTime: 0
};

// Endpoint de métricas
app.get('/metrics', (req, res) => {
  res.json({
    ...stats,
    successRate: (stats.successfulTasks / stats.totalRequests * 100).toFixed(2),
    uptime: process.uptime()
  });
});
```

## 🚨 Troubleshooting Rápido

### Problema: "Token inválido"
**Solução:**
1. Verificar se token está correto no .env
2. Verificar se token não expirou
3. Testar token direto na API ClickUp

### Problema: "List not found"
**Solução:**
1. Verificar LIST_ID no ClickUp
2. Confirmar permissões de acesso à lista
3. Testar com outro LIST_ID

### Problema: "PLN não reconhece pedido"
**Solução:**
1. Adicionar mais frases de treino na ChatGuru
2. Verificar se flow está ativo
3. Treinar com variações das frases

### Problema: "Timeout na requisição"
**Solução:**
1. Verificar se middleware está online
2. Aumentar timeout na ChatGuru (se possível)
3. Verificar logs do servidor

### Problema: "Dados não chegam no ClickUp"
**Solução:**
1. Verificar logs do middleware
2. Testar endpoint manualmente
3. Verificar mapeamento de campos
4. Confirmar estrutura do JSON

## 📱 Exemplo de Conversa Final

```
Cliente: "Oi, preciso fazer um pedido"

Bot: "Entendi que você quer fazer um pedido! 😊
Vou te ajudar a registrar sua solicitação.
Vamos começar?"

Cliente: "Sim"

Bot: "Primeiro, me diga seu nome completo:"

Cliente: "Maria Silva"

Bot: "Agora me conte: o que você precisa?
Seja o mais detalhado possível:"

Cliente: "Preciso criar um site para minha empresa de consultoria"

Bot: "Como você classificaria a urgência do seu pedido?"
[🔴 Alta] [🟡 Média] [🟢 Baixa]

Cliente: [clica em Média]

Bot: "Qual o prazo desejado para este pedido?"

Cliente: "30 dias"

Bot: "Por último, confirme seu melhor contato:"

Cliente: "maria.silva@email.com"

Bot: "✅ Pedido registrado com sucesso!

📋 ID da Tarefa: CU-abc123
🔗 Link: https://app.clickup.com/t/abc123

Nossa equipe recebeu sua solicitação e entrará em contato em breve!

Obrigado pela preferência! 😊"
```

## 🎯 Próximos Passos (Melhorias Futuras)

### Versão 2.0 - Recursos Avançados
- [ ] **Webhooks bidirecionais**: Notificar ChatGuru quando status da tarefa mudar
- [ ] **Templates**: Criar templates específicos por tipo de pedido
- [ ] **Anexos**: Permitir envio de arquivos/imagens
- [ ] **Multi-idioma**: Suporte a outros idiomas
- [ ] **Analytics**: Dashboard detalhado de métricas

### Integração com Outros Sistemas
- [ ] **CRM**: Integrar com Pipedrive/RD Station
- [ ] **Email**: Notificações