// Handlers para arquitetura event-driven
pub mod health;
pub mod webhook;
pub mod worker;

// pub mod db_check; // DESABILITADO - sem banco
// pub mod migrate; // DESABILITADO - sem banco
// pub mod sync_clickup; // DESABILITADO - sem banco

pub use health::*;
pub use webhook::*;
pub use worker::*;

// pub use db_check::*; // DESABILITADO
// pub use migrate::*; // DESABILITADO
// pub use sync_clickup::*; // DESABILITADO

// OAuth2 handlers agora estão em src/auth/handlers.rs (módulo separado)