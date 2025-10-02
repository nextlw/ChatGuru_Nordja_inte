/// Teste standalone para visualizar como o prompt é montado e enviado para a LLM
/// Execute com: cargo run --bin test_prompt

use chatguru_clickup_middleware::services::ai_prompt_loader::AiPromptConfig;
use chatguru_clickup_middleware::services::openai_fallback::OpenAIService;
use chatguru_clickup_middleware::models::WebhookPayload;

#[tokio::main]
async fn main() {
    println!("\n{}", "=".repeat(80));
    println!("TESTE: Visualização do Prompt Real Enviado à LLM");
    println!("{}\n", "=".repeat(80));

    // 1. Carregar configuração do prompt
    let config = match AiPromptConfig::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("❌ Erro ao carregar configuração: {}", e);
            return;
        }
    };

    println!("✅ Configuração carregada com sucesso\n");

    // 2. Simular uma mensagem real de usuário
    let mensagem_usuario = "Preciso comprar presentes de aniversário para minha mãe";
    let nome_usuario = "João Silva";
    let campanha = "WhatsApp";
    let origem = "whatsapp";

    // 3. Montar contexto como o sistema faz
    let context = format!(
        "Campanha: {}\nOrigem: {}\nNome: {}\nMensagem: {}\nTags: []",
        campanha, origem, nome_usuario, mensagem_usuario
    );

    println!("📝 CONTEXTO DA MENSAGEM:");
    println!("{}\n", context);
    println!("{}\n", "-".repeat(80));

    // 4. Gerar prompt completo (método estático - sem campos dinâmicos)
    let prompt_completo = config.generate_prompt(&context);

    println!("🤖 PROMPT COMPLETO ENVIADO À LLM:");
    println!("{}", "=".repeat(80));
    println!("{}", prompt_completo);
    println!("{}\n", "=".repeat(80));

    // 5. Estatísticas do prompt
    let num_linhas = prompt_completo.lines().count();
    let num_chars = prompt_completo.len();
    let num_palavras = prompt_completo.split_whitespace().count();

    println!("📊 ESTATÍSTICAS DO PROMPT:");
    println!("  • Linhas: {}", num_linhas);
    println!("  • Caracteres: {}", num_chars);
    println!("  • Palavras: {}", num_palavras);
    println!("  • Tamanho estimado (tokens): ~{}", num_palavras / 2);
    println!();

    // 6. Mostrar categorias disponíveis
    println!("📋 CATEGORIAS CONFIGURADAS:");
    for cat in &config.categories {
        println!("  • {}", cat);
    }
    println!();

    // 7. Mostrar tipos de atividade
    println!("🏷️  TIPOS DE ATIVIDADE:");
    for at in &config.activity_types {
        println!("  • {}: {}", at.name, at.description);
    }
    println!();

    // 8. Mostrar status options
    println!("⚡ STATUS BACK OFFICE:");
    for status in &config.status_options {
        println!("  • {}", status.name);
    }
    println!();

    println!("{}", "=".repeat(80));
    println!("✅ Teste concluído! O prompt acima é EXATAMENTE o que é enviado à LLM.");
    println!("{}\n", "=".repeat(80));

    // Verificar se OpenAI está configurada
    println!("\n{}", "=".repeat(80));
    println!("TESTE COM LLM REAL (OpenAI Fallback)");
    println!("{}\n", "=".repeat(80));

    let openai_service = OpenAIService::new(None).await;

    if openai_service.is_none() {
        println!("⚠️  OpenAI não configurada. Configure OPENAI_API_KEY para testar com LLM real.");
        println!("   Continuando apenas com visualização do prompt...\n");
    } else {
        println!("✅ OpenAI configurada! Vamos enviar prompts reais e ver as respostas.\n");
    }

    // Teste adicional com múltiplos cenários
    println!("\n{}", "=".repeat(80));
    println!("TESTE ADICIONAL: Múltiplos Cenários");
    println!("{}\n", "=".repeat(80));

    let scenarios = vec![
        ("Preciso agendar uma consulta médica para amanhã", "Agendamento de consulta"),
        ("Comprar 2 kg de arroz e 1 litro de leite no mercado", "Compra de mercado"),
        ("Reservar voo para São Paulo dia 15", "Viagem"),
        ("Pagar boleto da luz que vence hoje", "Pagamento financeiro"),
        ("Oi, tudo bem?", "Mensagem casual (não é atividade)"),
        ("Preciso enviar documentos via motoboy", "Logística"),
    ];

    for (i, (mensagem, descricao)) in scenarios.iter().enumerate() {
        println!("{}", "=".repeat(80));
        println!("CENÁRIO {}: {}", i + 1, descricao);
        println!("{}\n", "=".repeat(80));

        let context = format!(
            "Campanha: WhatsApp\nOrigem: whatsapp\nNome: Usuário Teste\nMensagem: {}\nTags: []",
            mensagem
        );

        let prompt = config.generate_prompt(&context);

        println!("📝 Mensagem: {}", mensagem);
        println!("🤖 Tamanho do prompt: {} caracteres", prompt.len());

        // Se OpenAI está configurada, enviar prompt real e mostrar resposta
        if let Some(ref openai) = openai_service {
            println!("\n⏳ Enviando para OpenAI...");

            match openai.classify_activity_fallback(&context).await {
                Ok(classification) => {
                    println!("✅ RESPOSTA DA LLM:");
                    println!("{}", "-".repeat(80));
                    println!("📊 Classificação:");
                    println!("  • É atividade: {}", if classification.is_activity { "✅ SIM" } else { "❌ NÃO" });
                    println!("  • Razão: {}", classification.reason);

                    if classification.is_activity {
                        if let Some(ref tipo) = classification.tipo_atividade {
                            println!("  • Tipo de Atividade: {}", tipo);
                        }
                        if let Some(ref cat) = classification.category {
                            println!("  • Categoria: {}", cat);
                        }
                        if let Some(ref subcat) = classification.sub_categoria {
                            println!("  • Subcategoria: {}", subcat);
                        }
                        if let Some(ref status) = classification.status_back_office {
                            println!("  • Status: {}", status);
                        }
                        if !classification.subtasks.is_empty() {
                            println!("  • Subtarefas:");
                            for (idx, subtask) in classification.subtasks.iter().enumerate() {
                                println!("    {}. {}", idx + 1, subtask);
                            }
                        }
                    }
                    println!("{}", "-".repeat(80));

                    // Mostrar como a tarefa será montada para o ClickUp
                    if classification.is_activity {
                        println!("\n📦 MONTAGEM DA TAREFA PARA O CLICKUP:");
                        println!("{}", "-".repeat(80));

                        // Criar payload REAL como o ChatGuru envia
                        let mut campos_personalizados = std::collections::HashMap::new();
                        campos_personalizados.insert("Info_1".to_string(), serde_json::Value::String("Conta Teste".to_string()));
                        campos_personalizados.insert("Info_2".to_string(), serde_json::Value::String("João Silva".to_string()));

                        let payload = WebhookPayload::ChatGuru(chatguru_clickup_middleware::models::ChatGuruPayload {
                            campanha_id: "901300373349".to_string(),
                            campanha_nome: "WhatsApp Bot".to_string(),
                            origem: "whatsapp".to_string(),
                            email: "joao.silva@email.com".to_string(),
                            nome: "João Silva".to_string(),
                            tags: vec!["urgente".to_string()],
                            texto_mensagem: mensagem.to_string(),
                            media_url: None,
                            media_type: None,
                            campos_personalizados,
                            bot_context: Some(chatguru_clickup_middleware::models::BotContext {
                                chat_guru: Some(true),
                            }),
                            responsavel_nome: Some("Atendente Bot".to_string()),
                            responsavel_email: Some("bot@chatguru.app".to_string()),
                            link_chat: "https://s15.chatguru.app/chat/625584ce6fdcb7bda7d94aa8/12345".to_string(),
                            celular: "+5511987654321".to_string(),
                            phone_id: Some("5511987654321".to_string()),
                            chat_id: Some("12345".to_string()),
                            chat_created: Some("2025-10-02T10:30:00Z".to_string()),
                        });

                        // Converter classificação OpenAI para ActivityClassification
                        let ai_classification = chatguru_clickup_middleware::services::vertex_ai::ActivityClassification {
                            is_activity: classification.is_activity,
                            activity_type: classification.tipo_atividade.clone(),
                            category: classification.category.clone(),
                            subtasks: classification.subtasks.clone(),
                            priority: None,
                            reason: classification.reason.clone(),
                            cliente_solicitante_id: None,
                            tipo_atividade: classification.tipo_atividade.clone(),
                            sub_categoria: classification.sub_categoria.clone(),
                            status_back_office: classification.status_back_office.clone(),
                        };

                        // Gerar dados da tarefa como o middleware faz
                        let task_data = payload.to_clickup_task_data_with_ai(Some(&ai_classification));

                        println!("\n📤 PAYLOAD ENVIADO PARA O CLICKUP:");
                        println!("{}", "=".repeat(80));
                        println!("{}", serde_json::to_string_pretty(&task_data).unwrap());
                        println!("{}", "=".repeat(80));

                        // Mostrar endpoint e detalhes da requisição
                        println!("\n🌐 DETALHES DA REQUISIÇÃO HTTP:");
                        println!("{}", "-".repeat(80));
                        println!("Método: POST");
                        println!("URL: https://api.clickup.com/api/v2/list/901300373349/task");
                        println!("\nHeaders:");
                        println!("  Authorization: <CLICKUP_API_TOKEN>");
                        println!("  Content-Type: application/json");
                        println!("\nBody: (JSON acima)");
                        println!("{}", "-".repeat(80));

                        // Explicar os campos
                        println!("\n📋 EXPLICAÇÃO DOS CAMPOS:");
                        println!("{}", "-".repeat(80));
                        if let Some(name) = task_data.get("name") {
                            println!("• name: {} (Título da tarefa no ClickUp)", name.as_str().unwrap_or(""));
                        }
                        if let Some(desc) = task_data.get("description") {
                            let desc_str = desc.as_str().unwrap_or("");
                            let preview = if desc_str.len() > 100 {
                                desc_str.chars().take(100).collect::<String>() + "..."
                            } else {
                                desc_str.to_string()
                            };
                            println!("• description: {} (Descrição completa com contexto)", preview);
                        }
                        if let Some(status) = task_data.get("status") {
                            println!("• status: {} (Status inicial da tarefa)", status.as_str().unwrap_or(""));
                        }
                        if let Some(priority) = task_data.get("priority") {
                            println!("• priority: {} (1=Urgente, 2=Alta, 3=Normal, 4=Baixa)", priority);
                        }
                        if let Some(custom_fields) = task_data.get("custom_fields").and_then(|v| v.as_array()) {
                            println!("• custom_fields: {} campos personalizados mapeados", custom_fields.len());
                            for (idx, field) in custom_fields.iter().enumerate() {
                                if let (Some(id), Some(value)) = (field.get("id"), field.get("value")) {
                                    println!("  {}. Campo ID {}: {}", idx + 1,
                                        id.as_str().unwrap_or(""),
                                        value.as_str().unwrap_or(&value.to_string())
                                    );
                                }
                            }
                        }
                        println!("{}", "-".repeat(80));
                    }
                }
                Err(e) => {
                    println!("❌ ERRO ao chamar OpenAI: {}", e);
                    println!("{}", "-".repeat(80));
                }
            }

            // Pequena pausa entre requisições para não sobrecarregar a API
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        } else {
            // Apenas mostrar o prompt sem enviar
            let lines: Vec<&str> = prompt.lines().collect();
            if lines.len() > 15 {
                println!("\n📄 Últimas 20 linhas do prompt:");
                println!("{}", "-".repeat(80));
                for line in lines.iter().skip(lines.len().saturating_sub(20)) {
                    println!("{}", line);
                }
                println!("{}", "-".repeat(80));
            }
        }
        println!();
    }

    println!("✅ Teste de múltiplos cenários concluído!\n");
}
