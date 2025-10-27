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


    /// Cria uma tarefa no ClickUp a partir de dados JSON
    ///
    /// # Descrição
    ///
    /// Cria uma tarefa na lista especificada. Suporta duas formas:
    /// 1. `list_id` presente em `task_data` (SmartFolderFinder insere isso)
    /// 2. Fallback para `self.list_id` se não encontrado
    ///
    /// # Endpoint da API
    ///
    /// `POST /api/v2/list/{list_id}/task`
    ///
    /// # Argumentos
    ///
    /// - `task_data`: JSON com estrutura da tarefa do ClickUp:
    ///   ```json
    ///   {
    ///     "name": "Título da tarefa",
    ///     "description": "Descrição markdown",
    ///     "priority": 3,
    ///     "tags": ["tag1", "tag2"],
    ///     "custom_fields": [
    ///       {"id": "field-id", "value": "valor"}
    ///     ],
    ///     "list_id": "123456" // Opcional, usado se presente
    ///   }
    ///   ```
    ///
    /// # Retorno
    ///
    /// - `Ok(Value)`: Tarefa criada com sucesso, retorna JSON com `id`, `url`, etc
    /// - `Err(ClickUpError)`: Falha na criação
    ///
    /// # Erros Comuns
    ///
    /// - **401 Unauthorized**: Token inválido ou expirado
    /// - **403 Forbidden**: Sem permissão para criar tarefas nesta lista
    /// - **404 Not Found**: list_id não existe
    /// - **400 Bad Request**: Estrutura JSON inválida ou custom_field_id incorreto
    pub async fn create_task_from_json(&self, task_data: &Value) -> Result<Value> {
        // Extrair list_id do task_data (SmartFolderFinder insere isso) ou usar fallback
        let list_id_str;
        let list_id = if let Some(id) = task_data.get("list_id").and_then(|v| v.as_str()) {
            tracing::info!("🎯 Usando list_id do task_data: {}", id);
            id
        } else if let Some(ref id) = self.list_id {
            tracing::info!("⚠️ list_id não encontrado no task_data, usando fallback: {}", id);
            list_id_str = id.clone();
            &list_id_str
        } else {
            return Err(ClickUpError::ValidationError(
                "list_id não encontrado no task_data e TaskManager não tem list_id configurado".to_string()
            ));
        };

        // Remover list_id do task_data antes de enviar (API espera na URL, não no body)
        let mut clean_task_data = task_data.clone();
        if let Some(obj) = clean_task_data.as_object_mut() {
            obj.remove("list_id");
        }

        // POST /list/{list_id}/task
        let endpoint = format!("/list/{}/task", list_id);
        let task: Value = self.client.post_json(&endpoint, &clean_task_data).await?;

        Ok(task)
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
    /// - `Ok(Some(Value))`: Tarefa encontrada, retorna JSON completo da tarefa
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
    ) -> Result<Option<Value>> {
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
            for task in tasks {
                if let Some(task_name) = task.get("name").and_then(|v| v.as_str()) {
                    if task_name == title {
                        tracing::info!("✅ Tarefa existente encontrada: '{}'", title);
                        return Ok(Some(task.clone()));
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
    pub async fn update_task(&self, task_id: &str, task_data: &Value) -> Result<Value> {
        let endpoint = format!("/task/{}", task_id);

        // PUT /task/{task_id}
        let updated_task: Value = self.client.put_json(&endpoint, task_data).await?;

        tracing::debug!("✅ Task {} atualizada", task_id);
        Ok(updated_task)
    }

}
