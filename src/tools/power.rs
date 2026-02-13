//! Tool for reading the electric network's production, consumption, and satisfaction.
//!
//! Finds the first electric pole on the player's surface and reads its
//! `electric_network_statistics`. Returns zero values if no poles exist yet.
//! Satisfaction < 1.0 means the factory is experiencing brownouts.

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::SenseiError;
use crate::lua;
use crate::rcon_ext::{execute_lua_json, SharedRcon};

/// Reads the power grid via the first electric pole's network statistics.
pub struct GetPowerStats {
    pub(crate) rcon: SharedRcon,
}

impl GetPowerStats {
    pub fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetPowerStats`]. Takes no parameters.
#[derive(Debug, Deserialize)]
pub struct GetPowerStatsArgs {}

/// Electric network summary for the player's surface.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerStats {
    /// Total power being generated across the network.
    pub production_watts: f64,
    /// Total power being consumed across the network.
    pub consumption_watts: f64,
    /// Ratio of production to consumption (1.0 = fully satisfied, <1.0 = brownout).
    pub satisfaction: f64,
}

impl Tool for GetPowerStats {
    const NAME: &'static str = "get_power_stats";
    type Error = SenseiError;
    type Args = GetPowerStatsArgs;
    type Output = PowerStats;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_power_stats".to_string(),
            description: "Get the power grid statistics: total production, consumption, and satisfaction ratio".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::power_stats();
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_power_stats() {
        let json = r#"{"production_watts":5000000,"consumption_watts":3500000,"satisfaction":1.0}"#;
        let stats: PowerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.production_watts, 5000000.0);
        assert_eq!(stats.consumption_watts, 3500000.0);
        assert_eq!(stats.satisfaction, 1.0);
    }

    #[test]
    fn test_parse_no_power() {
        let json = r#"{"production_watts":0,"consumption_watts":0,"satisfaction":1.0}"#;
        let stats: PowerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.production_watts, 0.0);
        assert_eq!(stats.satisfaction, 1.0);
    }

    #[test]
    fn test_parse_brownout() {
        let json = r#"{"production_watts":1000,"consumption_watts":2000,"satisfaction":0.5}"#;
        let stats: PowerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.satisfaction, 0.5);
    }
}
