# Integração ChatGuru × ClickUp - Empresa Nordja

## 📋 Visão Geral

Esta documentação detalha a implementação da integração entre a plataforma de chatbot **ChatGuru** e o sistema de gestão de projetos **ClickUp** para a empresa **Nordja**. O objetivo é criar um fluxo automatizado que reconheça pedidos nas conversas do chatbot e crie tarefas automaticamente no ClickUp.

## 🎯 Objetivo

Migrar a funcionalidade existente no ChatGuru para a ChatGuru, implementando:
- **Reconhecimento automático de pedidos** via PLN (Processamento de Linguagem Natural)
- **Criação automática de tarefas** no ClickUp via API
- **Fluxo conversacional** fluido e natural
- **Captura de dados** relevantes para a tarefa

## 🏗️ Arquitetura da Solução

### Componentes da Integração

```mermaid
graph LR
    A[Cliente WhatsApp] --> B[ChatGuru Chatbot]
    B --> C[ChatGuru Flow - PLN]
    C --> D[Requisição HTTP]
    D --> E[Middleware/API]
    E --> F[ClickUp API]
    F --> G[Tarefa Criada]
    G --> H[Confirmação para Cliente]
```

### Fluxo de Dados

1. **Cliente** envia mensagem no WhatsApp
2. **ChatGuru PLN** identifica intenção de "criar pedido/tarefa"
3. **Flow** executa captura de dados necessários
4. **Requisição HTTP** envia dados para middleware
5. **Middleware** processa e chama API do ClickUp
6. **ClickUp** cria a tarefa
7. **Confirmação** é enviada ao cliente

## 🔧 Componentes Técnicos

### 1. ChatGuru Flow - Reconhecimento de Pedidos

#### Gatilho PLN (Frases de Treino)
```
Frases de exemplo para reconhecer pedidos:
- "Preciso fazer um pedido"
- "Quero solicitar um serviço"
- "Gostaria de encomendar"
- "Preciso de um orçamento"
- "Quero fazer uma solicitação"
- "Tenho uma demanda"
- "Preciso contratar"
- "Quero um projeto"
```

#### Captura de Dados
O fluxo deve capturar:
- **Nome do cliente**
- **Descrição do pedido**
- **Prioridade** (Alta/Média/Baixa)
- **Prazo desejado**
- **Contato adicional** (email, telefone)
- **Observações especiais**

### 2. ClickUp API Integration

#### Autenticação
```http
Authorization: Bearer {access_token}
Content-Type: application/json
```

#### Endpoint para Criação de Tarefas
```http
POST https://api.clickup.com/api/v2/list/{list_id}/task
```

#### Payload de Exemplo
```json
{
  "name": "Pedido - [Nome do Cliente]",
  "description": "[Descrição detalhada do pedido]",
  "assignees": [12345],
  "tags": ["pedido", "chatguru-bot"],
  "status": "to do",
  "priority": 3,
  "due_date": 1640995200000,
  "custom_fields": [
    {
      "id": "cf_cliente",
      "value": "Nome do Cliente"
    },
    {
      "id": "cf_origem",
      "value": "ChatGuru WhatsApp"
    }
  ]
}
```

### 3. Middleware/API Bridge

#### Estrutura do Servidor
```typescript
// server.ts
import express from 'express';
import axios from 'axios';

const app = express();
app.use(express.json());

// Endpoint para receber dados da ChatGuru
app.post('/chatguru/create-task', async (req, res) => {
  try {
    const { cliente, descricao, prioridade, prazo, contato } = req.body;
    
    // Criar tarefa no ClickUp
    const clickupResponse = await createClickUpTask({
      cliente,
      descricao,
      prioridade,
      prazo,
      contato
    });
    
    res.json({
      success: true,
      taskId: clickupResponse.data.id,
      taskUrl: clickupResponse.data.url
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});
```

#### Função de Criação de Tarefa
```typescript
async function createClickUpTask(pedidoData) {
  const payload = {
    name: `Pedido - ${pedidoData.cliente}`,
    description: `
**Cliente:** ${pedidoData.cliente}
**Descrição:** ${pedidoData.descricao}
**Contato:** ${pedidoData.contato}
**Prazo Desejado:** ${pedidoData.prazo}
**Origem:** ChatGuru WhatsApp Bot
**Data/Hora:** ${new Date().toLocaleString('pt-BR')}
    `,
    assignees: [NORDJA_TEAM_MEMBER_ID],
    tags: ["pedido", "chatguru-bot", "whatsapp"],
    status: "Open",
    priority: getPriorityValue(pedidoData.prioridade),
    due_date: calculateDueDate(pedidoData.prazo),
    custom_fields: [
      {
        id: "cf_cliente_nome",
        value: pedidoData.cliente
      },
      {
        id: "cf_origem_canal",
        value: "WhatsApp - ChatGuru Bot"
      }
    ]
  };

  return await axios.post(
    `https://api.clickup.com/api/v2/list/${CLICKUP_LIST_ID}/task`,
    payload,
    {
      headers: {
        'Authorization': `Bearer ${CLICKUP_API_TOKEN}`,
        'Content-Type': 'application/json'
      }
    }
  );
}
```

## 🔄 Implementação do Fluxo ChatGuru

### Estrutura do Flow

1. **Gatilho PLN**: Reconhecimento de intenção de pedido
2. **Captura de Nome**: "Qual é o seu nome?"
3. **Captura de Descrição**: "Descreva o que você precisa:"
4. **Captura de Prioridade**: Botões (Alta/Média/Baixa)
5. **Captura de Prazo**: "Qual o prazo desejado?"
6. **Captura de Contato**: "Confirme seu contato:"
7. **Requisição HTTP**: Envio para middleware
8. **Confirmação**: Mensagem de sucesso com ID da tarefa

### Configuração da Requisição HTTP no ChatGuru

#### URL do Endpoint
```
https://your-middleware.com/chatguru/create-task
```

#### Método
```
POST
```

#### Headers
```json
{
  "Content-Type": "application/json",
  "Authorization": "Bearer YOUR_MIDDLEWARE_TOKEN"
}
```

#### Body (Dados a enviar)
```json
{
  "cliente": "{{nome_cliente}}",
  "descricao": "{{descricao_pedido}}",
  "prioridade": "{{prioridade_selecionada}}",
  "prazo": "{{prazo_desejado}}",
  "contato": "{{contato_cliente}}"
}
```

#### Tratamento da Resposta
```
Retorno de Sucesso (200):
"✅ Pedido registrado com sucesso!
📋 ID da Tarefa: {{response.taskId}}
🔗 Link: {{response.taskUrl}}

