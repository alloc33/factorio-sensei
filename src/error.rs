use thiserror::Error;

/// Unified error type for factorio-sensei operations.
#[derive(Debug, Error)]
pub enum SenseiError {
    #[error("RCON error: {0}")]
    Rcon(#[from] factorio_rcon::RconError),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Lua error from Factorio: {0}")]
    LuaError(String),

    #[error("No player connected")]
    NoPlayer,

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),
}
