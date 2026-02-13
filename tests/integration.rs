//! Integration tests for all 10 Rig tools against a live Factorio instance.
//!
//! These tests require a running Factorio server with RCON enabled.
//! Run with: `cargo test -- --ignored`
//!
//! Environment variables:
//! - `FACTORIO_RCON_ADDR` — default `127.0.0.1:27015`
//! - `FACTORIO_RCON_PASS` — default `factorio`

use std::sync::Arc;
use tokio::sync::Mutex;

use factorio_rcon::RconClient;
use factorio_sensei::tools::*;
use factorio_sensei::SharedRcon;
use rig::tool::Tool;

fn rcon_addr() -> String {
    std::env::var("FACTORIO_RCON_ADDR").unwrap_or_else(|_| "127.0.0.1:27015".to_string())
}

fn rcon_pass() -> String {
    std::env::var("FACTORIO_RCON_PASS").unwrap_or_else(|_| "factorio".to_string())
}

async fn shared_rcon() -> SharedRcon {
    let client = RconClient::connect(&rcon_addr(), &rcon_pass())
        .await
        .expect("Failed to connect — is Factorio running as multiplayer host?");
    Arc::new(Mutex::new(client))
}

#[tokio::test]
#[ignore]
async fn test_get_player_position() {
    let rcon = shared_rcon().await;
    let tool = GetPlayerPosition::new(rcon);
    let result = tool.call(GetPlayerPositionArgs {}).await.unwrap();
    // Player should be on nauvis by default
    assert_eq!(result.surface, "nauvis");
}

#[tokio::test]
#[ignore]
async fn test_get_player_inventory() {
    let rcon = shared_rcon().await;
    let tool = GetPlayerInventory::new(rcon);
    let result = tool.call(GetPlayerInventoryArgs {}).await.unwrap();
    // Just verify it parses — inventory may be empty
    let _ = result.items;
}

#[tokio::test]
#[ignore]
async fn test_get_production_stats() {
    let rcon = shared_rcon().await;
    let tool = GetProductionStats::new(rcon);
    let result = tool
        .call(GetProductionStatsArgs {
            item: "iron-plate".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(result.item, "iron-plate");
}

#[tokio::test]
#[ignore]
async fn test_get_power_stats() {
    let rcon = shared_rcon().await;
    let tool = GetPowerStats::new(rcon);
    let result = tool.call(GetPowerStatsArgs {}).await.unwrap();
    // Satisfaction should be between 0 and 1 (inclusive)
    assert!(result.satisfaction >= 0.0 && result.satisfaction <= 1.0);
}

#[tokio::test]
#[ignore]
async fn test_get_research_status() {
    let rcon = shared_rcon().await;
    let tool = GetResearchStatus::new(rcon);
    let result = tool.call(GetResearchStatusArgs {}).await.unwrap();
    // Just verify it parses — research may or may not be active
    let _ = result.queue;
}

#[tokio::test]
#[ignore]
async fn test_get_recipe() {
    let rcon = shared_rcon().await;
    let tool = GetRecipe::new(rcon);
    let result = tool
        .call(GetRecipeArgs {
            recipe_name: "iron-gear-wheel".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(result.name, "iron-gear-wheel");
    assert!(!result.ingredients.is_empty());
    assert!(!result.products.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_get_nearby_entities() {
    let rcon = shared_rcon().await;
    let tool = GetNearbyEntities::new(rcon);
    let result = tool
        .call(GetNearbyEntitiesArgs { radius: Some(10.0) })
        .await
        .unwrap();
    // Just verify it parses — may be empty in a fresh game
    let _ = result.entities;
}

#[tokio::test]
#[ignore]
async fn test_get_nearby_resources() {
    let rcon = shared_rcon().await;
    let tool = GetNearbyResources::new(rcon);
    let result = tool
        .call(GetNearbyResourcesArgs { radius: Some(50.0) })
        .await
        .unwrap();
    // Just verify it parses — should find some ore on nauvis spawn
    let _ = result.resources;
}

#[tokio::test]
#[ignore]
async fn test_get_assemblers() {
    let rcon = shared_rcon().await;
    let tool = GetAssemblers::new(rcon);
    let result = tool
        .call(GetAssemblersArgs { limit: Some(10) })
        .await
        .unwrap();
    // Just verify it parses — may be empty early game
    let _ = result.assemblers;
}

#[tokio::test]
#[ignore]
async fn test_get_furnaces() {
    let rcon = shared_rcon().await;
    let tool = GetFurnaces::new(rcon);
    let result = tool
        .call(GetFurnacesArgs { limit: Some(10) })
        .await
        .unwrap();
    // Just verify it parses — may be empty early game
    let _ = result.furnaces;
}
