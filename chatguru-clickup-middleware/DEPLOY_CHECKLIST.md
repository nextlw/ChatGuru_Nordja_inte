# ✅ Checklist de Deploy - Ambiente de Teste GCP

## 📋 Status das Credenciais e Configurações

### ✅ Arquivos Atualizados:

1. **`.env.test`**
   - ✅ PROJECT_ID: buzzlightear
   - ✅ PROJECT_NUMBER: 707444002434
   - ✅ REGION: southamerica-east1
   - ✅ Service URL atualizada
   - ✅ Credenciais ClickUp e ChatGuru configuradas

2. **`deploy-test.sh`**
   - ✅ Service Account: buzzlightear@appspot.gserviceaccount.com
   - ✅ Região: southamerica-east1
   - ✅ Configurações de recursos (CPU: 1, RAM: 512Mi)

3. **`cloudbuild-test.yaml`**
   - ✅ Service Account corrigido
   - ✅ Steps de build configurados
   - ✅ Variáveis de substituição definidas

4. **`deploy-test-local.sh`** (NOVO!)
   - ✅ Script simplificado criado
   - ✅ Usa Artifact Registry (já existe: chatguru-integrations)
   - ✅ Configurações otimizadas para teste local

## 🔐 Credenciais Verificadas:

### Google Cloud:
- **Projeto ID**: buzzlightear
- **Projeto Número**: 707444002434
- **Service Account**: buzzlightear@appspot.gserviceaccount.com (existente)
- **Região**: southamerica-east1
- **Artifact Registry**: chatguru-integrations (existente)

### APIs Externas:
- **ClickUp Token**: pk_81165687_S6WX0Z5NF5BG3QWRS10ALHFAY9VCE379
- **ClickUp List**: 901300373349
- **ChatGuru Token**: a6a387a5-68fe-4933-bc56-3eb614e2fa7f
- **ChatGuru Account**: 96c44fa9-8d8f-426f-94e8-0e264e29b47f

## 🚀 Como Fazer o Deploy:

### Opção 1: Script Simplificado (RECOMENDADO)
```bash
# Usar o novo script simplificado
./deploy-test-local.sh
```

### Opção 2: Script Completo
```bash
# Exportar credenciais
export CLICKUP_API_TOKEN="pk_81165687_S6WX0Z5NF5BG3QWRS10ALHFAY9VCE379"
export CHATGURU_API_TOKEN="a6a387a5-68fe-4933-bc56-3eb614e2fa7f"
export CHATGURU_ACCOUNT_ID="96c44fa9-8d8f-426f-94e8-0e264e29b47f"

# Executar deploy
./deploy-test.sh
```

### Opção 3: Cloud Build
```bash
gcloud builds submit --config=cloudbuild-test.yaml --project=buzzlightear
```

## 📊 Recursos do Ambiente de Teste:

| Recurso | Valor |
|---------|-------|
| CPU | 1 vCPU |
| Memória | 512 Mi |
| Min Instâncias | 0 |
| Max Instâncias | 5 |
| Timeout | 60s |
| Concorrência | 80 |
| Autenticação | Pública (allow-unauthenticated) |

## 🔍 Verificação Pós-Deploy:

```bash
# 1. Verificar se o serviço está rodando
gcloud run services list --region=southamerica-east1

# 2. Obter URL do serviço
gcloud run services describe chatguru-middleware-test \
  --region=southamerica-east1 \
  --format='value(status.url)'

# 3. Testar health check
curl https://YOUR-SERVICE-URL/health

# 4. Ver logs
gcloud run logs tail --service=chatguru-middleware-test \
  --region=southamerica-east1
```

## ⚠️ Notas Importantes:

1. **Service Account**: Usando o padrão do App Engine (já existe)
2. **Artifact Registry**: Repositório `chatguru-integrations` já existe
3. **APIs**: Todas necessárias estão habilitadas
4. **Cloud Tasks**: Desabilitado inicialmente (USE_CLOUD_TASKS=false)
5. **Logs**: Configurado para nível INFO em teste

## 🎯 Status Final:

✅ **TUDO PRONTO PARA DEPLOY!**

Todas as credenciais foram verificadas e atualizadas. O ambiente GCP está configurado e pronto para receber o deploy de teste.