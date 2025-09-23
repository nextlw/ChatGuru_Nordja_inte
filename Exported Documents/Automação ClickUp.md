# Automação ClickUp

## 11:18 seg., 15 de set. • 9 min

[Account](https://spellar.ai/account/meetings/68c820393000002300165012?setapp=true&utm_source=spellar_api)

# Notas da reunião

`5 Tópicos`, `14 Tarefas`

Reunião técnica sobre redução de custos de API e melhoria da automação que transforma áudios em tarefas no ClickUp. Discutiram troca da API de transcrição OpenAI → Google (Gemini), arquitetura de processamento na nuvem (instalar 'Eli'), lógica para evitar duplicação de tarefas, criação de subtarefas e atualização de classificação de categorias com input da Anny. Planejaram ajustes e próximos passos com previsões de custo.

## Otimização de custos de APIs

`3 Pontos-chave`
Avaliar e migrar transcrição de OpenAI para Google (Gemini) para reduzir gastos mensais e aproveitar créditos/free tier do Google Cloud.

### Pontos-chave:

✔️ **Custo atual OpenAI:**
Gastam cerca de R$500-600/mês com OpenAI para transcrições.

⚡️ **Migrar para Google:**
Decisão de testar Google Gemini para reduzir custo; validar compatibilidade com Chat Guru.

🤝 **Redução de custo prioritária:**
Concordaram que reduzir custo de transcrição é benéfico e prioridade.

### Tarefas:

- [ ] **Testar integração de transcrição via Gemini (Google) e comparar custos com OpenAI**:
*Validar se o chat/serviço atual aceita Gemini; caso negativo, implementar fluxo para enviar áudio ao Google, receber transcrição e seguir fluxo atual.*
- [ ] **Calcular estimativa de custos mensais pós-migração**:
*Usar volume atual (~1.000-1.200 atendimentos/tasks/mês) para projetar custos Google vs OpenAI e documentar economia esperada.*
- [ ] **Configurar credenciais Google Cloud na infraestrutura**:
*Preparar credenciais seguras e permissões necessárias para a API de transcrição na nuvem onde o sistema roda.*

---

## Integração ClickUp e lógica de tarefas

`3 Pontos-chave`
Melhorar correspondência de tarefas, evitar duplicações, permitir alterações e criação de subtarefas; usar campo 'info two' para identificar cliente.

### Pontos-chave:

📍 **Fonte do nome do cliente:**
O sistema lê o nome do cliente do campo 'info two'.

⚡️ **Detecção de duplicidade:**
Necessidade de checar existência prévia de tarefa para evitar duplicação.

🤝 **Usar ID numérico da pasta:**
Concordaram em usar o número da lista/pasta (do URL) para localizar a pasta correta no ClickUp.

### Tarefas:

- [ ] **Implementar verificação de existência de tarefa no ClickUp**:
*Puxar lista pelo ID numérico da pasta/lista, comparar com informações essenciais (cliente, título, data) e decidir criar, atualizar ou transformar em subtarefa.*
- [ ] **Adicionar lógica para criação de subtarefas e atualização em vez de duplicação**:
*Quando for mesma tarefa/tema, atualizar a tarefa existente ou criar subtarefa em vez de nova tarefa duplicada.*
- [ ] **Mapear campos usados para matching (ex.: info two, nome da pasta, identificador)**:
*Documentar quais campos serão usados e tolerância de matching (exatas, fuzzy) para evitar falsos positivos/negativos.*

---

## Implantação do agente 'Eli' na nuvem

`3 Pontos-chave`
Instalar e configurar o agente (Eli) que fará gerenciamento das conversas, processamento e comunicação com ClickUp.

### Pontos-chave:

⚡️ **Instalação local de Eli:**
Será instalado na nuvem do cliente para gerenciar processos.

✔️ **Funções do Eli:**
Eli fará parsing das conversas, processará e enviará tarefas ao ClickUp.

🤝 **Redução de custos/latência:**
Esperam reduzir custos e melhorar controle ao processar localmente.

### Tarefas:

- [ ] **Instalar Eli na nuvem do cliente e validar operações end-to-end**:
*Configurar ambiente, permissões, endpoints do ClickUp e da API de transcrição; testar fluxo: áudio → transcrição → entendimento → ClickUp.*
- [ ] **Monitorar consumo de API após implantação de Eli**:
*Coletar métricas de uso (tokens, chamadas) para avaliar economia e ajustar configuração.*

---

## Classificação e subcategorias

`3 Pontos-chave`
Atualizar mapeamento de categorias/subcategorias que o agente preenche automaticamente, com input da Anny.

### Pontos-chave:

⚡️ **Anny revisará categorias:**
Anny fará a proposta de nova classificação e enviará para configuração.

🤝 **Preenchimento automático:**
Agente continuará preenchendo automaticamente, mas com nova classificação após atualização.

✔️ **Configuração na API:**
Desenvolvedor atualizará a API conforme a nova taxonomia.

### Tarefas:

- [ ] **Receber nova classificação de categorias da Anny**:
*Agendar alinhamento com Anny; coletar a nova estrutura de categorias e exemplos de mapeamento.*
- [ ] **Atualizar configuração da API para refletir nova classificação**:
*Alterar regras de preenchimento automático para usar a nova taxonomia e testar contra amostras reais.*

---

## Volume, previsão e próximos passos

`3 Pontos-chave`
Estimativa de volume (1.000–1.200 tasks/mês) e impacto financeiro; definição de próximas ações e entregáveis para curto prazo.

### Pontos-chave:

✔️ **Volume mensal:**
Estimativa de ~1.000–1.200 tasks/mês.

📍 **Aumento de custo previsto:**
Preveram um aumento modesto de US$50–60 se necessário.

⚡️ **Prazo de retorno:**
Wilian/Equipe dará retorno no mesmo dia após falar com Anny.

### Tarefas:

- [ ] **Apresentar retorno com alinhamento da Anny hoje**:
*Confirmar reunião da Anny e enviar feedback sobre classificação e prioridades até o final do dia.*
- [ ] **Gerar projeção de custo pós-migração e impact report**:
*Baseado no volume atual, estimar custo adicional/receita economizada e apresentar plano financeiro.*

---

## Outras tarefas

    - Fornecer nova classificação/mapa de categorias para configuração na API
    - Testes de ponta a ponta (áudio→transcrição→ClickUp)

# Transcrição

Ó eu tenho aqui sinceramente eu não sei nem qual seria a melhor alternativa a essa daqui é mínimo fifteen licenças então, ainda tem isso, né? Eu tenho essa aqui, ó. adicionar em programas avançados por r33,00 o diário mês. Eu já pago r29,00 e eu vou aumentar para r33,00. Então vai para r50,00 e poucos. Não, é muito caro, muito caro. É muito caro. Aí tem o brain, que também adiciona mais r33,00. Cara, não dá. É muito caro, muito caro. Aí o que acontece, ó.

Com essa automação inteligente, como tu tem uma api da openai, que eu vi que tu tá com segurada lá, quanto que tu paga de openai? Tu tem ideia? Uns r500, r600 todo mês. Que serve pra fazer transcrição de áudio, não é isso? Isso, isso, exatamente. Pronto, é r500, r600 por mês, né? Dá aí mais ou menos. 对,50美元,大概是250300美金然后还有google cloud,大概是500600美金每个月对,为什么呢?因为这个做法是很.我可以说.我看了一下,这个做法是很厉害的所以我们必须降低这个价值好吗?非常好非常好,非常非常好因为,例如,最便宜的api.

Para transcrição de áudio é a do google. E como você já está na cloud, você deveria estar usando ela e não a da openai. Não entende? Tá. Aí, lá, por exemplo, até 500 mil caracteres você não paga. Não entende? Então, assim, reduz muito o custo, né? Só que aí o que acontece? Para poder a gente utilizar lá, o chat guru, ele tem uma. Com a openai. Eu tenho que ver se dá para colocar a api do geminiu. Se não der, a gente puxa o áudio lá para dentro do google e aí faz essa transcrição lá. Entendeu? Dá para fazer das duas formas. Ainda tem esse problema na suri. A suri não tem como fazer transcrição de áudio. Não tem não. Então, é a minha operação também, né? Pois é. exatamente.

Exatamente. Beleza. Porque o que acontece? Vem o áudio, aí no áudio vem o pedido, a integração entende o pedido e gera a tarefa no clickup. Então tem todo um. Isso. Um negocinho ali rodando por trás, né? Isso, é. só que aí o que acontece? Para poder a gente fazer. Porque tem duas opções aqui, ó. 为了让我们变得更聪明你会有费用增加与抵押 如果我们能够使用谷歌自动转辑的方式,我们将得到谷歌的价格减少所以,我认为这将有益于您的工作,您将得到更好的管理,通过您的工作应用程序,您将得到.

Basicamente o mesmo curso. Pode ser que passe, pode. Depende do volume, né? Aí é bom tu dar uma olhada, por exemplo, na quantidade de volume. Tu sabe quantos atendimentos tu faz por mês? O que que você considera um atendimento? Quantas tarefas, pronto, quantas tasks são feitas por mês? Umas mil. Umas mil tasks por mês. 1.000, 1.200, mais ou menos. É muito, né? Bastante, né? É. tá. Tá entendendo por que a meta tá querendo me tirar do ar? Pois é. então, acredito que tu vai ter aí, tu vai ter aí, no máximo, estourando um custo aí a mais fifty, sixty dólares a mais, tá? Então tem que se preparar um pouco.

Para isso, se você quiser realmente ser mais inteligente. Então, nesse fluxo. E aí, o que eu vou fazer? O eli, eu vou instalar ele na tua nuvem. E aí ele vai fazer esse gerenciamento. Ele vai receber os dados da conversa e ele mesmo vai processar e vai enviar para o clickup e ele vai manusear o clickup. Tá? Tá. Another thing, how does he understand who this task is? Because it will come in the conversation, right? The persons name, right? No, right here. He takes it here. He takes this field here, info two. This info two field. Ana flávia tavares. She is a client. Here it has to be exactly.

Como está escrito com quem está na flávia tavares agora? A flávia tavares. Beleza. Boa noite. Oh jesus! Here, see? He understands here, ana flávia tavares, ana flávia tavares, so its from here, he plays ana flávias request here. I see. Thats where he does the match. Okay, i see. To play it right in the folder. Because in the clickup api, in fact, you dont get the folder name, you get that number that is there, 25090512001, you know? Where? Up there.

No navegador, no endereço, né? Tem uma parte que é só o número, 2509, no endereço do site. Gente, eu não tô vendo isso, não. Ah, é que eu tô vendo no aplicativo. Ah, é. pronto. Enfim, ele pega o número, ele pega. Eu tava olhando, era o meu aqui, no caso, que eu também tô com um aberto, né? Mas o meu tava só com a barra pra fora, assim, eu pensava que era a sua barra. Mas aí ele pega pelo número, né? Aí, quando ele cria uma tarefa, é por isso que ele duplica. Ele não tem uma verificação se essa tarefa, se esse usuário já tem essa tarefa, né? Entendi. O número da lista, e aí ele conseguiria, por exemplo, puxar e ver se a tarefa já existe, né? Aí precisa ter, obviamente, o mínimo de entendimento para saber se a tarefa é igual ou não. Entendi. Entendi.

Não dá para fazer, senhor. Show de bola. Então, tu quer que eu faça isso? Eu tente rolochar o tradutor para o google para você dar uma economizada nos tokens. Eu vou economizar do lado e ganhar do outro. Mas só que você vai ganhar do outro porque aí você não vai ter mais tarefa duplicada, vai ter alteração, vai ter subtarefa, esse tipo de coisa. Entendi. E a anne estava querendo mexer nessas categorias aqui, subcategorias. Aí a gente te passa como vai ficar a nova classificação para ele preencher? Porque ele acaba preenchendo automático. Isso. Você me passa como é que vai ficar a nova classificação e aí eu configuro lá na api. Perfeito. Eu acho que é isso, william. Tá. Beleza. Tem mais algum detalhe que você queria aproveitar para a gente.

Eu queria inclusive usar esse teu caso como um case então eu queria aproveitar isso se você tiver mais alguma coisa é que a anny estava com uma reunião agendada agora para as eleven deixa eu ver com ela e aí eu te falo o que a gente alinhou e aí, provavelmente, ela que usa mais, que o time usa mais, deve saber melhor aqui do que fazer ou não. Tá, tá bom. Então, beleza. Só pra recapitular. Aí eu te passo isso hoje ainda. Só para recapitular, vou alterar a api para o gemini de transcrição, vou tornar a escolha da tarefa mais inteligente, com a condicional para ver se a tarefa já existe ou não, e habilitar também a parte de fazer subtarefas, se for necessário. Está certo? Certo.

E transcrição por gente. Tá. Eu puxo a anne e eu te dou um retorno ainda hoje para ver como que a gente melhora isso. Tá bom. Beleza. Tchau. Você pode contar comigo aí, viu? Obrigada, william. Obrigada mesmo. Nada. Valeu, viu? Um abraço. Outro. Tchau, tchau.

––
*Created from [Spellar](https://spellar.ai)*