//! Tool for listing the player's main inventory contents.
//!
//! Iterates every slot in `get_main_inventory()`, aggregates stacks of the
//! same item, and returns `[{name, count}]`. Lets Sensei check whether
//! the player has enough materials for a suggested build.

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Returns every item in the player's main inventory, deduplicated by name.
pub struct GetPlayerInventory {
    pub(crate) rcon: SharedRcon,
}

impl GetPlayerInventory {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetPlayerInventory`]. Takes no parameters.
#[derive(Debug, Deserialize)]
pub struct GetPlayerInventoryArgs {}

/// A single item stack aggregated across all inventory slots.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    /// Item prototype name (e.g. "iron-plate", "transport-belt").
    pub name: String,
    /// Total count across all stacks of this item.
    pub count: u64,
}

/// The player's main inventory contents, deduplicated by item name.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PlayerInventory {
    pub items: Vec<InventoryItem>,
}

impl Tool for GetPlayerInventory {
    const NAME: &'static str = "get_player_inventory";
    type Error = SenseiError;
    type Args = GetPlayerInventoryArgs;
    type Output = PlayerInventory;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_player_inventory".to_string(),
            description: "Get all items in the player's main inventory".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::player_inventory();
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inventory() {
        let json =
            r#"{"items":[{"name":"iron-plate","count":50},{"name":"copper-plate","count":25}]}"#;
        let inv: PlayerInventory = serde_json::from_str(json).unwrap();
        assert_eq!(inv.items.len(), 2);
        assert_eq!(inv.items[0].name, "iron-plate");
        assert_eq!(inv.items[0].count, 50);
    }

    #[test]
    fn test_parse_empty_inventory() {
        let json = r#"{"items":[]}"#;
        let inv: PlayerInventory = serde_json::from_str(json).unwrap();
        assert!(inv.items.is_empty());
    }
}
