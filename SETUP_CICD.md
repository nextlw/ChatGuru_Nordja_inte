# Configuração de CI/CD com Google Cloud Build

## Status Atual ✅
- Deploy manual funcionando em produção
- Imagem Docker: `gcr.io/buzzlightear/chatguru-clickup-middleware:fix-ai-v2`
- Serviço rodando: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app

## Arquivos de Configuração Criados

### 1. `chatguru-clickup-middleware/cloudbuild.yaml`
Arquivo de configuração do Cloud Build que define:
- Build da imagem Docker
- Push para Google Container Registry
- Deploy automático para Cloud Run
- Configuração de secrets (incluindo OPENAI_API_KEY)

### 2. `chatguru-clickup-middleware/trigger-config.yaml`
Configuração do trigger que define:
- Repositório: `nextlw/ChatGuru_Nordja_inte`
- Branch: `main`
- Arquivos monitorados: `chatguru-clickup-middleware/**`

## Como Conectar o Repositório ao GCP (Manual)

### Opção 1: Via Console do Google Cloud

1. Acesse: https://console.cloud.google.com/cloud-build/triggers?project=buzzlightear

2. Clique em **"Conectar repositório"** ou **"Connect repository"**

3. Selecione **GitHub** como fonte

4. Autorize o Google Cloud Build a acessar sua conta GitHub

5. Selecione:
   - Organização/Owner: `nextlw`
   - Repositório: `ChatGuru_Nordja_inte`

6. Clique em **"Criar trigger"** ou **"Create trigger"**

7. Configure o trigger:
   - Nome: `chatguru-clickup-middleware-trigger`
   - Evento: Push para branch
   - Branch: `^main$` (regex)
   - Arquivo de configuração: `chatguru-clickup-middleware/cloudbuild.yaml`
   - Arquivos incluídos: `chatguru-clickup-middleware/**`

8. Salve o trigger

### Opção 2: Via Cloud Shell

```bash
# No Cloud Shell ou terminal com gcloud configurado:

# 1. Instale o componente beta
gcloud components install beta --quiet

# 2. Configure a conexão com GitHub (requer interação no browser)
gcloud beta builds connections create github chatguru-github-connection \
  --region=southamerica-east1 \
  --project=buzzlightear

# 3. Crie o repositório linkado
gcloud beta builds repositories create chatguru-nordja-repo \
  --remote-uri=https://github.com/nextlw/ChatGuru_Nordja_inte.git \
  --connection=chatguru-github-connection \
  --region=southamerica-east1 \
  --project=buzzlightear

# 4. Crie o trigger
gcloud beta builds triggers create github \
  --name=chatguru-clickup-middleware-trigger \
  --repository=projects/buzzlightear/locations/southamerica-east1/connections/chatguru-github-connection/repositories/chatguru-nordja-repo \
  --branch-pattern="^main$" \
  --build-config=chatguru-clickup-middleware/cloudbuild.yaml \
  --region=southamerica-east1 \
  --project=buzzlightear
```

## Deploy Manual (Caso Necessário)

Se precisar fazer deploy manual enquanto o CI/CD não está configurado:

```bash
# 1. Build local
cd chatguru-clickup-middleware
docker build -t gcr.io/buzzlightear/chatguru-clickup-middleware:latest .

# 2. Push para GCR
docker push gcr.io/buzzlightear/chatguru-clickup-middleware:latest

# 3. Deploy para Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --project buzzlightear
```

## Verificação

Após configurar o trigger:

1. Faça um commit de teste:
```bash
echo "# Test CI/CD" >> README.md
git add README.md
git commit -m "test: Trigger CI/CD pipeline"
git push origin main
```

2. Verifique o build:
```bash
# Lista builds recentes
gcloud builds list --limit=5 --project=buzzlightear

# Verifica logs do último build
gcloud builds log $(gcloud builds list --limit=1 --format="value(id)") --project=buzzlightear
```

3. Verifique o deploy:
```bash
# Verifica status do serviço
gcloud run services describe chatguru-clickup-middleware \
  --region=southamerica-east1 \
  --project=buzzlightear \
  --format="value(status.latestReadyRevisionName)"
```

## Secrets Necessários

Certifique-se de que os seguintes secrets existem no Secret Manager:

- `clickup-api-token`
- `clickup-list-id`
- `gcp-project-id`
- `OPENAI_API_KEY` ✅ (já configurado)

Para criar um secret:
```bash
echo -n "YOUR_SECRET_VALUE" | gcloud secrets create SECRET_NAME --data-file=- --project=buzzlightear
```

## Monitoramento

- Cloud Build Dashboard: https://console.cloud.google.com/cloud-build/builds?project=buzzlightear
- Cloud Run Service: https://console.cloud.google.com/run/detail/southamerica-east1/chatguru-clickup-middleware?project=buzzlightear
- Logs: https://console.cloud.google.com/logs?project=buzzlightear

## Suporte

Em caso de problemas:
1. Verifique os logs do Cloud Build
2. Verifique os logs do Cloud Run
3. Certifique-se de que todos os secrets estão configurados
4. Verifique as permissões da service account