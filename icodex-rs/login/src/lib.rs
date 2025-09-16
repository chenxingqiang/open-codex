mod pkce;
mod server;

pub use server::LoginServer;
pub use server::ServerOptions;
pub use server::ShutdownHandle;
pub use server::run_login_server;

// Re-export commonly used auth types and helpers from icodex-core for compatibility
pub use icodex_core::AuthManager;
pub use icodex_core::CodexAuth;
pub use icodex_core::auth::AuthDotJson;
pub use icodex_core::auth::CLIENT_ID;
pub use icodex_core::auth::OPENAI_API_KEY_ENV_VAR;
pub use icodex_core::auth::get_auth_file;
pub use icodex_core::auth::login_with_api_key;
pub use icodex_core::auth::logout;
pub use icodex_core::auth::try_read_auth_json;
pub use icodex_core::auth::write_auth_json;
pub use icodex_core::token_data::TokenData;
pub use icodex_protocol::mcp_protocol::AuthMode;
