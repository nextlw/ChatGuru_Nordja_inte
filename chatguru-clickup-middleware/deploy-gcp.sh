#!/bin/bash

# ==============================================================================
# Script de Deploy do ChatGuru-ClickUp Middleware para Google Cloud
# ==============================================================================

set -e  # Exit on error

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configurações do projeto
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-clickup-middleware"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

# Função para imprimir mensagens coloridas
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Header do script
echo "=============================================="
echo "   ChatGuru-ClickUp Middleware Deployment    "
echo "=============================================="
echo ""

# 1. Verificar se gcloud está instalado
print_status "Verificando instalação do gcloud CLI..."
if ! command -v gcloud &> /dev/null; then
    print_error "gcloud CLI não está instalado. Por favor, instale primeiro."
    echo "Visite: https://cloud.google.com/sdk/docs/install"
    exit 1
fi
print_success "gcloud CLI encontrado!"

# 2. Verificar autenticação
print_status "Verificando autenticação do Google Cloud..."
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    print_warning "Não há conta ativa. Iniciando processo de autenticação..."
    gcloud auth login
fi
ACTIVE_ACCOUNT=$(gcloud auth list --filter=status:ACTIVE --format="value(account)")
print_success "Autenticado como: $ACTIVE_ACCOUNT"

# 3. Configurar projeto
print_status "Configurando projeto Google Cloud..."
gcloud config set project ${PROJECT_ID}
print_success "Projeto configurado: ${PROJECT_ID}"

# 4. Habilitar APIs necessárias
print_status "Habilitando APIs necessárias..."
gcloud services enable \
    cloudbuild.googleapis.com \
    run.googleapis.com \
    containerregistry.googleapis.com \
    pubsub.googleapis.com \
    secretmanager.googleapis.com \
    --quiet

print_success "APIs habilitadas com sucesso!"

# 5. Criar secrets no Secret Manager
print_status "Configurando secrets no Secret Manager..."

# Função para criar ou atualizar secret
create_or_update_secret() {
    local SECRET_NAME=$1
    local SECRET_VALUE=$2
    
    if gcloud secrets describe ${SECRET_NAME} --project=${PROJECT_ID} &> /dev/null; then
        print_warning "Secret ${SECRET_NAME} já existe. Atualizando..."
        echo -n "${SECRET_VALUE}" | gcloud secrets versions add ${SECRET_NAME} --data-file=- --project=${PROJECT_ID}
    else
        print_status "Criando secret ${SECRET_NAME}..."
        echo -n "${SECRET_VALUE}" | gcloud secrets create ${SECRET_NAME} --data-file=- --replication-policy="automatic" --project=${PROJECT_ID}
    fi
}

# Solicitar valores dos secrets
echo ""
print_status "Configuração de Secrets (valores sensíveis)"
echo "=============================================="

# ClickUp API Token
if [ -z "${CLICKUP_API_TOKEN}" ]; then
    read -p "Digite o ClickUp API Token (pk_...): " CLICKUP_API_TOKEN
fi
create_or_update_secret "clickup-api-token" "${CLICKUP_API_TOKEN}"

# ClickUp List ID
if [ -z "${CLICKUP_LIST_ID}" ]; then
    CLICKUP_LIST_ID="901300373349"  # Valor padrão do documento
    read -p "Digite o ClickUp List ID [${CLICKUP_LIST_ID}]: " input
    CLICKUP_LIST_ID="${input:-$CLICKUP_LIST_ID}"
fi
create_or_update_secret "clickup-list-id" "${CLICKUP_LIST_ID}"

# GCP Project ID
create_or_update_secret "gcp-project-id" "${PROJECT_ID}"

print_success "Secrets configurados com sucesso!"

# 6. Criar tópicos Pub/Sub
print_status "Configurando Google Pub/Sub..."

# Criar tópico se não existir
if ! gcloud pubsub topics describe chatguru-events --project=${PROJECT_ID} &> /dev/null; then
    print_status "Criando tópico chatguru-events..."
    gcloud pubsub topics create chatguru-events --project=${PROJECT_ID}
