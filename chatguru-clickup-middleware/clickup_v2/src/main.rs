use clap::{Parser, Subcommand};
use clickup_v2::auth::oauth::OAuthFlow;
use clickup_v2::client::api::{
    ClickUpClient, EntityType, CreateTaskRequest, TaskPriority,
    CustomFieldValue, CustomField
};
use serde_json::json;
use chrono::{DateTime, NaiveDate, Utc};

/// ClickUp v2 CLI - Interface de linha de comando para a API do ClickUp
#[derive(Parser)]
#[command(name = "clickup_v2")]
#[command(author = "William Duarte")]
#[command(version = "0.1.0")]
#[command(about = "CLI para integração com ClickUp API v2", long_about = None)]
struct Cli {
    /// Token de acesso do ClickUp (ou use CLICKUP_ACCESS_TOKEN env var)
    #[arg(short = 't', long, env = "CLICKUP_ACCESS_TOKEN", global = true)]
    token: Option<String>,

    /// URL base da API
    #[arg(long, env = "CLICKUP_API_BASE_URL", default_value = "https://api.clickup.com/api/v2", global = true)]
    api_url: String,

    /// Team/Workspace ID (opcional)
    #[arg(long, env = "CLICKUP_TEAM_ID", global = true)]
    team_id: Option<String>,

    /// Formato de saída (json, pretty)
    #[arg(short = 'o', long, default_value = "pretty", global = true)]
    output: OutputFormat,

    /// Modo verbose para debug
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    /// Comando a executar
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, PartialEq)]
enum OutputFormat {
    Json,
    Pretty,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "pretty" => Ok(OutputFormat::Pretty),
            _ => Err(format!("Formato desconhecido: {}", s)),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Autentica via OAuth2 e salva o token
    Auth {
        /// Força reautenticação mesmo se já houver token
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Cria uma nova task
    CreateTask {
        /// ID da lista onde criar a task (ou use --list-name)
        #[arg(short = 'l', long, conflicts_with = "list_name")]
        list_id: Option<String>,

        /// Nome da lista (busca automaticamente o ID)
        #[arg(long, conflicts_with = "list_id")]
        list_name: Option<String>,

        /// Nome/título da task
        #[arg(short = 'n', long)]
        name: String,

        /// Descrição da task
        #[arg(short = 'd', long)]
        description: Option<String>,

        /// Usar markdown na descrição
        #[arg(long)]
        markdown: bool,

        /// Prioridade (1=urgent, 2=high, 3=normal, 4=low)
        #[arg(short = 'p', long)]
        priority: Option<u8>,

        /// Status da task
        #[arg(short = 's', long)]
        status: Option<String>,

        /// IDs dos responsáveis (separados por vírgula)
        #[arg(short = 'a', long)]
        assignees: Option<String>,

        /// Tags (separadas por vírgula)
        #[arg(long)]
        tags: Option<String>,

        /// Data de vencimento (formato: YYYY-MM-DD ou timestamp)
        #[arg(long)]
        due_date: Option<String>,

        /// Incluir horário na data de vencimento
        #[arg(long)]
        due_time: bool,

        /// Data de início (formato: YYYY-MM-DD ou timestamp)
        #[arg(long)]
        start_date: Option<String>,

        /// Incluir horário na data de início
        #[arg(long)]
        start_time: bool,

        /// Tempo estimado em horas
        #[arg(long)]
        time_estimate_hours: Option<f64>,

        /// Notificar todos os envolvidos
        #[arg(long)]
        notify_all: bool,

        /// Campos personalizados (formato: id:tipo:valor, separados por vírgula)
        /// Exemplo: field_uuid:text:valor,field2:number:42,field3:date:2024-01-01
        #[arg(long)]
        custom_fields: Option<String>,
    },

    /// Cria uma task simples (apenas nome e descrição)
    QuickTask {
        /// ID da lista
        #[arg(short = 'l', long)]
        list_id: String,

        /// Nome da task
        #[arg(short = 'n', long)]
        name: String,

        /// Descrição (opcional)
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// Pesquisa por entidades
    Search {
        /// Nome da entidade a pesquisar
        #[arg(short = 'n', long)]
        name: String,

        /// Tipo de entidade (space, folder, list, task)
        #[arg(short = 'e', long)]
        entity: String,

        /// Limpar cache antes de pesquisar
        #[arg(long)]
        no_cache: bool,
    },

    /// Lista spaces de um team
    ListSpaces {
        /// Team ID (usa padrão se não fornecido)
        #[arg(long)]
        team_id: Option<String>,
    },

    /// Lista folders de um space
    ListFolders {
        /// Space ID
        #[arg(short = 's', long)]
        space_id: String,
    },

    /// Lista listas de um space ou folder
    ListLists {
        /// Space ID
        #[arg(short = 's', long)]
        space_id: Option<String>,

        /// Folder ID
        #[arg(short = 'f', long)]
        folder_id: Option<String>,
    },

    /// Mostra campos personalizados de uma lista
    ShowFields {
        /// ID da lista
        #[arg(short = 'l', long)]
        list_id: String,
    },

    /// Busca uma lista por nome
    FindList {
        /// Nome da lista
        #[arg(short = 'n', long)]
        name: String,
    },

    /// Obtém detalhes de uma task
    GetTask {
        /// ID da task
        #[arg(short = 't', long)]
        task_id: String,
    },

    /// Atualiza um campo personalizado de uma task
    UpdateField {
        /// ID da task
        #[arg(short = 't', long)]
        task_id: String,

        /// ID do campo personalizado
        #[arg(short = 'f', long)]
        field_id: String,

        /// Tipo do campo (text, number, date, dropdown, rating, etc)
        #[arg(long)]
        field_type: String,

        /// Valor do campo
        #[arg(short = 'v', long)]
        value: String,
    },

    /// Obtém informações do usuário autenticado
    User,

    /// Lista teams/workspaces disponíveis
    Teams,

    /// Testa a conexão com a API
    Test,

    /// Mostra estatísticas do cache
    CacheStats,

    /// Limpa o cache de pesquisas
    CacheClear,
}

/// Estrutura para resposta padronizada
#[derive(serde::Serialize)]
struct CliResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

impl CliResponse {
    fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Configura logging
    if cli.verbose {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("debug")
        ).init();
    } else {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or("error")
        ).init();
    }

