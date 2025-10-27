//! Task types do ClickUp
//!
//! Estrutura completa de uma tarefa do ClickUp, incluindo todos os campos
//! disponíveis na API v2.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{CustomField, Priority, Status, User};

/// Representa uma tarefa completa do ClickUp
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    // ==================== IDENTIFICAÇÃO ====================
    /// ID da tarefa (retornado pela API após criação)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Nome/título da tarefa (obrigatório na criação)
    pub name: String,

    /// Descrição da tarefa (suporta Markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // ==================== ORGANIZAÇÃO ====================
    /// ID da lista onde a tarefa será criada (obrigatório na criação)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_id: Option<String>,

    /// ID do folder (read-only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<JsonValue>,

    /// ID do space (read-only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space: Option<JsonValue>,

    /// ID do projeto (read-only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<JsonValue>,

    // ==================== STATUS & PRIORIDADE ====================
    /// Status da tarefa (e.g., "pendente", "em andamento", "concluído")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,

    /// Prioridade da tarefa (1=Urgente, 2=Alta, 3=Normal, 4=Baixa)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    // ==================== RESPONSÁVEIS ====================
    /// Lista de usuários atribuídos à tarefa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<User>>,

    /// Watchers da tarefa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchers: Option<Vec<User>>,

    // ==================== DATAS ====================
    /// Data de criação (timestamp em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_created: Option<String>,

    /// Data de última atualização (timestamp em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_updated: Option<String>,

    /// Data de fechamento (timestamp em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_closed: Option<String>,

    /// Data de início (timestamp em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,

    /// Data de entrega (timestamp em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<i64>,

    /// Include time in due date?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_time: Option<bool>,

    // ==================== TEMPO ====================
    /// Estimativa de tempo (em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_estimate: Option<i64>,

    /// Tempo rastreado (em milissegundos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_spent: Option<i64>,

    // ==================== RELACIONAMENTOS ====================
    /// ID da tarefa pai (para subtasks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// Dependências (tasks que esta task bloqueia)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<TaskDependency>>,

    /// Tasks linkadas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_tasks: Option<Vec<LinkedTask>>,

    // ==================== CAMPOS PERSONALIZADOS ====================
    /// Campos personalizados
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomField>>,

    // ==================== TAGS & CATEGORIZAÇÃO ====================
    /// Tags da tarefa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    // ==================== ANEXOS & COMENTÁRIOS ====================
    /// Número de comentários
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_count: Option<i32>,

    /// Anexos da tarefa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,

    // ==================== OUTROS ====================
    /// URL da tarefa no ClickUp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Task está arquivada?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,

    /// Ordem da task na lista
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderindex: Option<String>,

    /// Criador da task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<User>,

    /// Pontos (se list usar pontos)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<f64>,

    /// Notificar todos os assignees?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_all: Option<bool>,

    /// Markdown description?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_description: Option<String>,
}

/// Representa uma dependência de tarefa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDependency {
    /// ID da tarefa dependente
    pub task_id: String,

    /// Tipo de dependência:
    /// - "waiting on" = esta task espera pela outra
    /// - "blocking" = esta task bloqueia a outra
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<String>,

    /// ID do link de dependência
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_of: Option<String>,
}

/// Representa uma tarefa linkada
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedTask {
    /// ID da tarefa linkada
    pub task_id: String,

    /// Tipo de link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,

    /// Data de criação do link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_created: Option<String>,
}

/// Representa um anexo de tarefa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    /// ID do anexo
    pub id: String,

    /// Nome do arquivo
    pub title: String,

    /// URL do arquivo
    pub url: String,

    /// Tamanho em bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// Data de upload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Usuário que fez upload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

