#!/bin/bash

# Script para monitorar logs de produção no Google Cloud Run
# Útil para acompanhar o processamento dos testes

echo "=================================================="
echo "📡 MONITOR DE PRODUÇÃO - CHATGURU CLICKUP"
echo "=================================================="
echo ""

SERVICE="chatguru-clickup-middleware"
REGION="southamerica-east1"

# Cores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Função para mostrar menu
show_menu() {
    echo -e "${BLUE}Escolha uma opção:${NC}"
    echo "1) Ver logs em tempo real (tail)"
    echo "2) Ver últimos 100 logs"
    echo "3) Buscar logs com erro"
    echo "4) Buscar logs de processamento (Vertex AI)"
    echo "5) Buscar logs de tarefas criadas"
    echo "6) Ver status do serviço"
    echo "7) Ver métricas de requisições"
    echo "q) Sair"
    echo ""
    read -p "Opção: " choice
}

# Loop principal
while true; do
    clear
    echo "=================================================="
    echo "📡 MONITOR DE PRODUÇÃO - CHATGURU CLICKUP"
    echo "=================================================="
    echo -e "${YELLOW}Serviço:${NC} $SERVICE"
    echo -e "${YELLOW}Região:${NC} $REGION"
    echo ""
    
    show_menu
    
    case $choice in
        1)
            echo -e "${GREEN}Iniciando stream de logs...${NC}"
            echo "Pressione Ctrl+C para parar"
            echo ""
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\"" \
                --project=buzzlightear \
                --format="table(timestamp,textPayload)" \
                --limit=50 \
                --freshness=1m
            ;;
        2)
            echo -e "${GREEN}Últimos 100 logs:${NC}"
            echo ""
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\"" \
                --project=buzzlightear \
                --format="table(timestamp,textPayload)" \
                --limit=100
            ;;
        3)
            echo -e "${RED}Logs com erro:${NC}"
            echo ""
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"ERROR\" OR textPayload:\"WARN\" OR textPayload:\"Failed\")" \
                --project=buzzlightear \
                --format="table(timestamp,textPayload)" \
                --limit=50
            ;;
        4)
            echo -e "${PURPLE}Logs de processamento Vertex AI:${NC}"
            echo ""
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"Vertex\" OR textPayload:\"OAuth\" OR textPayload:\"classification\" OR textPayload:\"Gemini\")" \
                --project=buzzlightear \
                --format="table(timestamp,textPayload)" \
                --limit=50
            ;;
        5)
            echo -e "${GREEN}Tarefas criadas:${NC}"
            echo ""
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"task created\" OR textPayload:\"tarefa criada\" OR textPayload:\"ClickUp\")" \
                --project=buzzlightear \
                --format="table(timestamp,textPayload)" \
                --limit=50
            ;;
        6)
            echo -e "${BLUE}Status do serviço:${NC}"
            echo ""
            gcloud run services describe $SERVICE --region=$REGION --format="yaml" | grep -E "status:|ready:|url:|latestCreatedRevisionName:|observedGeneration:"
            echo ""
            echo -e "${YELLOW}URL do serviço:${NC}"
            gcloud run services describe $SERVICE --region=$REGION --format="value(status.url)"
            ;;
        7)
            echo -e "${BLUE}Métricas das últimas 24h:${NC}"
            echo ""
            echo "Requisições por hora:"
            gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND httpRequest.requestMethod=\"POST\"" \
                --project=buzzlightear \
                --format="table(timestamp.date('%Y-%m-%d %H:00'),httpRequest.status)" \
                --limit=100 | sort | uniq -c
            ;;
        q)
            echo -e "${GREEN}Saindo do monitor...${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Opção inválida!${NC}"
            sleep 2
            ;;
    esac
    
    echo ""
    read -p "Pressione Enter para continuar..."
done