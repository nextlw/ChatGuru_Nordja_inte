#!/bin/bash

# ==============================================================================
# Script de Deploy Completo - ChatGuru-ClickUp Middleware
# ==============================================================================

set -e  # Exit on error

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configurações do projeto
PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
SERVICE_NAME="chatguru-clickup-middleware"

# Configurações da aplicação
CLICKUP_API_TOKEN="pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657"
CLICKUP_LIST_ID="901300373349"

# Configurações do ChatGuru
CHATGURU_API_TOKEN="TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK"
CHATGURU_API_ENDPOINT="https://s15.chatguru.app"
CHATGURU_ACCOUNT_ID="625584ce6fdcb7bda7d94aa8"

# Configurações da IA
AI_ENABLED="true"
OPENAI_API_KEY="${OPENAI_API_KEY:-}"  # Definir via variável de ambiente

# Funções para output colorido
print_header() {
    echo ""
    echo -e "${CYAN}============================================${NC}"
    echo -e "${CYAN}   $1${NC}"
    echo -e "${CYAN}============================================${NC}"
    echo ""
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Header principal
clear
print_header "Deploy ChatGuru-ClickUp Middleware"

# 1. Verificar pré-requisitos
print_status "Verificando pré-requisitos..."

# Verificar se está no diretório correto
if [ ! -f "Cargo.toml" ]; then
    print_error "Execute este script do diretório chatguru-clickup-middleware/"
    exit 1
fi
print_success "Diretório correto: $(pwd)"

# Verificar gcloud CLI
if ! command -v gcloud &> /dev/null; then
    print_error "gcloud CLI não está instalado"
    echo "Instale em: https://cloud.google.com/sdk/docs/install"
    exit 1
fi
print_success "gcloud CLI instalado"

# 2. Autenticação
print_status "Verificando autenticação Google Cloud..."
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    print_warning "Não há conta ativa. Fazendo login..."
    gcloud auth login
fi
ACTIVE_ACCOUNT=$(gcloud auth list --filter=status:ACTIVE --format="value(account)")
print_success "Autenticado como: $ACTIVE_ACCOUNT"

# 3. Configurar projeto
print_status "Configurando projeto Google Cloud..."
gcloud config set project ${PROJECT_ID} --quiet
print_success "Projeto configurado: ${PROJECT_ID}"

# 4. Verificar/Habilitar APIs necessárias
print_status "Verificando APIs necessárias..."
APIS_NEEDED="run.googleapis.com cloudbuild.googleapis.com artifactregistry.googleapis.com aiplatform.googleapis.com"

for api in $APIS_NEEDED; do
    if gcloud services list --enabled --filter="name:${api}" --format="value(name)" | grep -q "${api}"; then
        print_success "API ${api} já habilitada"
    else
        print_warning "Habilitando API ${api}..."
        gcloud services enable ${api} --quiet
        print_success "API ${api} habilitada"
    fi
done

# 4.1 Configurar Service Account para Vertex AI
print_status "Configurando permissões para Vertex AI..."

# Obter a service account padrão do Cloud Run
# O número do projeto é 707444002434
SERVICE_ACCOUNT="707444002434-compute@developer.gserviceaccount.com"
print_status "Service Account: ${SERVICE_ACCOUNT}"

# Verificar/adicionar role para Vertex AI
VERTEX_ROLE="roles/aiplatform.user"
if gcloud projects get-iam-policy ${PROJECT_ID} \
    --flatten="bindings[].members" \
    --format="table(bindings.role)" \
    --filter="bindings.members:${SERVICE_ACCOUNT}" | grep -q "${VERTEX_ROLE}"; then
    print_success "Permissão ${VERTEX_ROLE} já configurada"
else
    print_warning "Adicionando permissão ${VERTEX_ROLE}..."
    gcloud projects add-iam-policy-binding ${PROJECT_ID} \
        --member="serviceAccount:${SERVICE_ACCOUNT}" \
        --role="${VERTEX_ROLE}" \
        --quiet > /dev/null 2>&1
    print_success "Permissão ${VERTEX_ROLE} adicionada"
fi

# 5. Escolher método de deploy
echo ""
print_header "Método de Deploy"
echo "Escolha o método de deploy:"
echo ""
echo "  1) Deploy direto do código (RECOMENDADO - mais simples e rápido)"
echo "  2) Build local + Docker push (requer Docker instalado)"
echo "  3) Apenas verificar status do serviço"
echo ""
read -p "Digite sua escolha (1-3) [1]: " DEPLOY_METHOD
DEPLOY_METHOD=${DEPLOY_METHOD:-1}

case $DEPLOY_METHOD in
    1)
        # Deploy direto do código fonte
        print_header "Deploy Direto do Código"
        
        print_status "Verificando arquivos necessários..."
        
        # Verificar Dockerfile
        if [ ! -f "Dockerfile" ]; then
            print_error "Dockerfile não encontrado!"
            exit 1
        fi
        print_success "Dockerfile encontrado"
        
        # Verificar se PORT não está no Dockerfile
        if grep -q "ENV PORT=" Dockerfile; then
            print_warning "Dockerfile contém ENV PORT - isso pode causar erro no Cloud Run"
            print_status "Removendo ENV PORT do Dockerfile temporariamente..."
            sed -i.bak '/ENV PORT=/d' Dockerfile
            print_success "ENV PORT removido"
        fi
        
        # Fazer o deploy
        print_status "Iniciando deploy no Cloud Run..."
        print_warning "Isso pode levar 3-5 minutos..."
        echo ""
        
        if gcloud run deploy ${SERVICE_NAME} \
            --source . \
            --region ${REGION} \
            --allow-unauthenticated \
            --project ${PROJECT_ID} \
            --memory 512Mi \
            --cpu 1 \
            --timeout 300 \
            --min-instances 0 \
            --max-instances 10 \
            --set-env-vars "CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN},CLICKUP_LIST_ID=${CLICKUP_LIST_ID},CHATGURU__API_TOKEN=${CHATGURU_API_TOKEN},CHATGURU__API_ENDPOINT=${CHATGURU_API_ENDPOINT},CHATGURU__ACCOUNT_ID=${CHATGURU_ACCOUNT_ID},AI__ENABLED=${AI_ENABLED},GCP__PROJECT_ID=${PROJECT_ID},OPENAI_API_KEY=${OPENAI_API_KEY},RUST_LOG=info"; then
            
            print_success "Deploy concluído com sucesso!"
            
            # Restaurar Dockerfile se foi modificado
            if [ -f "Dockerfile.bak" ]; then
                mv Dockerfile.bak Dockerfile
            fi
        else
            print_error "Deploy falhou!"
            # Restaurar Dockerfile se foi modificado
            if [ -f "Dockerfile.bak" ]; then
                mv Dockerfile.bak Dockerfile
            fi
            exit 1
        fi
        ;;
        
    2)
        # Build local e push
        print_header "Build Local + Docker Push"
        
        # Verificar Docker
        if ! command -v docker &> /dev/null; then
            print_error "Docker não está instalado!"
            exit 1
        fi
        print_success "Docker instalado"
        
        # Configurar Artifact Registry
        REPOSITORY="chatguru-integrations"
        IMAGE_URI="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPOSITORY}/${SERVICE_NAME}"
        
        print_status "Configurando Artifact Registry..."
        
        # Criar repositório se não existir
        if ! gcloud artifacts repositories describe ${REPOSITORY} \
            --location=${REGION} --project=${PROJECT_ID} &> /dev/null; then
            print_status "Criando repositório ${REPOSITORY}..."
            gcloud artifacts repositories create ${REPOSITORY} \
                --repository-format=docker \
                --location=${REGION} \
                --project=${PROJECT_ID} \
                --quiet
        fi
        print_success "Artifact Registry configurado"
        
        # Autenticar Docker
        print_status "Configurando autenticação Docker..."
        gcloud auth configure-docker ${REGION}-docker.pkg.dev --quiet
        print_success "Docker autenticado"
        
        # Build da imagem específica para amd64/linux (requerido pelo Cloud Run)
        print_status "Construindo imagem Docker para linux/amd64..."
        
        # Garantir que estamos usando o builder padrão para evitar problemas com manifests OCI
        docker buildx use default 2>/dev/null || true
        
        # Build tradicional para garantir compatibilidade
        if docker build \
            --platform linux/amd64 \
            --tag ${IMAGE_URI}:latest \
            --tag ${IMAGE_URI}:$(date +%Y%m%d-%H%M%S) \
            .; then
            print_success "Imagem construída com sucesso!"
            
            # Push das imagens
            print_status "Enviando imagem para Artifact Registry..."
            docker push ${IMAGE_URI}:latest
            docker push ${IMAGE_URI}:$(date +%Y%m%d-%H%M%S)
            print_success "Imagem enviada!"
        else
            print_error "Build falhou!"
            exit 1
        fi
        
        # Deploy da imagem
        print_status "Fazendo deploy da imagem..."
        gcloud run deploy ${SERVICE_NAME} \
            --image ${IMAGE_URI}:latest \
            --region ${REGION} \
            --allow-unauthenticated \
            --project ${PROJECT_ID} \
            --memory 512Mi \
            --cpu 1 \
            --timeout 300 \
            --min-instances 0 \
            --max-instances 10 \
            --set-env-vars "CLICKUP_API_TOKEN=${CLICKUP_API_TOKEN},CLICKUP_LIST_ID=${CLICKUP_LIST_ID},CHATGURU__API_TOKEN=${CHATGURU_API_TOKEN},CHATGURU__API_ENDPOINT=${CHATGURU_API_ENDPOINT},CHATGURU__ACCOUNT_ID=${CHATGURU_ACCOUNT_ID},AI__ENABLED=${AI_ENABLED},GCP__PROJECT_ID=${PROJECT_ID},OPENAI_API_KEY=${OPENAI_API_KEY},RUST_LOG=info" \
            --quiet
            
        print_success "Deploy concluído!"
        ;;
        
    3)
        # Apenas verificar status
        print_header "Verificação de Status"
        ;;
        
    *)
        print_error "Opção inválida!"
        exit 1
        ;;
