# Setup Local com Pub/Sub Emulator

Este guia explica como rodar o middleware **localmente** com o **Google Cloud Pub/Sub Emulator**, permitindo testar todo o fluxo de webhooks → fila → worker sem precisar do GCP.

## 🎯 O que você vai conseguir

- ✅ Rodar o middleware localmente (sem Cloud Run)
- ✅ Simular Pub/Sub localmente (sem GCP Project)
- ✅ Testar webhooks → batching → publicação
- ✅ Debugar com logs em tempo real
- ✅ Desenvolver sem custo algum

## 📋 Pré-requisitos

1. **Google Cloud SDK** instalado:
   ```bash
   # Verificar instalação
   gcloud --version

   # Se não tiver, instale:
   # macOS:
   brew install --cask google-cloud-sdk

   # Linux:
   curl https://sdk.cloud.google.com | bash
   ```

2. **Rust** instalado (você já tem):
   ```bash
   cargo --version
   ```

3. **jq** para formatar JSON (opcional):
   ```bash
   brew install jq  # macOS
   ```

## 🚀 Passo a Passo

### 1️⃣ Iniciar o Pub/Sub Emulator

Em um **Terminal 1**:

```bash
cd /Users/williamduarte/NCMproduto/integrações/ChatGuru_Nordja_inte/chatguru-clickup-middleware

./start-pubsub-emulator.sh
```

**O que esse script faz:**
- Instala o emulador (se necessário)
- Inicia o servidor Pub/Sub em `localhost:8085`
- Cria diretório `./pubsub-data/` para persistência

**Output esperado:**
```
╔════════════════════════════════════════════════════════════════╗
║             INICIALIZANDO PUB/SUB EMULATOR LOCAL              ║
╚════════════════════════════════════════════════════════════════╝

📋 Configuração:
   Project ID: local-dev
   Host: localhost:8085
   ...
[pubsub] This is the Google Pub/Sub fake.
[pubsub] Implementation may be incomplete or differ from the real system.
```

💡 **Deixe esse terminal aberto!** O emulador precisa ficar rodando.

---

### 2️⃣ Criar Tópicos e Subscriptions

Em um **Terminal 2**:

```bash
cd /Users/williamduarte/NCMproduto/integrações/ChatGuru_Nordja_inte/chatguru-clickup-middleware

./setup-pubsub-topics.sh
```

**O que esse script faz:**
- Conecta ao emulador (`localhost:8085`)
- Cria tópico `chatguru-webhook-events`
- Cria tópico `clickup-webhook-events`
- Cria subscription `chatguru-worker-sub`

**Output esperado:**
```
╔════════════════════════════════════════════════════════════════╗
║            CONFIGURANDO TÓPICOS NO PUB/SUB EMULATOR           ║
╚════════════════════════════════════════════════════════════════╝

🔗 Conectando ao emulador: localhost:8085

📝 Criando tópicos...
   ✅ Tópico 'chatguru-webhook-events' criado
   ✅ Tópico 'clickup-webhook-events' criado

📬 Criando subscription...
   ✅ Subscription 'chatguru-worker-sub' criada

✅ Configuração concluída!
```

---

### 3️⃣ Iniciar o Servidor Rust Local

Em um **Terminal 3**:

```bash
cd /Users/williamduarte/NCMproduto/integrações/ChatGuru_Nordja_inte/chatguru-clickup-middleware

# Carregar variáveis de ambiente
source .env.local

# Iniciar servidor
cargo run
```

**O que acontece:**
- Rust detecta `PUBSUB_EMULATOR_HOST=localhost:8085`
- Conecta automaticamente ao emulador (não precisa de credenciais GCP!)
- Inicia servidor HTTP em `http://localhost:8080`

**Output esperado:**
```
🚀 Servidor iniciado em http://0.0.0.0:8080
✅ Message Queue Scheduler iniciado - COM CALLBACK para Pub/Sub (8 msgs ou 180s por chat)
```

💡 **Deixe esse terminal aberto para ver os logs!**

---

### 4️⃣ Testar o Fluxo Completo

