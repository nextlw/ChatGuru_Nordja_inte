# 🧪 Guia de Testes - ChatGuru ClickUp Middleware

## 📋 Scripts de Teste Disponíveis

### 1. `test_production.sh` - Teste Completo de Produção
Script principal para validar o funcionamento completo do sistema.

#### Uso:
```bash
# Teste local
./test_production.sh

# Teste em produção
./test_production.sh prod
```

#### Cenários Testados:
1. **Compra de Material** - Categoria: Compras
2. **Agendamento Médico** - Categoria: Agendamento  
3. **Reembolso Médico** - Categoria: Plano de Saúde
4. **Logística Urgente** - Categoria: Logística
5. **Pagamento de Conta** - Categoria: Pagamentos
6. **Pesquisa/Orçamento** - Categoria: Pesquisas
7. **Viagem Corporativa** - Categoria: Viagens
8. **Documentação** - Categoria: Documentos
9. **Saudação** - Não deve criar tarefa
10. **Agradecimento** - Não deve criar tarefa

### 2. `monitor_prod.sh` - Monitor de Produção
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
- Mensagens de trabalho devem gerar tarefas
- Saudações/agradecimentos não devem gerar tarefas
- Categoria deve corresponder ao conteúdo

#### 2. **Formato das Anotações**
Exemplo esperado:
```
Tarefa: Atividade Identificada: Compra de Material de Restaurante
Tipo de Atividade: Urgente
Categoria: Compras
Sub Categoria: Material Profissional
Status Back Office: Executar
```

#### 3. **Integração Vertex AI**
- OAuth2 deve autenticar corretamente
- Modelo gemini-2.0-flash-001 deve responder
- Classificação deve ser precisa

#### 4. **Scheduler**
- Processar em até 10s (local) ou 100s (produção)
- Agrupar mensagens do mesmo contato
- Processar imediatamente com 3+ mensagens

## 🚀 Deploy para Produção

### Pré-requisitos:
1. Docker instalado
2. gcloud CLI configurado
3. Permissões no projeto buzzlightear

### Comandos:
```bash
# Build da imagem
docker build -t gcr.io/buzzlightear/chatguru-clickup-middleware:latest . --platform linux/amd64

# Push para GCR
docker push gcr.io/buzzlightear/chatguru-clickup-middleware:latest

# Deploy no Cloud Run
gcloud run deploy chatguru-clickup-middleware \
  --image gcr.io/buzzlightear/chatguru-clickup-middleware:latest \
  --region southamerica-east1 \
  --platform managed
```

## 📊 Monitoramento

### Logs de Produção:
```bash
# Logs em tempo real
gcloud run logs tail chatguru-clickup-middleware --region=southamerica-east1

# Últimos 100 logs
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 --limit=100

# Buscar erros
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep ERROR
```

### Métricas:
- Latência média: < 500ms esperado
- Taxa de sucesso: > 95% esperado
- Uso de Vertex AI: ~$0.01 por classificação

## 🐛 Troubleshooting

### Problema: "Failed to get OAuth2 access token"
**Solução**: Normal em ambiente local. Em produção, verificar:
- Conta de serviço tem role `roles/aiplatform.user`
- Região está correta (us-central1)

### Problema: "Número não informado ou incompleto"
**Solução**: Formato do telefone deve ser E.164 (+5511999998888)

### Problema: Scheduler não processa
**Solução**: Verificar:
- Servidor está rodando
- Logs mostram "Added job verificar_e_enviar_mensagens"
- Aguardar intervalo configurado (10s local, 100s prod)

### Problema: Classificação incorreta
**Solução**: Verificar:
- Prompt em `config/ai_prompt.yaml`
- Campos dinâmicos do ClickUp
- Resposta do Vertex AI nos logs

## 📈 Resultados Esperados

### Taxa de Classificação:
- **Atividades válidas**: 90%+ precisão
- **Não-atividades**: 95%+ precisão
- **Categorização**: 85%+ precisão

### Performance:
- **Latência webhook**: < 100ms
- **Processamento Vertex AI**: < 2s
- **Criação tarefa ClickUp**: < 1s

### Custos:
- **Vertex AI**: ~$0.01 por requisição
- **Cloud Run**: ~$10/mês (baixo volume)
- **Total estimado**: < $50/mês para 5000 mensagens

## 🔗 URLs Importantes

- **Produção**: https://chatguru-clickup-middleware-707444002434.southamerica-east1.run.app
- **Health Check**: /health
- **Status**: /status
- **Webhook**: /webhooks/chatguru

## 📝 Notas

- Scheduler agrupa mensagens por contato
- Vertex AI usa cache para economizar
- ClickUp API tem rate limit de 100 req/min
- ChatGuru espera resposta em < 5s