    // Armazena o formato de saída antes de passar cli para execute_command
    let output_format = cli.output.clone();

    // Executa comando
    let result = execute_command(&cli).await;

    // Handle do resultado
    match result {
        Ok(response) => {
            let exit_code = if response.success { 0 } else { 1 };
            output_response(response, &output_format);
            std::process::exit(exit_code);
        },
        Err(e) => {
            eprintln!("❌ Erro: {}", e);
            std::process::exit(1);
        }
    }
}

async fn execute_command(cli: &Cli) -> Result<CliResponse, Box<dyn std::error::Error>> {
    match &cli.command {
        Commands::Auth { force } => {
            handle_auth(*force).await
        },

        Commands::CreateTask {
            list_id, list_name, name, description, markdown, priority,
            status, assignees, tags, due_date, due_time, start_date,
            start_time, time_estimate_hours, notify_all, custom_fields
        } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            // Resolve list_id se necessário
            let list_id = match (list_id, list_name) {
                (Some(id), _) => id.clone(),
                (_, Some(name)) => {
                    client.find_list_by_name(&name, cli.team_id.clone())
                        .await?
                        .ok_or_else(|| format!("Lista '{}' não encontrada", name))?
                },
                _ => return Err("Forneça --list-id ou --list-name".into()),
            };

            // Constrói o request
            let mut builder = CreateTaskRequest::builder(name.clone());

            if let Some(desc) = description {
                if *markdown {
                    builder = builder.markdown_content(desc.clone());
                } else {
                    builder = builder.content(desc.clone());
                }
            }

            if let Some(p) = priority {
                let priority = match *p {
                    1 => TaskPriority::Urgent,
                    2 => TaskPriority::High,
                    3 => TaskPriority::Normal,
                    4 => TaskPriority::Low,
                    _ => TaskPriority::Normal,
                };
                builder = builder.priority(priority);
            }

            if let Some(s) = status {
                builder = builder.status(s.clone());
            }

            if let Some(a) = assignees {
                let ids: Vec<i64> = a.split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                builder = builder.assignees(ids);
            }

            if let Some(t) = tags {
                let tags: Vec<String> = t.split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                builder = builder.tags(tags);
            }

            if let Some(dd) = due_date {
                let timestamp = parse_date_to_timestamp(&dd)?;
                builder = builder.due_date(timestamp, *due_time);
            }

            if let Some(sd) = start_date {
                let timestamp = parse_date_to_timestamp(&sd)?;
                builder = builder.start_date(timestamp, *start_time);
            }

            if let Some(hours) = time_estimate_hours {
                let millis = (hours * 3600.0 * 1000.0) as i64;
                builder = builder.time_estimate(millis);
            }

            if *notify_all {
                builder = builder.notify_all(true);
            }

            // Processa campos personalizados
            if let Some(fields_str) = custom_fields {
                let fields = parse_custom_fields(&fields_str)?;
                builder = builder.custom_fields(fields);
            }

            let request = builder.build();

            // Cria a task
            match client.create_task(&list_id, request).await {
                Ok(task) => {
                    Ok(CliResponse::success(json!({
                        "id": task.id,
                        "name": task.name,
                        "url": task.url,
                        "status": task.status,
                        "created": task.date_created,
                        "message": "Task criada com sucesso!"
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::QuickTask { list_id, name, description } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.create_simple_task(&list_id, &name, description.as_deref()).await {
                Ok(task) => {
                    Ok(CliResponse::success(json!({
                        "id": task.id,
                        "name": task.name,
                        "url": task.url,
                        "message": "Task criada com sucesso!"
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::Search { name, entity, no_cache } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            if *no_cache {
                client.clear_search_cache();
            }

            let entity_type = match entity.to_lowercase().as_str() {
                "space" => EntityType::Space,
                "folder" => EntityType::Folder,
                "list" => EntityType::List,
                "task" => EntityType::Task,
                _ => return Err(format!("Tipo de entidade inválido: {}", entity).into()),
            };

            match client.search_entity(&name, entity_type, cli.team_id.clone()).await {
                Ok(result) => {
                    Ok(CliResponse::success(json!({
                        "found": result.found,
                        "count": result.items.len(),
                        "from_cache": result.cached_at.is_some(),
                        "items": result.items,
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::ListSpaces { team_id } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            let team_id = team_id.clone().or(cli.team_id.clone())
                .or_else(|| std::env::var("CLICKUP_TEAM_ID").ok())
                .unwrap_or_else(|| String::new());

            let team_id = if team_id.is_empty() {
                client.get_first_workspace_id().await?
            } else {
                team_id
            };

            match client.get_spaces(&team_id).await {
                Ok(spaces) => Ok(CliResponse::success(spaces)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::ListFolders { space_id } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_folders(&space_id).await {
                Ok(folders) => Ok(CliResponse::success(folders)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::ListLists { space_id, folder_id } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_lists(space_id.as_deref(), folder_id.as_deref()).await {
                Ok(lists) => Ok(CliResponse::success(lists)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::ShowFields { list_id } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_custom_fields(&list_id).await {
                Ok(fields) => Ok(CliResponse::success(fields)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::FindList { name } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.find_list_by_name(&name, cli.team_id.clone()).await {
                Ok(Some(id)) => {
                    Ok(CliResponse::success(json!({
                        "found": true,
                        "list_id": id,
                        "message": format!("Lista '{}' encontrada", name)
                    })))
                },
                Ok(None) => {
                    Ok(CliResponse::success(json!({
                        "found": false,
                        "message": format!("Lista '{}' não encontrada", name)
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::GetTask { task_id } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_task(&task_id).await {
                Ok(task) => Ok(CliResponse::success(task)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::UpdateField { task_id, field_id, field_type, value } => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            let field_value = parse_field_value(&field_type, &value)?;

            match client.update_custom_field(&task_id, &field_id, field_value).await {
                Ok(result) => {
                    Ok(CliResponse::success(json!({
                        "success": true,
                        "message": "Campo atualizado com sucesso",
                        "result": result
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::User => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_user_info().await {
                Ok(user) => {
                    Ok(CliResponse::success(json!({
                        "id": user.id,
                        "username": user.username,
                        "email": user.email,
                        "initials": user.initials,
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::Teams => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.get_teams_info().await {
                Ok(teams) => {
                    let teams_json: Vec<_> = teams.iter().map(|t| json!({
                        "id": t.id,
                        "name": t.name,
                        "color": t.color,
                    })).collect();

                    Ok(CliResponse::success(json!({
                        "count": teams_json.len(),
                        "teams": teams_json
                    })))
                },
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::Test => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());

            match client.test_connection().await {
                Ok(result) => Ok(CliResponse::success(result)),
                Err(e) => Ok(CliResponse::error(e.to_string())),
            }
        },

        Commands::CacheStats => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());
            let stats = client.get_cache_stats();
            Ok(CliResponse::success(stats))
        },

        Commands::CacheClear => {
            let token = get_token(cli)?;
            let client = ClickUpClient::new(token, cli.api_url.clone());
            client.clear_search_cache();
            Ok(CliResponse::success(json!({
                "message": "Cache limpo com sucesso"
            })))
        },
    }
}

fn get_token(cli: &Cli) -> Result<String, Box<dyn std::error::Error>> {
    cli.token.clone()
        .or_else(|| std::env::var("CLICKUP_ACCESS_TOKEN").ok())
        .ok_or_else(|| "Token não fornecido. Use --token ou defina CLICKUP_ACCESS_TOKEN".into())
}

async fn handle_auth(force: bool) -> Result<CliResponse, Box<dyn std::error::Error>> {
    let oauth_flow = OAuthFlow::new()?;

    if !force && oauth_flow.is_authenticated().await {
        return Ok(CliResponse::success(json!({
            "message": "Já autenticado. Use --force para reautenticar"
        })));
    }

    println!("🔐 Iniciando fluxo de autenticação OAuth2...");
    println!("📌 Um navegador será aberto para você autorizar o acesso.");

    let token = if force {
        oauth_flow.force_reauth().await?
    } else {
        oauth_flow.authenticate().await?
    };

    Ok(CliResponse::success(json!({
        "message": "Autenticação concluída com sucesso!",
        "token_preview": format!("{}...{}", &token[..4], &token[token.len()-4..]),
        "note": "Token salvo no ambiente"
    })))
}

fn parse_date_to_timestamp(date_str: &str) -> Result<i64, Box<dyn std::error::Error>> {
    // Tenta parse como timestamp direto
    if let Ok(timestamp) = date_str.parse::<i64>() {
        return Ok(timestamp);
    }

    // Tenta parse como data YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0)
            .ok_or("Erro ao criar datetime")?;
        let timestamp = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc).timestamp_millis();
        return Ok(timestamp);
    }

    Err(format!("Formato de data inválido: {}. Use YYYY-MM-DD ou timestamp", date_str).into())
}

fn parse_custom_fields(fields_str: &str) -> Result<Vec<CustomField>, Box<dyn std::error::Error>> {
    let mut fields = Vec::new();

    for field_str in fields_str.split(',') {
        let parts: Vec<&str> = field_str.trim().split(':').collect();

        if parts.len() != 3 {
            return Err(format!("Formato inválido: {}. Use id:tipo:valor", field_str).into());
        }

        let id = parts[0].to_string();
        let field_type = parts[1];
        let value_str = parts[2];

        let value = parse_field_value(field_type, value_str)?;

        fields.push(CustomField { id, value });
    }

    Ok(fields)
}

fn parse_field_value(field_type: &str, value_str: &str) -> Result<CustomFieldValue, Box<dyn std::error::Error>> {
    match field_type.to_lowercase().as_str() {
        "text" => Ok(CustomFieldValue::Text(value_str.to_string())),
        "number" => {
            let num = value_str.parse::<f64>()
                .map_err(|_| format!("Valor numérico inválido: {}", value_str))?;
            Ok(CustomFieldValue::Number(num))
        },
        "boolean" | "bool" => {
            let val = value_str.to_lowercase() == "true" || value_str == "1";
            Ok(CustomFieldValue::Boolean(val))
        },
        "date" => {
            let timestamp = parse_date_to_timestamp(value_str)?;
            Ok(CustomFieldValue::Date(timestamp))
        },
        "url" => Ok(CustomFieldValue::Url(value_str.to_string())),
        "email" => Ok(CustomFieldValue::Email(value_str.to_string())),
        "phone" => Ok(CustomFieldValue::Phone(value_str.to_string())),
        "dropdown" => Ok(CustomFieldValue::DropdownOption(value_str.to_string())),
        "rating" => {
            let rating = value_str.parse::<i32>()
                .map_err(|_| format!("Rating inválido: {}", value_str))?;
            if rating < 1 || rating > 5 {
                return Err("Rating deve ser entre 1 e 5".into());
            }
            Ok(CustomFieldValue::Rating(rating))
        },
        "currency" | "money" => {
            let amount = value_str.parse::<f64>()
                .map_err(|_| format!("Valor monetário inválido: {}", value_str))?;
            Ok(CustomFieldValue::Currency(amount))
        },
        _ => Err(format!("Tipo de campo desconhecido: {}", field_type).into()),
    }
}

fn output_response(response: CliResponse, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&response).unwrap());
        },
        OutputFormat::Pretty => {
            if response.success {
                if let Some(data) = response.data {
                    println!("✅ Sucesso!");
                    println!("{}", serde_json::to_string_pretty(&data).unwrap());
                }
            } else if let Some(error) = response.error {
                eprintln!("❌ Erro: {}", error);
            }
        },
    }
}