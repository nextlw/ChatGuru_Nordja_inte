#!/bin/bash
# Script para listar todos os spaces do workspace ClickUp

CLICKUP_TOKEN=$(gcloud secrets versions access latest --secret="clickup-api-token" --project=buzzlightear 2>/dev/null)

if [ -z "$CLICKUP_TOKEN" ]; then
    echo "Erro: Não foi possível obter o token do ClickUp"
    exit 1
fi

echo "=== Listando todos os Spaces do ClickUp ==="
echo ""

curl -s -X GET "https://api.clickup.com/api/v2/team" \
  -H "Authorization: ${CLICKUP_TOKEN}" \
  -H "Content-Type: application/json" | jq -r '
.teams[].spaces[] | 
"Space: \(.name)
ID: \(.id)
---"
'
