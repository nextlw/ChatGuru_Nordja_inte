#!/bin/bash

# Script para configurar os secrets no Google Secret Manager
# Uso: ./scripts/setup-secrets.sh

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Configuração do Google Secret Manager ===${NC}"

# Verifica se gcloud está instalado
if ! command -v gcloud &> /dev/null; then
    echo -e "${RED}Erro: gcloud CLI não está instalado${NC}"
    echo "Por favor, instale o Google Cloud SDK: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Obtém o projeto atual
PROJECT_ID=$(gcloud config get-value project 2>/dev/null)

if [ -z "$PROJECT_ID" ]; then
    echo -e "${RED}Erro: Nenhum projeto GCP configurado${NC}"
    echo "Use: gcloud config set project SEU_PROJETO_ID"
    exit 1
fi

echo -e "${GREEN}Projeto GCP: ${PROJECT_ID}${NC}"

# Habilita a API do Secret Manager
echo -e "${YELLOW}Habilitando a API do Secret Manager...${NC}"
gcloud services enable secretmanager.googleapis.com --project=$PROJECT_ID

# Função para criar ou atualizar um secret
create_or_update_secret() {
    local SECRET_NAME=$1
    local SECRET_VALUE=$2
    local DESCRIPTION=$3
    
    echo -e "${YELLOW}Configurando secret: ${SECRET_NAME}${NC}"
    
    # Verifica se o secret já existe
    if gcloud secrets describe $SECRET_NAME --project=$PROJECT_ID &>/dev/null; then
        echo "  Secret já existe. Criando nova versão..."
        echo -n "$SECRET_VALUE" | gcloud secrets versions add $SECRET_NAME \
            --data-file=- \
            --project=$PROJECT_ID
    else
        echo "  Criando novo secret..."
        echo -n "$SECRET_VALUE" | gcloud secrets create $SECRET_NAME \
            --data-file=- \
            --replication-policy="automatic" \
            --project=$PROJECT_ID
        
        if [ -n "$DESCRIPTION" ]; then
            gcloud secrets update $SECRET_NAME \
                --update-labels="description=$DESCRIPTION" \
                --project=$PROJECT_ID
        fi
    fi
    
    echo -e "${GREEN}  ✓ Secret ${SECRET_NAME} configurado${NC}"
}

# Solicita as informações ao usuário
echo ""
echo -e "${YELLOW}Por favor, forneça as seguintes informações:${NC}"

# ClickUp API Token
read -p "ClickUp API Token (começa com pk_): " CLICKUP_API_TOKEN
if [ -z "$CLICKUP_API_TOKEN" ]; then
    echo -e "${RED}Erro: Token do ClickUp é obrigatório${NC}"
    exit 1
fi

# ClickUp List ID
read -p "ClickUp List ID [padrão: 901300373349]: " CLICKUP_LIST_ID
CLICKUP_LIST_ID=${CLICKUP_LIST_ID:-901300373349}

echo ""
echo -e "${GREEN}=== Criando Secrets ===${NC}"

# Cria os secrets
create_or_update_secret "clickup-api-token" "$CLICKUP_API_TOKEN" "Token da API do ClickUp"
create_or_update_secret "clickup-list-id" "$CLICKUP_LIST_ID" "ID da lista do ClickUp"

echo ""
echo -e "${GREEN}=== Configurando Permissões IAM ===${NC}"

# Obtém a conta de serviço do Cloud Run
SERVICE_ACCOUNT=$(gcloud run services describe chatguru-clickup-middleware \
    --region=us-central1 \
    --format="value(spec.template.spec.serviceAccountName)" \
    --project=$PROJECT_ID 2>/dev/null)

if [ -z "$SERVICE_ACCOUNT" ]; then
    # Se o serviço ainda não existe, usa a conta de serviço padrão do Compute Engine
    PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")
    SERVICE_ACCOUNT="${PROJECT_NUMBER}-compute@developer.gserviceaccount.com"
    echo -e "${YELLOW}Serviço Cloud Run ainda não existe. Usando conta de serviço padrão: ${SERVICE_ACCOUNT}${NC}"
else
    echo -e "${GREEN}Conta de serviço do Cloud Run: ${SERVICE_ACCOUNT}${NC}"
fi

# Concede permissão para acessar os secrets
echo -e "${YELLOW}Concedendo permissões para a conta de serviço...${NC}"

for SECRET_NAME in "clickup-api-token" "clickup-list-id"; do
    gcloud secrets add-iam-policy-binding $SECRET_NAME \
        --member="serviceAccount:${SERVICE_ACCOUNT}" \
        --role="roles/secretmanager.secretAccessor" \
        --project=$PROJECT_ID \
        --condition=None
    echo -e "${GREEN}  ✓ Permissão concedida para ${SECRET_NAME}${NC}"
done

echo ""
echo -e "${GREEN}=== Configuração Completa ===${NC}"
echo ""
echo "Os seguintes secrets foram configurados:"
echo "  • clickup-api-token"
echo "  • clickup-list-id"
echo ""
echo "A conta de serviço ${SERVICE_ACCOUNT} tem acesso aos secrets."
echo ""
echo -e "${YELLOW}Próximos passos:${NC}"
echo "1. Deploy o middleware: ./quick-deploy.sh"
echo "2. Verifique os logs: gcloud run logs read chatguru-clickup-middleware --region=us-central1"
echo ""
echo -e "${GREEN}Para testar localmente com os secrets:${NC}"
echo "  export GOOGLE_APPLICATION_CREDENTIALS=/caminho/para/sua/key.json"
echo "  export GCP_PROJECT_ID=$PROJECT_ID"
echo "  cargo run"
echo ""