#!/bin/bash

# Script para configurar atualização automática dos campos do ClickUp
# Pode ser executado via cron ou systemd timer

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
UPDATE_SCRIPT="$SCRIPT_DIR/update_clickup_fields.js"

echo "🔧 Configurando atualização automática dos campos do ClickUp"

# Opção 1: Adicionar ao crontab (executa a cada 6 horas)
read -p "Deseja adicionar ao crontab para executar a cada 6 horas? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    # Adiciona ao crontab se não existir
    CRON_CMD="0 */6 * * * cd $SCRIPT_DIR && /usr/bin/node update_clickup_fields.js >> /tmp/clickup_fields_update.log 2>&1"
    
    # Verifica se já existe
    if ! crontab -l 2>/dev/null | grep -q "update_clickup_fields.js"; then
        (crontab -l 2>/dev/null; echo "$CRON_CMD") | crontab -
        echo "✅ Adicionado ao crontab com sucesso!"
        echo "   Execução: a cada 6 horas"
        echo "   Logs em: /tmp/clickup_fields_update.log"
    else
        echo "⚠️  Já existe uma entrada no crontab para este script"
    fi
fi

# Opção 2: Criar script de atualização manual
MANUAL_SCRIPT="$SCRIPT_DIR/../update_fields.sh"
cat > "$MANUAL_SCRIPT" << 'EOF'
#!/bin/bash
# Script para atualizar manualmente os campos do ClickUp

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

echo "🔄 Atualizando campos do ClickUp..."
cd "$SCRIPT_DIR/scripts" && npm run update

if [ $? -eq 0 ]; then
    echo "✅ Campos atualizados com sucesso!"
    echo "📁 Arquivo: config/clickup_fields_static.yaml"
else
    echo "❌ Erro ao atualizar campos"
    exit 1
fi
EOF

chmod +x "$MANUAL_SCRIPT"
echo "✅ Script de atualização manual criado: update_fields.sh"

# Executar uma vez agora
read -p "Deseja executar a atualização agora? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
    cd "$SCRIPT_DIR" && node update_clickup_fields.js
fi

echo ""
echo "📋 Resumo da configuração:"
echo "   - Script de atualização: scripts/update_clickup_fields.js"
echo "   - Arquivo de saída: config/clickup_fields_static.yaml"
echo "   - Backup automático: config/clickup_fields_static.backup.yaml"
echo ""
echo "💡 Comandos úteis:"
echo "   - Atualizar manualmente: ./update_fields.sh"
echo "   - Ver logs do cron: tail -f /tmp/clickup_fields_update.log"
echo "   - Listar crontab: crontab -l"
echo ""
echo "✨ Configuração concluída!"