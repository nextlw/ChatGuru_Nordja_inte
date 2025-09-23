#!/bin/bash

# ==============================================================================
# QUICK DEPLOY - Script Simplificado para Deploy Rápido
# ==============================================================================

echo "======================================"
echo "   QUICK DEPLOY - ChatGuru ClickUp   "
echo "======================================"
echo ""

# Verificar se está no diretório correto
if [ ! -f "Cargo.toml" ]; then
    echo "❌ ERRO: Execute este script do diretório chatguru-clickup-middleware/"
    exit 1
fi

echo "📍 Diretório: $(pwd)"
echo ""

# Opções de deployment
echo "Escolha uma opção de deployment:"
echo ""
echo "1) Deploy COMPLETO com Artifact Registry (Recomendado)"
echo "2) Deploy COMPLETO com Container Registry" 
echo "3) Deploy DIRETO do código fonte (mais simples)"
echo "4) Apenas testar localmente com Docker"
echo "5) Ver status do serviço existente"
echo ""

read -p "Digite sua escolha (1-5): " choice

case $choice in
    1)
        echo "🚀 Iniciando deploy com Artifact Registry..."
        ./deploy-artifact-registry.sh
        ;;
    2)
        echo "🚀 Iniciando deploy com Container Registry..."
        ./deploy-gcp.sh
        ;;
    3)
        echo "🚀 Deploy direto do código fonte..."
        echo ""
        echo "⚠️  Este método é mais simples mas pode demorar mais"
        echo ""
        
        # Autenticar se necessário
        if ! gcloud auth list --filter=status:ACTIVE --format="value(account)" | grep -q .; then
            echo "🔐 Fazendo login no Google Cloud..."
            gcloud auth login
        fi
        
        # Configurar projeto
        gcloud config set project buzzlightear
        
        # Deploy direto
        echo "📦 Fazendo build e deploy (isso pode levar alguns minutos)..."
        gcloud run deploy chatguru-clickup-middleware \
            --source . \
            --region southamerica-east1 \
            --allow-unauthenticated \
            --set-env-vars "CLICKUP_API_TOKEN=pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657" \
            --set-env-vars "CLICKUP_LIST_ID=901300373349" \
            --set-env-vars "RUST_LOG=info" \
            --set-env-vars "PORT=8080"
        
        # Obter URL
        SERVICE_URL=$(gcloud run services describe chatguru-clickup-middleware \
            --region southamerica-east1 \
            --format 'value(status.url)')
        
        echo ""
        echo "✅ Deploy concluído!"
        echo "🌐 URL do serviço: $SERVICE_URL"
        echo ""
        echo "📝 Configure no ChatGuru:"
        echo "   URL: ${SERVICE_URL}/webhooks/chatguru"
        echo "   Método: POST"
        ;;
    4)
        echo "🐳 Testando localmente com Docker..."
        echo ""
        
        # Build local
        docker build -t chatguru-clickup-local .
        
        # Run local
        echo "🏃 Rodando container local na porta 8080..."
        docker run -p 8080:8080 \
            -e CLICKUP_API_TOKEN="pk_106092691_UC96HSQJH4ZUS3NJATYHIQ06BQQYM657" \
            -e CLICKUP_LIST_ID="901300373349" \
            -e RUST_LOG="debug" \
            -e PORT="8080" \
            chatguru-clickup-local
        ;;
    5)
        echo "📊 Verificando status do serviço..."
        echo ""
        
        # Verificar se o serviço existe
        if gcloud run services describe chatguru-clickup-middleware \
            --region southamerica-east1 \
            --project buzzlightear &> /dev/null; then
            
            # Obter informações
            SERVICE_URL=$(gcloud run services describe chatguru-clickup-middleware \
                --region southamerica-east1 \
                --format 'value(status.url)')
            
            echo "✅ Serviço encontrado!"
            echo "🌐 URL: $SERVICE_URL"
            echo ""
            
            # Testar health check
            echo "🏥 Testando health check..."
            curl -s "${SERVICE_URL}/health" | jq . || echo "Health check falhou"
            
            echo ""
            echo "📝 Últimos logs:"
            gcloud logs read --service=chatguru-clickup-middleware \
                --project buzzlightear \
                --limit=10
        else
            echo "❌ Serviço não encontrado no Cloud Run"
            echo "Execute uma das opções de deploy (1-3)"
        fi
        ;;
    *)
        echo "❌ Opção inválida"
        exit 1
        ;;
esac

echo ""
echo "======================================"
echo "        Processo Finalizado           "
echo "======================================"