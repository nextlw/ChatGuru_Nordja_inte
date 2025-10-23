# Dockerfile para Cloud Run Job de sincronização do ClickUp
# Este job é executado diariamente via Cloud Scheduler
FROM curlimages/curl:latest

# Definir variáveis de ambiente
ENV SERVICE_URL="https://chatguru-clickup-middleware-pcwqxktwfq-rj.a.run.app/admin/sync-clickup"

# Script de entrada que faz o POST request
ENTRYPOINT ["/bin/sh", "-c", "curl -X POST -H 'Content-Type: application/json' ${SERVICE_URL} && echo 'Sync completed successfully'"]
