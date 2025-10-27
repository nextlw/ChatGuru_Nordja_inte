// ============================================================================
// Task Manager - Gerenciamento de tarefas do ClickUp
// ============================================================================
//
// Este módulo encapsula todas as operações de gerenciamento de tarefas:
//
// 1. **CRUD de Tarefas:**
//    - Criação de tarefas (create_task)
//    - Atualização de tarefas existentes (update_task)
//    - Busca de duplicatas (find_existing_task)
//    - Adição de comentários (add_comment)
//
// 2. **Administração:**
//    - Teste de conectividade (test_connection)
//    - Obtenção de informações de lista (get_list_info)
//
// # Autenticação
//
// Suporta dois modos:
// - **Personal Token**: Token fixo com todas as permissões
// - **OAuth2 Access Token**: Token dinâmico com permissões limitadas
//
// # Tratamento de Erros OAuth2
//
// Quando usando OAuth2, alguns endpoints podem retornar erro `OAUTH_027`
// (Team not authorized). O código trata isso gracefully retornando `None`
// ao invés de falhar completamente.

use crate::client::ClickUpClient;
use crate::error::{ClickUpError, Result};
use crate::types::Task;
use serde_json::{json, Value};

/// Gerenciador de tarefas do ClickUp
///
/// # Campos
///
/// - `client`: Cliente HTTP abstrato (ClickUpClient) com suporte a OAuth2
/// - `list_id`: ID da lista padrão/fallback para criação de tarefas (opcional)
///
/// # Autenticação
///
/// O token de autenticação é gerenciado pelo `ClickUpClient`:
/// - **Personal Token**: `pk_xxxxx` (todas as permissões)
/// - **OAuth2 Token**: Access token (permissões limitadas)
///
/// # Thread-Safety
///
/// Este struct implementa `Clone` e pode ser compartilhado entre threads via `Arc<>`.
#[derive(Clone)]
pub struct TaskManager {
    client: ClickUpClient,
    list_id: Option<String>,
}

impl TaskManager {
    /// Cria uma nova instância do TaskManager
    ///
    /// # Argumentos
    ///
    /// - `client`: Cliente ClickUp já configurado com autenticação
    /// - `list_id`: ID da lista padrão (opcional)
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// use clickup::ClickUpClient;
    /// use clickup::tasks::TaskManager;
    ///
    /// let client = ClickUpClient::new("pk_your_token")?;
    /// let manager = TaskManager::new(client, Some("list_123".to_string()));
    /// ```
    pub fn new(client: ClickUpClient, list_id: Option<String>) -> Self {
        Self { client, list_id }
    }

    /// Cria um TaskManager a partir de um token (conveniência)
    ///
    /// # Argumentos
    ///
    /// - `api_token`: Token de autenticação (Personal ou OAuth2)
    /// - `list_id`: ID da lista padrão (opcional)
    pub fn from_token(api_token: String, list_id: Option<String>) -> Result<Self> {
        let client = ClickUpClient::new(api_token)?;
        Ok(Self::new(client, list_id))
    }


