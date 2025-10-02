# 🧪 Guia de Testes - ChatGuru ClickUp Middleware

## 📋 Scripts de Teste Disponíveis

### 1. `test-scenarios.js` - Cenários de Mundo Real
Testa fluxos completos de conversação e interações de usuário.

#### Uso:
```bash
# Mostrar menu de cenários disponíveis
node test-scenarios.js

# Executar cenário específico
node test-scenarios.js 1    # Agregação de mensagens curtas
node test-scenarios.js 2    # Transcrição de áudio
node test-scenarios.js 3    # Pedido completo válido
node test-scenarios.js 4    # Modificação de pedido (embeddings)
node test-scenarios.js 5    # Conversas casuais (não criar tarefas)
node test-scenarios.js 6    # Pedido de orçamento
node test-scenarios.js 7    # Adições ao pedido
node test-scenarios.js 8    # Mensagens ambíguas
node test-scenarios.js 9    # Pedido urgente
node test-scenarios.js 10   # Cancelamento

# Executar TODOS os cenários
node test-scenarios.js all
```

#### Cenários Testados:
1. **Agregação de Mensagens Curtas** - Teste contexto com ≤3 palavras
2. **Transcrição de Áudio** - Vertex AI → Whisper fallback
3. **Pedido Completo Válido** - Criação de tarefa normal
4. **Modificação de Pedido** - Detecção por embeddings (similaridade > 0.7)
5. **Conversas Casuais** - Não deve criar tarefas
6. **Pedido de Orçamento** - Classificação de serviços
7. **Adições ao Pedido** - Baixa similaridade = nova tarefa
8. **Mensagens Ambíguas** - Teste de classificação IA
9. **Pedido Urgente** - Detecção de prioridade
10. **Cancelamento** - Modificação com cancelamento

### 2. `test-ai-features.js` - Funcionalidades de IA
Testes focados nas capacidades de IA do sistema.

#### Uso:
```bash
# Mostrar menu de testes
node test-ai-features.js

# Executar teste específico
node test-ai-features.js 1    # Fluxo de transcrição de áudio
node test-ai-features.js 2    # Embeddings e similaridade
node test-ai-features.js 3    # Agregação de contexto
node test-ai-features.js 4    # Fallback seguro

# Executar TODOS os testes
node test-ai-features.js all
```

#### Testes de IA:
1. **Transcrição de Áudio** - Pipeline completo Vertex AI + Whisper
2. **Embeddings e Similaridade** - Detecção semântica de modificações
3. **Agregação de Contexto** - Mensagens curtas agregam histórico
4. **Fallback Seguro** - Comportamento quando IA indisponível

### 3. `test_production.sh` - Teste Completo de Produção (legado)
Script anterior para validação básica do sistema.

#### Uso:
```bash
# Teste local
./test_production.sh

# Teste em produção
./test_production.sh prod
```

### 4. `monitor_prod.sh` - Monitor de Produção
Ferramenta interativa para monitorar logs e métricas em produção.

#### Funcionalidades:
- Stream de logs em tempo real
- Busca por erros
- Monitoramento de processamento Vertex AI
- Visualização de tarefas criadas
- Status do serviço
- Métricas de requisições

#### Uso:
```bash
./monitor_prod.sh
```

## 🔍 Verificações de Teste

### ✅ Critérios de Sucesso

#### 1. **Classificação Correta**
- Mensagens com pedidos/solicitações devem gerar tarefas (`is_activity: true`)
- Saudações/agradecimentos não devem gerar tarefas (`is_activity: false`)
- Categoria deve corresponder ao conteúdo

#### 2. **Formato da Resposta**
Exemplo esperado para criação de tarefa:
```json
{
  "action": "created",
  "task_id": "abc123",
  "task_url": "https://app.clickup.com/t/abc123",
  "is_activity": true,
  "message": "Tarefa criada com sucesso",
  "annotations": [
    {
      "type": "info",
      "text": "Tarefa criada: [Campanha] João Silva"
    }
  ]
}
```

Exemplo para não-atividade:
```json
{
  "action": "none",
  "is_activity": false,
  "message": "Mensagem não requer tarefa",
  "reason": "Saudação casual sem solicitação específica"
}
```

#### 3. **Integração Vertex AI**
- OAuth2 deve autenticar corretamente
- Modelo gemini-2.0-flash-001 deve responder
- Classificação deve ser precisa
- Transcrições de áudio devem retornar texto literal

