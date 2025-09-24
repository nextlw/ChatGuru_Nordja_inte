#!/bin/bash

# Script simples para ver logs do Cloud Run

SERVICE="chatguru-clickup-middleware"
PROJECT="buzzlightear"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}📋 Logs do ChatGuru-ClickUp Middleware${NC}"
echo ""

# Verificar parâmetro
if [ "$1" == "error" ]; then
    echo -e "${RED}Mostrando apenas ERROS:${NC}"
    gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND severity>=ERROR" \
        --project=$PROJECT \
        --format="table(timestamp,severity,textPayload)" \
        --limit=50
elif [ "$1" == "tail" ]; then
    echo -e "${YELLOW}Logs em tempo real (últimos 2 minutos):${NC}"
    watch -n 5 "gcloud logging read 'resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\"' \
        --project=$PROJECT \
        --format='table(timestamp,textPayload)' \
        --limit=20 \
        --freshness=2m"
elif [ "$1" == "vertex" ]; then
    echo -e "${YELLOW}Logs do Vertex AI:${NC}"
    gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND textPayload:(\"Vertex\" OR \"OAuth\" OR \"Gemini\" OR \"classification\")" \
        --project=$PROJECT \
        --format="table(timestamp,textPayload)" \
        --limit=30
else
    echo -e "${GREEN}Últimos 50 logs:${NC}"
    gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\"" \
        --project=$PROJECT \
        --format="table(timestamp,textPayload)" \
        --limit=50
fi