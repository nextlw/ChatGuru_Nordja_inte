#!/bin/bash

# Script para configurar o Google Cloud Build Trigger
# Este script configura o gatilho para builds automáticos quando há push no GitHub

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configurações
PROJECT_ID="buzzlightear"
REPO_NAME="chatguru-nordja-inte"
GITHUB_OWNER="seu-usuario-github"  # AJUSTE ISSO
TRIGGER_NAME="chatguru-middleware-trigger"
BUCKET_NAME="chatguru-middleware-config"
REGION="southamerica-east1"

echo -e "${GREEN}=== Configurando Google Cloud Build ===${NC}"

# 1. Verificar se está autenticado
echo -e "${YELLOW}Verificando autenticação...${NC}"
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    echo -e "${RED}Erro: Você não está autenticado no gcloud${NC}"
    echo "Execute: gcloud auth login"
    exit 1
fi

# 2. Configurar projeto
echo -e "${YELLOW}Configurando projeto ${PROJECT_ID}...${NC}"
gcloud config set project ${PROJECT_ID}

# 3. Habilitar APIs necessárias
echo -e "${YELLOW}Habilitando APIs necessárias...${NC}"
gcloud services enable cloudbuild.googleapis.com \
    containerregistry.googleapis.com \
    run.googleapis.com \
    secretmanager.googleapis.com \
    pubsub.googleapis.com \
    storage.googleapis.com

# 4. Criar bucket para configurações (se não existir)
echo -e "${YELLOW}Criando bucket para configurações...${NC}"
if ! gsutil ls -b gs://${BUCKET_NAME} &>/dev/null; then
    gsutil mb -p ${PROJECT_ID} -l ${REGION} gs://${BUCKET_NAME}/
    gsutil versioning set on gs://${BUCKET_NAME}/
    echo -e "${GREEN}Bucket criado: gs://${BUCKET_NAME}${NC}"
    
    # Criar estrutura de pastas
    echo "placeholder" | gsutil cp - gs://${BUCKET_NAME}/backups/.keep
else
    echo -e "${GREEN}Bucket já existe: gs://${BUCKET_NAME}${NC}"
fi

# 5. Configurar permissões do Cloud Build
echo -e "${YELLOW}Configurando permissões do Cloud Build...${NC}"
PROJECT_NUMBER=$(gcloud projects describe ${PROJECT_ID} --format='value(projectNumber)')
CLOUD_BUILD_SA="${PROJECT_NUMBER}@cloudbuild.gserviceaccount.com"

# Dar permissões necessárias ao Cloud Build
gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${CLOUD_BUILD_SA}" \
    --role="roles/run.admin"

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${CLOUD_BUILD_SA}" \
    --role="roles/secretmanager.secretAccessor"

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${CLOUD_BUILD_SA}" \
    --role="roles/storage.admin"

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
    --member="serviceAccount:${CLOUD_BUILD_SA}" \
    --role="roles/pubsub.admin"

# 6. Conectar repositório GitHub (se ainda não conectado)
echo -e "${YELLOW}Conectando repositório GitHub...${NC}"
if ! gcloud builds repositories list --connection=github --region=${REGION} 2>/dev/null | grep -q ${REPO_NAME}; then
    echo -e "${YELLOW}Por favor, siga estes passos manualmente:${NC}"
    echo "1. Acesse: https://console.cloud.google.com/cloud-build/triggers/connect"
    echo "2. Selecione GitHub e autorize o acesso"
    echo "3. Selecione o repositório: ${GITHUB_OWNER}/${REPO_NAME}"
    echo "4. Após conectar, pressione ENTER para continuar..."
    read -p ""
fi

# 7. Criar trigger do Cloud Build
echo -e "${YELLOW}Criando trigger do Cloud Build...${NC}"

# Verificar se o trigger já existe
if gcloud builds triggers list --region=${REGION} --filter="name:${TRIGGER_NAME}" --format="value(name)" | grep -q .; then
    echo -e "${YELLOW}Trigger já existe. Atualizando...${NC}"
    gcloud builds triggers delete ${TRIGGER_NAME} --region=${REGION} --quiet
fi

# Criar novo trigger
cat > trigger-config.yaml <<EOF
name: ${TRIGGER_NAME}
description: "Trigger automático para chatguru-clickup-middleware"
github:
  owner: ${GITHUB_OWNER}
  name: ${REPO_NAME}
  push:
    branch: "^main$"
