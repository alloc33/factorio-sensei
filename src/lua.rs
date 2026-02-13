/// Lua IIFE command builders for Factorio 2.x.
///
/// Each function returns a Lua IIFE string (without `/c` or `rcon.print` wrapper).
/// The wrapping is done by `execute_lua_json()` in `rcon_ext.rs`.
///
/// All IIFEs follow these rules:
/// 1. Wrapped in `(function() ... end)()`
/// 2. Check `game.connected_players[1]` exists (return `{error="no_player"}` if not)
/// 3. Build plain Lua tables (no userdata) for JSON serialization
/// 4. Use DOT syntax for Factorio 2.x API
/// 5. Cap entity results to avoid huge responses
const PLAYER_CHECK: &str =
    "local p = game.connected_players[1] if not p then return {error=\"no_player\"} end";

pub fn player_position() -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         return {{x=p.position.x, y=p.position.y, surface=p.surface.name}} \
         end)()"
    )
}

pub fn player_inventory() -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local inv = p.get_main_inventory() \
         local items = {{}} \
         if inv then \
           for i = 1, #inv do \
             local stack = inv[i] \
             if stack.valid_for_read then \
               local found = false \
               for _, item in ipairs(items) do \
                 if item.name == stack.name then \
                   item.count = item.count + stack.count \
                   found = true \
                   break \
                 end \
               end \
               if not found then \
                 items[#items+1] = {{name=stack.name, count=stack.count}} \
               end \
             end \
           end \
         end \
         return {{items=items}} \
         end)()"
    )
}

pub fn production_stats(item: &str) -> String {
    // Sanitize item name to prevent Lua injection
    let safe_item = sanitize_lua_string(item);
    format!(
        "(function() {PLAYER_CHECK} \
         local stats = p.force.get_item_production_statistics(\"nauvis\") \
         local produced = stats.get_input_count(\"{safe_item}\") \
         local consumed = stats.get_output_count(\"{safe_item}\") \
         return {{item=\"{safe_item}\", produced=produced, consumed=consumed}} \
         end)()"
    )
}

pub fn power_stats() -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local poles = p.surface.find_entities_filtered{{type=\"electric-pole\", limit=1}} \
         if #poles == 0 then \
           return {{production_watts=0, consumption_watts=0, satisfaction=1.0}} \
         end \
         local network = poles[1].electric_network_statistics \
         if not network then \
           return {{production_watts=0, consumption_watts=0, satisfaction=1.0}} \
         end \
         local prod = network.get_flow_count{{input=true, precision_index=defines.flow_precision_index.one_second}} \
         local cons = network.get_flow_count{{input=false, precision_index=defines.flow_precision_index.one_second}} \
         local satisfaction = 1.0 \
         if cons > 0 then satisfaction = math.min(1.0, prod / cons) end \
         return {{production_watts=prod, consumption_watts=cons, satisfaction=satisfaction}} \
         end)()"
    )
}

pub fn research_status() -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local force = p.force \
         local current = force.current_research \
         local result = {{}} \
         if current then \
           result.current = current.name \
           result.progress = force.research_progress \
         end \
         local queue = {{}} \
         if force.research_queue then \
           for i, tech in ipairs(force.research_queue) do \
             if i > 10 then break end \
             queue[#queue+1] = tech.name \
           end \
         end \
         result.queue = queue \
         return result \
         end)()"
    )
}

pub fn recipe(name: &str) -> String {
    let safe_name = sanitize_lua_string(name);
    format!(
        "(function() \
         local r = prototypes.recipe[\"{safe_name}\"] \
         if not r then return {{error=\"recipe_not_found\"}} end \
         local ingredients = {{}} \
         for _, ing in ipairs(r.ingredients) do \
           ingredients[#ingredients+1] = {{name=ing.name, type=ing.type, amount=ing.amount}} \
         end \
         local products = {{}} \
         for _, prod in ipairs(r.products) do \
           products[#products+1] = {{name=prod.name, type=prod.type, amount=prod.amount}} \
         end \
         return {{name=r.name, energy=r.energy, ingredients=ingredients, products=products}} \
         end)()"
    )
}

