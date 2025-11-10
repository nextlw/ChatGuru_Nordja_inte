# 🚀 Quick Start - Desenvolvimento Local

Setup rápido para rodar o middleware localmente com Pub/Sub Emulator.

## ⚡ 4 Passos Simples

### Terminal 1: Emulador
```bash
./start-pubsub-emulator.sh
```
✅ Deixe rodando

### Terminal 2: Configurar Topics (uma vez só)
```bash
./setup-pubsub-topics.sh
```
✅ Execute apenas na primeira vez

### Terminal 3: Servidor Rust
```bash
source .env.local
cargo run
```
✅ Deixe rodando

### Terminal 4: Testar
```bash
./test-local-pubsub.sh
```

---

## 📊 Monitorar Mensagens

### Ver mensagens (uma vez)
```bash
./pubsub-pull.sh
```

### Monitorar em tempo real
```bash
./monitor-pubsub.sh
```

---

## 🔧 Comandos Úteis

```bash
# Ver status do emulador
curl http://localhost:8085

# Ver health do servidor
curl http://localhost:8080/health

# Enviar webhook manual
curl -X POST http://localhost:8080/webhooks/chatguru \
  -H "Content-Type: application/json" \
  -d '{
    "chat_id": "test@c.us",
    "celular": "5511999999999",
    "sender_name": "Teste",
    "texto_mensagem": "Mensagem de teste",
    "message_type": "text",
    "campos_personalizados": {
      "Info_1": "Nexcode",
      "Info_2": "William"
    }
  }'
```

---

## ✅ Checklist

- [ ] Emulador rodando (Terminal 1)
- [ ] Topics criados (Terminal 2)
- [ ] Servidor rodando (Terminal 3)
- [ ] Teste executado (Terminal 4)

---

## 📚 Docs Completas

Ver [LOCAL_SETUP.md](LOCAL_SETUP.md) para documentação detalhada.