Em um **Terminal 4**:

```bash
cd /Users/williamduarte/NCMproduto/integrações/ChatGuru_Nordja_inte/chatguru-clickup-middleware

./test-local-pubsub.sh
```

**O que esse script faz:**
1. Verifica se servidor local está rodando
2. Envia payload de teste para `/webhooks/chatguru`
3. Aguarda 5 segundos
4. Verifica se mensagem foi publicada no Pub/Sub

**Output esperado:**
```
╔════════════════════════════════════════════════════════════════╗
║          TESTE LOCAL COM PUB/SUB EMULATOR + WEBHOOK           ║
╚════════════════════════════════════════════════════════════════╝

📋 Informações do Teste:
   Test ID: LOCAL-TEST-1762809000
   Local URL: http://localhost:8080
   Pub/Sub: localhost:8085

🚀 Enviando para webhook local...
✅ Webhook respondeu em 0s
   HTTP Status: 200
   Response: { "message": "Success", ... }

⏳ Aguardando 5 segundos para verificar logs...

📬 Verificando mensagens no Pub/Sub...
   ⚠️  Nenhuma mensagem na subscription ainda (aguardando batch)
```

---

## 📊 Monitorando

### Ver logs do servidor (Terminal 3)
```
📥 WEBHOOK RECEBIDO - RequestID: abc123 | ChatID: LOCAL-TEST-...
📬 Chat 'LOCAL-TEST-...@c.us': 1 mensagens na fila (aguardando análise SmartContextManager)
```

### Ver mensagens no Pub/Sub
```bash
# Configurar variáveis
export PUBSUB_EMULATOR_HOST=localhost:8085
export PUBSUB_PROJECT_ID=local-dev

# Pull mensagens da subscription
gcloud pubsub subscriptions pull chatguru-worker-sub \
  --project=local-dev \
  --limit=10 \
  --format=json
```

### Monitorar em tempo real
```bash
# Terminal 5 (opcional)
export PUBSUB_EMULATOR_HOST=localhost:8085

watch -n 2 'gcloud pubsub subscriptions pull chatguru-worker-sub \
  --project=local-dev \
  --limit=1 \
  --format=json'
```

---

## 🧪 Cenários de Teste

### Teste 1: Enviar 1 mensagem (aguarda batch)
```bash
./test-local-pubsub.sh
# Resultado: Mensagem enfileirada, aguardando 8 msgs ou 180s
```

### Teste 2: Enviar 8 mensagens (dispara batch)
```bash
for i in {1..8}; do
  curl -X POST http://localhost:8080/webhooks/chatguru \
    -H "Content-Type: application/json" \
    -d "{
      \"chat_id\": \"TEST-BATCH-$i@c.us\",
      \"celular\": \"5511999999999\",
      \"sender_name\": \"Teste $i\",
      \"texto_mensagem\": \"Mensagem $i de teste\",
      \"message_type\": \"text\",
      \"campos_personalizados\": {
        \"Info_1\": \"Nexcode\",
        \"Info_2\": \"William Duarte\"
      }
    }"
  sleep 0.5
done

# Resultado: 8ª mensagem dispara publicação no Pub/Sub
```

### Teste 3: Timeout de 180s
```bash
# Enviar 1 mensagem
./test-local-pubsub.sh

# Aguardar 3 minutos
sleep 180

# Verificar se foi publicada
gcloud pubsub subscriptions pull chatguru-worker-sub \
  --project=local-dev \
  --limit=1
```

---

## 🐛 Troubleshooting

### Problema: "Topic does not exist"

**Causa:** Tópicos não foram criados no emulador

**Solução:**
```bash
# Terminal 2
./setup-pubsub-topics.sh
```

---

### Problema: "Connection refused to localhost:8085"

**Causa:** Emulador não está rodando

**Solução:**
```bash
# Terminal 1
./start-pubsub-emulator.sh
```

---

### Problema: Mensagens não aparecem no Pub/Sub

**Causa:** Aguardando batch (8 mensagens ou 180s)

