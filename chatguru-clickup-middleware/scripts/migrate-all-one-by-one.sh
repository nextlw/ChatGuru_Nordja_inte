#!/bin/bash

echo "🚀 MIGRAÇÃO DE TODAS AS CATEGORIAS - UMA POR VEZ"
echo "================================================"
echo ""

# Array com todos os IDs antigos das categorias
CATEGORIAS=(
  "a4a4e85c-4eb5-44f9-9175-f98594da5c70"  # ADM
  "25102629-2b4f-4863-847c-51468e484362"  # Agendamento
  "e0b80ba4-3b2e-47bc-9a4f-d322045d6480"  # Atividades Corporativas
  "1b61be8c-6eaa-4cd4-959d-c48f8b78ca3e"  # Atividades Pessoais / Domésticas
  "4799d43c-c1be-478e-ad58-57fdfa95b292"  # Compras
  "e57a5888-49cc-4c3a-bb4a-e235c8c58e77"  # Documentos
  "4232858d-8052-4fdc-9ce9-628f34d2edac"  # Educação / Academia
  "07c402c5-1f0b-4ae0-b38e-349553fd7a81"  # Eventos Corporativos
  "2330057a-4637-4ab6-94af-905904fdb4c3"  # Gestão de Funcionário Doméstico
  "68539506-88b8-44a5-bac8-0496b2b2f148"  # Lazer
  "a1bc0a49-2a9d-41bd-a91a-a6b4af7677b4"  # Logistica
  "18c2c60c-cfd0-4ef8-af94-7542bd9b30c7"  # Festas / Reuniões / Recepção
  "b64b9e80-fdc4-4521-8975-45e335109b49"  # Pagamentos
  "80ad2f74-7074-4eec-a4fd-8fc92d0fe0dd"  # Pesquisas / Orçamentos
  "2aebf637-6534-487c-bd35-d28334c8d685"  # Viagens
  "3496ba6c-9d7f-495f-951e-6017dfdbd55b"  # Plano de Saúde
  "16c6bd8a-05d3-4bd7-8fd9-906ef3e8b2d2"  # Controle Interno
  "8fb02d9e-febb-4b94-9e9e-cdf8ff5a28fb"  # Compra/Venda/Aluguel
  "14f5e8ff-df26-4446-8766-053e47c001c7"  # Atividade Baixada/Duplicada
)

TOTAL=${#CATEGORIAS[@]}
CONTADOR=1

for CAT_ID in "${CATEGORIAS[@]}"; do
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "[$CONTADOR/$TOTAL] Processando categoria: $CAT_ID"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  node migrate-categoria-individual.js "$CAT_ID"

  CONTADOR=$((CONTADOR + 1))

  echo ""
  echo "Aguardando 2 segundos antes da próxima..."
  sleep 2
done

echo ""
echo "✅ TODAS AS CATEGORIAS PROCESSADAS!"
echo ""
