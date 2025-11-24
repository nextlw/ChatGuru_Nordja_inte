#!/bin/bash

# Script para limpar e reiniciar o Pub/Sub do projeto buzzlightear
# Uso: ./clear-and-restart-pubsub.sh

set -e

PROJECT_ID="buzzlightear"
TOPIC_CHATGURU="chatguru-webhook-raw"
SUBSCRIPTION_WORKER="chatguru-events-subscription"

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧹 Limpando e reiniciando Pub/Sub do projeto ${PROJECT_ID}${NC}"
echo ""

# Verificar se está autenticado
echo -e "${BLUE}🔐 Verificando autenticação...${NC}"
if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
    echo -e "${RED}❌ Você precisa estar autenticado no gcloud${NC}"
    echo -e "   Execute: ${YELLOW}gcloud auth login${NC}"
    exit 1
fi

# Configurar projeto
echo -e "${BLUE}⚙️  Configurando projeto: ${PROJECT_ID}${NC}"
gcloud config set project ${PROJECT_ID}

# Listar subscriptions existentes
echo -e "${BLUE}📋 Listando subscriptions existentes...${NC}"
SUBSCRIPTIONS=$(gcloud pubsub subscriptions list --project=${PROJECT_ID} --format="value(name)" 2>/dev/null || echo "")

if [ -z "$SUBSCRIPTIONS" ]; then
    echo -e "${YELLOW}⚠️  Nenhuma subscription encontrada${NC}"
else
    echo -e "${GREEN}✅ Subscriptions encontradas:${NC}"
    echo "$SUBSCRIPTIONS" | while read sub; do
        if [ ! -z "$sub" ]; then
            echo "   - $sub"
        fi
    done
fi

# Limpar mensagens pendentes de todas as subscriptions
echo ""
echo -e "${BLUE}🗑️  Limpando mensagens pendentes...${NC}"
for sub in $SUBSCRIPTIONS; do
    if [ ! -z "$sub" ]; then
        echo -e "   Limpando subscription: ${YELLOW}$sub${NC}"
        
        # Contar mensagens pendentes
        PENDING=$(gcloud pubsub subscriptions describe "$sub" --project=${PROJECT_ID} --format="value(numUndeliveredMessages)" 2>/dev/null || echo "0")
        
        if [ "$PENDING" != "0" ] && [ ! -z "$PENDING" ]; then
            echo -e "      ${YELLOW}⚠️  $PENDING mensagens pendentes${NC}"
            
            # Fazer seek para limpar mensagens (mover para o final)
            gcloud pubsub subscriptions seek "$sub" \
                --time=$(date -u +%Y-%m-%dT%H:%M:%SZ) \
                --project=${PROJECT_ID} 2>/dev/null && \
                echo -e "      ${GREEN}✅ Mensagens limpas${NC}" || \
                echo -e "      ${YELLOW}⚠️  Não foi possível limpar (pode não ter mensagens)${NC}"
        else
            echo -e "      ${GREEN}✅ Nenhuma mensagem pendente${NC}"
        fi
    fi
done

# Verificar e criar tópico se não existir
echo ""
echo -e "${BLUE}📝 Verificando tópico ${TOPIC_CHATGURU}...${NC}"
if gcloud pubsub topics describe ${TOPIC_CHATGURU} --project=${PROJECT_ID} &>/dev/null; then
    echo -e "   ${GREEN}✅ Tópico ${TOPIC_CHATGURU} existe${NC}"
else
    echo -e "   ${YELLOW}⚠️  Tópico não existe, criando...${NC}"
    gcloud pubsub topics create ${TOPIC_CHATGURU} --project=${PROJECT_ID}
    echo -e "   ${GREEN}✅ Tópico criado${NC}"
fi

# Verificar e criar subscription se não existir
echo ""
echo -e "${BLUE}📬 Verificando subscription ${SUBSCRIPTION_WORKER}...${NC}"
if gcloud pubsub subscriptions describe ${SUBSCRIPTION_WORKER} --project=${PROJECT_ID} &>/dev/null; then
    echo -e "   ${GREEN}✅ Subscription ${SUBSCRIPTION_WORKER} existe${NC}"
    
    # Recriar subscription para limpar completamente
    echo -e "   ${YELLOW}🔄 Recriando subscription para limpar completamente...${NC}"
    gcloud pubsub subscriptions delete ${SUBSCRIPTION_WORKER} --project=${PROJECT_ID} 2>/dev/null || true
    sleep 2
fi

# Criar subscription nova
echo -e "   ${BLUE}📝 Criando subscription ${SUBSCRIPTION_WORKER}...${NC}"
gcloud pubsub subscriptions create ${SUBSCRIPTION_WORKER} \
    --topic=${TOPIC_CHATGURU} \
    --ack-deadline=60 \
    --project=${PROJECT_ID} 2>/dev/null || \
    echo -e "   ${YELLOW}⚠️  Subscription já existe ou erro ao criar${NC}"

echo ""
echo -e "${GREEN}✅ Pub/Sub limpo e reiniciado com sucesso!${NC}"
echo ""
echo -e "${BLUE}📊 Status final:${NC}"
echo -e "   Projeto: ${YELLOW}${PROJECT_ID}${NC}"
echo -e "   Tópico: ${YELLOW}${TOPIC_CHATGURU}${NC}"
echo -e "   Subscription: ${YELLOW}${SUBSCRIPTION_WORKER}${NC}"
echo ""

