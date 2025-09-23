#!/usr/bin/env node

/**
 * Exemplo de webhook que retorna instrução para o ChatGuru criar anotação
 * Este código pode ser usado no webhook buzzlightear ou em um novo webhook
 */

const express = require('express');
const app = express();

app.use(express.json());

// Webhook que recebe eventos do diálogo nova_api
app.post('/webhook', (req, res) => {
    console.log('Webhook recebido:', JSON.stringify(req.body, null, 2));
    
    const { 
        event_type, 
        dialog_id, 
        chat_number, 
        variables = {},
        account,
        phone 
    } = req.body;
    
    // Verificar se é o diálogo nova_api
    if (dialog_id === 'nova_api') {
        // Processar as variáveis e criar texto formatado para anotação
        const anotacao = formatarAnotacao(variables);
        
        // OPÇÃO 1: Retornar instrução para o ChatGuru criar anotação
        // (Funciona se o ChatGuru suportar este formato de resposta)
        res.json({
            success: true,
            action: 'add_annotation',
            annotation: {
                text: anotacao,
                type: 'task',
                priority: variables.prioridade || 'Normal'
            },
            message: '✅ Tarefa registrada com sucesso!'
        });
        
        // OPÇÃO 2: Apenas confirmar recebimento
        // (Se o ChatGuru já está configurado para criar anotação automaticamente)
        /*
        res.json({
            success: true,
            message: 'Tarefa processada'
        });
        */
        
        // OPÇÃO 3: Chamar API do ChatGuru para adicionar anotação
        // (Descomentar se preferir esta abordagem)
        /*
        adicionarAnotacaoViaChatGuruAPI(chat_number, anotacao)
            .then(() => {
                res.json({ success: true });
            })
            .catch(error => {
                res.status(500).json({ error: error.message });
            });
        */
    } else {
        // Outros diálogos
        res.json({
            success: true,
            message: 'Evento recebido'
        });
    }
});

// Função para formatar a anotação
function formatarAnotacao(variables) {
    const {
        tarefa = 'Tarefa não especificada',
        tipo_atividade = 'Geral',
        categoria = 'Atividades em geral',
        prioridade = 'Normal',
        responsavel = 'A definir',
        descricao = '',
        subtarefas = []
    } = variables;
    
    let texto = `📋 NOVA TAREFA IDENTIFICADA
━━━━━━━━━━━━━━━━━━━━━━
📌 Tarefa: ${tarefa}
📊 Tipo: ${tipo_atividade}
📁 Categoria: ${categoria}
🔴 Prioridade: ${prioridade}
👤 Responsável: ${responsavel}`;
    
    if (descricao) {
        texto += `\n📝 Descrição: ${descricao}`;
    }
    
    texto += '\n\n📍 Subtarefas:';
    
    if (Array.isArray(subtarefas) && subtarefas.length > 0) {
        subtarefas.forEach(subtarefa => {
            texto += `\n  • ${subtarefa}`;
        });
    } else if (variables.subtarefa1 || variables.subtarefa2) {
        if (variables.subtarefa1) texto += `\n  • ${variables.subtarefa1}`;
        if (variables.subtarefa2) texto += `\n  • ${variables.subtarefa2}`;
    } else {
        texto += '\n  • Análise inicial\n  • Implementação\n  • Testes';
    }
    
    texto += `\n\n⏰ Criado em: ${new Date().toLocaleString('pt-BR')}`;
    texto += '\n━━━━━━━━━━━━━━━━━━━━━━';
    
    return texto;
}

// Função para chamar API do ChatGuru (opcional)
async function adicionarAnotacaoViaChatGuruAPI(chatNumber, anotacao) {
    const https = require('https');
    
    const config = {
        apiKey: 'TXUKDWXS92SSN9KP3MCLX9AADSXAYVGB2MWWER0ESYNRZE80ZNLUQ9HYCXKXQ1BK',
        accountId: '625584ce6fdcb7bda7d94aa8',
        phoneId: '6537de23b6d5b0bb0b80421a'
    };
    
    const payload = JSON.stringify({
        chat_number: chatNumber,
        note: anotacao,
        key: config.apiKey,
        account_id: config.accountId,
        phone_id: config.phoneId
    });
    
    return new Promise((resolve, reject) => {
        const options = {
            hostname: 's15.chatguru.app',
            port: 443,
            path: '/note_add',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(payload),
                'APIKey': config.apiKey
            }
        };
        
        const req = https.request(options, (res) => {
            let data = '';
            res.on('data', chunk => data += chunk);
            res.on('end', () => {
                if (res.statusCode === 200 || res.statusCode === 201) {
                    resolve(JSON.parse(data));
                } else {
                    reject(new Error(`API retornou status ${res.statusCode}: ${data}`));
                }
            });
        });
        
        req.on('error', reject);
        req.write(payload);
        req.end();
    });
}

// Health check
app.get('/health', (req, res) => {
    res.json({ status: 'ok', service: 'nova-api-webhook' });
});

// Iniciar servidor (apenas para teste local)
if (require.main === module) {
    const PORT = process.env.PORT || 3000;
    app.listen(PORT, () => {
        console.log(`
╔════════════════════════════════════════════════╗
║   WEBHOOK NOVA_API COM ANOTAÇÃO                ║
╚════════════════════════════════════════════════╝

Servidor rodando na porta ${PORT}

Endpoints disponíveis:
- POST /webhook - Recebe eventos do ChatGuru
- GET /health - Health check

Teste com:
curl -X POST http://localhost:${PORT}/webhook \\
  -H "Content-Type: application/json" \\
  -d '{
    "dialog_id": "nova_api",
    "chat_number": "5585989530473",
    "variables": {
      "tarefa": "Teste de tarefa",
      "tipo_atividade": "Desenvolvimento",
      "categoria": "Backend",
      "prioridade": "Alta"
    }
  }'
        `);
    });
}

module.exports = app;