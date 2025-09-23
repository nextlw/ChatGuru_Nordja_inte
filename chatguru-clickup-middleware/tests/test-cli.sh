#!/bin/bash

# Script de teste para o ChatGuru Dialog CLI

echo "🧪 Testando ChatGuru Dialog CLI"
echo "================================"
echo ""
echo "Este script demonstra como usar a ferramenta CLI"
echo ""

# Teste 1: Listar diálogos
echo "1️⃣ Listando todos os diálogos..."
echo "1" | node chatguru-dialog-cli.js | head -50

echo ""
echo "================================"
echo "✅ Teste concluído!"
echo ""
echo "Para usar a ferramenta interativamente, execute:"
echo "  node chatguru-dialog-cli.js"
echo ""
echo "Opções disponíveis:"
echo "  1. Listar diálogos"
echo "  2. Ver detalhes"
echo "  3. Criar diálogo"
echo "  4. Atualizar diálogo"
echo "  5. Deletar diálogo"
echo "  6. Configurar webhook"
echo "  7. Testar webhook"
echo "  8. Executar diálogo"
echo "  9. Configurações"
echo "  0. Sair"