esac

# 6. Obter informações do serviço
print_header "Informações do Serviço"

if gcloud run services describe ${SERVICE_NAME} \
    --region ${REGION} \
    --project ${PROJECT_ID} &> /dev/null; then
    
    # Obter URL
    SERVICE_URL=$(gcloud run services describe ${SERVICE_NAME} \
        --region ${REGION} \
        --project ${PROJECT_ID} \
        --format 'value(status.url)')
    
    print_success "Serviço encontrado!"
    echo ""
    echo -e "${GREEN}📍 URL do Serviço:${NC}"
    echo "   ${SERVICE_URL}"
    echo ""
    echo -e "${GREEN}🔗 Endpoints disponíveis:${NC}"
    echo "   Health Check: ${SERVICE_URL}/health"
    echo "   Ready Check:  ${SERVICE_URL}/ready"
    echo "   Status:       ${SERVICE_URL}/status"
    echo "   Webhook:      ${SERVICE_URL}/webhooks/chatguru"
    echo "   ClickUp Test: ${SERVICE_URL}/clickup/test"
    echo ""
    
    # Testar health check
    print_status "Testando health check..."
    if curl -s "${SERVICE_URL}/health" 2>/dev/null | grep -q "healthy"; then
        print_success "Serviço está respondendo corretamente!"
        echo ""
        echo "Resposta do health check:"
        curl -s "${SERVICE_URL}/health" | jq . 2>/dev/null || curl -s "${SERVICE_URL}/health"
    else
        print_warning "Health check falhou ou serviço ainda está iniciando"
    fi
    
    echo ""
    print_header "Configuração para ChatGuru"
    echo -e "${YELLOW}Configure o webhook no ChatGuru com:${NC}"
    echo ""
    echo "   URL:    ${SERVICE_URL}/webhooks/chatguru"
    echo "   Método: POST"
    echo "   Headers:"
    echo "     Content-Type: application/json"
    echo ""
    
    print_header "Comandos Úteis"
    echo "📊 Ver logs em tempo real:"
    echo "   gcloud run logs tail ${SERVICE_NAME} --region ${REGION}"
    echo ""
    echo "📝 Ver logs recentes:"
    echo "   gcloud run logs read ${SERVICE_NAME} --region ${REGION} --limit 50"
    echo ""
    echo "🔄 Fazer redeploy:"
    echo "   ./deploy.sh"
    echo ""
    echo "🧪 Testar webhook localmente:"
    echo "   node test-webhook-production.js"
    echo ""
    
else
    print_error "Serviço não encontrado!"
    echo "Execute o deploy primeiro (opção 1 ou 2)"
fi

echo ""
print_success "Script finalizado!"
echo ""