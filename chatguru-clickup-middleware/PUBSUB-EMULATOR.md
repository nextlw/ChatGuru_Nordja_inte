# Pub/Sub Emulator - Guia de Uso

## ⚠️ IMPORTANTE

O Pub/Sub Emulator **NÃO suporta comandos `gcloud pubsub`**!

Segundo a [documentação oficial](https://cloud.google.com/pubsub/docs/emulator):
> The emulator does not support Google Cloud console or gcloud pubsub commands

Para interagir com o emulator, você deve usar:
- ✅ Cloud Client Libraries (Python, Java, Go, etc.)
- ✅ API REST (com curl)
- ❌ Comandos `gcloud pubsub` (NÃO funcionam)

## 🚀 Setup Rápido

### Modo 1: Desenvolvimento com Emulator (RECOMENDADO)

Este modo simula a arquitetura completa em produção localmente:

**Terminal 1 - Iniciar o Emulator**
```bash
./start-pubsub-emulator.sh
```
Deixe rodando.

**Terminal 2 - Criar Topics e Subscriptions**
```bash
./setup-pubsub-rest.sh
```

**Terminal 3 - Rodar a aplicação conectada ao emulator**
```bash
./run-dev-with-emulator.sh
```

**Terminal 4 - Enviar mensagens de teste**
```bash
./test-local.sh
```

**Terminal 5 (opcional) - Monitorar mensagens no Pub/Sub**
```bash
./monitor-pubsub-live.sh
```

### Modo 2: Desenvolvimento sem Emulator (direto)

Se você NÃO quer usar o emulator e quer que o webhook chame o worker diretamente:

```bash
# SEM setar PUBSUB_EMULATOR_HOST ou FORCE_PUBSUB
cargo run
```

### 3. Testar a Configuração (apenas emulator)

**Publicar mensagem de teste:**
```bash
./pubsub-publish-test.sh
```

**Verificar mensagens:**
```bash
./pubsub-pull-direct.sh
```

**Monitorar em tempo real:**
```bash
./monitor-pubsub-live.sh
```

## 📝 Scripts Disponíveis

### Gerenciamento do Emulator

| Script | Descrição |
|--------|-----------|
| `start-pubsub-emulator.sh` | Inicia o emulator na porta 8085 |
| `setup-pubsub-rest.sh` | Cria topics e subscriptions via API REST |

### Testes e Monitoramento

| Script | Descrição |
|--------|-----------|
| `pubsub-publish-test.sh` | Publica mensagem de teste |
| `pubsub-pull-direct.sh` | Faz pull de mensagens via API REST (uma vez) |
| `monitor-pubsub-live.sh` | Monitora mensagens em tempo real (atualiza a cada 2s) |

### Scripts Legados (❌ Deletados - não funcionavam)

Os seguintes scripts foram removidos pois usavam `gcloud pubsub` (não compatível com emulator):
- `setup-pubsub-topics.sh` - Substituído por `setup-pubsub-rest.sh`
- `pubsub-pull.sh` - Substituído por `pubsub-pull-direct.sh`
- `monitor-pubsub.sh` - Substituído por `monitor-pubsub-live.sh`

## 🔧 Configuração do Emulator

### Recursos Criados

**Topics:**
- `chatguru-webhook-events` - Eventos do webhook do ChatGuru
- `clickup-webhook-events` - Eventos do webhook do ClickUp

**Subscriptions:**
- `chatguru-worker-sub` - Consumer que processa eventos do ChatGuru
  - Topic: `chatguru-webhook-events`
  - Ack Deadline: 600 segundos

### Variáveis de Ambiente

Para que sua aplicação use o emulator:

```bash
export PUBSUB_EMULATOR_HOST=localhost:8085
export PUBSUB_PROJECT_ID=local-dev
```

No Rust (já configurado em `main.rs`):
```rust
// Detecta automaticamente PUBSUB_EMULATOR_HOST
let config = ClientConfig::default().with_auth().await?;
let client = Client::new(config).await?;
```

### Como o Código Detecta o Emulator

O código em [src/main.rs](src/main.rs) tem lógica inteligente:

```rust
// Linhas 589-593
let force_pubsub = std::env::var("FORCE_PUBSUB").unwrap_or_default() == "true"
    || std::env::var("PUBSUB_EMULATOR_HOST").is_ok();

if (cfg!(debug_assertions) || std::env::var("RUST_ENV") == "development") && !force_pubsub {
    // Chama worker diretamente (SEM Pub/Sub)
} else {
    // Usa Pub/Sub (emulator ou produção)
}
```

**Comportamento:**
- **Sem variáveis**: Webhook → Worker direto (sem Pub/Sub)
- **Com `PUBSUB_EMULATOR_HOST`**: Webhook → Pub/Sub Emulator → Worker
- **Com `FORCE_PUBSUB=true`**: Webhook → Pub/Sub (emulator ou produção) → Worker
- **Produção**: Sempre usa Pub/Sub (GCP)

## 🔗 API REST do Emulator

Base URL: `http://localhost:8085/v1`

### Criar Topic
```bash
curl -X PUT "http://localhost:8085/v1/projects/local-dev/topics/my-topic" \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Criar Subscription
```bash
curl -X PUT "http://localhost:8085/v1/projects/local-dev/subscriptions/my-sub" \
  -H "Content-Type: application/json" \
  -d '{
    "topic": "projects/local-dev/topics/my-topic",
    "ackDeadlineSeconds": 600
  }'
