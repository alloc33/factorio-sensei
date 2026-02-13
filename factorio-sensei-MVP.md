# factorio-sensei — MVP Plan

## Vision
A Rust-native AI coach for Factorio 2.x. You play normally on Steam. Sensei watches your game via RCON — reads your inventory, production rates, power grid, research, factory layout — and coaches you to play like a pro. You ask questions via terminal or in-game chat, it answers with real analysis of your actual game state, backed by Factorio wiki knowledge.

## How it works

```
You play Factorio (Steam, macOS)
    ↓ RCON (TCP, port 27015)
factorio-sensei (Rust binary)
    ├── RCON client ─── reads game state (inventory, production, power, research, entities)
    ├── Rig agent ───── Claude via Anthropic API (multi-turn, typed tools)
    ├── RAG engine ──── Factorio wiki + recipes (rig-sqlite + embeddings)
    ├── Chat bridge ─── in-game chat via Lua mod (/coach command)
    └── Terminal UI ─── interactive REPL for detailed conversations
```

## User choices
- Interface: terminal + in-game chat
- Coaching style: reactive (ask questions, get answers). Proactive alerts are post-MVP.
- Factorio: 2.x (Space Age) on Steam/macOS
- LLM: Anthropic API (Claude) via Rig

## Crate dependencies

| Crate | Purpose |
|-------|---------|
| `factorio-rcon` | Our async RCON client (workspace sibling, published to crates.io) |
| `rig-core` | LLM agent framework (tools, multi-turn, RAG) |
| `rig-sqlite` | Vector store for wiki/recipe RAG |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization |
| `clap` | CLI argument parsing |
| `reedline` or `rustyline` | Terminal REPL |
| `thiserror` / `anyhow` | Error handling |
| `tracing` | Logging |