**Explicação:** O sistema agrupa mensagens por chat antes de publicar. Veja logs:
```
📬 Chat 'TEST@c.us': 1 mensagens na fila (aguardando análise SmartContextManager)
```

**Solução:** Envie mais mensagens OU aguarde timeout

---

### Problema: Servidor não inicia

**Causa:** Variáveis de ambiente não carregadas

**Solução:**
```bash
# Carregar .env.local
source .env.local

# Verificar
echo $PUBSUB_EMULATOR_HOST
# Output: localhost:8085

# Iniciar servidor
cargo run
```

---

## 📚 Comandos Úteis

### Pub/Sub Emulator

```bash
# Listar tópicos
gcloud pubsub topics list --project=local-dev

# Listar subscriptions
gcloud pubsub subscriptions list --project=local-dev

# Publicar mensagem manualmente
gcloud pubsub topics publish chatguru-webhook-events \
  --message='{"test": "manual"}' \
  --project=local-dev

# Deletar todos os tópicos (reset)
gcloud pubsub topics delete chatguru-webhook-events --project=local-dev
gcloud pubsub topics delete clickup-webhook-events --project=local-dev
```

### Servidor Local

```bash
# Build release (mais rápido)
cargo build --release
./target/release/chatguru-clickup-middleware

# Ver logs detalhados
RUST_LOG=trace cargo run

# Verificar health
curl http://localhost:8080/health
curl http://localhost:8080/ready
curl http://localhost:8080/status
```

---

## 🔄 Workflow Completo

```
┌─────────────────────────────────────────────────────────────────┐
│  1. WEBHOOK (Terminal 4)                                        │
│     curl POST /webhooks/chatguru                                │
│     ↓                                                            │
│  2. MIDDLEWARE (Terminal 3)                                     │
│     - Recebe payload                                            │
│     - Enfileira em MessageQueueService                          │
│     - SmartContextManager analisa                               │
│     - Dispara quando: 8 msgs OU 180s OU regras inteligentes    │
│     ↓                                                            │
│  3. PUB/SUB EMULATOR (Terminal 1)                               │
│     - Recebe mensagem publicada                                 │
│     - Armazena na subscription 'chatguru-worker-sub'            │
│     ↓                                                            │
│  4. WORKER (TODO - Terminal 5)                                  │
│     - Pull da subscription                                      │
│     - Processa batch agregado                                   │
│     - Classifica com OpenAI                                     │
│     - Cria task no ClickUp                                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✅ Checklist de Setup

- [ ] Terminal 1: `./start-pubsub-emulator.sh` rodando
- [ ] Terminal 2: `./setup-pubsub-topics.sh` executado
- [ ] Terminal 3: `source .env.local && cargo run` rodando
- [ ] Terminal 4: `./test-local-pubsub.sh` funcionando
- [ ] Logs aparecem no Terminal 3
- [ ] Webhook retorna HTTP 200

---

## 🎓 Diferenças: Local vs Produção

| Aspecto | Local (Emulator) | Produção (GCP) |
|---------|------------------|----------------|
| **Pub/Sub** | `localhost:8085` | `pubsub.googleapis.com` |
| **Auth** | Não precisa | ADC ou Service Account |
| **Project ID** | `local-dev` | `buzzlightear` |
| **Persistência** | `./pubsub-data/` | GCP gerencia |
| **Custo** | $0 (grátis!) | Paga por mensagem |
| **Performance** | Lento (simulado) | Produção real |

---

## 📖 Próximos Passos

1. ✅ **Setup Local** (você está aqui!)
2. 🔄 **Implementar Worker Local** (subscriber)
3. 🧪 **Testes de Integração**
4. 🚀 **Deploy para Produção**

---

## 📞 Suporte

- **Docs Pub/Sub Emulator:** https://cloud.google.com/pubsub/docs/emulator
- **Rust Pub/Sub Client:** https://github.com/yoshidan/google-cloud-rust
- **CLAUDE.md:** Instruções gerais do projeto

---

**Criado em:** 2025-11-10
**Autor:** Claude Code
**Versão:** 1.0