```

### Publicar Mensagem
```bash
MESSAGE_BASE64=$(echo -n "Hello World" | base64)
curl -X POST "http://localhost:8085/v1/projects/local-dev/topics/my-topic:publish" \
  -H "Content-Type: application/json" \
  -d "{
    \"messages\": [{
      \"data\": \"${MESSAGE_BASE64}\"
    }]
  }"
```

### Pull Mensagens
```bash
curl -X POST "http://localhost:8085/v1/projects/local-dev/subscriptions/my-sub:pull" \
  -H "Content-Type: application/json" \
  -d '{"maxMessages": 10}'
```

### ACK Mensagens
```bash
curl -X POST "http://localhost:8085/v1/projects/local-dev/subscriptions/my-sub:acknowledge" \
  -H "Content-Type: application/json" \
  -d '{"ackIds": ["ACK_ID_AQUI"]}'
```

## 🐛 Troubleshooting

### Erro: "Topic not found" ou "Subscription does not exist"

**Causa:** O emulator foi reiniciado e perdeu os recursos criados (não persiste entre reinícios).

**Solução:**
```bash
./setup-pubsub-rest.sh
```

### Erro: "NOT_FOUND: Resource not found... authenticated as email@gmail.com"

**Causa:** Está tentando usar `gcloud pubsub` que não funciona com o emulator.

**Solução:** Use os scripts corretos:
- ✅ `pubsub-publish-test.sh` (em vez de `gcloud pubsub topics publish`)
- ✅ `pubsub-pull-direct.sh` (em vez de `gcloud pubsub subscriptions pull`)
- ✅ `monitor-pubsub-live.sh` (para monitoramento em tempo real)

### Emulator não está respondendo

**Verificar se está rodando:**
```bash
curl -s "http://localhost:8085" > /dev/null && echo "✅ OK" || echo "❌ Não está rodando"
```

**Reiniciar:**
```bash
# Ctrl+C no terminal do emulator
./start-pubsub-emulator.sh
./setup-pubsub-rest.sh  # Recriar topics/subscriptions
```

## 📚 Referências

- [Pub/Sub Emulator Documentation](https://cloud.google.com/pubsub/docs/emulator)
- [Pub/Sub REST API Reference](https://cloud.google.com/pubsub/docs/reference/rest)
- [Using the Emulator with Client Libraries](https://cloud.google.com/pubsub/docs/samples/pubsub-use-emulator)

## ⚡ Exemplo Completo

```bash
# Terminal 1 - Iniciar emulator
./start-pubsub-emulator.sh

# Terminal 2 - Setup e testes
./setup-pubsub-rest.sh                    # Criar recursos
./pubsub-publish-test.sh "Test message"   # Publicar mensagem
./pubsub-pull-direct.sh                   # Ver mensagens (uma vez)

# Terminal 3 - Monitorar em tempo real
./monitor-pubsub-live.sh                  # Monitoramento contínuo
```