#### 4. **OpenAI Embeddings**
- Embeddings gerados para todas atividades
- Formato: array de 1536 floats (model: text-embedding-3-small)
- Similaridade coseno calculada corretamente
- Threshold: >0.7 = modificação, <0.5 = nova tarefa

#### 5. **Contexto e Agregação**
- Mensagens ≤3 palavras devem agregar últimas 5 mensagens
- Logs mostram: "Short message detected" e "Aggregated context"
- Classificação usa contexto completo

#### 6. **Scheduler**
- Processar em até 10s (local) ou 100s (produção)
- Agrupar mensagens do mesmo contato
- Processar imediatamente com 3+ mensagens

#### 7. **Áudio**
- Tentativa Vertex AI primeiro
- Fallback para Whisper em caso de falha
- Transcrição retornada como anotação
- Logs mostram: "🎤 Iniciando transcrição de áudio"

## 📊 Detalhamento dos Cenários

### Cenário 1: Agregação de Mensagens Curtas
**Objetivo**: Validar que mensagens com ≤3 palavras agregam últimas 5 mensagens

**Sequência**:
1. "Olá" (1 palavra) → Deve agregar
2. "então" (1 palavra) → Deve agregar
3. "preciso" (1 palavra) → Deve agregar
4. "de 10 caixas de parafusos" (5 palavras) → Cria tarefa com contexto completo

**Esperado**:
- Primeiras 3 mensagens não criam tarefas
- Última mensagem cria tarefa com contexto: "Olá então preciso de 10 caixas de parafusos"
- Logs mostram "Short message detected" para primeiras 3

### Cenário 4: Modificação de Pedido (Embeddings)
**Objetivo**: Testar detecção de modificação por similaridade semântica

**Sequência**:
1. "Quero 20 metros de cabo elétrico 2.5mm" → Cria tarefa + gera embedding
2. Aguardar 5s para processamento
3. "Na verdade, troca para 30 metros e cabo 4mm" → Detectar modificação

**Esperado**:
- Primeira mensagem: `action: "created"`, task_id retornado
- Embedding gerado e armazenado
- Segunda mensagem: similaridade > 0.7
- `action: "updated"`, mesmo task_id
- Comentário adicionado à tarefa com histórico

**Validação**:
- Logs: "🧠 Embeddings gerados"
- Logs: "Similaridade coseno: 0.XX"
- ClickUp: Tarefa tem comentário com modificação

### Cenário 7: Adições ao Pedido
**Objetivo**: Diferenciar adições (nova tarefa) de modificações

**Sequência**:
1. "Preciso de 10 lâmpadas LED" → Cria tarefa
2. Aguardar 4s
3. "E também 5 soquetes E27" → Nova tarefa (baixa similaridade)

**Esperado**:
- Primeira mensagem: tarefa criada
- Segunda mensagem: similaridade < 0.5
- `action: "created"` novamente
- Dois task_id diferentes

## 🚀 Deploy para Produção

### Pré-requisitos:
1. Docker instalado
2. gcloud CLI configurado com conta `voilaassist@gmail.com`
3. Permissões no projeto buzzlightear
4. Rust 1.90 (para build local)

### Secrets Configurados:
```bash
# Secrets no GCP Secret Manager
openai-api-key         # OpenAI API key
clickup-api-token      # ClickUp personal token
clickup-list-id        # ClickUp list ID (901300373349)

# IAM: buzzlightear@appspot.gserviceaccount.com tem acesso
```

### Comandos:
```bash
# 1. Limpar cache se necessário
cargo clean

# 2. Build da imagem (Rust 1.90-slim)
docker build -t gcr.io/buzzlightear/chatguru-clickup-middleware:latest . --platform linux/amd64

# 3. Push para GCR
docker push gcr.io/buzzlightear/chatguru-clickup-middleware:latest

# 4. Deploy no Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --platform managed
```

### Validação Pós-Deploy:
```bash
# Health check
curl https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/health

# Status completo
curl https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app/status

# Logs em tempo real
gcloud run logs tail chatguru-clickup-middleware --region=southamerica-east1
```

## 📊 Monitoramento Durante Testes