Em breve nossa equipe entrará em contato!"

Retorno de Erro (500):
"❌ Ops! Ocorreu um problema ao registrar seu pedido.
Por favor, tente novamente ou entre em contato diretamente."
```

## ⚙️ Configurações Necessárias

### 1. ClickUp
- **API Token**: Gerar token pessoal ou OAuth
- **List ID**: Identificar a lista onde criar as tarefas
- **Team Member IDs**: IDs dos responsáveis padrão
- **Custom Fields**: Configurar campos personalizados se necessário

### 2. ChatGuru
- **Flow**: Criar o fluxo de captura de pedidos
- **PLN Training**: Treinar com frases variadas
- **Variables**: Configurar variáveis de captura
- **HTTP Request**: Configurar requisição para middleware

### 3. Middleware
- **Server**: Configurar servidor Express.js
- **Authentication**: Implementar segurança (tokens)
- **Error Handling**: Tratamento de erros robusto
- **Logging**: Logs para debugging

## 🚀 Passos para Implementação

### Fase 1: Preparação
1. **Obter credenciais do ClickUp**
2. **Identificar List ID** onde criar tarefas
3. **Configurar servidor middleware**
4. **Testar conectividade** com API ClickUp

### Fase 2: Desenvolvimento
1. **Implementar middleware** com endpoint para ChatGuru
2. **Criar função** de integração com ClickUp
3. **Implementar tratamento** de erros
4. **Adicionar logging** e monitoramento

### Fase 3: Configuração ChatGuru
1. **Criar Flow** com gatilhos PLN
2. **Configurar captura** de dados
3. **Implementar requisição HTTP**
4. **Configurar respostas** de sucesso/erro

### Fase 4: Testes
1. **Testes unitários** do middleware
2. **Testes de integração** ChatGuru ↔ ClickUp
3. **Testes de usabilidade** do fluxo
4. **Validação** com equipe Nordja

### Fase 5: Deploy e Monitoramento
1. **Deploy do middleware** em servidor/cloud
2. **Configurar monitoramento** de logs e métricas
3. **Implementar alertas** para falhas
4. **Documentar** para equipe de suporte

## 📊 Métricas de Sucesso

### KPIs Principais
- **Taxa de conversão**: Pedidos iniciados vs concluídos
- **Tempo médio de resposta**: Middleware → ClickUp
- **Taxa de erro**: Falhas na criação de tarefas
- **Volume de pedidos**: Total por dia/semana/mês

### Monitoramento
- **Uptime do middleware**: > 99.5%
- **Latência média**: < 2 segundos
- **Taxa de sucesso**: > 95%
- **Satisfação do usuário**: Feedback positivo

## 🔒 Segurança e Compliance

### Medidas de Segurança
- **HTTPS** em todas as comunicações
- **Autenticação** por tokens seguros
- **Validação** de dados de entrada
- **Rate limiting** para prevenir abuse
- **Logs** criptografados e auditáveis

### LGPD Compliance
- **Consentimento** para coleta de dados
- **Anonimização** quando necessário
- **Direito ao esquecimento** implementado
- **Portabilidade** de dados garantida

## 🚨 Troubleshooting

### Problemas Comuns

#### 1. Falha na criação de tarefa
**Causa**: Token inválido ou expirado
**Solução**: Renovar token no ClickUp

#### 2. Timeout na requisição
**Causa**: Latência alta na rede
**Solução**: Aumentar timeout ou implementar retry

#### 3. Dados não capturados
**Causa**: Variáveis mal configuradas no ChatGuru
**Solução**: Revisar mapeamento de variáveis

#### 4. Mensagem de erro genérica
**Causa**: Erro não tratado no middleware
**Solução**: Implementar tratamento específico

## 📚 Recursos Adicionais

### Documentação
- [ChatGuru API Docs](https://docs.chatguru.com)
- [ClickUp API Reference](https://clickup.com/api)
- [Webhook Best Practices](https://webhooks.dev)

### Ferramentas Úteis
- **Postman**: Testar APIs
- **ngrok**: Expor localhost para testes
- **Sentry**: Monitoramento de erros
- **Grafana**: Dashboard de métricas

## 🎉 Conclusão

Esta integração permite que a empresa Nordja automatize completamente o processo de criação de tarefas a partir de conversas no WhatsApp, melhorando significativamente a eficiência operacional e a experiência do cliente.

---

**Versão**: 1.0.0  
**Última atualização**: Janeiro 2025  
**Autor**: Equipe de Integração  
**Contato**: integracao@nordja.com