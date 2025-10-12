// Handlers para arquitetura event-driven
pub mod health;
pub mod webhook;
pub mod worker;
pub mod clickup;
pub mod db_check;
pub mod migrate;
pub mod sync_clickup;

pub use health::*;
pub use webhook::*;
pub use worker::*;
pub use clickup::*;
pub use db_check::*;
pub use migrate::*;
pub use sync_clickup::*;

// OAuth2 handlers agora estão em src/auth/handlers.rs (módulo separado)