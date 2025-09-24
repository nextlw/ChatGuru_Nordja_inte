# Guia de Deploy para Ambiente de Teste no Google Cloud

## 📋 Pré-requisitos

1. **Google Cloud CLI** instalado e configurado
2. **Docker** instalado e rodando
3. **Conta no GCP** com permissões adequadas
4. **Projeto GCP**: `buzzlightear`

## 🚀 Deploy Rápido (Método Recomendado)

### Opção 1: Script Automatizado

```bash
# Exportar variáveis de ambiente necessárias
export CLICKUP_API_TOKEN="pk_81165687_S6WX0Z5NF5BG3QWRS10ALHFAY9VCE379"
export CHATGURU_API_TOKEN="a6a387a5-68fe-4933-bc56-3eb614e2fa7f"
export CHATGURU_ACCOUNT_ID="96c44fa9-8d8f-426f-94e8-0e264e29b47f"

# Executar script de deploy
./deploy-test.sh
```

### Opção 2: Cloud Build (CI/CD)

```bash
# Submeter build para Cloud Build
gcloud builds submit \
  --config=cloudbuild-test.yaml \
  --project=buzzlightear \
  --region=southamerica-east1
```

## 📝 Deploy Manual Passo a Passo

### 1. Configurar gcloud

```bash
# Login
gcloud auth login

# Configurar projeto
gcloud config set project buzzlightear

# Habilitar APIs necessárias
gcloud services enable run.googleapis.com containerregistry.googleapis.com
```

### 2. Build da Imagem Docker

```bash
# Build com Dockerfile de teste
docker build -f Dockerfile.test -t chatguru-middleware-test:latest .

# Tag para GCR
docker tag chatguru-middleware-test:latest \
  gcr.io/buzzlightear/chatguru-middleware-test:latest
```

### 3. Push para Google Container Registry

```bash
# Configurar Docker para usar gcloud
gcloud auth configure-docker

# Push da imagem
docker push gcr.io/buzzlightear/chatguru-middleware-test:latest
```

### 4. Deploy no Cloud Run

```bash
gcloud run deploy chatguru-middleware-test \
  --image gcr.io/buzzlightear/chatguru-middleware-test:latest \
  --region southamerica-east1 \
  --platform managed \
  --allow-unauthenticated \
  --port 8080 \
  --cpu 1 \
  --memory 512Mi \
  --min-instances 0 \
  --max-instances 10 \
  --set-env-vars "RUST_ENV=test,IS_TEST_ENVIRONMENT=true"
```

## 🧪 Testar o Deploy

### 1. Obter URL do Serviço

```bash
# Obter URL
gcloud run services describe chatguru-middleware-test \
  --region southamerica-east1 \
  --format 'value(status.url)'
```

### 2. Testar Endpoints

```bash
# Health Check
curl https://YOUR-SERVICE-URL/health

# Status
curl https://YOUR-SERVICE-URL/status

# Teste de Webhook
./test-webhook-gcp.sh https://YOUR-SERVICE-URL
```

## 📊 Monitoramento

### Ver Logs

```bash
# Últimos 50 logs
gcloud run logs read \
  --service chatguru-middleware-test \
  --region southamerica-east1 \
  --tail 50

# Logs em tempo real
gcloud run logs tail \
  --service chatguru-middleware-test \
  --region southamerica-east1
```

### Ver Métricas

```bash
# Descrição do serviço
gcloud run services describe chatguru-middleware-test \
  --region southamerica-east1

# Métricas no Console
# Acesse: https://console.cloud.google.com/run
```

## 🔧 Gerenciamento

### Atualizar Variáveis de Ambiente

```bash
gcloud run services update chatguru-middleware-test \
  --update-env-vars KEY=VALUE \
  --region southamerica-east1
```

### Escalar o Serviço

```bash
# Aumentar instâncias mínimas (para testes de carga)
gcloud run services update chatguru-middleware-test \
  --min-instances 1 \
  --max-instances 20 \
  --region southamerica-east1
```

### Rollback

```bash
# Listar revisões
gcloud run revisions list \
  --service chatguru-middleware-test \
  --region southamerica-east1

# Voltar para revisão anterior
gcloud run services update-traffic chatguru-middleware-test \
  --to-revisions REVISION_NAME=100 \
  --region southamerica-east1
```

## 🗑️ Limpeza

### Deletar Serviço de Teste

```bash
gcloud run services delete chatguru-middleware-test \
  --region southamerica-east1 \
  --quiet
```

### Deletar Imagens do GCR

```bash
# Listar imagens
gcloud container images list --repository=gcr.io/buzzlightear

# Deletar imagem
gcloud container images delete \
  gcr.io/buzzlightear/chatguru-middleware-test:latest \
  --quiet
```

## 📋 Checklist de Validação

- [ ] Health check retorna `{"status": "healthy"}`
- [ ] Webhook aceita payloads do ChatGuru
- [ ] Logs aparecem no Cloud Logging
- [ ] Métricas aparecem no Cloud Monitoring
- [ ] Autoscaling funciona (0 a 10 instâncias)
- [ ] Latência < 500ms para webhooks
- [ ] Sem erros críticos nos logs

## 🔐 Segurança

### Service Account (Opcional)

```bash
# Criar service account
gcloud iam service-accounts create chatguru-middleware \
  --display-name="ChatGuru Middleware Service Account"

# Dar permissões necessárias
gcloud projects add-iam-policy-binding buzzlightear \
  --member="serviceAccount:chatguru-middleware@buzzlightear.iam.gserviceaccount.com" \
  --role="roles/logging.logWriter"

# Usar no deploy
gcloud run services update chatguru-middleware-test \
  --service-account chatguru-middleware@buzzlightear.iam.gserviceaccount.com \
  --region southamerica-east1
```

## 🚨 Troubleshooting

### Problema: Build falha

```bash
# Verificar logs do build
gcloud builds list --limit 5

# Ver detalhes do build
gcloud builds describe BUILD_ID
```

### Problema: Serviço não responde

```bash
# Verificar status
gcloud run services describe chatguru-middleware-test \
  --region southamerica-east1

# Verificar logs de erro
gcloud run logs read \
  --service chatguru-middleware-test \
  --region southamerica-east1 \
  --filter "severity>=ERROR"
```

### Problema: Permissões negadas

```bash
# Verificar IAM
gcloud projects get-iam-policy buzzlightear

# Adicionar permissão necessária
gcloud projects add-iam-policy-binding buzzlightear \
  --member="user:SEU_EMAIL" \
  --role="roles/run.admin"
```

## 📞 Suporte

- **Documentação Cloud Run**: https://cloud.google.com/run/docs
- **Status GCP**: https://status.cloud.google.com
- **Console**: https://console.cloud.google.com/run

## 🎯 Próximos Passos

1. **Ativar Cloud Tasks** quando estiver pronto para teste assíncrono
2. **Configurar Pub/Sub** para eventos (Fase 2)
3. **Implementar métricas customizadas** (Fase 3)
4. **Deploy em produção** após validação completa