// ================================================
// TESTE DE INTEGRAÇÃO COMPLETO - ChatGuru ClickUp Middleware
// ================================================
// Testa a aplicação implantada no Google Cloud Run
// URL: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app
// ================================================

const axios = require('axios');
const chalk = require('chalk');

// ============= CONFIGURAÇÃO =============
const CONFIG = {
    // URL da aplicação no GCP
    baseUrl: 'https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app',
    
    // URL local para testes de desenvolvimento
    localUrl: 'http://localhost:8080',
    
    // Usar produção (GCP) ou local
    useProduction: true,
    
    // Webhook secret para assinatura
    webhookSecret: process.env.WEBHOOK_SECRET || 'test-secret',
    
    // Headers padrão
    headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'ChatGuru-Integration-Test/1.0'
    }
};

// URL base para os testes
const BASE_URL = CONFIG.useProduction ? CONFIG.baseUrl : CONFIG.localUrl;

// ============= UTILITÁRIOS =============

// Cores para output
const log = {
    success: (msg) => console.log(chalk.green('✓'), msg),
    error: (msg) => console.log(chalk.red('✗'), msg),
    info: (msg) => console.log(chalk.blue('ℹ'), msg),
    warning: (msg) => console.log(chalk.yellow('⚠'), msg),
    header: (msg) => console.log(chalk.cyan.bold(`\n${'='.repeat(50)}\n${msg}\n${'='.repeat(50)}`)),
    subheader: (msg) => console.log(chalk.magenta.bold(`\n--- ${msg} ---`))
};

// Função para criar assinatura HMAC
function createWebhookSignature(payload, secret) {
    const crypto = require('crypto');
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(JSON.stringify(payload));
    return 'sha256=' + hmac.digest('hex');
}

// Função para gerar timestamp RFC3339
function generateTimestamp(minutesOffset = 0) {
    const date = new Date();
    date.setMinutes(date.getMinutes() + minutesOffset);
    return date.toISOString();
}