### Logs de Produção:
```bash
# Logs em tempo real
gcloud run logs tail chatguru-clickup-middleware --region=southamerica-east1

# Últimos 100 logs
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 --limit=100

# Buscar erros
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep ERROR

# Filtrar por padrões específicos
gcloud logging read "resource.type=cloud_run_revision AND \
  resource.labels.service_name=chatguru-clickup-middleware AND \
  textPayload=~\"🎤|🧠|📤\"" \
  --limit 50 --format=json
```

### Padrões de Log Importantes:

#### Sucesso:
- `🎤 Iniciando transcrição de áudio` - Processamento de áudio iniciado
- `🧠 Embeddings gerados` - Embeddings criados com sucesso
- `📤 Criando tarefa no ClickUp` - Nova tarefa sendo criada
- `🔄 Tarefa já existe` - Atualização de tarefa existente
- `Similaridade coseno: X.XX` - Cálculo de similaridade
- `Short message detected` - Agregação de contexto ativada
- `Aggregated context` - Contexto agregado

#### Erros:
- `❌` - Qualquer erro
- `⚠️ Falha na transcrição Vertex AI, tentando Whisper` - Fallback ativado
- `Failed to` - Falhas em operações
- `ERROR` - Erros gerais

### Métricas Esperadas:
- **Latência webhook**: < 100ms (resposta imediata)
- **Processamento Vertex AI**: 1-3s
- **Geração embeddings**: 1-2s
- **Criação tarefa ClickUp**: < 1s
- **Taxa de sucesso**: > 95%
- **Classificação precisa**: > 90%

## 🐛 Troubleshooting

### Problema: Áudio não está sendo transcrito
**Causas possíveis**:
- URL do áudio inválida ou inacessível
- Vertex AI falhou e Whisper também
- OPENAI_API_KEY não configurada

**Validação**:
```bash
# Verificar se OpenAI key está no Secret Manager
gcloud secrets versions access latest --secret="openai-api-key"

# Verificar logs de tentativa
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep "🎤"
```

**Solução**:
- Confirmar que `media_url` é acessível publicamente
- Verificar formato do áudio (MP3, OGG, WAV suportados)
- Confirmar OpenAI API key tem créditos

### Problema: Embeddings não sendo gerados
**Sintoma**: Mensagens similares criam tarefas duplicadas

**Validação**:
```bash
# Buscar logs de embeddings
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep "🧠"
```

**Solução**:
- Verificar OpenAI API key no Secret Manager
- Confirmar modelo: "text-embedding-3-small"
- Verificar quota/rate limits OpenAI
- Checar se `is_activity: true` (embeddings só são gerados para atividades)

### Problema: Modificações criando tarefas novas
**Causa**: Similaridade abaixo do threshold (0.7)

**Validação**:
```bash
# Ver cálculos de similaridade
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep "Similaridade"
```

