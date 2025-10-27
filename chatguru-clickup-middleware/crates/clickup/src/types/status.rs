//! Status de tarefas do ClickUp
//!
//! Status são configurados por space/list no ClickUp. Exemplos comuns:
//! - "to do" / "pendente"
//! - "in progress" / "em andamento"
//! - "review" / "revisão"
//! - "done" / "concluído"
//!
//! IMPORTANTE: Status não são globais - cada lista pode ter seus próprios status personalizados.

use serde::{Deserialize, Serialize};

/// Representa um status de tarefa do ClickUp
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// ID do status (opcional em algumas operações)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Nome do status (e.g., "pendente", "em andamento", "concluído")
    pub status: String,

    /// Cor do status (hex color, e.g., "#FF0000")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Ordem do status na lista
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderindex: Option<i32>,

    /// Tipo do status (open, closed, custom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

impl Status {
    /// Cria um novo status com apenas o nome
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            id: None,
            status: status.into(),
            color: None,
            orderindex: None,
            type_: None,
        }
    }

    /// Cria um status com nome e ID
    pub fn with_id(id: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            status: status.into(),
            color: None,
            orderindex: None,
            type_: None,
        }
    }

    /// Cria um status completo
    pub fn full(
        id: impl Into<String>,
        status: impl Into<String>,
        color: impl Into<String>,
        orderindex: i32,
        type_: impl Into<String>,
    ) -> Self {
        Self {
            id: Some(id.into()),
            status: status.into(),
            color: Some(color.into()),
            orderindex: Some(orderindex),
            type_: Some(type_.into()),
        }
    }
}

/// Status pré-definidos comuns (nomes em português para projeto Nordja)
impl Status {
    /// Status "Pendente" (padrão)
    pub fn pendente() -> Self {
        Self::new("pendente")
    }

    /// Status "Em Andamento"
    pub fn em_andamento() -> Self {
        Self::new("em andamento")
    }

    /// Status "Revisão"
    pub fn revisao() -> Self {
        Self::new("revisão")
    }

    /// Status "Concluído"
    pub fn concluido() -> Self {
        Self::new("concluído")
    }

    /// Status "Bloqueado"
    pub fn bloqueado() -> Self {
        Self::new("bloqueado")
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_new() {
        let status = Status::new("pendente");
        assert_eq!(status.status, "pendente");
        assert!(status.id.is_none());
        assert!(status.color.is_none());
    }

    #[test]
    fn test_status_with_id() {
        let status = Status::with_id("123", "em andamento");
        assert_eq!(status.id, Some("123".to_string()));
        assert_eq!(status.status, "em andamento");
    }

    #[test]
    fn test_status_presets() {
        assert_eq!(Status::pendente().status, "pendente");
        assert_eq!(Status::em_andamento().status, "em andamento");
        assert_eq!(Status::revisao().status, "revisão");
        assert_eq!(Status::concluido().status, "concluído");
        assert_eq!(Status::bloqueado().status, "bloqueado");
    }

    #[test]
    fn test_status_display() {
        let status = Status::new("teste");
        assert_eq!(status.to_string(), "teste");
    }
}
