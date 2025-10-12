// Serviços necessários para arquitetura event-driven
pub mod clickup;
pub mod chatguru;
pub mod config_db;
pub mod estrutura;
pub mod openai;
pub mod secrets;
pub mod prompts;
pub mod vertex;
pub mod media_sync;

// Re-exports
pub use clickup::*;
pub use chatguru::*;
pub use config_db::*;
pub use estrutura::*;
pub use openai::*;
pub use vertex::*;
pub use media_sync::*;

// OAuth2 module agora está em src/auth/ (módulo separado e isolado)