// Função para criar evento ChatGuru válido
function createChatGuruEvent(overrides = {}) {
    const baseEvent = {
        event_id: `test_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        event_type: 'payment_created',
        timestamp: generateTimestamp(),
        data: {
            amount: 100.00,
            customer_name: 'Cliente Teste',
            payment_method: 'pix',
            transaction_id: `trans_${Date.now()}`
        },
        source: 'test_suite',
        metadata: {
            test_run: new Date().toISOString(),
            environment: CONFIG.useProduction ? 'production' : 'local'
        }
    };

    // Merge com overrides
    return {
        ...baseEvent,
        ...overrides,
        data: {
            ...baseEvent.data,
            ...(overrides.data || {})
        },
        metadata: {
            ...baseEvent.metadata,
            ...(overrides.metadata || {})
        }
    };
}

// ============= TESTES DE HEALTH CHECK =============

async function testHealthEndpoints() {
    log.header('TESTES DE HEALTH CHECK');
    
    const endpoints = [
        { path: '/health', description: 'Health Check' },
        { path: '/ready', description: 'Readiness Check' },
        { path: '/status', description: 'Status Check' }
    ];
    
    const results = [];
    
    for (const endpoint of endpoints) {
        log.subheader(`Testando ${endpoint.description}`);
        
        try {
            const response = await axios.get(`${BASE_URL}${endpoint.path}`, {
                headers: CONFIG.headers
            });
            
            if (response.status === 200) {
                log.success(`${endpoint.path} - Status: ${response.status}`);
                log.info(`Resposta: ${JSON.stringify(response.data, null, 2)}`);
                results.push({ endpoint: endpoint.path, status: 'success' });
            } else {
                log.warning(`${endpoint.path} - Status inesperado: ${response.status}`);
                results.push({ endpoint: endpoint.path, status: 'warning' });
            }
        } catch (error) {
            log.error(`${endpoint.path} - Erro: ${error.message}`);
            results.push({ endpoint: endpoint.path, status: 'error', error: error.message });
        }
    }
    
    return results;
}

// ============= TESTES DE VALIDAÇÃO DE WEBHOOK =============

async function testWebhookValidation() {
    log.header('TESTES DE VALIDAÇÃO DE WEBHOOK');
    
    const tests = [
        {
            name: 'Evento válido',
            event: createChatGuruEvent(),
            expectedStatus: [200, 500], // 500 se ClickUp falhar
            shouldPass: true
        },
        {
            name: 'Evento sem event_type',
            event: (() => {
                const event = createChatGuruEvent();
                delete event.event_type;
                return event;
            })(),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento sem event_id',
            event: (() => {
                const event = createChatGuruEvent();
                delete event.event_id;
                return event;
            })(),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento sem timestamp',
            event: (() => {
                const event = createChatGuruEvent();
                delete event.timestamp;
                return event;
            })(),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento com timestamp inválido',
            event: createChatGuruEvent({
                timestamp: 'invalid-timestamp'
            }),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento muito antigo (> 5 minutos)',
            event: createChatGuruEvent({
                timestamp: generateTimestamp(-10) // 10 minutos no passado
            }),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento sem data',
            event: (() => {
                const event = createChatGuruEvent();
                event.data = null;
                return event;
            })(),
            expectedStatus: [400],
            shouldPass: false
        },
        {
            name: 'Evento com data vazia',
            event: createChatGuruEvent({
                data: {}
            }),
            expectedStatus: [400],
            shouldPass: false
        }
    ];
    
    const results = [];
    
    for (const test of tests) {
        log.subheader(`Teste: ${test.name}`);
        
        try {
            const response = await axios.post(
                `${BASE_URL}/webhooks/chatguru`,
                test.event,
                {
                    headers: {
                        ...CONFIG.headers,
                        'X-ChatGuru-Signature': createWebhookSignature(test.event, CONFIG.webhookSecret)
                    },
                    validateStatus: () => true // Aceitar qualquer status
                }
            );
            
            const statusMatch = test.expectedStatus.includes(response.status);
            
            if (statusMatch) {
                log.success(`Status esperado: ${response.status} ∈ [${test.expectedStatus.join(', ')}]`);
                results.push({ test: test.name, status: 'success' });
            } else {
                log.error(`Status inesperado: ${response.status} ∉ [${test.expectedStatus.join(', ')}]`);
                results.push({ test: test.name, status: 'error', actual: response.status });
            }
            
            if (response.data) {
                log.info(`Resposta: ${JSON.stringify(response.data, null, 2)}`);
            }
        } catch (error) {
            log.error(`Erro na requisição: ${error.message}`);
            results.push({ test: test.name, status: 'error', error: error.message });
        }
    }
    
    return results;
}

// ============= TESTES DE DIFERENTES TIPOS DE EVENTOS =============

async function testEventTypes() {
    log.header('TESTES DE DIFERENTES TIPOS DE EVENTOS');
    
    const eventTypes = [
        {
            type: 'payment_created',
            data: {
                amount: 299.90,
                customer_name: 'João Silva',
                payment_method: 'credit_card',
                transaction_id: 'txn_001'
            }
        },
        {
            type: 'payment_completed',
            data: {
                amount: 150.00,
                customer_name: 'Maria Santos',
                payment_method: 'pix',
                transaction_id: 'txn_002'
            }
        },
        {
            type: 'payment_failed',
            data: {
                amount: 500.00,
                customer_name: 'Pedro Oliveira',
                reason: 'Saldo insuficiente',
                transaction_id: 'txn_003'
            }
        },
        {
            type: 'pix_received',
            data: {
                amount: 75.50,
                sender_name: 'Ana Costa',
                pix_key: 'email@example.com'
            }
        },
        {
            type: 'customer_created',
            data: {
                name: 'Carlos Ferreira',
                email: 'carlos@example.com',
                phone: '+5511999999999'
            }
        },
        {
            type: 'invoice_generated',
            data: {
                invoice_number: 'NF-2024-001',
                amount: 1200.00,
                customer_name: 'Empresa ABC Ltda',
                invoice_url: 'https://example.com/invoice/001'
            }
        },
        {
            type: 'new_lead',
            data: {
                lead_name: 'Roberto Almeida',
                project_name: 'Projeto X',
                phone: '+5511888888888'
            }
        },
        {
            type: 'appointment_scheduled',
            data: {
                lead_name: 'Juliana Martins',
                appointment_type: 'Consultoria',
                date: '2024-12-01T14:00:00Z'
            }
        },
        {
            type: 'status_change',
            data: {
                lead_name: 'Fernando Lima',
                old_status: 'Em análise',
                new_status: 'Aprovado'
            }
        },
        {
            type: 'mensagem_recebida',
            data: {
                sender_name: 'Cliente WhatsApp',
                chat_name: 'Chat #123',
                message: 'Gostaria de mais informações sobre o produto'
            }
        }
    ];
    
    const results = [];
    
    for (const eventType of eventTypes) {
        log.subheader(`Testando evento: ${eventType.type}`);
        
        const event = createChatGuruEvent({
            event_type: eventType.type,
            data: eventType.data
        });
        
        try {
            const response = await axios.post(
                `${BASE_URL}/webhooks/chatguru`,
                event,
                {
                    headers: {
                        ...CONFIG.headers,
                        'X-ChatGuru-Signature': createWebhookSignature(event, CONFIG.webhookSecret)
                    },
                    validateStatus: () => true
                }
            );
            
            if (response.status === 200 || response.status === 500) {
                log.success(`Evento ${eventType.type} processado - Status: ${response.status}`);
                
                if (response.data) {
                    log.info(`Título gerado: ${extractTaskTitle(response.data)}`);
                    if (response.data.clickup_task_id) {
                        log.info(`Task ID no ClickUp: ${response.data.clickup_task_id}`);
                    }
                }
                
                results.push({ event: eventType.type, status: 'success' });
            } else {
                log.error(`Evento ${eventType.type} - Status inesperado: ${response.status}`);
                results.push({ event: eventType.type, status: 'error', httpStatus: response.status });
            }
        } catch (error) {
            log.error(`Erro ao processar evento ${eventType.type}: ${error.message}`);
            results.push({ event: eventType.type, status: 'error', error: error.message });
        }
    }
    
    return results;
}

// ============= TESTE DE ATUALIZAÇÃO DE TAREFA (MESMO TÍTULO) =============

async function testTaskUpdate() {
    log.header('TESTE DE ATUALIZAÇÃO DE TAREFA EXISTENTE');
    
    const taskTitle = 'Task para Atualização - ' + Date.now();
    
    // Primeiro evento - criar tarefa
    const firstEvent = createChatGuruEvent({
        event_type: 'new_lead',
        data: {
            task_title: taskTitle,
            lead_name: 'Cliente Original',
            project_name: 'Projeto Inicial'
        }
    });
    
    // Segundo evento - atualizar tarefa (mesmo título)
    const secondEvent = createChatGuruEvent({
        event_type: 'status_change',
        data: {
            task_title: taskTitle,
            lead_name: 'Cliente Original',
            new_status: 'Status Atualizado',
            additional_info: 'Informações adicionais da atualização'
        }
    });
    
    const results = [];
    
    try {
        // 1. Criar tarefa inicial
        log.subheader('Criando tarefa inicial');
        
        const firstResponse = await axios.post(
            `${BASE_URL}/webhooks/chatguru`,
            firstEvent,
            {
                headers: {
                    ...CONFIG.headers,
                    'X-ChatGuru-Signature': createWebhookSignature(firstEvent, CONFIG.webhookSecret)
                },
                validateStatus: () => true
            }
        );
        
        if (firstResponse.status === 200) {
            log.success('Tarefa criada com sucesso');
            const taskId = firstResponse.data.clickup_task_id;
            if (taskId) {
                log.info(`Task ID: ${taskId}`);
            }
            results.push({ step: 'create', status: 'success', taskId });
        } else {
            log.error(`Falha ao criar tarefa - Status: ${firstResponse.status}`);
            results.push({ step: 'create', status: 'error' });
            return results;
        }
        
        // Aguardar um pouco antes de atualizar
        log.info('Aguardando 3 segundos antes de atualizar...');
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // 2. Atualizar tarefa existente
        log.subheader('Atualizando tarefa existente');
        
        const secondResponse = await axios.post(
            `${BASE_URL}/webhooks/chatguru`,
            secondEvent,
            {
                headers: {
                    ...CONFIG.headers,
                    'X-ChatGuru-Signature': createWebhookSignature(secondEvent, CONFIG.webhookSecret)
                },
                validateStatus: () => true
            }
        );
        
        if (secondResponse.status === 200) {
            log.success('Tarefa atualizada com sucesso');
            
            const action = secondResponse.data.clickup_task_action;
            if (action === 'updated') {
                log.success('Confirmado: Tarefa foi ATUALIZADA (não criada nova)');
                results.push({ step: 'update', status: 'success', action });
            } else {
                log.warning(`Ação inesperada: ${action}`);
                results.push({ step: 'update', status: 'warning', action });
            }
        } else {
            log.error(`Falha ao atualizar tarefa - Status: ${secondResponse.status}`);
            results.push({ step: 'update', status: 'error' });
        }
        
    } catch (error) {
        log.error(`Erro no teste de atualização: ${error.message}`);
        results.push({ step: 'error', status: 'error', error: error.message });
    }
    
    return results;
}

// ============= TESTE DE ENDPOINTS CLICKUP =============

async function testClickUpEndpoints() {
    log.header('TESTE DE ENDPOINTS CLICKUP');
    
    const endpoints = [
        { path: '/clickup/test', description: 'Teste de Conexão ClickUp' },
        { path: '/clickup/list', description: 'Informações da Lista' },
        { path: '/clickup/tasks', description: 'Listar Tarefas' }
    ];
    
    const results = [];
    
    for (const endpoint of endpoints) {
        log.subheader(`Testando ${endpoint.description}`);
        
        try {
            const response = await axios.get(`${BASE_URL}${endpoint.path}`, {
                headers: CONFIG.headers,
                validateStatus: () => true
            });
            
            if (response.status === 200) {
                log.success(`${endpoint.path} - Status: ${response.status}`);
                
                // Mostrar informações relevantes
                if (endpoint.path === '/clickup/test' && response.data.user) {
                    log.info(`Usuário ClickUp: ${response.data.user.username}`);
                }
                if (endpoint.path === '/clickup/list' && response.data.name) {
                    log.info(`Lista: ${response.data.name}`);
                }
                if (endpoint.path === '/clickup/tasks' && response.data.tasks) {
                    log.info(`Tarefas na lista: ${response.data.tasks.length}`);
                }
                
                results.push({ endpoint: endpoint.path, status: 'success' });
            } else {
                log.error(`${endpoint.path} - Status: ${response.status}`);
                if (response.data && response.data.error) {
                    log.error(`Erro: ${response.data.error}`);
                }
                results.push({ endpoint: endpoint.path, status: 'error', httpStatus: response.status });
            }
        } catch (error) {
            log.error(`${endpoint.path} - Erro: ${error.message}`);
            results.push({ endpoint: endpoint.path, status: 'error', error: error.message });
        }
    }
    
    return results;
}

// ============= UTILITÁRIOS AUXILIARES =============

function extractTaskTitle(responseData) {
    if (!responseData) return 'N/A';
    
    // Tentar extrair o título da resposta
    if (responseData.clickup_task_name) {
        return responseData.clickup_task_name;
    }
    
    return 'Título não disponível';
}

// ============= RELATÓRIO FINAL =============

function generateReport(allResults) {
    log.header('RELATÓRIO FINAL DE TESTES');
    
    const summary = {
        totalTests: 0,
        passed: 0,
        failed: 0,
        warnings: 0
    };
    
    // Processar resultados
    Object.values(allResults).forEach(categoryResults => {
        categoryResults.forEach(result => {
            summary.totalTests++;
            if (result.status === 'success') {
                summary.passed++;
            } else if (result.status === 'warning') {
                summary.warnings++;
            } else {
                summary.failed++;
            }
        });
    });
    
    // Exibir resumo
    console.log('\n' + '='.repeat(50));
    console.log(chalk.cyan.bold('RESUMO DOS TESTES'));
    console.log('='.repeat(50));
    
    console.log(`Total de testes: ${summary.totalTests}`);
    console.log(chalk.green(`✓ Aprovados: ${summary.passed}`));
    console.log(chalk.yellow(`⚠ Avisos: ${summary.warnings}`));
    console.log(chalk.red(`✗ Falhados: ${summary.failed}`));
    
    const successRate = ((summary.passed / summary.totalTests) * 100).toFixed(2);
    console.log(`\nTaxa de sucesso: ${successRate}%`);
    
    if (successRate >= 80) {
        console.log(chalk.green.bold('\n✨ TESTES APROVADOS! ✨'));
    } else if (successRate >= 60) {
        console.log(chalk.yellow.bold('\n⚠️ TESTES COM AVISOS ⚠️'));
    } else {
        console.log(chalk.red.bold('\n❌ TESTES REPROVADOS ❌'));
    }
    
    // Detalhes por categoria
    console.log('\n' + '='.repeat(50));
    console.log(chalk.cyan.bold('DETALHES POR CATEGORIA'));
    console.log('='.repeat(50));
    
    Object.entries(allResults).forEach(([category, results]) => {
        const passed = results.filter(r => r.status === 'success').length;
        const total = results.length;
        console.log(`\n${category}: ${passed}/${total} aprovados`);
    });
}

// ============= EXECUÇÃO PRINCIPAL =============

async function runAllTests() {
    console.log(chalk.cyan.bold('\n' + '='.repeat(60)));
    console.log(chalk.cyan.bold('INICIANDO TESTES DE INTEGRAÇÃO - CHATGURU CLICKUP MIDDLEWARE'));
    console.log(chalk.cyan.bold('='.repeat(60)));
    
    console.log(`\nAmbiente: ${CONFIG.useProduction ? 'PRODUÇÃO (GCP)' : 'LOCAL'}`);
    console.log(`URL Base: ${BASE_URL}`);
    console.log(`Timestamp: ${new Date().toISOString()}\n`);
    
    const allResults = {};
    
    try {
        // 1. Health Checks
        allResults['Health Checks'] = await testHealthEndpoints();
        
        // 2. Validação de Webhook
        allResults['Validação de Webhook'] = await testWebhookValidation();
        
        // 3. Tipos de Eventos
        allResults['Tipos de Eventos'] = await testEventTypes();
        
        // 4. Atualização de Tarefa
        allResults['Atualização de Tarefa'] = await testTaskUpdate();
        
        // 5. Endpoints ClickUp
        allResults['Endpoints ClickUp'] = await testClickUpEndpoints();
        
    } catch (error) {
        log.error(`Erro durante execução dos testes: ${error.message}`);
    }
    
    // Gerar relatório final
    generateReport(allResults);
}

// ============= VERIFICAR DEPENDÊNCIAS =============

async function checkDependencies() {
    const dependencies = ['axios', 'chalk'];
    const missing = [];
    
    for (const dep of dependencies) {
        try {
            require(dep);
        } catch (error) {
            missing.push(dep);
        }
    }
    
    if (missing.length > 0) {
        console.log('⚠️ Dependências faltando. Instalando...');
        const { exec } = require('child_process');
        
        return new Promise((resolve) => {
            exec(`npm install ${missing.join(' ')}`, (error, stdout, stderr) => {
                if (error) {
                    console.error(`Erro ao instalar dependências: ${error}`);
                    process.exit(1);
                }
                console.log('✓ Dependências instaladas com sucesso');
                resolve();
            });
        });
    }
}

// ============= PONTO DE ENTRADA =============

(async () => {
    // Verificar e instalar dependências se necessário
    await checkDependencies();
    
    // Executar todos os testes
    await runAllTests();
})();