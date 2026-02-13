//! Tool for looking up a recipe's ingredients, products, and crafting time.
//!
//! Reads from `prototypes.recipe[name]` — this is a prototype lookup, not a
//! game-state query, so it doesn't require a connected player. Returns a
//! `LuaError` if the recipe name doesn't exist.

use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::SenseiError;
use crate::lua;
use crate::rcon_ext::{execute_lua_json, SharedRcon};

/// Looks up a recipe prototype by name and returns its crafting details.
pub struct GetRecipe {
    pub(crate) rcon: SharedRcon,
}

impl GetRecipe {
    pub fn new(rcon: SharedRcon) -> Self {
        Self { rcon }
    }
}

/// Arguments for [`GetRecipe`].
#[derive(Debug, Deserialize)]
pub struct GetRecipeArgs {
    /// Recipe prototype name (e.g. "iron-gear-wheel", "electronic-circuit").
    pub recipe_name: String,
}

/// A single input required by the recipe.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RecipeIngredient {
    /// Ingredient prototype name.
    pub name: String,
    /// "item" or "fluid".
    #[serde(rename = "type")]
    pub kind: String,
    /// Number of units needed per craft.
    pub amount: f64,
}

/// A single output produced by the recipe.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RecipeProduct {
    /// Product prototype name.
    pub name: String,
    /// "item" or "fluid".
    #[serde(rename = "type")]
    pub kind: String,
    /// Number of units produced per craft.
    pub amount: f64,
}

/// Full recipe details from the prototype data.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RecipeInfo {
    /// Recipe prototype name.
    pub name: String,
    /// Base crafting time in seconds (before speed modifiers).
    pub energy: f64,
    /// Items/fluids consumed per craft.
    pub ingredients: Vec<RecipeIngredient>,
    /// Items/fluids produced per craft.
    pub products: Vec<RecipeProduct>,
}

impl Tool for GetRecipe {
    const NAME: &'static str = "get_recipe";
    type Error = SenseiError;
    type Args = GetRecipeArgs;
    type Output = RecipeInfo;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "get_recipe".to_string(),
            description:
                "Look up a recipe's ingredients, products, and crafting time by prototype name"
                    .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "recipe_name": {
                        "type": "string",
                        "description": "The recipe prototype name (e.g. 'iron-gear-wheel', 'electronic-circuit')"
                    }
                },
                "required": ["recipe_name"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let lua = lua::recipe(&args.recipe_name);
        let json = execute_lua_json(&self.rcon, &lua).await?;
        Ok(serde_json::from_str(&json)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_recipe() {
        let json = r#"{
            "name": "iron-gear-wheel",
            "energy": 0.5,
            "ingredients": [{"name": "iron-plate", "type": "item", "amount": 2}],
            "products": [{"name": "iron-gear-wheel", "type": "item", "amount": 1}]
        }"#;
        let recipe: RecipeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(recipe.name, "iron-gear-wheel");
        assert_eq!(recipe.energy, 0.5);
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.ingredients[0].name, "iron-plate");
        assert_eq!(recipe.ingredients[0].amount, 2.0);
        assert_eq!(recipe.products[0].name, "iron-gear-wheel");
    }

    #[test]
    fn test_parse_multi_ingredient_recipe() {
        let json = r#"{
            "name": "electronic-circuit",
            "energy": 0.5,
            "ingredients": [
                {"name": "iron-plate", "type": "item", "amount": 1},
                {"name": "copper-cable", "type": "item", "amount": 3}
            ],
            "products": [{"name": "electronic-circuit", "type": "item", "amount": 1}]
        }"#;
        let recipe: RecipeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(recipe.ingredients.len(), 2);
    }
}