impl Task {
    /// Cria uma nova tarefa com campos mínimos obrigatórios
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: None,
            list_id: None,
            folder: None,
            space: None,
            project: None,
            status: None,
            priority: None,
            assignees: None,
            watchers: None,
            date_created: None,
            date_updated: None,
            date_closed: None,
            start_date: None,
            due_date: None,
            due_date_time: None,
            time_estimate: None,
            time_spent: None,
            parent: None,
            dependencies: None,
            linked_tasks: None,
            custom_fields: None,
            tags: None,
            comment_count: None,
            attachments: None,
            url: None,
            archived: None,
            orderindex: None,
            creator: None,
            points: None,
            notify_all: None,
            markdown_description: None,
        }
    }

    /// Builder: define a descrição
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builder: define a lista
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Builder: define a prioridade
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Builder: define o status
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Builder: adiciona assignees
    pub fn with_assignees(mut self, assignees: Vec<User>) -> Self {
        self.assignees = Some(assignees);
        self
    }

    /// Builder: adiciona tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Builder: adiciona custom fields
    pub fn with_custom_fields(mut self, custom_fields: Vec<CustomField>) -> Self {
        self.custom_fields = Some(custom_fields);
        self
    }

    /// Builder: define due date (timestamp em milissegundos)
    pub fn with_due_date(mut self, timestamp_ms: i64) -> Self {
        self.due_date = Some(timestamp_ms);
        self
    }

    /// Builder: define tarefa pai (para subtasks)
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent = Some(parent_id.into());
        self
    }

    /// Builder: notificar todos assignees
    pub fn notify_all(mut self, notify: bool) -> Self {
        self.notify_all = Some(notify);
        self
    }
}

/// Builder auxiliar para criação de tarefas
pub struct TaskBuilder {
    task: Task,
}

impl TaskBuilder {
    /// Cria um novo builder com nome obrigatório
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            task: Task::new(name),
        }
    }

    /// Define a descrição
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.task = self.task.with_description(description);
        self
    }

    /// Define a lista
    pub fn list_id(mut self, list_id: impl Into<String>) -> Self {
        self.task = self.task.with_list_id(list_id);
        self
    }

    /// Define a prioridade
    pub fn priority(mut self, priority: Priority) -> Self {
        self.task = self.task.with_priority(priority);
        self
    }

    /// Define o status
    pub fn status(mut self, status: Status) -> Self {
        self.task = self.task.with_status(status);
        self
    }

    /// Adiciona assignees
    pub fn assignees(mut self, assignees: Vec<User>) -> Self {
        self.task = self.task.with_assignees(assignees);
        self
    }

    /// Adiciona tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.task = self.task.with_tags(tags);
        self
    }

    /// Adiciona custom fields
    pub fn custom_fields(mut self, custom_fields: Vec<CustomField>) -> Self {
        self.task = self.task.with_custom_fields(custom_fields);
        self
    }

    /// Define due date
    pub fn due_date(mut self, timestamp_ms: i64) -> Self {
        self.task = self.task.with_due_date(timestamp_ms);
        self
    }

    /// Define tarefa pai (para subtasks)
    pub fn parent(mut self, parent_id: impl Into<String>) -> Self {
        self.task = self.task.with_parent(parent_id);
        self
    }

    /// Notificar todos assignees
    pub fn notify_all(mut self, notify: bool) -> Self {
        self.task = self.task.notify_all(notify);
        self
    }

    /// Constrói a task
    pub fn build(self) -> Task {
        self.task
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_new() {
        let task = Task::new("Test Task");
        assert_eq!(task.name, "Test Task");
        assert!(task.id.is_none());
        assert!(task.description.is_none());
    }

    #[test]
    fn test_task_builder() {
        let task = Task::new("Test")
            .with_description("Description")
            .with_list_id("123")
            .with_priority(Priority::High)
            .with_tags(vec!["urgent".to_string()]);

        assert_eq!(task.name, "Test");
        assert_eq!(task.description, Some("Description".to_string()));
        assert_eq!(task.list_id, Some("123".to_string()));
        assert_eq!(task.priority, Some(Priority::High));
        assert_eq!(task.tags, Some(vec!["urgent".to_string()]));
    }

    #[test]
    fn test_task_builder_pattern() {
        let task = TaskBuilder::new("Builder Test")
            .description("Using builder pattern")
            .list_id("456")
            .priority(Priority::Urgent)
            .notify_all(true)
            .build();

        assert_eq!(task.name, "Builder Test");
        assert_eq!(task.list_id, Some("456".to_string()));
        assert_eq!(task.notify_all, Some(true));
    }
}