Note: the existing `rcon` crate on crates.io (panicbit/rust-rcon) is broken for Factorio (multi-packet responses don't work, last updated ~2017, sync-first API). We'll build `factorio-rcon` — a standalone async RCON crate, published to crates.io. The protocol is simple (length-prefixed TCP packets, 3 packet types). Reference: `factorio-ai-companion/src/rcon/client.ts`.

## MVP Phases

### Phase 1: RCON Client (`factorio-rcon` crate)
Build an async RCON client as a standalone, publishable crate. Lives as a workspace sibling to factorio-sensei.

**Why a separate crate:**
- The existing `rcon` crate on crates.io is broken for Factorio (multi-packet responses fail, abandoned since ~2017)
- Real gap on crates.io — other Factorio-Rust projects need this
- Clean portfolio piece on its own (async, well-typed, small API surface)
- Trivial overhead — same code, just a separate `Cargo.toml` with crate metadata

**What to build:**
- `factorio-rcon/Cargo.toml` — crate metadata (license, description, keywords)
- `factorio-rcon/README.md` — usage examples
- `factorio-rcon/src/lib.rs` — public API
- `factorio-rcon/src/client.rs` — async TCP connection, auth, command execution
- `factorio-rcon/src/protocol.rs` — packet framing (length-prefixed, type=3 auth, type=2 command)
- `factorio-rcon/src/error.rs` — RCON error types

**Public API (target):**
```rust
let rcon = RconClient::connect("127.0.0.1:27015", "password").await?;
let response = rcon.execute("/version").await?;
let lua_result = rcon.execute("/c rcon.print(game.tick)").await?;
```

**RCON protocol (from factorio-ai-companion reference):**
- TCP connection to `127.0.0.1:27015`
- Auth packet: 4-byte length + 4-byte request_id + 4-byte type(3) + password + 2 null bytes
- Command packet: 4-byte length + 4-byte request_id + 4-byte type(2) + command + 2 null bytes
- Response: same framing, parse payload as UTF-8 string
- Must handle Factorio's multi-packet response fragmentation correctly

**Workspace setup:**
```
factorio-projects/
├── factorio-rcon/          # standalone crate, published to crates.io
│   ├── Cargo.toml
│   └── src/
└── factorio-sensei/        # depends on factorio-rcon via path during dev
    ├── Cargo.toml           # factorio-rcon = { path = "../factorio-rcon" }
    └── src/
```

**Verification:** Connect to a Factorio server, send `/version`, get response. Then `cargo publish`.

### Phase 2: Game State Tools (Rig Tools)
Implement typed Rig tools that read game state via RCON + Lua commands.

**Tools to implement (MVP set — all read-only, no cheating):**

| Tool | What it reads | Lua pattern |
|------|--------------|-------------|
| `get_player_inventory` | All items in player's main inventory | `game.connected_players[1].get_main_inventory()` |
| `get_player_position` | Player x,y coordinates | `game.connected_players[1].position` |
| `get_production_stats` | Production/consumption for an item | `game.forces["player"].get_item_production_statistics("nauvis")` |
| `get_power_stats` | Electricity production, consumption, satisfaction | `electric_network_statistics.get_flow_count()` |
| `get_research_status` | Current research, progress, queue | `game.forces["player"].current_research` |
| `get_nearby_entities` | Buildings/machines within radius | `surface.find_entities_filtered{position=pos, radius=r}` |
| `get_nearby_resources` | Ore patches near player | `find_entities_filtered{type="resource"}` |
| `get_assemblers` | Assembling machines + their recipes | `find_entities_filtered{type="assembling-machine"}` |
| `get_furnaces` | Furnaces + contents + fuel | `find_entities_filtered{type="furnace"}` |
| `get_recipe` | Recipe ingredients/products for an item | `prototypes.recipe[name]` |

Each tool is a Rust struct implementing Rig's `Tool` trait with typed `Args`/`Output`.

**Key Lua patterns (from factorio_llm reference):**
- All queries wrapped in `(function() ... end)()` IIFE
- Results via `rcon.print(serpent.line(...))`
- Use DOT syntax, not colon (Factorio 2.x)
- Parse serpent output → serde_json on the Rust side

**Verification:** Each tool returns valid typed data from a live Factorio game.

### Phase 3: Rig Agent (The Coach)
Wire tools into a Rig agent with Claude as the LLM.

**What to build:**
- `src/agent/mod.rs` — agent construction and configuration
- `src/agent/prompts.rs` — system prompt for coaching behavior

**Agent setup:**
```rust
let agent = anthropic_client
    .agent("claude-sonnet-4-5-20250929")
    .preamble(COACH_SYSTEM_PROMPT)
    .tool(GetPlayerInventory::new(rcon.clone()))
    .tool(GetProductionStats::new(rcon.clone()))
    .tool(GetPowerStats::new(rcon.clone()))
    .tool(GetResearchStatus::new(rcon.clone()))
    .tool(GetNearbyEntities::new(rcon.clone()))
    .tool(GetAssemblers::new(rcon.clone()))
    .tool(GetFurnaces::new(rcon.clone()))
    .tool(GetRecipe::new(rcon.clone()))
    .default_max_turns(10)
    .build();
```

**System prompt key points:**
- You are a Factorio coaching expert. Your job is to analyze and teach, not to play.
- Always use tools to check actual game state before giving advice.
- When analyzing production, check ratios against known optimal ratios.
- Explain WHY something is a bottleneck, not just WHAT to build.
- Reference specific numbers (e.g., "you're producing 15 iron plates/min but consuming 22").
- Keep responses concise — player is in-game, not reading essays.

**Verification:** Ask "what's in my inventory?" and get a real answer. Ask "is my power grid okay?" and get analysis.

### Phase 4: Terminal REPL
Interactive terminal interface for chatting with the coach.

**What to build:**
- `src/ui/terminal.rs` — REPL loop with history, colored output
- `src/main.rs` — CLI entry point with clap

**Features:**
- `reedline` or `rustyline` for readline-like input (history, Ctrl+R search)
- Colored output (tool calls in dim, coach responses in normal)
- `/status` command — quick game state summary
- `/quit` command
- Conversation history maintained across turns

**CLI args:**
```
factorio-sensei --host 127.0.0.1 --port 27015 --password <rcon_password>
```

**Verification:** Run binary, type questions, get coaching responses.

### Phase 5: Lua Mod + In-Game Chat
Thin Factorio mod that bridges in-game chat to sensei.

**What to build:**
- `factorio-mod/info.json` — mod metadata (Factorio 2.0+)
- `factorio-mod/control.lua` — registers `/coach` command, message queue

**How it works:**
1. Player types `/coach why is my iron production slow?` in Factorio
2. Lua mod stores message in `storage.coach_messages` table
3. Rust binary polls via RCON: `/c rcon.print(helpers.table_to_json(storage.coach_messages))`
4. Sensei processes the question, calls tools, generates advice
5. Rust sends response back via RCON: `/c game.print("[Sensei] Your furnaces are...")`

**Message format:**
```lua
storage.coach_messages = {
    { player = "nshv", message = "why is my iron slow?", tick = 12345, read = false }
}
```

**Verification:** Type `/coach` in Factorio, see response in game chat.

### Phase 6: RAG — Factorio Wiki + Recipes
Add Factorio knowledge so the coach gives expert advice.

**What to build:**
- `src/knowledge/mod.rs` — RAG pipeline
- `src/knowledge/recipes.rs` — structured recipe database
- `src/knowledge/wiki.rs` — wiki content embedder
- `data/recipes.json` — all Factorio 2.x recipes (extracted from game data)
- `data/wiki/` — key wiki articles as text files

**Two knowledge systems:**

1. **Structured recipe lookup** (not RAG — direct tool):
   - `GetRecipe` tool queries a JSON/SQLite database of all recipes
   - Includes: ingredients, products, craft time, machine type, optimal ratios
   - Source: extract from Factorio's data.raw via a one-time Lua script

2. **Wiki RAG** (semantic search via rig-sqlite):
   - Embed key wiki articles: production ratios, oil processing, train signals, nuclear power, circuit networks, common blueprints/patterns
   - Agent gets `.dynamic_context(2, wiki_index)` — top 2 relevant articles per query
   - Embedding model: Anthropic or a local model via `rig-fastembed`

**Verification:** Ask "how should I set up oil processing?" and get an answer that references actual game mechanics and ratios.

## Project structure (final)

```
factorio-projects/
├── factorio-rcon/              # standalone crate → crates.io
│   ├── Cargo.toml
│   ├── CLAUDE.md
│   ├── README.md
│   └── src/
│       ├── lib.rs              # public API
│       ├── client.rs           # async RCON client
│       ├── protocol.rs         # packet framing
│       └── error.rs
│
├── factorio-sensei/
│   ├── Cargo.toml              # depends on factorio-rcon = { path = "../factorio-rcon" }
│   ├── CLAUDE.md
│   ├── factorio-sensei-MVP.md      # this MVP plan
│   ├── src/
│   │   ├── main.rs             # CLI entry point
│   │   ├── tools/
│   │   ├── mod.rs
│   │   ├── inventory.rs        # GetPlayerInventory
│   │   ├── production.rs       # GetProductionStats
│   │   ├── power.rs            # GetPowerStats
│   │   ├── research.rs         # GetResearchStatus
│   │   ├── entities.rs         # GetNearbyEntities, GetAssemblers, GetFurnaces
│   │   └── recipes.rs          # GetRecipe (structured lookup)
│   ├── agent/
│   │   ├── mod.rs              # agent construction
│   │   └── prompts.rs          # system prompt
│   ├── knowledge/
│   │   ├── mod.rs
│   │   └── wiki.rs             # RAG pipeline
│   └── ui/
│       └── terminal.rs         # REPL
│   ├── factorio-mod/
│   │   ├── info.json
│   │   └── control.lua         # /coach command
│   └── data/
│       ├── recipes.json        # all Factorio 2.x recipes
│       └── wiki/               # wiki articles for RAG
│
├── rig/                        # reference repos (not our code)
├── factorio_llm/
├── factorio-ai-companion/
└── factorio-learning-environment/
```

## Implementation order

Phase 1 (factorio-rcon crate + publish to crates.io) → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6

Each phase is independently testable. Phase 1 produces a standalone published crate. Phase 4 (terminal) gives you a working product. Phase 5 (in-game chat) makes it seamless. Phase 6 (RAG) makes the advice expert-level.

## What we're NOT building (MVP scope)
- No proactive alerts (post-MVP)
- No action execution (no placing entities, mining, crafting — read-only coach)
- No multiplayer support (single player hosted as multiplayer for RCON)
- No MCP server (post-MVP — would let it work as Claude Code extension)
- No visual/screenshot analysis
- No blueprint analysis