else
    print_warning "Tópico chatguru-events já existe"
fi

# Criar subscription se não existir
if ! gcloud pubsub subscriptions describe chatguru-events-subscription --project=${PROJECT_ID} &> /dev/null; then
    print_status "Criando subscription chatguru-events-subscription..."
    gcloud pubsub subscriptions create chatguru-events-subscription \
        --topic=chatguru-events \
        --ack-deadline=60 \
        --project=${PROJECT_ID}
else
    print_warning "Subscription chatguru-events-subscription já existe"
fi

print_success "Pub/Sub configurado com sucesso!"

# 7. Build da imagem Docker
print_status "Construindo imagem Docker..."
docker build -t ${IMAGE_NAME}:latest -f Dockerfile .
print_success "Imagem Docker construída com sucesso!"

# 8. Push da imagem para Container Registry
print_status "Configurando Docker para usar gcloud..."
gcloud auth configure-docker --quiet

print_status "Enviando imagem para Container Registry..."
docker push ${IMAGE_NAME}:latest
print_success "Imagem enviada com sucesso!"

# 9. Deploy no Cloud Run
print_status "Fazendo deploy no Cloud Run..."

gcloud run deploy ${SERVICE_NAME} \
    --image ${IMAGE_NAME}:latest \
    --region ${REGION} \
    --platform managed \
    --allow-unauthenticated \
    --memory 512Mi \
    --cpu 1 \
    --timeout 300 \
    --concurrency 100 \
    --min-instances 0 \
    --max-instances 10 \
    --port 8080 \
    --set-env-vars "RUST_LOG=info,RUN_MODE=production,PORT=8080" \
    --set-secrets "CLICKUP_API_TOKEN=clickup-api-token:latest,CLICKUP_LIST_ID=clickup-list-id:latest,GCP_PROJECT_ID=gcp-project-id:latest" \
    --project ${PROJECT_ID} \
    --quiet

# 10. Obter URL do serviço
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format 'value(status.url)' --project ${PROJECT_ID})

# 11. Testar o serviço
print_status "Testando o serviço deployado..."
echo ""

# Health check
print_status "Executando health check..."
HEALTH_RESPONSE=$(curl -s "${SERVICE_URL}/health" || echo "ERRO")
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    print_success "Health check passou!"
    echo "Response: $HEALTH_RESPONSE"
else
    print_error "Health check falhou!"
    echo "Response: $HEALTH_RESPONSE"
fi

echo ""
# Status check
print_status "Verificando status das integrações..."
STATUS_RESPONSE=$(curl -s "${SERVICE_URL}/status" || echo "ERRO")
echo "Status response: $STATUS_RESPONSE"

# Resumo final
echo ""
echo "=============================================="
echo "         DEPLOYMENT CONCLUÍDO!                "
echo "=============================================="
echo ""
print_success "Serviço deployado com sucesso!"
echo ""
echo "📌 Informações do serviço:"
echo "   URL: ${SERVICE_URL}"
echo "   Região: ${REGION}"
echo "   Projeto: ${PROJECT_ID}"
echo ""
echo "📊 Endpoints disponíveis:"
echo "   - ${SERVICE_URL}/health (GET) - Health check"
echo "   - ${SERVICE_URL}/status (GET) - Status das integrações"
echo "   - ${SERVICE_URL}/webhooks/chatguru (POST) - Webhook ChatGuru"
echo "   - ${SERVICE_URL}/clickup/tasks (GET) - Listar tarefas"
echo "   - ${SERVICE_URL}/clickup/list (GET) - Info da lista"
echo "   - ${SERVICE_URL}/clickup/test (GET) - Testar conexão"
echo ""
echo "🔧 Para configurar o webhook no ChatGuru, use:"
echo "   URL: ${SERVICE_URL}/webhooks/chatguru"
echo "   Método: POST"
echo ""
echo "📝 Logs do serviço:"
echo "   gcloud logs read --service=${SERVICE_NAME} --project=${PROJECT_ID}"
echo ""
echo "🔄 Para fazer redeploy, execute este script novamente!"
echo ""