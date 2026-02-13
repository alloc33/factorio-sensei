//! Tool for listing furnaces with their smelting recipes, fuel, and output.
//!
//! Finds entities of `type="furnace"` up to a configurable limit. For each
//! furnace, inspects the recipe, fuel inventory (first fuel item), and output
//! inventory (first output item). All three are optional — an idle furnace
//! with no fuel will have all `None`.

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::SenseiError;
use crate::lua;
use crate::rcon_ext::{execute_lua_json, SharedRcon};

/// Lists furnaces with their positions, active recipes, fuel types, and output items.
pub struct GetFurnaces {
    pub(crate) rcon: SharedRcon,
}

impl GetFurnaces {
    pub fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetFurnaces`]. All optional.
#[derive(Debug, Deserialize)]
pub struct GetFurnacesArgs {
    /// Max furnaces to return. Defaults to 30 if omitted.
    pub limit: Option<u32>,
}

/// A single furnace's state snapshot.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FurnaceInfo {
    /// Entity prototype name (e.g. "stone-furnace", "steel-furnace", "electric-furnace").
    pub name: String,
    /// World x coordinate.
    pub x: f64,
    /// World y coordinate.
    pub y: f64,
    /// Active smelting recipe, or `None` if the furnace is idle.
    pub recipe: Option<String>,
    /// First item in the fuel inventory (e.g. "coal"), or `None` if empty/electric.
    pub fuel_type: Option<String>,
    /// First item in the output inventory, or `None` if empty.
    pub output_item: Option<String>,
}

/// Top-level response wrapper for the furnaces list.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Furnaces {
    pub furnaces: Vec<FurnaceInfo>,
}

impl Tool for GetFurnaces {
    const NAME: &'static str = "get_furnaces";
    type Error = SenseiError;
    type Args = GetFurnacesArgs;
    type Output = Furnaces;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_furnaces".to_string(),
            description:
                "Get furnaces on the map with their recipes, fuel types, and output items"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of furnaces to return (default: 30)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let limit = args.limit.unwrap_or(30);
        let lua = lua::furnaces(limit);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_furnaces() {
        let json = r#"{"furnaces":[
            {"name":"stone-furnace","x":1.0,"y":2.0,"recipe":"iron-plate","fuel_type":"coal","output_item":"iron-plate"},
            {"name":"steel-furnace","x":4.0,"y":2.0,"recipe":null,"fuel_type":null,"output_item":null}
        ]}"#;
        let result: Furnaces = serde_json::from_str(json).unwrap();
        assert_eq!(result.furnaces.len(), 2);
        assert_eq!(
            result.furnaces[0].recipe.as_deref(),
            Some("iron-plate")
        );
        assert_eq!(
            result.furnaces[0].fuel_type.as_deref(),
            Some("coal")
        );
        assert_eq!(result.furnaces[1].recipe, None);
        assert_eq!(result.furnaces[1].fuel_type, None);
    }

    #[test]
    fn test_parse_no_furnaces() {
        let json = r#"{"furnaces":[]}"#;
        let result: Furnaces = serde_json::from_str(json).unwrap();
        assert!(result.furnaces.is_empty());
    }
}