includedFiles:
  - "chatguru-clickup-middleware/**"
  - "cloudbuild.yaml"
ignoredFiles:
  - "*.md"
  - ".github/**"
  - "docs/**"
filename: cloudbuild.yaml
substitutions:
  _REGION: ${REGION}
  _SERVICE_NAME: chatguru-clickup-middleware
  _CONFIG_BUCKET: gs://${BUCKET_NAME}
EOF

gcloud builds triggers create github \
    --region=${REGION} \
    --repo-name=${REPO_NAME} \
    --repo-owner=${GITHUB_OWNER} \
    --branch-pattern="^main$" \
    --build-config="cloudbuild.yaml" \
    --name=${TRIGGER_NAME} \
    --description="Deploy automático do middleware ChatGuru-ClickUp" \
    --include-files="chatguru-clickup-middleware/**,cloudbuild.yaml" \
    --ignored-files="*.md,.github/**,docs/**" \
    --substitutions="_REGION=${REGION},_SERVICE_NAME=chatguru-clickup-middleware,_CONFIG_BUCKET=gs://${BUCKET_NAME}"

echo -e "${GREEN}Trigger criado com sucesso!${NC}"

# 8. Sincronizar configurações iniciais
echo -e "${YELLOW}Sincronizando configurações iniciais para o bucket...${NC}"
if [ -d "chatguru-clickup-middleware/config" ]; then
    gsutil -m rsync -r -d chatguru-clickup-middleware/config/ gs://${BUCKET_NAME}/
    echo -e "${GREEN}Configurações sincronizadas!${NC}"
else
    echo -e "${YELLOW}Diretório de configuração não encontrado. Execute este script da raiz do repositório.${NC}"
fi

# 9. Criar secrets necessários (se não existirem)
echo -e "${YELLOW}Verificando secrets...${NC}"

declare -a secrets=("openai-api-key" "clickup-api-token" "clickup-oauth-token" "chatguru-api-token" "database-url")

for secret in "${secrets[@]}"; do
    if ! gcloud secrets describe ${secret} &>/dev/null; then
        echo -e "${YELLOW}Secret ${secret} não existe. Criando...${NC}"
        echo -n "Digite o valor para ${secret}: "
        read -s secret_value
        echo
        echo -n "${secret_value}" | gcloud secrets create ${secret} --data-file=-
        echo -e "${GREEN}Secret ${secret} criado!${NC}"
    else
        echo -e "${GREEN}Secret ${secret} já existe${NC}"
    fi
done

# 10. Criar arquivo de configuração para download de configs do bucket
echo -e "${YELLOW}Criando script de download de configurações...${NC}"
cat > download-configs.sh <<'DOWNLOAD_SCRIPT'
#!/bin/bash
# Script para baixar configurações do bucket GCS

BUCKET="gs://chatguru-middleware-config"
LOCAL_DIR="chatguru-clickup-middleware/config"

echo "Baixando configurações do bucket ${BUCKET}..."
mkdir -p ${LOCAL_DIR}
gsutil -m rsync -r -d ${BUCKET}/ ${LOCAL_DIR}/
echo "Configurações baixadas com sucesso!"
DOWNLOAD_SCRIPT

chmod +x download-configs.sh

echo -e "${GREEN}=== Configuração Completa! ===${NC}"
echo ""
echo -e "${GREEN}Próximos passos:${NC}"
echo "1. Ajuste a variável GITHUB_OWNER neste script com seu usuário GitHub"
echo "2. Certifique-se de que o repositório está conectado ao Cloud Build"
echo "3. Faça um push para a branch main para testar o trigger"
echo "4. Use ./download-configs.sh para baixar configs do bucket"
echo ""
echo -e "${YELLOW}Comandos úteis:${NC}"
echo "  # Ver logs do build:"
echo "  gcloud builds log"
echo ""
echo "  # Listar triggers:"
echo "  gcloud builds triggers list --region=${REGION}"
echo ""
echo "  # Ver configurações no bucket:"
echo "  gsutil ls -l gs://${BUCKET_NAME}/"
echo ""
echo "  # Testar build manualmente:"
echo "  gcloud builds submit --config cloudbuild.yaml"