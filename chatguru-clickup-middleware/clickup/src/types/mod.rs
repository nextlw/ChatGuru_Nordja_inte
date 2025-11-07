//! Tipos do ClickUp API
//!
//! Este módulo contém todas as estruturas de dados type-safe para interagir
//! com a API do ClickUp, incluindo:
//!
//! - **Task**: Estrutura completa de tarefas
//! - **Priority**: Níveis de prioridade (1-4)
//! - **Status**: Status de tarefas
//! - **User/Assignee**: Usuários e responsáveis
//! - **CustomField**: 18 tipos de campos personalizados
//!
//! ## ⚠️ Notas Importantes
//!
//! - **Checkbox fields**: Usam string "true"/"false", NÃO boolean
//! - **Timestamps**: Sempre em milissegundos (i64), nunca segundos
//! - **Priority**: Valores limitados a 1-4
//! - **Status**: Não são globais, cada lista tem seus próprios status

pub mod custom_field;
pub mod priority;
pub mod status;
pub mod task;
pub mod user;

// Re-exports principais para facilitar uso
pub use custom_field::{
    CustomField, CustomFieldValue, DropdownOption, FileValue, LocationValue, TypeConfig,
};
pub use priority::Priority;
pub use status::Status;
pub use task::{Attachment, LinkedTask, Task, TaskBuilder, TaskDependency};
pub use user::{Assignee, AssigneeList, User};
