#!/bin/bash

# Script para criar trigger via API REST do Cloud Build

PROJECT_ID="buzzlightear"
REGION="southamerica-east1"
CONNECTION_NAME="integracao_cli_gu"
REPO_NAME="nextlw-ChatGuru_Nordja_inte"
TRIGGER_NAME="chatguru-clickup-middleware-auto-deploy"

# Cria o trigger usando gcloud
gcloud builds triggers create \
  --name="${TRIGGER_NAME}" \
  --repository="projects/${PROJECT_ID}/locations/${REGION}/connections/${CONNECTION_NAME}/repositories/${REPO_NAME}" \
  --branch-pattern="^main$" \
  --build-config="chatguru-clickup-middleware/cloudbuild.yaml" \
  --region="${REGION}" \
  --project="${PROJECT_ID}" \
  --description="Auto deploy ChatGuru-ClickUp middleware on push to main"

echo "Trigger criado com sucesso!"

# Lista triggers para verificar
echo "Verificando triggers existentes:"
gcloud builds triggers list --region="${REGION}" --project="${PROJECT_ID}"