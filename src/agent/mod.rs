pub mod prompts;

use rig::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    providers::{anthropic, anthropic::completion::CompletionModel},
};

use crate::{tools::*, SharedRcon};

pub const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

/// Build the Sensei agent backed by Claude, with all game-state tools registered.
///
/// Reads `ANTHROPIC_API_KEY` from the environment. Pass `model` to override the
/// default Claude model (e.g. `"claude-opus-4-0"`).
pub fn build_sensei(
    rcon: &SharedRcon,
    model: Option<&str>,
    wiki_articles: &[String],
) -> Agent<CompletionModel> {
    let client = anthropic::Client::from_env();
    let model = model.unwrap_or(DEFAULT_MODEL);

    let preamble = if wiki_articles.is_empty() {
        prompts::SENSEI_SYSTEM_PROMPT.to_string()
    } else {
        let mut parts = vec![prompts::SENSEI_SYSTEM_PROMPT.to_string()];
        parts.push("\n\n--- KNOWLEDGE BASE ---\nUse the following verified reference material for exact ratios, formulas, and game mechanics.\n".to_string());
        for article in wiki_articles {
            parts.push(article.clone());
            parts.push("\n---\n".to_string());
        }
        parts.concat()
    };

    client
        .agent(model)
        .preamble(&preamble)
        .tool(GetPlayerPosition::new(rcon.clone()))
        .tool(GetPlayerInventory::new(rcon.clone()))
        .tool(GetProductionStats::new(rcon.clone()))
        .tool(GetPowerStats::new(rcon.clone()))
        .tool(GetResearchStatus::new(rcon.clone()))
        .tool(GetNearbyEntities::new(rcon.clone()))
        .tool(GetNearbyResources::new(rcon.clone()))
        .tool(GetAssemblers::new(rcon.clone()))
        .tool(GetFurnaces::new(rcon.clone()))
        .tool(GetRecipe::new(rcon.clone()))
        .default_max_turns(10)
        .build()
}
