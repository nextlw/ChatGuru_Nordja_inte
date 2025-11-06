//! Priority levels para tarefas do ClickUp
//!
//! A API do ClickUp aceita valores de 1 a 4:
//! - 1 = Urgente
//! - 2 = Alta
//! - 3 = Normal (padrão)
//! - 4 = Baixa

use serde::{Deserialize, Deserializer, Serialize};

/// Representa os níveis de prioridade do ClickUp
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Priority {
    /// Prioridade urgente (valor 1)
    Urgent = 1,
    /// Prioridade alta (valor 2)
    High = 2,
    /// Prioridade normal (valor 3) - padrão
    Normal = 3,
    /// Prioridade baixa (valor 4)
    Low = 4,
}

// Deserializer customizado que aceita null e valores inválidos
impl<'de> Deserialize<'de> for Priority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Option::<i64>::deserialize(deserializer)?;
        match value {
            Some(1) => Ok(Priority::Urgent),
            Some(2) => Ok(Priority::High),
            Some(3) => Ok(Priority::Normal),
            Some(4) => Ok(Priority::Low),
            Some(_) => Ok(Priority::default()), // Valor inválido → usa default
            None => Ok(Priority::default()),    // null → usa default
        }
    }
}

impl Priority {
    /// Converte para o valor inteiro usado pela API
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// Cria a partir de um valor inteiro
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Priority::Urgent),
            2 => Some(Priority::High),
            3 => Some(Priority::Normal),
            4 => Some(Priority::Low),
            _ => None,
        }
    }

    /// Retorna o nome legível da prioridade
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Urgent => "Urgente",
            Priority::High => "Alta",
            Priority::Normal => "Normal",
            Priority::Low => "Baixa",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_values() {
        assert_eq!(Priority::Urgent.as_i32(), 1);
        assert_eq!(Priority::High.as_i32(), 2);
        assert_eq!(Priority::Normal.as_i32(), 3);
        assert_eq!(Priority::Low.as_i32(), 4);
    }

    #[test]
    fn test_priority_from_i32() {
        assert_eq!(Priority::from_i32(1), Some(Priority::Urgent));
        assert_eq!(Priority::from_i32(2), Some(Priority::High));
        assert_eq!(Priority::from_i32(3), Some(Priority::Normal));
        assert_eq!(Priority::from_i32(4), Some(Priority::Low));
        assert_eq!(Priority::from_i32(5), None);
        assert_eq!(Priority::from_i32(0), None);
    }

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn test_priority_display() {
        assert_eq!(Priority::Urgent.to_string(), "Urgente");
        assert_eq!(Priority::High.to_string(), "Alta");
        assert_eq!(Priority::Normal.to_string(), "Normal");
        assert_eq!(Priority::Low.to_string(), "Baixa");
    }
}