**Solução**:
- Revisar threshold em [conversation_tracker.rs:93](src/services/conversation_tracker.rs#L93)
- Verificar se embeddings estão sendo armazenados
- Confirmar que mensagens são do mesmo `celular`

### Problema: Contexto não está sendo agregado
**Sintoma**: Mensagens curtas não criam tarefas completas

**Validação**:
```bash
# Ver detecção de mensagens curtas
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep "Short message"
```

**Solução**:
- Verificar se mensagem tem ≤3 palavras
- Confirmar histórico de mensagens existe para o contato
- Verificar método `aggregate_recent_context` está sendo chamado

### Problema: "Failed to get OAuth2 access token"
**Contexto**: Vertex AI authentication

**Solução**:
- Normal em ambiente local (usa credenciais do gcloud)
- Em produção, verificar:
  - Conta de serviço `buzzlightear@appspot.gserviceaccount.com`
  - Role `roles/aiplatform.user` atribuído
  - Região correta (us-central1 para Vertex AI)

### Problema: "Número não informado ou incompleto"
**Solução**: Formato do telefone deve ser E.164 (+5511999998888)

### Problema: Scheduler não processa
**Solução**: Verificar:
- Servidor está rodando
- Logs mostram "Added job verificar_e_enviar_mensagens"
- Aguardar intervalo configurado (10s local, 100s prod)

### Problema: Tarefas duplicadas para mesmo pedido
**Causas**:
1. Embeddings não gerados/armazenados
2. Similaridade calculada incorretamente
3. Timeout entre mensagens muito longo

**Solução**:
- Verificar logs de embeddings: `grep "🧠"`
- Confirmar similaridade sendo calculada: `grep "Similaridade"`
- Enviar mensagens em sequência rápida (< 5 minutos)

### Problema: Classificação incorreta
**Solução**: Verificar:
- Prompt em `config/ai_prompt.yaml`
- Campos dinâmicos do ClickUp
- Resposta do Vertex AI nos logs
- Se contexto agregado está sendo usado

## 📈 Resultados Esperados

### Taxa de Classificação:
- **Atividades válidas**: 90%+ precisão
- **Não-atividades**: 95%+ precisão
- **Detecção de modificação**: 85%+ (similaridade > 0.7)
- **Diferenciação adição vs modificação**: 80%+

### Performance:
- **Latência webhook**: < 100ms (resposta imediata)
- **Processamento Vertex AI**: 1-3s
- **Geração embeddings**: 1-2s
- **Criação tarefa ClickUp**: < 1s
- **Transcrição áudio**: 5-15s (depende do tamanho)

### Custos Estimados:
- **Vertex AI**: ~$0.01 por classificação
- **OpenAI Embeddings**: ~$0.0001 por mensagem (text-embedding-3-small)
- **OpenAI Whisper**: ~$0.006 por minuto de áudio
- **Cloud Run**: ~$10-20/mês (baixo volume)
- **Total estimado**: < $100/mês para 5000 mensagens + 100 áudios

## ✅ Checklist de Validação Pós-Teste

Após executar os testes, verificar:

- [ ] Tarefas criadas no ClickUp (list 901300373349)
- [ ] Títulos seguem formato: `[Campanha] Nome Cliente`
- [ ] Descrições contêm o conteúdo da mensagem
- [ ] Status inicial é "pendente"
- [ ] Comentários adicionados para modificações
- [ ] Sem tarefas duplicadas para mesmo pedido
- [ ] Mensagens casuais não criam tarefas
- [ ] Tentativas de transcrição de áudio nos logs
- [ ] Embeddings gerados para atividades
- [ ] Agregação de contexto para mensagens curtas
- [ ] Sem crashes ou 500 errors
- [ ] Todas respostas em JSON válido
- [ ] Logs mostram padrões esperados (🎤, 🧠, 📤, etc.)

## 🔗 URLs e Recursos

### URLs de Produção:
- **Base**: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app
- **Health**: /health
- **Status**: /status
- **Webhook**: /webhooks/chatguru

### GCP Resources:
- **Project**: buzzlightear
- **Region**: southamerica-east1
- **Service Account**: buzzlightear@appspot.gserviceaccount.com
- **ClickUp List**: 901300373349

### Secret Manager:
- `openai-api-key` - OpenAI API key (embeddings + Whisper)
- `clickup-api-token` - ClickUp personal token
- `clickup-list-id` - ClickUp list ID

## 📝 Notas Importantes

### Comportamento do Sistema:
- Scheduler agrupa mensagens por contato
- Mensagens ≤3 palavras agregam últimas 5 mensagens
- Embeddings só gerados para `is_activity: true`
- Similaridade > 0.7 = modificação (atualiza tarefa)
- Similaridade < 0.5 = nova tarefa (cria nova)
- 0.5 ≤ similaridade ≤ 0.7 = zona cinza (comportamento indefinido)

### Limitações Conhecidas:
- ClickUp API: 100 req/min rate limit
- ChatGuru: espera resposta em < 5s (já resolvido com webhook assíncrono)
- Vertex AI: pode falhar ocasionalmente (fallback para Whisper)
- OpenAI: rate limits dependem do plano contratado

### Melhores Práticas:
- Executar testes em horário de baixo tráfego
- Monitorar logs em tempo real durante testes
- Validar tarefas no ClickUp após cada cenário
- Aguardar 3-5s entre requisições para evitar rate limits
- Limpar tarefas de teste do ClickUp periodicamente

## 🚀 Próximos Passos

Após validação completa dos testes:

1. **Monitoramento Contínuo**: Observar uso em produção por 1 semana
2. **Ajuste de Thresholds**: Refinar similaridade se necessário (atualmente 0.7)
3. **Novos Cenários**: Adicionar testes baseados em padrões reais de uso
4. **Automação**: Considerar CI/CD com testes automatizados
5. **Métricas**: Implementar dashboard de métricas (Grafana/Cloud Monitoring)
6. **Alertas**: Configurar alertas para falhas, latência alta, etc.