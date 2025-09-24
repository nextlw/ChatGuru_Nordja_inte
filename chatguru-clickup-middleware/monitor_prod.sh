#!/bin/bash

# Script para monitorar logs de produção no Google Cloud Run
# Com cores e formatação melhorada para facilitar leitura

echo "=================================================="
echo "📡 MONITOR DE PRODUÇÃO - CHATGURU CLICKUP"
echo "=================================================="
echo ""

SERVICE="chatguru-clickup-middleware"
REGION="southamerica-east1"

# Cores e estilos
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Emojis para diferentes tipos de log
SUCCESS="✅"
ERROR="❌"
WARNING="⚠️"
INFO="ℹ️"
TASK="📋"
AI="🤖"
WEBHOOK="🔔"
TIME="⏰"
PROCESSING="⚙️"

# Função para formatar logs com cores
format_logs() {
    local filter="$1"
    local title="$2"
    local color="$3"
    
    echo ""
    echo -e "${color}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${color}${title}${NC}"
    echo -e "${color}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    # Buscar logs e aplicar colorização
    gcloud logging read "$filter" \
        --project=buzzlightear \
        --format="csv[no-heading](timestamp,textPayload)" \
        --limit=50 | while IFS=',' read -r timestamp payload; do
        
        # Formatar timestamp
        time_formatted=$(echo "$timestamp" | cut -d'T' -f2 | cut -d'.' -f1)
        date_formatted=$(echo "$timestamp" | cut -d'T' -f1)
        
        # Colorir baseado no conteúdo
        if [[ "$payload" == *"ERROR"* ]] || [[ "$payload" == *"error"* ]] || [[ "$payload" == *"Failed"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${RED}${ERROR} ${payload}${NC}"
        elif [[ "$payload" == *"WARN"* ]] || [[ "$payload" == *"warning"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${YELLOW}${WARNING} ${payload}${NC}"
        elif [[ "$payload" == *"task created"* ]] || [[ "$payload" == *"Task criada"* ]] || [[ "$payload" == *"ClickUp task"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${GREEN}${TASK} ${payload}${NC}"
        elif [[ "$payload" == *"Vertex"* ]] || [[ "$payload" == *"Gemini"* ]] || [[ "$payload" == *"classificação"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${PURPLE}${AI} ${payload}${NC}"
        elif [[ "$payload" == *"webhook"* ]] || [[ "$payload" == *"Webhook"* ]] || [[ "$payload" == *"Request received"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${CYAN}${WEBHOOK} ${payload}${NC}"
        elif [[ "$payload" == *"Processing"* ]] || [[ "$payload" == *"processando"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${BLUE}${PROCESSING} ${payload}${NC}"
        elif [[ "$payload" == *"Success"* ]] || [[ "$payload" == *"successfully"* ]] || [[ "$payload" == *"✓"* ]]; then
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${GREEN}${SUCCESS} ${payload}${NC}"
        else
            echo -e "${GRAY}${date_formatted} ${WHITE}${time_formatted}${NC} ${INFO} ${payload}${NC}"
        fi
    done
}

# Função para logs em tempo real com cores
stream_logs() {
    echo -e "${CYAN}${BOLD}🔴 LOGS EM TEMPO REAL${NC}"
    echo -e "${DIM}Pressione Ctrl+C para parar${NC}"
    echo ""
    
    # Loop infinito para simular tail -f
    while true; do
        gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND timestamp>=\"$(date -u -v-1M '+%Y-%m-%dT%H:%M:%S')\"" \
            --project=buzzlightear \
            --format="csv[no-heading](timestamp,textPayload)" \
            --limit=10 | while IFS=',' read -r timestamp payload; do
            
            time_formatted=$(echo "$timestamp" | cut -d'T' -f2 | cut -d'.' -f1)
            
            # Aplicar cores baseadas no tipo de log
            if [[ "$payload" == *"ERROR"* ]]; then
                echo -e "${WHITE}${time_formatted}${NC} ${RED}${ERROR} ${payload}${NC}"
            elif [[ "$payload" == *"WARN"* ]]; then
                echo -e "${WHITE}${time_formatted}${NC} ${YELLOW}${WARNING} ${payload}${NC}"
            elif [[ "$payload" == *"task created"* ]]; then
                echo -e "${WHITE}${time_formatted}${NC} ${GREEN}${TASK} ${BOLD}${payload}${NC}"
            elif [[ "$payload" == *"Vertex"* ]] || [[ "$payload" == *"Gemini"* ]]; then
                echo -e "${WHITE}${time_formatted}${NC} ${PURPLE}${AI} ${payload}${NC}"
            elif [[ "$payload" == *"webhook"* ]]; then
                echo -e "${WHITE}${time_formatted}${NC} ${CYAN}${WEBHOOK} ${payload}${NC}"
            else
                echo -e "${WHITE}${time_formatted}${NC} ${INFO} ${payload}${NC}"
            fi
        done
        sleep 2
    done
}

# Função para mostrar estatísticas com cores
show_stats() {
    echo ""
    echo -e "${CYAN}${BOLD}📊 ESTATÍSTICAS DAS ÚLTIMAS 24 HORAS${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    # Contar diferentes tipos de eventos
    echo -e "${YELLOW}📈 Resumo de Eventos:${NC}"
    echo ""
    
    # Total de requisições
    total=$(gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND timestamp>=\"$(date -u -v-24H '+%Y-%m-%dT%H:%M:%S')\"" \
        --project=buzzlightear --format="value(textPayload)" --limit=1000 | wc -l)
    echo -e "  ${WHITE}Total de logs:${NC} ${BOLD}${total}${NC}"
    
    # Erros
    errors=$(gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"ERROR\" OR textPayload:\"Failed\") AND timestamp>=\"$(date -u -v-24H '+%Y-%m-%dT%H:%M:%S')\"" \
        --project=buzzlightear --format="value(textPayload)" --limit=1000 | wc -l)
    echo -e "  ${RED}Erros:${NC} ${errors}"
    
    # Warnings
    warnings=$(gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND textPayload:\"WARN\" AND timestamp>=\"$(date -u -v-24H '+%Y-%m-%dT%H:%M:%S')\"" \
        --project=buzzlightear --format="value(textPayload)" --limit=1000 | wc -l)
    echo -e "  ${YELLOW}Avisos:${NC} ${warnings}"
    
    # Tarefas criadas
    tasks=$(gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND textPayload:\"task created\" AND timestamp>=\"$(date -u -v-24H '+%Y-%m-%dT%H:%M:%S')\"" \
        --project=buzzlightear --format="value(textPayload)" --limit=1000 | wc -l)
    echo -e "  ${GREEN}Tarefas criadas:${NC} ${tasks}"
    
    # Webhooks recebidos
    webhooks=$(gcloud logging read "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND textPayload:\"webhook\" AND timestamp>=\"$(date -u -v-24H '+%Y-%m-%dT%H:%M:%S')\"" \
        --project=buzzlightear --format="value(textPayload)" --limit=1000 | wc -l)
    echo -e "  ${CYAN}Webhooks:${NC} ${webhooks}"
    
    echo ""
}

# Função para mostrar menu com cores
show_menu() {
    echo -e "${BLUE}${BOLD}📋 MENU DE OPÇÕES${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "  ${WHITE}1)${NC} ${CYAN}🔴${NC} Ver logs em tempo real"
    echo -e "  ${WHITE}2)${NC} ${BLUE}📜${NC} Ver últimos 100 logs"
    echo -e "  ${WHITE}3)${NC} ${RED}${ERROR}${NC} Buscar logs com erro"
    echo -e "  ${WHITE}4)${NC} ${PURPLE}${AI}${NC} Logs de processamento (Vertex AI)"
    echo -e "  ${WHITE}5)${NC} ${GREEN}${TASK}${NC} Logs de tarefas criadas"
    echo -e "  ${WHITE}6)${NC} ${CYAN}${WEBHOOK}${NC} Logs de webhooks recebidos"
    echo -e "  ${WHITE}7)${NC} ${YELLOW}📊${NC} Ver estatísticas (últimas 24h)"
    echo -e "  ${WHITE}8)${NC} ${BLUE}ℹ️${NC} Status do serviço"
    echo -e "  ${WHITE}9)${NC} ${PURPLE}🔍${NC} Buscar texto específico"
    echo -e "  ${WHITE}q)${NC} ${RED}🚪${NC} Sair"
    echo ""
    echo -e "${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    read -p "$(echo -e ${WHITE}Escolha uma opção: ${NC})" choice
}

# Loop principal
while true; do
    clear
    echo -e "${CYAN}${BOLD}=================================================="
    echo -e "📡 MONITOR DE PRODUÇÃO - CHATGURU CLICKUP"
    echo -e "==================================================${NC}"
    echo -e "${GRAY}Serviço:${NC} ${WHITE}${SERVICE}${NC}"
    echo -e "${GRAY}Região:${NC} ${WHITE}${REGION}${NC}"
    echo -e "${GRAY}Projeto:${NC} ${WHITE}buzzlightear${NC}"
    echo -e "${GRAY}Hora:${NC} ${WHITE}$(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo ""
    
    show_menu
    
    case $choice in
        1)
            clear
            stream_logs
            ;;
        2)
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\"" \
                "📜 ÚLTIMOS 100 LOGS" \
                "$BLUE"
            ;;
        3)
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"ERROR\" OR textPayload:\"WARN\" OR textPayload:\"Failed\")" \
                "${ERROR} LOGS DE ERRO E AVISOS" \
                "$RED"
            ;;
        4)
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"Vertex\" OR textPayload:\"OAuth\" OR textPayload:\"classification\" OR textPayload:\"Gemini\")" \
                "${AI} PROCESSAMENTO VERTEX AI" \
                "$PURPLE"
            ;;
        5)
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"task created\" OR textPayload:\"tarefa criada\" OR textPayload:\"ClickUp\")" \
                "${TASK} TAREFAS CRIADAS NO CLICKUP" \
                "$GREEN"
            ;;
        6)
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND (textPayload:\"webhook\" OR textPayload:\"Webhook\" OR textPayload:\"Request received\")" \
                "${WEBHOOK} WEBHOOKS RECEBIDOS" \
                "$CYAN"
            ;;
        7)
            show_stats
            ;;
        8)
            echo ""
            echo -e "${BLUE}${BOLD}ℹ️ STATUS DO SERVIÇO${NC}"
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            
            # Status básico
            status=$(gcloud run services describe $SERVICE --region=$REGION --format="value(status.conditions[0].status)" 2>/dev/null)
            if [[ "$status" == "True" ]]; then
                echo -e "  ${GREEN}${SUCCESS} Serviço está ATIVO${NC}"
            else
                echo -e "  ${RED}${ERROR} Serviço com problemas${NC}"
            fi
            
            # URL
            url=$(gcloud run services describe $SERVICE --region=$REGION --format="value(status.url)" 2>/dev/null)
            echo -e "  ${WHITE}URL:${NC} ${CYAN}${url}${NC}"
            
            # Última revisão
            revision=$(gcloud run services describe $SERVICE --region=$REGION --format="value(status.latestCreatedRevisionName)" 2>/dev/null)
            echo -e "  ${WHITE}Revisão:${NC} ${revision}"
            
            # Tráfego
            echo -e "  ${WHITE}Tráfego:${NC} 100% para ${revision}"
            ;;
        9)
            echo ""
            read -p "$(echo -e ${WHITE}Digite o texto para buscar: ${NC})" search_text
            format_logs \
                "resource.type=\"cloud_run_revision\" AND resource.labels.service_name=\"$SERVICE\" AND textPayload:\"$search_text\"" \
                "🔍 BUSCA: $search_text" \
                "$PURPLE"
            ;;
        q)
            echo ""
            echo -e "${GREEN}${BOLD}👋 Saindo do monitor...${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}${ERROR} Opção inválida!${NC}"
            sleep 2
            ;;
    esac
    
    echo ""
    echo -e "${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    read -p "$(echo -e ${WHITE}Pressione Enter para continuar...${NC})"
done