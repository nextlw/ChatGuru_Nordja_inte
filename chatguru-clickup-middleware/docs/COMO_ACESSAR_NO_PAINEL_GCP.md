# 📍 ONDE ENCONTRAR O DEPLOYMENT NO GOOGLE CLOUD CONSOLE

## 🌐 Acesso ao Console

### 1. Acesse o Google Cloud Console
```
https://console.cloud.google.com
```

### 2. Faça login com suas credenciais Google

### 3. Selecione o projeto correto
- No topo da página, clique no seletor de projetos
- Procure e selecione: **`buzzlightear`**

## 📂 Onde Encontrar Cada Componente

### 🚀 CLOUD RUN (Onde está o middleware deployado)

**Caminho no menu:**
```
☰ Menu → Serverless → Cloud Run
```

**URL direta:**
```
https://console.cloud.google.com/run?project=buzzlightear
```

**O que você verá:**
- Lista de serviços deployados
- Procure por: **`chatguru-clickup-middleware`**
- Status: ✅ (verde se estiver rodando)
- Clique no nome para ver detalhes

**Informações disponíveis:**
- URL do serviço (endpoint público)
- Métricas de uso
- Logs
- Revisões deployadas
- Configurações de ambiente

---

### 🔐 SECRET MANAGER (Credenciais)

**Caminho no menu:**
```
☰ Menu → Security → Secret Manager
```

**URL direta:**
```
https://console.cloud.google.com/security/secret-manager?project=buzzlightear
```

**Secrets criados:**
- `clickup-api-token`
- `clickup-list-id`
- `gcp-project-id`

---

### 📦 ARTIFACT REGISTRY (Imagens Docker)

**Caminho no menu:**
```
☰ Menu → CI/CD → Artifact Registry
```

**URL direta:**
```
https://console.cloud.google.com/artifacts?project=buzzlightear
```

**Repositório:**
- Nome: `chatguru-integrations`
- Tipo: Docker
- Região: southamerica-east1

---

### 📊 CLOUD PUB/SUB (Mensageria)

**Caminho no menu:**
```
☰ Menu → Big Data → Pub/Sub → Topics
```

**URL direta:**
```
https://console.cloud.google.com/cloudpubsub/topic/list?project=buzzlightear
```

**Tópicos criados:**
- `chatguru-events`
- Subscription: `chatguru-events-subscription`

---

### 🔨 CLOUD BUILD (CI/CD)

**Caminho no menu:**
```
☰ Menu → CI/CD → Cloud Build → History
```

**URL direta:**
```
https://console.cloud.google.com/cloud-build/builds?project=buzzlightear
```

**O que você verá:**
- Histórico de builds
- Status de cada deployment
- Logs detalhados

---

### 📈 MONITORING (Logs e Métricas)

**Caminho no menu:**
```
☰ Menu → Operations → Logging → Logs Explorer
```

**URL direta para logs do serviço:**
```
https://console.cloud.google.com/logs/query;query=resource.type%3D%22cloud_run_revision%22%20resource.labels.service_name%3D%22chatguru-clickup-middleware%22?project=buzzlightear
```

---

## 🎯 Painel Principal do Serviço

### Acesso Rápido ao Serviço Deployado:

1. **Método 1 - Direto:**
   ```
   https://console.cloud.google.com/run/detail/southamerica-east1/chatguru-clickup-middleware/metrics?project=buzzlightear
   ```

2. **Método 2 - Navegação:**
   - Vá para Cloud Run
   - Região: `southamerica-east1`
   - Clique em `chatguru-clickup-middleware`

### O que você encontrará na página do serviço:

#### 📊 Aba METRICS
- Requests por segundo
- Latência
- Uso de CPU e memória
- Erros

#### 📝 Aba LOGS
- Logs em tempo real
- Filtros por severidade
- Busca por texto

#### ⚙️ Aba DETAILS
- URL do serviço (copie para usar no ChatGuru)
- Variáveis de ambiente
- Configurações de recursos
- Região e zone

#### 🔄 Aba REVISIONS
- Histórico de deployments
- Rollback para versões anteriores
- Traffic splitting

#### 🔧 Aba YAML
- Configuração completa em YAML
- Pode editar e redeploy direto daqui

---

## 💡 Dicas Úteis no Console

### Para ver a URL do serviço rapidamente:
1. Cloud Run → `chatguru-clickup-middleware`
2. No topo da página, copie a URL ao lado de "URL:"
3. Formato: `https://chatguru-clickup-middleware-xxxxx-rj.a.run.app`

### Para ver os logs em tempo real:
1. Na página do serviço, clique em "LOGS"
2. Ou use o comando:
   ```bash
   gcloud logs tail --service=chatguru-clickup-middleware
   ```

### Para testar o serviço:
1. Copie a URL do serviço
2. Adicione `/health` no final
3. Acesse no navegador ou curl:
   ```bash
   curl https://[SUA-URL-DO-SERVICO]/health
   ```

### Para fazer redeploy:
1. Na página do serviço
2. Clique em "EDIT & DEPLOY NEW REVISION"
3. Faça as alterações necessárias
4. Clique em "DEPLOY"

---

## 🔍 Verificação Rápida

### Status do Deployment no Console:

1. **Serviço Rodando:**
   - Cloud Run → Status: ✅ (check verde)
   
2. **URL Funcionando:**
   - Details → URL → Clique para abrir
   
3. **Logs Ativos:**
   - Logs → Deve mostrar atividade
   
4. **Métricas:**
   - Metrics → Gráficos devem aparecer após primeiras requisições

---

## 📱 App Mobile do Google Cloud

Você também pode monitorar pelo app:
- **iOS**: https://apps.apple.com/app/google-cloud-console/id1005120814
- **Android**: https://play.google.com/store/apps/details?id=com.google.android.apps.cloudconsole

---

## 🆘 Não está aparecendo?

Se o serviço não aparecer no console:

1. **Verifique o projeto:**
   - Confirme que está em `buzzlightear`
   
2. **Verifique a região:**
   - Filtro de região: `southamerica-east1`
   
3. **Execute o comando de verificação:**
   ```bash
   gcloud run services list --project=buzzlightear
   ```

4. **Se não foi deployado ainda:**
   ```bash
   cd chatguru-clickup-middleware
   ./quick-deploy.sh
   # Escolha opção 3
   ```

---

## 📌 Links Úteis

- **Console Principal**: https://console.cloud.google.com
- **Cloud Run**: https://console.cloud.google.com/run
- **Documentação**: https://cloud.google.com/run/docs
- **Pricing Calculator**: https://cloud.google.com/products/calculator

---

**Última atualização**: 22 de Janeiro de 2025