pub fn nearby_entities(radius: f64) -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local ents = p.surface.find_entities_filtered{{position=p.position, radius={radius}}} \
         local result = {{}} \
         local count = 0 \
         for _, e in ipairs(ents) do \
           if count >= 50 then break end \
           if e.type ~= \"resource\" and e.type ~= \"tree\" and e.type ~= \"simple-entity\" then \
             result[#result+1] = {{name=e.name, type=e.type, x=e.position.x, y=e.position.y}} \
             count = count + 1 \
           end \
         end \
         return {{entities=result}} \
         end)()"
    )
}

pub fn nearby_resources(radius: f64) -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local ents = p.surface.find_entities_filtered{{type=\"resource\", position=p.position, radius={radius}}} \
         local grouped = {{}} \
         for _, e in ipairs(ents) do \
           local key = e.name \
           if not grouped[key] then \
             grouped[key] = {{name=key, total_amount=0, sum_x=0, sum_y=0, count=0}} \
           end \
           local g = grouped[key] \
           g.total_amount = g.total_amount + e.amount \
           g.sum_x = g.sum_x + e.position.x \
           g.sum_y = g.sum_y + e.position.y \
           g.count = g.count + 1 \
         end \
         local result = {{}} \
         for _, g in pairs(grouped) do \
           result[#result+1] = {{ \
             name=g.name, \
             total_amount=g.total_amount, \
             center_x=g.sum_x/g.count, \
             center_y=g.sum_y/g.count \
           }} \
         end \
         return {{resources=result}} \
         end)()"
    )
}

pub fn assemblers(limit: u32) -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local ents = p.surface.find_entities_filtered{{type=\"assembling-machine\", limit={limit}}} \
         local result = {{}} \
         for _, e in ipairs(ents) do \
           local recipe_name = nil \
           local r = e.get_recipe() \
           if r then recipe_name = r.name end \
           result[#result+1] = {{ \
             name=e.name, \
             x=e.position.x, \
             y=e.position.y, \
             recipe=recipe_name, \
             crafting_speed=e.crafting_speed \
           }} \
         end \
         return {{assemblers=result}} \
         end)()"
    )
}

pub fn furnaces(limit: u32) -> String {
    format!(
        "(function() {PLAYER_CHECK} \
         local ents = p.surface.find_entities_filtered{{type=\"furnace\", limit={limit}}} \
         local result = {{}} \
         for _, e in ipairs(ents) do \
           local recipe_name = nil \
           local r = e.get_recipe() \
           if r then recipe_name = r.name end \
           local fuel_type = nil \
           local fuel_inv = e.get_fuel_inventory() \
           if fuel_inv then \
             for i = 1, #fuel_inv do \
               local stack = fuel_inv[i] \
               if stack.valid_for_read then fuel_type = stack.name break end \
             end \
           end \
           local output_item = nil \
           local output_inv = e.get_output_inventory() \
           if output_inv then \
             for i = 1, #output_inv do \
               local stack = output_inv[i] \
               if stack.valid_for_read then output_item = stack.name break end \
             end \
           end \
           result[#result+1] = {{ \
             name=e.name, \
             x=e.position.x, \
             y=e.position.y, \
             recipe=recipe_name, \
             fuel_type=fuel_type, \
             output_item=output_item \
           }} \
         end \
         return {{furnaces=result}} \
         end)()"
    )
}

