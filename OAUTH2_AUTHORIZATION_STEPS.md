# 🔐 Autorização OAuth2 - Passo a Passo

**Data**: 10 de Outubro de 2025
**Status**: ⏳ Aguardando autorização

---

## ✅ Deploy Concluído

- **Build ID**: `0953f541-c9d0-4493-99bc-5add70e947db`
- **Status**: SUCCESS
- **Duration**: 12m52s
- **Image**: `gcr.io/buzzlightear/chatguru-clickup-middleware:latest`

---

## 🚀 Próximo Passo: Autorizar OAuth2

### 1. **Acesse a URL de Autorização**

Abra no navegador:

```
https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/auth/clickup
```

### 2. **Na Página do ClickUp**

- ClickUp vai pedir para autorizar a aplicação
- ⚠️ **IMPORTANTE**: Selecione **TODOS os workspaces**
- Principalmente: **Nordja (ID: 9013037641)**
- Clique em **"Authorize"**

### 3. **Na Página de Sucesso**

O sistema vai:

- ✅ Receber o authorization code
- ✅ Trocar por access token OAuth2
- ✅ Validar o token
- ✅ Consultar workspaces autorizados
- ✅ Tentar salvar no Secret Manager (pode falhar - OK!)
- ✅ **Exibir o token** na página

### 4. **Copiar e Salvar o Token**

- Clique no botão **"📋 Copiar Token"**
- Execute o comando para salvar:

```bash
echo "106092691_5a823a64061246a2fb9498f37fecb77540478eb28423d778f37aabcafbd602b9" | gcloud secrets versions add clickup-oauth-token --data-file=- --project=buzzlightear
```

---

## 🔍 Como Verificar se Deu Certo

### 1. Verificar Token Salvo

```bash
gcloud secrets versions access latest --secret=clickup-oauth-token --project=buzzlightear | head -c 50
```

**Esperado**: Token diferente do Personal Token (não começa com `106092691_`)

### 2. Testar Criação de Folder

```bash
TOKEN=$(gcloud secrets versions access latest --secret=clickup-oauth-token --project=buzzlightear)

curl -X POST "https://api.clickup.com/api/v2/space/90130085983/folder" \
  -H "Authorization: $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "TEST_OAUTH2_FOLDER"}'
```

**Esperado**:

```json
{"id": "...", "name": "TEST_OAUTH2_FOLDER", ...}
```

**Se der erro OAUTH_019**: Token ainda é Personal Token

---

## ⚠️  Troubleshooting

### Erro: "OAUTH_027 - Team not authorized"

**Causa**: Workspace não foi autorizado durante OAuth
**Solução**: Acesse `/auth/clickup` novamente e autorize TODOS os workspaces

### Erro: "OAUTH_019 - Oauth token not found"

**Causa**: Ainda está usando Personal Token
**Solução**: Copie o token da página de sucesso e salve no Secret Manager

### Erro: "Failed to save token"

**Causa**: Método `create_or_update_secret` não implementado completamente
**Solução**: Copie manualmente da página e salve via gcloud (comando acima)

---

## 📋 Checklist

- [ ] Acessar `/auth/clickup`
- [ ] Autorizar TODOS os workspaces no ClickUp
- [ ] Ver página de sucesso com lista de workspaces
- [ ] Copiar token da página
- [ ] Salvar no Secret Manager via gcloud
- [ ] Verificar token salvo (diferente do Personal)
- [ ] Testar criação de folder
- [ ] Re-habilitar Pub/Sub subscription

---

## 🎯 Após Autorização

### Reativar Processamento de Webhooks

```bash
# Voltar ack deadline para normal
gcloud pubsub subscriptions update chatguru-webhook-worker-sub --ack-deadline=60 --project=buzzlightear
```

### Testar com Webhook Real

- Enviar mensagem no ChatGuru
- Verificar logs do Cloud Run
- Confirmar task criada no ClickUp (sem erro OAUTH_027)

---

## 🔗 Links Úteis

- **URL OAuth**: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/auth/clickup
- **Logs Cloud Run**: https://console.cloud.google.com/run/detail/southamerica-east1/chatguru-clickup-middleware/logs?project=buzzlightear
- **Secret Manager**: https://console.cloud.google.com/security/secret-manager?project=buzzlightear
