use std::sync::Arc;
use tokio::sync::Mutex;

use factorio_rcon::RconClient;

use crate::error::SenseiError;

/// Shared RCON connection for all tools.
/// `RconClient::execute()` needs `&mut self`, but Rig `Tool::call()` takes `&self`.
/// `Arc<Mutex<..>>` provides interior mutability across tools.
pub type SharedRcon = Arc<Mutex<RconClient>>;

/// Execute a Lua IIFE via RCON, wrapping it with `helpers.table_to_json()` and `rcon.print()`.
/// Returns the raw JSON string from Factorio.
///
/// The IIFE should return a plain Lua table (no userdata).
/// This function checks for `{"error":"no_player"}` and converts it to `SenseiError::NoPlayer`.
pub async fn execute_lua_json(rcon: &SharedRcon, lua_iife: &str) -> Result<String, SenseiError> {
    let command = format!("/c rcon.print(helpers.table_to_json({}))", lua_iife);
    let mut client = rcon.lock().await;
    let response = client.execute(&command).await?;

    // Check for Lua-side error responses
    if response.contains(r#""error":"no_player""#) {
        return Err(SenseiError::NoPlayer);
    }
    if let Some(msg) = extract_lua_error(&response) {
        return Err(SenseiError::LuaError(msg));
    }

    Ok(response)
}

/// Extract error message from `{"error":"..."}` pattern in Lua responses.
fn extract_lua_error(response: &str) -> Option<String> {
    let parsed: serde_json::Value = serde_json::from_str(response).ok()?;
    let err = parsed.get("error")?.as_str()?;
    if err == "no_player" {
        return None; // Handled separately
    }
    Some(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_lua_error_with_error() {
        let response = r#"{"error":"some lua problem"}"#;
        assert_eq!(
            extract_lua_error(response),
            Some("some lua problem".to_string())
        );
    }

    #[test]
    fn test_extract_lua_error_no_player_returns_none() {
        let response = r#"{"error":"no_player"}"#;
        assert_eq!(extract_lua_error(response), None);
    }

    #[test]
    fn test_extract_lua_error_valid_json() {
        let response = r#"{"x":1.5,"y":-3.2}"#;
        assert_eq!(extract_lua_error(response), None);
    }

    #[test]
    fn test_extract_lua_error_not_json() {
        let response = "not json at all";
        assert_eq!(extract_lua_error(response), None);
    }
}
