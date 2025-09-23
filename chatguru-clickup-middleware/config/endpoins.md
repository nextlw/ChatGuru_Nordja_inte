endpoins.md


## 📍 Endpoint do Webhook no Cloud

Com base na análise completa do projeto, identifiquei as informações do endpoint do webhook:

### **Endpoint Principal do Webhook**

O middleware está configurado com o seguinte endpoint para receber eventos:

**`POST /webhooks/chatguru`** (versão Rust atual)

Nota: No arquivo [`main.rs`](chatguru-clickup-middleware/src/main.rs:64), linha 64, o endpoint está configurado como `/webhooks/chatguru`.

### **URLs de Deploy no Cloud**

De acordo com a documentação do [`middleware-nodejs-deploy.md`](middleware-nodejs-deploy.md:728), o projeto foi deployado no Google App Engine:

**URL Base no Cloud:** `https://buzzlightear.rj.r.appspot.com`

### **Endpoint Completo no Cloud:**

```
https://buzzlightear.rj.r.appspot.com/webhooks/chatguru
```


### **Outros Endpoints Disponíveis:**

- `GET https://buzzlightear.rj.r.appspot.com/health` - Health check
- `GET https://buzzlightear.rj.r.appspot.com/ready` - Readiness check
- `GET https://buzzlightear.rj.r.appspot.com/status` - Status da aplicação
- `GET https://buzzlightear.rj.r.appspot.com/clickup/tasks` - Listar tasks
- `GET https://buzzlightear.rj.r.appspot.com/clickup/list` - Info da lista
- `GET https://buzzlightear.rj.r.appspot.com/clickup/test` - Testar conexão

### **Configuração no ChatGuru:**

Para configurar o webhook no sistema ChatGuru, você deve usar:

- **URL do Webhook:** `https://buzzlightear.rj.r.appspot.com/webhooks/chatguru`
- **Método:** POST
- **Headers opcionais:**
  - `X-ChatGuru-Signature` (para validação de assinatura)

### **Informações do Projeto no GCP:**

- **Projeto GCP:** `buzzlightear`
- **Região:** `rj.r` (Rio de Janeiro)
- **Serviço:** Google App Engine

O webhook está pronto para receber eventos do chatbot e criar automaticamente tasks no ClickUp com list ID `901300373349`.
