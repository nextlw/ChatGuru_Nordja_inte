#!/bin/bash

# ==============================================================================
# Script de Deploy usando Google Artifact Registry (Recomendado)
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
REPOSITORY_NAME="chatguru-integrations"
ARTIFACT_REGISTRY_LOCATION="southamerica-east1"
IMAGE_NAME="${ARTIFACT_REGISTRY_LOCATION}-docker.pkg.dev/${PROJECT_ID}/${REPOSITORY_NAME}/${SERVICE_NAME}"

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
echo "   (Using Artifact Registry - Recommended)   "
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
    artifactregistry.googleapis.com \
    cloudbuild.googleapis.com \
    run.googleapis.com \
    pubsub.googleapis.com \
    secretmanager.googleapis.com \
    --quiet

print_success "APIs habilitadas com sucesso!"

# 5. Criar repositório no Artifact Registry
print_status "Configurando Artifact Registry..."

if ! gcloud artifacts repositories describe ${REPOSITORY_NAME} \
    --location=${ARTIFACT_REGISTRY_LOCATION} \
    --project=${PROJECT_ID} &> /dev/null; then
    print_status "Criando repositório ${REPOSITORY_NAME}..."
    gcloud artifacts repositories create ${REPOSITORY_NAME} \
        --repository-format=docker \
        --location=${ARTIFACT_REGISTRY_LOCATION} \
        --description="Repository for ChatGuru integrations" \
        --project=${PROJECT_ID}
else
    print_warning "Repositório ${REPOSITORY_NAME} já existe"
fi

# Configurar Docker para usar Artifact Registry
print_status "Configurando Docker authentication..."
gcloud auth configure-docker ${ARTIFACT_REGISTRY_LOCATION}-docker.pkg.dev --quiet

print_success "Artifact Registry configurado!"

# 6. Criar secrets no Secret Manager
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
        
        # Dar permissão ao Cloud Run para acessar o secret
        gcloud secrets add-iam-policy-binding ${SECRET_NAME} \
            --member="serviceAccount:${PROJECT_ID}-compute@developer.gserviceaccount.com" \
            --role="roles/secretmanager.secretAccessor" \
            --project=${PROJECT_ID} &> /dev/null
    fi
}

# Solicitar valores dos secrets
echo ""
print_status "Configuração de Secrets (valores sensíveis)"
echo "=============================================="

# ClickUp API Token
CLICKUP_API_TOKEN="${CLICKUP_API_TOKEN:-pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657}"
read -p "Digite o ClickUp API Token [${CLICKUP_API_TOKEN}]: " input
CLICKUP_API_TOKEN="${input:-$CLICKUP_API_TOKEN}"
create_or_update_secret "clickup-api-token" "${CLICKUP_API_TOKEN}"

# ClickUp List ID
CLICKUP_LIST_ID="${CLICKUP_LIST_ID:-901300373349}"
read -p "Digite o ClickUp List ID [${CLICKUP_LIST_ID}]: " input
CLICKUP_LIST_ID="${input:-$CLICKUP_LIST_ID}"
create_or_update_secret "clickup-list-id" "${CLICKUP_LIST_ID}"

# GCP Project ID
create_or_update_secret "gcp-project-id" "${PROJECT_ID}"

print_success "Secrets configurados com sucesso!"

# 7. Criar tópicos Pub/Sub
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

# 8. Build da imagem Docker
print_status "Construindo imagem Docker..."

# Verificar se o Dockerfile existe
if [ ! -f "Dockerfile" ]; then
    print_error "Dockerfile não encontrado no diretório atual!"
    exit 1
fi

# Build com tag específica e latest
docker build -t ${IMAGE_NAME}:latest -t ${IMAGE_NAME}:$(date +%Y%m%d-%H%M%S) -f Dockerfile .
print_success "Imagem Docker construída com sucesso!"

# 9. Push da imagem para Artifact Registry
print_status "Enviando imagem para Artifact Registry..."
docker push --all-tags ${IMAGE_NAME}
print_success "Imagem enviada com sucesso!"

# 10. Deploy no Cloud Run
print_status "Fazendo deploy no Cloud Run..."

# Obter account de serviço do Cloud Run
SERVICE_ACCOUNT="${PROJECT_ID}-compute@developer.gserviceaccount.com"

gcloud run deploy ${SERVICE_NAME} \
    --image ${IMAGE_NAME}:latest \
    --region ${REGION} \
    --platform managed \
    --allow-unauthenticated \
    --service-account ${SERVICE_ACCOUNT} \
    --memory 512Mi \
    --cpu 1 \
    --timeout 300 \
    --concurrency 100 \
    --min-instances 0 \
    --max-instances 10 \
    --set-secrets="CLICKUP_API_TOKEN=clickup-api-token:latest,CLICKUP_LIST_ID=clickup-list-id:latest,GCP_PROJECT_ID=gcp-project-id:latest" \
    --set-env-vars="RUST_LOG=info,PORT=8080" \
    --labels="app=chatguru-clickup,env=production" \
    --quiet

# 11. Obter URL do serviço
SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} --region ${REGION} --format 'value(status.url)')

# 12. Configurar permissões IAM
print_status "Configurando permissões IAM..."

# Dar permissão ao service account para Pub/Sub
gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${SERVICE_ACCOUNT}" \
    --role="roles/pubsub.subscriber" &> /dev/null

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${SERVICE_ACCOUNT}" \
    --role="roles/pubsub.publisher" &> /dev/null

print_success "Permissões IAM configuradas!"

# 13. Resumo final
echo ""
echo "=============================================="
echo "           DEPLOY CONCLUÍDO!                 "
echo "=============================================="
echo ""
print_success "Serviço deployado com sucesso!"
echo ""
echo "📌 Detalhes do Deploy:"
echo "   - Projeto: ${PROJECT_ID}"
echo "   - Serviço: ${SERVICE_NAME}"
echo "   - Região: ${REGION}"
echo "   - URL: ${SERVICE_URL}"
echo ""
echo "📝 Próximos Passos:"
echo "   1. Configure o webhook no ChatGuru com a URL:"
echo "      ${SERVICE_URL}/webhooks/chatguru"
echo ""
echo "   2. Teste os endpoints:"
echo "      - Health: ${SERVICE_URL}/health"
echo "      - Ready: ${SERVICE_URL}/ready"
echo "      - Status: ${SERVICE_URL}/status"
echo ""
echo "   3. Monitor logs:"
echo "      gcloud logging read \"resource.type=cloud_run_revision AND resource.labels.service_name=${SERVICE_NAME}\" --limit 50"
echo ""
echo "   4. Para debug completo:"
echo "      gcloud run services logs read ${SERVICE_NAME} --region ${REGION}"
echo ""
echo "=============================================="

# Testar o health endpoint
echo ""
print_status "Testando endpoint de health..."
sleep 5  # Aguardar serviço estar pronto
if curl -s "${SERVICE_URL}/health" | grep -q "healthy"; then
    print_success "Serviço está respondendo corretamente!"
else
    print_warning "Serviço pode ainda estar inicializando. Tente novamente em alguns segundos."
fi

echo ""
echo "✅ Script finalizado com sucesso!"