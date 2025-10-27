//! Tipos relacionados a usuários do ClickUp (assignees)
//!
//! Usuários podem ser atribuídos a tarefas como responsáveis (assignees).

use serde::{Deserialize, Serialize};

/// Representa um usuário do ClickUp
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_snake_case)] // profilePicture matches ClickUp API field name
pub struct User {
    /// ID do usuário (sempre presente)
    pub id: u32,

    /// Nome de usuário
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Email do usuário
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Cor associada ao usuário (hex color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// URL da foto de perfil
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profilePicture: Option<String>,

    /// Iniciais do usuário
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initials: Option<String>,
}

impl User {
    /// Cria um usuário apenas com ID (mínimo necessário)
    pub fn new(id: u32) -> Self {
        Self {
            id,
            username: None,
            email: None,
            color: None,
            profilePicture: None,
            initials: None,
        }
    }

    /// Cria um usuário com ID e username
    pub fn with_username(id: u32, username: impl Into<String>) -> Self {
        Self {
            id,
            username: Some(username.into()),
            email: None,
            color: None,
            profilePicture: None,
            initials: None,
        }
    }

    /// Cria um usuário com ID, username e email
    pub fn with_email(id: u32, username: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            username: Some(username.into()),
            email: Some(email.into()),
            color: None,
            profilePicture: None,
            initials: None,
        }
    }
}

/// Representa um assignee (responsável) de uma tarefa
///
/// NOTA: Assignees são essencialmente Users, mas separamos o tipo
/// para clareza semântica (um usuário pode ser assignee de uma tarefa).
pub type Assignee = User;

/// Lista de assignees para uma tarefa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssigneeList {
    /// IDs dos usuários atribuídos
    pub add: Vec<u32>,
    /// IDs dos usuários a remover (para updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rem: Option<Vec<u32>>,
}

impl AssigneeList {
    /// Cria uma lista vazia de assignees
    pub fn new() -> Self {
        Self {
            add: Vec::new(),
            rem: None,
        }
    }

    /// Cria uma lista com assignees para adicionar
    pub fn add(user_ids: Vec<u32>) -> Self {
        Self {
            add: user_ids,
            rem: None,
        }
    }

    /// Cria uma lista com assignees para adicionar e remover
    pub fn add_and_remove(add: Vec<u32>, rem: Vec<u32>) -> Self {
        Self {
            add,
            rem: Some(rem),
        }
    }

    /// Adiciona um assignee à lista
    pub fn add_assignee(&mut self, user_id: u32) {
        if !self.add.contains(&user_id) {
            self.add.push(user_id);
        }
    }

    /// Remove um assignee da lista
    pub fn remove_assignee(&mut self, user_id: u32) {
        if let Some(ref mut rem) = self.rem {
            if !rem.contains(&user_id) {
                rem.push(user_id);
            }
        } else {
            self.rem = Some(vec![user_id]);
        }
    }

    /// Verifica se a lista está vazia
    pub fn is_empty(&self) -> bool {
        self.add.is_empty() && self.rem.as_ref().map_or(true, |r| r.is_empty())
    }
}

impl Default for AssigneeList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_new() {
        let user = User::new(123);
        assert_eq!(user.id, 123);
        assert!(user.username.is_none());
    }

    #[test]
    fn test_user_with_username() {
        let user = User::with_username(456, "john");
        assert_eq!(user.id, 456);
        assert_eq!(user.username, Some("john".to_string()));
    }

    #[test]
    fn test_user_with_email() {
        let user = User::with_email(789, "jane", "jane@example.com");
        assert_eq!(user.id, 789);
        assert_eq!(user.username, Some("jane".to_string()));
        assert_eq!(user.email, Some("jane@example.com".to_string()));
    }

    #[test]
    fn test_assignee_list_new() {
        let list = AssigneeList::new();
        assert!(list.add.is_empty());
        assert!(list.rem.is_none());
        assert!(list.is_empty());
    }

    #[test]
    fn test_assignee_list_add() {
        let list = AssigneeList::add(vec![1, 2, 3]);
        assert_eq!(list.add, vec![1, 2, 3]);
        assert!(list.rem.is_none());
        assert!(!list.is_empty());
    }

    #[test]
    fn test_assignee_list_add_assignee() {
        let mut list = AssigneeList::new();
        list.add_assignee(100);
        list.add_assignee(200);
        list.add_assignee(100); // Duplicate should not be added
        assert_eq!(list.add, vec![100, 200]);
    }

    #[test]
    fn test_assignee_list_remove_assignee() {
        let mut list = AssigneeList::add(vec![1, 2, 3]);
        list.remove_assignee(2);
        list.remove_assignee(3);
        assert_eq!(list.rem, Some(vec![2, 3]));
    }
}
