//! Tool for checking the current technology research and queue.
//!
//! Reads `force.current_research`, `research_progress`, and up to 10 entries
//! from `research_queue`. All fields are optional — if no research is active,
//! `current` and `progress` will be `None`.

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::SenseiError;
use crate::lua;
use crate::rcon_ext::{execute_lua_json, SharedRcon};

/// Returns current research tech, completion progress, and queued techs.
pub struct GetResearchStatus {
    pub(crate) rcon: SharedRcon,
}

impl GetResearchStatus {
    pub fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetResearchStatus`]. Takes no parameters.
#[derive(Debug, Deserialize)]
pub struct GetResearchStatusArgs {}

/// Current research state for the player's force.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ResearchStatus {
    /// Technology currently being researched, or `None` if idle.
    pub current: Option<String>,
    /// Completion fraction (0.0–1.0) of the current research, or `None` if idle.
    pub progress: Option<f64>,
    /// Up to 10 queued technology names (in order).
    pub queue: Vec<String>,
}

impl Tool for GetResearchStatus {
    const NAME: &'static str = "get_research_status";
    type Error = SenseiError;
    type Args = GetResearchStatusArgs;
    type Output = ResearchStatus;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_research_status".to_string(),
            description: "Get current research technology, progress percentage, and research queue"
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::research_status();
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_active_research() {
        let json =
            r#"{"current":"automation-2","progress":0.45,"queue":["automation-2","logistics"]}"#;
        let status: ResearchStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.current.as_deref(), Some("automation-2"));
        assert_eq!(status.progress, Some(0.45));
        assert_eq!(status.queue, vec!["automation-2", "logistics"]);
    }

    #[test]
    fn test_parse_no_research() {
        let json = r#"{"queue":[]}"#;
        let status: ResearchStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.current, None);
        assert_eq!(status.progress, None);
        assert!(status.queue.is_empty());
    }
}