/// Sanitize a string for safe interpolation into Lua string literals.
/// Escapes backslashes, double quotes, and square brackets.
fn sanitize_lua_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_position_contains_iife() {
        let lua = player_position();
        assert!(lua.starts_with("(function()"));
        assert!(lua.ends_with("end)()"));
    }

    #[test]
    fn test_player_position_has_player_check() {
        let lua = player_position();
        assert!(lua.contains("game.connected_players[1]"));
        assert!(lua.contains("no_player"));
    }

    #[test]
    fn test_player_position_uses_dot_syntax() {
        let lua = player_position();
        assert!(lua.contains("p.position.x"));
        assert!(lua.contains("p.surface.name"));
    }

    #[test]
    fn test_player_inventory_loops_inventory() {
        let lua = player_inventory();
        assert!(lua.contains("get_main_inventory()"));
        assert!(lua.contains("valid_for_read"));
        assert!(lua.contains("stack.name"));
    }

    #[test]
    fn test_production_stats_uses_dot_syntax() {
        let lua = production_stats("iron-plate");
        assert!(lua.contains("get_item_production_statistics"));
        assert!(lua.contains("iron-plate"));
        // DOT syntax, not colon
        assert!(lua.contains("stats.get_input_count"));
        assert!(lua.contains("stats.get_output_count"));
    }

    #[test]
    fn test_production_stats_sanitizes_input() {
        let lua = production_stats(r#"iron"; os.execute("rm"#);
        // Quotes are escaped so Lua can't break out of the string literal
        assert!(lua.contains(r#"iron\"; os.execute(\"rm"#));
        // The unescaped quote pattern should NOT appear
        assert!(!lua.contains(r#"iron"; os"#));
    }

    #[test]
    fn test_nearby_entities_caps_at_50() {
        let lua = nearby_entities(20.0);
        assert!(lua.contains("count >= 50"));
    }

    #[test]
    fn test_nearby_entities_filters_noise() {
        let lua = nearby_entities(20.0);
        assert!(lua.contains("resource"));
        assert!(lua.contains("tree"));
        assert!(lua.contains("simple-entity"));
    }

    #[test]
    fn test_nearby_resources_aggregates() {
        let lua = nearby_resources(50.0);
        assert!(lua.contains("total_amount"));
        assert!(lua.contains("sum_x"));
        assert!(lua.contains("center_x"));
    }

    #[test]
    fn test_recipe_no_player_check() {
        let lua = recipe("iron-gear-wheel");
        // Recipe uses prototypes, doesn't need a player
        assert!(!lua.contains("connected_players"));
        assert!(lua.contains("prototypes.recipe"));
    }

    #[test]
    fn test_recipe_sanitizes_name() {
        let lua = recipe(r#"test"]game.tick--"#);
        assert!(lua.contains(r#"test\"\]game.tick--"#));
    }

    #[test]
    fn test_assemblers_respects_limit() {
        let lua = assemblers(15);
        assert!(lua.contains("limit=15"));
    }

    #[test]
    fn test_furnaces_checks_fuel_and_output() {
        let lua = furnaces(30);
        assert!(lua.contains("get_fuel_inventory()"));
        assert!(lua.contains("get_output_inventory()"));
    }

    #[test]
    fn test_research_status_handles_queue() {
        let lua = research_status();
        assert!(lua.contains("current_research"));
        assert!(lua.contains("research_queue"));
        assert!(lua.contains("research_progress"));
    }

    #[test]
    fn test_power_stats_finds_poles() {
        let lua = power_stats();
        assert!(lua.contains("electric-pole"));
        assert!(lua.contains("electric_network_statistics"));
        assert!(lua.contains("get_flow_count"));
        assert!(lua.contains("precision_index=defines.flow_precision_index.one_second"));
    }

    #[test]
    fn test_sanitize_lua_string() {
        assert_eq!(sanitize_lua_string("iron-plate"), "iron-plate");
        assert_eq!(sanitize_lua_string(r#"a"b"#), r#"a\"b"#);
        assert_eq!(sanitize_lua_string(r"a\b"), r"a\\b");
        assert_eq!(sanitize_lua_string("a[b]"), r"a\[b\]");
    }
}
