//! Tool for listing assembling machines and their current recipes.
//!
//! Finds entities of `type="assembling-machine"` up to a configurable limit.
//! For each machine, reports its prototype name, position, assigned recipe
//! (if any), and effective crafting speed (accounting for modules/beacons).

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Lists assembling machines with their positions, recipes, and crafting speeds.
pub struct GetAssemblers {
    pub(crate) rcon: SharedRcon,
}

impl GetAssemblers {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetAssemblers`]. All optional.
#[derive(Debug, Deserialize)]
pub struct GetAssemblersArgs {
    /// Max machines to return. Defaults to 30 if omitted.
    pub limit: Option<u32>,
}

/// A single assembling machine's state snapshot.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AssemblerInfo {
    /// Entity prototype name (e.g. "assembling-machine-1", "assembling-machine-3").
    pub name: String,
    /// World x coordinate.
    pub x: f64,
    /// World y coordinate.
    pub y: f64,
    /// Currently assigned recipe, or `None` if the machine is idle.
    pub recipe: Option<String>,
    /// Effective crafting speed (base speed * module bonuses).
    pub crafting_speed: f64,
}

/// Top-level response wrapper for the assemblers list.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Assemblers {
    pub assemblers: Vec<AssemblerInfo>,
}

impl Tool for GetAssemblers {
    const NAME: &'static str = "get_assemblers";
    type Error = SenseiError;
    type Args = GetAssemblersArgs;
    type Output = Assemblers;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_assemblers".to_string(),
            description:
                "Get assembling machines on the map with their recipes and crafting speeds"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of assemblers to return (default: 30)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let limit = args.limit.unwrap_or(30);
        let lua = lua::assemblers(limit);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assemblers() {
        let json = r#"{"assemblers":[
            {"name":"assembling-machine-1","x":5.0,"y":10.0,"recipe":"iron-gear-wheel","crafting_speed":0.5},
            {"name":"assembling-machine-2","x":8.0,"y":10.0,"recipe":null,"crafting_speed":0.75}
        ]}"#;
        let result: Assemblers = serde_json::from_str(json).unwrap();
        assert_eq!(result.assemblers.len(), 2);
        assert_eq!(
            result.assemblers[0].recipe.as_deref(),
            Some("iron-gear-wheel")
        );
        assert_eq!(result.assemblers[1].recipe, None);
    }

    #[test]
    fn test_parse_no_assemblers() {
        let json = r#"{"assemblers":[]}"#;
        let result: Assemblers = serde_json::from_str(json).unwrap();
        assert!(result.assemblers.is_empty());
    }
}
