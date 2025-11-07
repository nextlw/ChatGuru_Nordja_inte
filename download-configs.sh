#!/bin/bash

# Script para baixar configurações do bucket GCS
# Use este script para sincronizar configurações do Cloud Storage para o ambiente local

set -e

# Cores para output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configurações
BUCKET="gs://chatguru-middleware-config"
LOCAL_DIR="chatguru-clickup-middleware/config"
PROJECT_ID="buzzlightear"

echo -e "${GREEN}=== Download de Configurações do GCS ===${NC}"

# Verificar autenticação
echo -e "${YELLOW}Verificando autenticação no GCP...${NC}"
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    echo -e "${RED}Erro: Você não está autenticado no gcloud${NC}"
    echo "Execute: gcloud auth login"
    exit 1
fi

# Verificar projeto
CURRENT_PROJECT=$(gcloud config get-value project 2>/dev/null)
if [ "$CURRENT_PROJECT" != "$PROJECT_ID" ]; then
    echo -e "${YELLOW}Mudando para o projeto ${PROJECT_ID}...${NC}"
    gcloud config set project ${PROJECT_ID}
fi

# Criar diretório local se não existir
echo -e "${YELLOW}Criando diretório local ${LOCAL_DIR}...${NC}"
mkdir -p ${LOCAL_DIR}

# Verificar se o bucket existe
echo -e "${YELLOW}Verificando bucket ${BUCKET}...${NC}"
if ! gsutil ls -b ${BUCKET} &>/dev/null; then
    echo -e "${RED}Erro: Bucket ${BUCKET} não existe!${NC}"
    echo "Execute primeiro: ./setup-cloud-build.sh"
    exit 1
fi

# Listar arquivos no bucket
echo -e "${YELLOW}Arquivos disponíveis no bucket:${NC}"
gsutil ls -l ${BUCKET}/*.toml ${BUCKET}/*.yaml 2>/dev/null || echo "Nenhum arquivo de configuração encontrado"

# Fazer backup local antes de sincronizar
if [ -d "${LOCAL_DIR}" ] && [ "$(ls -A ${LOCAL_DIR})" ]; then
    BACKUP_DIR="chatguru-clickup-middleware/config.backup.$(date +%Y%m%d-%H%M%S)"
    echo -e "${YELLOW}Criando backup local em ${BACKUP_DIR}...${NC}"
    cp -r ${LOCAL_DIR} ${BACKUP_DIR}
fi

# Sincronizar configurações
echo -e "${YELLOW}Sincronizando configurações do bucket...${NC}"
gsutil -m rsync -r -d ${BUCKET}/ ${LOCAL_DIR}/ \
    -x "backups/.*" \
    -x ".*\.keep$"

# Verificar arquivos baixados
echo -e "${GREEN}Arquivos sincronizados:${NC}"
ls -la ${LOCAL_DIR}/*.toml ${LOCAL_DIR}/*.yaml 2>/dev/null || echo "Nenhum arquivo encontrado"

echo -e "${GREEN}=== Download concluído com sucesso! ===${NC}"
echo ""
echo -e "${YELLOW}Dica:${NC} Para enviar alterações locais para o bucket, use:"
echo "  gsutil -m rsync -r -d ${LOCAL_DIR}/ ${BUCKET}/"
echo ""
echo -e "${YELLOW}Para ver backups disponíveis:${NC}"
echo "  gsutil ls ${BUCKET}/backups/"