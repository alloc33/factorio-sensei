pub mod prompts;

use rig::{
    agent::Agent,
    client::{CompletionClient, ProviderClient},
    providers::{anthropic, anthropic::completion::CompletionModel},
};

use crate::{tools::*, SharedRcon};

pub const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

/// Build a coaching agent backed by Claude, with all game-state tools registered.
///
/// Reads `ANTHROPIC_API_KEY` from the environment. Pass `model` to override the
/// default Claude model (e.g. `"claude-opus-4-0"`).
pub fn build_coach(
    rcon: &SharedRcon,
    model: Option<&str>,
    wiki_articles: &[String],
) -> Agent<CompletionModel> {
    let client = anthropic::Client::from_env();
    let model = model.unwrap_or(DEFAULT_MODEL);

    let mut builder = client
        .agent(model)
        .preamble(prompts::COACH_SYSTEM_PROMPT)
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
        .default_max_turns(10);

    for article in wiki_articles {
        builder = builder.context(article);
    }

    builder.build()
}
