/// Middleware layer para o Axum router
///
/// Este módulo contém middleware customizados para:
/// - Autenticação de endpoints administrativos
/// - Validação de requisições
/// - Logging e observabilidade

pub mod admin_auth;

pub use admin_auth::require_admin_key;