    /// Cria uma tarefa no ClickUp (API tipada)
    ///
    /// # Descrição
    ///
    /// Cria uma tarefa na lista especificada. Suporta duas formas:
    /// 1. `task.list_id` presente (SmartFolderFinder configura isso)
    /// 2. Fallback para `self.list_id` se não configurado
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/list/{list_id}/task`
    ///
    /// # Argumentos
    ///
    /// - `task`: Tarefa a criar (use TaskBuilder para construir)
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// use clickup::{Task, TaskBuilder, Priority};
    ///
    /// let task = TaskBuilder::new("Nova tarefa")
    ///     .description("Descrição da tarefa")
    ///     .priority(Priority::High)
    ///     .list_id("list-123")
    ///     .build();
    ///
    /// let created = task_manager.create_task(&task).await?;
    /// println!("Task criada: {}", created.id.unwrap());
    /// ```
    ///
    /// # Retorno
    ///
    /// - `Ok(Task)`: Tarefa criada com sucesso (inclui ID, URL, timestamps)
    /// - `Err(ClickUpError)`: Falha na criação
    ///
    /// # Erros Comuns
    ///
    /// - **ValidationError**: list_id não encontrado na task nem no TaskManager
    /// - **401 Unauthorized**: Token inválido ou expirado
    /// - **403 Forbidden**: Sem permissão para criar tarefas nesta lista
    /// - **404 Not Found**: list_id não existe
    /// - **400 Bad Request**: Estrutura inválida ou custom_field_id incorreto
    pub async fn create_task(&self, task: &Task) -> Result<Task> {
        // Extrair list_id da task ou usar fallback
        let list_id = if let Some(ref id) = task.list_id {
            tracing::info!("🎯 Usando list_id da task: {}", id);
            id.clone()
        } else if let Some(ref id) = self.list_id {
            tracing::info!("⚠️ list_id não encontrado na task, usando fallback: {}", id);
            id.clone()
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não encontrado na task e TaskManager não tem list_id configurado".to_string()
            ));
        };

        // Serializar task para JSON
        let mut task_json = serde_json::to_value(task)?;

        // Remover list_id do body (API espera na URL, não no body)
        if let Some(obj) = task_json.as_object_mut() {
            obj.remove("list_id");
            // Também remover campos read-only que API não aceita no POST
            obj.remove("id");
            obj.remove("url");
            obj.remove("date_created");
            obj.remove("date_updated");
            obj.remove("date_closed");
            obj.remove("creator");
            obj.remove("folder");
            obj.remove("space");
            obj.remove("project");
        }

        // POST /list/{list_id}/task
        let endpoint = format!("/list/{}/task", list_id);
        let created_task: Task = self.client.post_json(&endpoint, &task_json).await?;

        tracing::info!("✅ Task criada: {}", created_task.id.as_ref().unwrap_or(&"?".to_string()));
        Ok(created_task)
    }

    /// Testa conectividade com a API do ClickUp
    ///
    /// # Descrição
    ///
    /// Faz uma chamada simples à API para validar:
    /// - Token de autenticação está válido
    /// - API está acessível
    /// - Permissões básicas estão corretas
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/user`
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Conexão bem-sucedida, retorna informações do usuário:
    ///   ```json
    ///   {
    ///     "user": {
    ///       "id": 123456,
    ///       "username": "user@example.com",
    ///       "email": "user@example.com",
    ///       "color": "#FF0000"
    ///     }
    ///   }
    ///   ```
    /// - `Err(ClickUpError)`: Falha na conexão ou autenticação
    ///
    /// # Uso
    ///
    /// - Health checks
    /// - Validação de configuração no startup
    /// - Debug de problemas de autenticação
    pub async fn test_connection(&self) -> Result<Value> {
        let user_info: Value = self.client.get_json("/user").await?;
        Ok(user_info)
    }

    /// Obtém informações detalhadas sobre uma lista
    ///
    /// # Descrição
    ///
    /// Busca metadados completos da lista, incluindo:
    /// - Nome e descrição da lista
    /// - Status disponíveis (To Do, In Progress, Done, etc)
    /// - Custom fields e seus IDs
    /// - Configurações de permissões
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/list/{list_id}`
    ///
    /// # Argumentos
    ///
    /// - `list_id`: ID da lista (se None, usa self.list_id)
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Informações da lista em formato JSON
    ///   ```json
    ///   {
    ///     "id": "901321080769",
    ///     "name": "OUTUBRO 2025",
    ///     "status": [...],
    ///     "custom_fields": [
    ///       {
    ///         "id": "field-uuid",
    ///         "name": "Categoria",
    ///         "type": "drop_down",
    ///         "type_config": { "options": [...] }
    ///       }
    ///     ]
    ///   }
    ///   ```
    /// - `Err(ClickUpError)`: Falha ao buscar informações
    ///
    /// # Uso
    ///
    /// - Debug: verificar custom fields disponíveis
    /// - Mapeamento: obter IDs de campos para usar em payloads
    /// - Validação: conferir se lista está corretamente configurada
    pub async fn get_list_info(&self, list_id: Option<&str>) -> Result<Value> {
        let id = if let Some(id) = list_id {
            id
        } else if let Some(ref id) = self.list_id {
            id
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não fornecido e TaskManager não tem list_id configurado".to_string()
            ));
        };

        let endpoint = format!("/list/{}", id);
        let list_info: Value = self.client.get_json(&endpoint).await?;
        Ok(list_info)
    }



    /// Lista todas as tarefas de uma lista (não arquivadas)
    ///
    /// # Descrição
    ///
    /// Retorna todas as tarefas não-arquivadas de uma lista específica.
    /// Usado para debug, listagens e verificações administrativas.
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/list/{list_id}/task?archived=false`
    ///
    /// # Argumentos
    ///
    /// - `list_id`: ID da lista (se None, usa self.list_id)
    ///
    /// # Retorno
    ///
    /// - `Ok(Vec<Task>)`: Array de tarefas encontradas (pode ser vazio)
    /// - `Err(ClickUpError)`: Erro na comunicação com API
    ///
    /// # Exemplo
    ///
    /// ```rust,no_run
    /// # use clickup::tasks::TaskManager;
    /// # async fn example(manager: &TaskManager) -> clickup::Result<()> {
    /// let tasks = manager.get_tasks_in_list(None).await?;
    /// println!("Total de tasks: {}", tasks.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tasks_in_list(&self, list_id: Option<&str>) -> Result<Vec<Task>> {
        let id = if let Some(id) = list_id {
            id
        } else if let Some(ref id) = self.list_id {
            id
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não fornecido e TaskManager não tem list_id configurado".to_string()
            ));
        };

        let endpoint = format!("/list/{}/task?archived=false", id);

        let json_resp: Value = self.client.get_json(&endpoint).await?;

        // Extrair array de tasks e desserializar
        if let Some(tasks_array) = json_resp.get("tasks").and_then(|v| v.as_array()) {
            let mut tasks = Vec::new();
            for task_value in tasks_array {
                match serde_json::from_value::<Task>(task_value.clone()) {
                    Ok(task) => tasks.push(task),
                    Err(e) => {
                        tracing::warn!("⚠️ Falha ao desserializar task: {}", e);
                        // Continua processando outras tasks
                    }
                }
            }
            tracing::info!("✅ Listadas {} tasks da lista {}", tasks.len(), id);
            Ok(tasks)
        } else {
            // Se não houver campo "tasks", retornar array vazio
            tracing::warn!("⚠️ Resposta da API sem campo 'tasks'");
            Ok(Vec::new())
        }
    }

    /// Busca uma tarefa existente na lista pelo título (detecção de duplicatas)
    ///
    /// # Descrição
    ///
    /// Lista todas as tarefas não-arquivadas da lista e busca por título exato.
    /// Usado para detectar duplicatas antes de criar nova tarefa.
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/list/{list_id}/task?archived=false`
    ///
    /// # Argumentos
    ///
    /// - `list_id`: ID da lista (se None, usa self.list_id)
    /// - `title`: Título exato da tarefa para buscar (case-sensitive)
    ///
    /// # Retorno
    ///
    /// - `Ok(Some(Task))`: Tarefa encontrada com título exato
    /// - `Ok(None)`: Nenhuma tarefa com este título encontrada
    /// - `Err(ClickUpError)`: Erro na comunicação com API
    ///
    /// # Tratamento Especial de Permissões OAuth2
    ///
    /// Se o token OAuth2 não tiver permissão para listar tarefas (erro OAUTH_027),
    /// retorna `Ok(None)` ao invés de falhar. Isso permite que o fluxo continue
    /// criando a tarefa mesmo sem poder verificar duplicatas.
    ///
    /// **Códigos de erro tolerados:**
    /// - `OAUTH_027`: Team not authorized
    /// - Mensagens contendo "Team not authorized"
    ///
    /// # Performance
    ///
    /// ATENÇÃO: Este método pode ser lento se a lista tiver muitas tarefas (>1000),
    /// pois a API do ClickUp não suporta busca por título. Considera-se implementar
    /// cache ou índice em banco de dados para listas grandes.
    pub async fn find_existing_task_in_list(
        &self,
        list_id: Option<&str>,
        title: &str,
    ) -> Result<Option<Task>> {
        let id = if let Some(id) = list_id {
            id
        } else if let Some(ref id) = self.list_id {
            id
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não fornecido e TaskManager não tem list_id configurado".to_string()
            ));
        };

        let endpoint = format!("/list/{}/task?archived=false", id);

        // Tentar listar tarefas da lista
        let json_resp: Value = match self.client.get_json(&endpoint).await {
            Ok(resp) => resp,
            Err(ClickUpError::ApiError { status, message }) => {
                // TRATAMENTO ESPECIAL OAUTH2: Se token não tem permissão para listar,
                // retornar None (assume que não há duplicatas) em vez de falhar
                if message.contains("OAUTH_027") || message.contains("Team not authorized") {
                    tracing::warn!(
                        "⚠️ Token OAuth2 sem permissão para listar tasks ({}). Assumindo que não há duplicatas.",
                        message
                    );
                    return Ok(None);
                }

                // Outros erros de API: propagar
                tracing::error!("Erro ao listar tasks na lista {}: {} - {}", id, status, message);
                return Err(ClickUpError::ApiError { status, message });
            }
            Err(e) => {
                // Outros tipos de erro (network, etc): propagar
                return Err(e);
            }
        };

        // Buscar tarefa com título exato (case-sensitive)
        if let Some(tasks) = json_resp.get("tasks").and_then(|v| v.as_array()) {
            for task_value in tasks {
                if let Some(task_name) = task_value.get("name").and_then(|v| v.as_str()) {
                    if task_name == title {
                        tracing::info!("✅ Tarefa existente encontrada: '{}'", title);
                        // Desserializar task de Value para Task
                        let task: Task = serde_json::from_value(task_value.clone())?;
                        return Ok(Some(task));
                    }
                }
            }
        }

        // Nenhuma tarefa com este título encontrada
        tracing::debug!("ℹ️ Nenhuma tarefa encontrada com título: '{}'", title);
        Ok(None)
    }

    /// Adiciona comentário a uma tarefa existente (preservar histórico)
    ///
    /// # Descrição
    ///
    /// Cria um novo comentário na tarefa especificada. Usado para:
    /// - Registrar histórico de atualizações
    /// - Preservar dados da versão anterior ao atualizar tarefa
    /// - Adicionar contexto sobre mudanças automáticas via webhook
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/task/{task_id}/comment`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa no ClickUp (string numérica ou UUID)
    /// - `comment`: Texto do comentário (suporta Markdown)
    ///
    /// # Retorno
    ///
    /// - `Ok(())`: Comentário adicionado com sucesso
    /// - `Err(ClickUpError)`: Falha ao adicionar comentário
    ///
    /// # Formato do Comentário de Histórico
    ///
    /// Comentários criados automaticamente seguem este formato:
    /// ```markdown
    /// 📝 **Atualização Automática via Webhook**
    ///
    /// **Timestamp:** 2025-10-14T16:20:00Z
    /// **Tipo de Payload:** ChatGuru
    ///
    /// ---
    ///
    /// **Histórico da Versão Anterior:**
    /// - **Título:** [título antigo]
    /// - **Última Atualização:** [timestamp]
    ///
    /// **Descrição Anterior:**
    /// ```
    /// [descrição antiga]
    /// ```
    /// ```
    ///
    /// # Uso
    ///
    /// ```rust,ignore
    /// task_manager.add_comment_to_task(
    ///     "task-123",
    ///     "📝 Tarefa atualizada automaticamente via webhook"
    /// ).await?;
    /// ```
    pub async fn add_comment_to_task(&self, task_id: &str, comment: &str) -> Result<()> {
        let endpoint = format!("/task/{}/comment", task_id);
        let body = json!({
            "comment_text": comment
        });

        // POST /task/{task_id}/comment
        let _response: Value = self.client.post_json(&endpoint, &body).await?;

        tracing::debug!("✅ Comentário adicionado à task {}", task_id);
        Ok(())
    }

    /// Atualiza uma tarefa existente no ClickUp
    ///
    /// # Descrição
    ///
    /// Atualiza campos de uma tarefa existente via API PUT. Apenas os campos
    /// presentes em `task_data` serão atualizados (patch parcial).
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa a atualizar
    /// - `task_data`: JSON com campos a atualizar:
    ///   ```json
    ///   {
    ///     "name": "Novo título",
    ///     "description": "Nova descrição",
    ///     "priority": 1,
    ///     "custom_fields": [...]
    ///   }
    ///   ```
    ///
    /// # Comportamento de Atualização
    ///
    /// - **Campos omitidos**: mantêm valor atual (não são limpos)
    /// - **Campos presentes**: substituem valor atual
    /// - **custom_fields**: SUBSTITUIÇÃO COMPLETA (não merge)
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Tarefa atualizada, retorna JSON completo da nova versão
    /// - `Err(ClickUpError)`: Falha na atualização
    ///
    /// # IMPORTANTE: Custom Fields
    ///
    /// Ao atualizar custom_fields, o array SUBSTITUI completamente os valores
    /// anteriores. Para preservar campos não modificados, inclua-os no payload.
    ///
    /// # Uso
    ///
    /// ```rust,ignore
    /// let updated = task_manager.update_task(
    ///     "task-123",
    ///     &json!({"name": "Novo título", "priority": 1})
    /// ).await?;
    /// ```
    pub async fn update_task(&self, task_id: &str, task_data: &Value) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);

        // PUT /task/{task_id}
        let updated_task: Task = self.client.put_json(&endpoint, task_data).await?;

        tracing::debug!("✅ Task {} atualizada", task_id);
        Ok(updated_task)
    }

    // ==================== ASSIGNEES (Responsáveis) ====================

    /// Atribui usuários a uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com campo `assignees.add`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `user_ids`: Lista de IDs de usuários a adicionar como assignees
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// task_manager.assign_task("task-123", &[123, 456]).await?;
    /// ```
    pub async fn assign_task(&self, task_id: &str, user_ids: &[u32]) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "assignees": {
                "add": user_ids
            }
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!("✅ Assignees {:?} adicionados à task {}", user_ids, task_id);
        Ok(updated_task)
    }

    /// Remove usuários de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com campo `assignees.rem`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `user_ids`: Lista de IDs de usuários a remover
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// task_manager.unassign_task("task-123", &[123]).await?;
    /// ```
    pub async fn unassign_task(&self, task_id: &str, user_ids: &[u32]) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "assignees": {
                "rem": user_ids
            }
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!("✅ Assignees {:?} removidos da task {}", user_ids, task_id);
        Ok(updated_task)
    }

    /// Substitui todos os assignees de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com campo `assignees.add` e `assignees.rem`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `add_user_ids`: Usuários a adicionar
    /// - `rem_user_ids`: Usuários a remover
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// // Remove 123, adiciona 456 e 789
    /// task_manager.update_assignees("task-123", &[456, 789], &[123]).await?;
    /// ```
    pub async fn update_assignees(
        &self,
        task_id: &str,
        add_user_ids: &[u32],
        rem_user_ids: &[u32],
    ) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "assignees": {
                "add": add_user_ids,
                "rem": rem_user_ids
            }
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!(
            "✅ Assignees atualizados: +{:?} -{:?} na task {}",
            add_user_ids,
            rem_user_ids,
            task_id
        );
        Ok(updated_task)
    }

    // ==================== STATUS ====================

    /// Atualiza o status de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com campo `status`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `status`: Nome do status (e.g., "pendente", "em andamento", "concluído")
    ///
    /// # IMPORTANTE
    ///
    /// Status não são globais no ClickUp - cada lista tem seus próprios status.
    /// Use nomes que existem na lista da tarefa, caso contrário a API retornará erro.
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// task_manager.update_task_status("task-123", "em andamento").await?;
    /// ```
    pub async fn update_task_status(&self, task_id: &str, status: &str) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "status": status
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!("✅ Status da task {} atualizado para: {}", task_id, status);
        Ok(updated_task)
    }

    // ==================== SUBTASKS ====================

    /// Cria uma subtask (tarefa filha) de uma tarefa existente
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/list/{list_id}/task` com campo `parent`
    ///
    /// # Argumentos
    ///
    /// - `parent_id`: ID da tarefa pai
    /// - `task_data`: Dados da subtask (deve incluir `name` no mínimo)
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// let subtask = task_manager.create_subtask(
    ///     "parent-task-123",
    ///     &json!({
    ///         "name": "Subtarefa 1",
    ///         "description": "Descrição da subtarefa"
    ///     })
    /// ).await?;
    /// ```
    pub async fn create_subtask(&self, parent_id: &str, task_data: &Value) -> Result<Task> {
        // Extract or validate list_id from task_data or use fallback
        let list_id = if let Some(id) = task_data.get("list_id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(ref id) = self.list_id {
            id.clone()
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não encontrado para criar subtask".to_string(),
            ));
        };

        // Add parent field to task_data
        let mut subtask_data = task_data.clone();
        if let Some(obj) = subtask_data.as_object_mut() {
            obj.insert("parent".to_string(), json!(parent_id));
            obj.remove("list_id"); // Remove list_id from body (goes in URL)
        }

        // POST /list/{list_id}/task
        let endpoint = format!("/list/{}/task", list_id);
        let subtask: Task = self.client.post_json(&endpoint, &subtask_data).await?;

        tracing::debug!("✅ Subtask criada: {} (pai: {})", subtask.id.as_ref().unwrap_or(&"?".to_string()), parent_id);
        Ok(subtask)
    }

    // ==================== DUE DATES ====================

    /// Define a data de entrega (due date) de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com campo `due_date`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `timestamp_ms`: Timestamp em MILISSEGUNDOS (não segundos!)
    /// - `include_time`: Se true, inclui horário; se false, apenas data
    ///
    /// # ⚠️ IMPORTANTE: Timestamps em MILISSEGUNDOS
    ///
    /// A API do ClickUp usa milissegundos, não segundos:
    /// - ✅ Correto: `1672531200000` (2023-01-01 00:00:00 UTC)
    /// - ❌ Errado: `1672531200` (segundos Unix)
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// use chrono::Utc;
    ///
    /// // Data: 2023-01-01 00:00:00 UTC
    /// let timestamp_ms = 1672531200000;
    /// task_manager.set_due_date("task-123", timestamp_ms, true).await?;
    ///
    /// // Ou usando chrono:
    /// let date = Utc::now() + chrono::Duration::days(7);
    /// let timestamp_ms = date.timestamp_millis();
    /// task_manager.set_due_date("task-123", timestamp_ms, false).await?;
    /// ```
    pub async fn set_due_date(
        &self,
        task_id: &str,
        timestamp_ms: i64,
        include_time: bool,
    ) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "due_date": timestamp_ms,
            "due_date_time": include_time
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!(
            "✅ Due date da task {} definida: {} (include_time: {})",
            task_id,
            timestamp_ms,
            include_time
        );
        Ok(updated_task)
    }

    /// Remove a data de entrega (due date) de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `PUT /api/v2/task/{task_id}` com `due_date: null`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// task_manager.clear_due_date("task-123").await?;
    /// ```
    pub async fn clear_due_date(&self, task_id: &str) -> Result<Task> {
        let endpoint = format!("/task/{}", task_id);
        let body = json!({
            "due_date": null,
            "due_date_time": false
        });

        let updated_task: Task = self.client.put_json(&endpoint, &body).await?;
        tracing::debug!("✅ Due date da task {} removida", task_id);
        Ok(updated_task)
    }

    // ==================== DEPENDENCIES ====================

    /// Adiciona uma dependência a uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/task/{task_id}/dependency`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `depends_on`: ID da tarefa da qual esta depende
    /// - `dependency_type`: Tipo de dependência:
    ///   - "waiting_on" = esta task espera pela outra completar
    ///   - "blocking" = esta task bloqueia a outra (opcional, padrão: "waiting_on")
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// // Task 123 depende de Task 456 (123 espera 456 completar)
    /// task_manager.add_dependency("123", "456", None).await?;
    /// ```
    pub async fn add_dependency(
        &self,
        task_id: &str,
        depends_on: &str,
        dependency_type: Option<&str>,
    ) -> Result<Value> {
        let endpoint = format!("/task/{}/dependency", task_id);
        let dep_type = dependency_type.unwrap_or("waiting_on");

        let body = json!({
            "depends_on": depends_on,
            "dependency_of": dep_type
        });

        let response: Value = self.client.post_json(&endpoint, &body).await?;
        tracing::debug!(
            "✅ Dependência adicionada: task {} {} task {}",
            task_id,
            dep_type,
            depends_on
        );
        Ok(response)
    }

    /// Remove uma dependência de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `DELETE /api/v2/task/{task_id}/dependency/{depends_on}`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    /// - `depends_on`: ID da tarefa dependente a remover
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// task_manager.remove_dependency("123", "456").await?;
    /// ```
    pub async fn remove_dependency(&self, task_id: &str, depends_on: &str) -> Result<Value> {
        let endpoint = format!("/task/{}/dependency/{}", task_id, depends_on);
        let response: Value = self.client.delete_json(&endpoint).await?;

        tracing::debug!("✅ Dependência removida: task {} não depende mais de {}", task_id, depends_on);
        Ok(response)
    }

    /// Lista todas as dependências de uma tarefa
    ///
    /// # Endpoint da API
    ///
    /// `GET /api/v2/task/{task_id}` e extrai campo `dependencies`
    ///
    /// # Argumentos
    ///
    /// - `task_id`: ID da tarefa
    ///
    /// # Retorno
    ///
    /// Retorna array de dependências ou array vazio se não houver.
    ///
    /// # Exemplo
    ///
    /// ```rust,ignore
    /// let deps = task_manager.get_dependencies("task-123").await?;
    /// println!("Dependências: {:?}", deps);
    /// ```
    pub async fn get_dependencies(&self, task_id: &str) -> Result<Vec<Value>> {
        let endpoint = format!("/task/{}", task_id);
        let task: Value = self.client.get_json(&endpoint).await?;

        let dependencies = task
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|arr| arr.clone())
            .unwrap_or_default();

        tracing::debug!("✅ Recuperadas {} dependências da task {}", dependencies.len(), task_id);
        Ok(dependencies)
    }

}
