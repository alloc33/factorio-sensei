//! Tool for discovering ore patches and other resource deposits near the player.
//!
//! Finds all `type="resource"` entities within a radius, then aggregates them
//! by name — summing amounts and averaging positions to produce one entry per
//! resource type with its total yield and center coordinates.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Aggregates nearby resource entities by type, returning total amounts and centers.
pub struct GetNearbyResources {
    pub(crate) rcon: SharedRcon,
}

impl GetNearbyResources {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetNearbyResources`]. All optional.
#[derive(Debug, Deserialize)]
pub struct GetNearbyResourcesArgs {
    /// Search radius in tiles around the player. Defaults to 50.
    pub radius: Option<f64>,
}

/// Aggregated info for one resource type (e.g. all iron-ore tiles combined).
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResourcePatch {
    /// Resource prototype name (e.g. "iron-ore", "crude-oil").
    pub name: String,
    /// Sum of all individual tile amounts within the radius.
    pub total_amount: u64,
    /// Average x position of all matching resource tiles.
    pub center_x: f64,
    /// Average y position of all matching resource tiles.
    pub center_y: f64,
}

/// Top-level response wrapper with one entry per resource type.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct NearbyResources {
    pub resources: Vec<ResourcePatch>,
}

impl Tool for GetNearbyResources {
    const NAME: &'static str = "get_nearby_resources";
    type Error = SenseiError;
    type Args = GetNearbyResourcesArgs;
    type Output = NearbyResources;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_nearby_resources".to_string(),
            description: "Get resource patches near the player, aggregated by type with total amounts and center positions".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "radius": {
                        "type": "number",
                        "description": "Search radius in tiles (default: 50)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let radius = args.radius.unwrap_or(50.0);
        let lua = lua::nearby_resources(radius);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resources() {
        let json = r#"{"resources":[
            {"name":"iron-ore","total_amount":50000,"center_x":10.5,"center_y":-20.3},
            {"name":"copper-ore","total_amount":30000,"center_x":50.0,"center_y":15.0}
        ]}"#;
        let result: NearbyResources = serde_json::from_str(json).unwrap();
        assert_eq!(result.resources.len(), 2);
        assert_eq!(result.resources[0].name, "iron-ore");
        assert_eq!(result.resources[0].total_amount, 50000);
    }

    #[test]
    fn test_parse_no_resources() {
        let json = r#"{"resources":[]}"#;
        let result: NearbyResources = serde_json::from_str(json).unwrap();
        assert!(result.resources.is_empty());
    }
}
