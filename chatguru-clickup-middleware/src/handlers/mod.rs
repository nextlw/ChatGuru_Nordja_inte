// Handlers para arquitetura event-driven
pub mod health;
pub mod webhook;
pub mod worker;
pub mod clickup;
pub mod clickup_webhook;  // ✅ Handler para webhooks do ClickUp → Pub/Sub
// pub mod db_check; // DESABILITADO - sem banco
// pub mod migrate; // DESABILITADO - sem banco
// pub mod sync_clickup; // DESABILITADO - sem banco

pub use health::*;
pub use webhook::*;
pub use worker::*;
pub use clickup::*;
pub use clickup_webhook::*;  // ✅ handle_clickup_webhook, list_registered_webhooks
// pub use db_check::*; // DESABILITADO
// pub use migrate::*; // DESABILITADO
// pub use sync_clickup::*; // DESABILITADO

// OAuth2 handlers agora estão em src/auth/handlers.rs (módulo separado)