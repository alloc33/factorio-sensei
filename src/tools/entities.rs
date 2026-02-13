//! Tool for scanning buildings and structures around the player.
//!
//! Uses `find_entities_filtered{position, radius}` and excludes noise entities
//! (resources, trees, simple-entities) to focus on player-built structures.
//! Capped at 50 results to keep RCON responses small.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Finds up to 50 non-resource, non-decorative entities near the player.
pub struct GetNearbyEntities {
    pub(crate) rcon: SharedRcon,
}

impl GetNearbyEntities {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetNearbyEntities`]. All optional.
#[derive(Debug, Deserialize)]
pub struct GetNearbyEntitiesArgs {
    /// Search radius in tiles around the player. Defaults to 20.
    pub radius: Option<f64>,
}

/// A single entity found near the player.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NearbyEntity {
    /// Entity prototype name (e.g. "stone-furnace", "inserter").
    pub name: String,
    /// Entity type (e.g. "furnace", "transport-belt", "inserter").
    #[serde(rename = "type")]
    pub kind: String,
    /// World x coordinate.
    pub x: f64,
    /// World y coordinate.
    pub y: f64,
}

/// Top-level response wrapper. Capped at 50 entities.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NearbyEntities {
    pub entities: Vec<NearbyEntity>,
}

impl Tool for GetNearbyEntities {
    const NAME: &'static str = "get_nearby_entities";
    type Error = SenseiError;
    type Args = GetNearbyEntitiesArgs;
    type Output = NearbyEntities;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_nearby_entities".to_string(),
            description: "Get buildings and structures near the player (excludes resources, trees, and decoratives). Returns up to 50 entities.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "radius": {
                        "type": "number",
                        "description": "Search radius in tiles (default: 20)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let radius = args.radius.unwrap_or(20.0);
        let lua = lua::nearby_entities(radius);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entities() {
        let json = r#"{"entities":[
            {"name":"stone-furnace","type":"furnace","x":1.5,"y":2.5},
            {"name":"transport-belt","type":"transport-belt","x":3.0,"y":4.0}
        ]}"#;
        let result: NearbyEntities = serde_json::from_str(json).unwrap();
        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.entities[0].name, "stone-furnace");
        assert_eq!(result.entities[0].kind, "furnace");
    }

    #[test]
    fn test_parse_empty_entities() {
        let json = r#"{"entities":[]}"#;
        let result: NearbyEntities = serde_json::from_str(json).unwrap();
        assert!(result.entities.is_empty());
    }
}
