//! Tool for retrieving the player's current world position.
//!
//! Queries `connected_players[1].position` and the surface name via RCON.
//! Useful for Sensei to understand where the player is and what they're
//! likely working on (e.g. near ore patches, at main bus, exploring).

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Reads the first connected player's x/y coordinates and surface name.
pub struct GetPlayerPosition {
    pub(crate) rcon: SharedRcon,
}

impl GetPlayerPosition {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetPlayerPosition`]. Takes no parameters.
#[derive(Debug, Deserialize)]
pub struct GetPlayerPositionArgs {}

/// The player's current location in the world.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerPosition {
    /// Tile x coordinate (east is positive).
    pub x: f64,
    /// Tile y coordinate (south is positive).
    pub y: f64,
    /// Surface name, typically "nauvis" for the default world.
    pub surface: String,
}

impl Tool for GetPlayerPosition {
    const NAME: &'static str = "get_player_position";
    type Error = SenseiError;
    type Args = GetPlayerPositionArgs;
    type Output = PlayerPosition;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_player_position".to_string(),
            description: "Get the current player's position and surface name".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::player_position();
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        let json = r#"{"x":1.5,"y":-3.2,"surface":"nauvis"}"#;
        let pos: PlayerPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.x, 1.5);
        assert_eq!(pos.y, -3.2);
        assert_eq!(pos.surface, "nauvis");
    }

    #[test]
    fn test_parse_position_integer_coords() {
        let json = r#"{"x":0,"y":0,"surface":"nauvis"}"#;
        let pos: PlayerPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }
}
