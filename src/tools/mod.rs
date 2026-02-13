//! Rig tools for reading Factorio game state via RCON.
//!
//! Each tool implements the [`rig::tool::Tool`] trait so it can be registered with
//! a Rig agent. Every tool holds a [`SharedRcon`](crate::SharedRcon) handle and
//! delegates Lua generation to [`crate::lua`], JSON transport to
//! [`crate::rcon_ext::execute_lua_json`], and deserialization to serde.
//!
//! Tools are read-only — they observe the game but never execute actions.

mod assemblers;
mod entities;
mod furnaces;
mod inventory;
mod position;
mod power;
mod production;
mod recipe;
mod research;
mod resources;

pub use assemblers::GetAssemblers;
pub use entities::GetNearbyEntities;
pub use furnaces::GetFurnaces;
pub use inventory::GetPlayerInventory;
pub use position::GetPlayerPosition;
pub use power::GetPowerStats;
pub use production::GetProductionStats;
pub use recipe::GetRecipe;
pub use research::GetResearchStatus;
pub use resources::GetNearbyResources;
