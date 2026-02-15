//! Tool for querying cumulative production/consumption of a specific item.
//!
//! Uses `force.get_item_production_statistics("nauvis")` to read the all-time
//! input (produced) and output (consumed) counts. Sensei can compare these
//! to spot bottlenecks (e.g. consuming more iron plates than producing).

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::SenseiError,
    lua,
    rcon_ext::{execute_lua_json, SharedRcon},
};

/// Fetches all-time produced/consumed counts for one item on nauvis.
pub struct GetProductionStats {
    pub(crate) rcon: SharedRcon,
}

impl GetProductionStats {
    pub const fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetProductionStats`].
#[derive(Debug, Deserialize)]
pub struct GetProductionStatsArgs {
    /// Item prototype name to query (e.g. "iron-plate", "electronic-circuit").
    pub item: String,
}

/// Cumulative production and consumption totals for a single item.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ProductionStats {
    /// The queried item name, echoed back for context.
    pub item: String,
    /// All-time count of this item produced (input side of statistics).
    pub produced: u64,
    /// All-time count of this item consumed (output side of statistics).
    pub consumed: u64,
}

impl Tool for GetProductionStats {
    const NAME: &'static str = "get_production_stats";
    type Error = SenseiError;
    type Args = GetProductionStatsArgs;
    type Output = ProductionStats;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_production_stats".to_string(),
            description: "Get total production and consumption statistics for a specific item"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "item": {
                        "type": "string",
                        "description": "The item prototype name (e.g. 'iron-plate', 'electronic-circuit')"
                    }
                },
                "required": ["item"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::production_stats(&args.item);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_production_stats() {
        let json = r#"{"item":"iron-plate","produced":1500,"consumed":800}"#;
        let stats: ProductionStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.item, "iron-plate");
        assert_eq!(stats.produced, 1500);
        assert_eq!(stats.consumed, 800);
    }

    #[test]
    fn test_parse_zero_stats() {
        let json = r#"{"item":"nuclear-fuel","produced":0,"consumed":0}"#;
        let stats: ProductionStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.produced, 0);
        assert_eq!(stats.consumed, 0);
    }
}
