# factorio-sensei

Rust-native AI coaching copilot for Factorio 2.x. Watches your game via RCON, analyzes your factory, and teaches you to play like a pro. Built with Rig (Rust LLM framework) + Claude.

## MVP plan
See `factorio-sensei.md` for full implementation plan with phases, tools, and project structure.

## Architecture
```
Factorio (Steam) → RCON → factorio-sensei (Rust) → Rig agent (Claude) → coaching advice
                                                  → RAG (wiki + recipes)
```

Read-only coach — observes and advises, does not execute game actions.

## Tech stack
- `factorio-rcon` — our async RCON crate (workspace sibling at `../factorio-rcon`, (not published yet to crates.io))
- Rig (`rig-core`, `rig-sqlite`) — LLM agent framework
- Anthropic API (Claude) — reasoning engine
- Tokio — async runtime
- Lua mod — in-game `/coach` chat command

## Reference repos (cloned in parent directory)
These repos are cloned at `/Users/nshv/personal-vault/Gaming/factorio-projects/`:

| Repo | What to reference |
|------|-------------------|
| `rig/` | Rig framework — Tool trait, agent builder, Anthropic provider, RAG |
| `factorio_llm/` | RCON wrapper pattern, Lua game state queries, serpent parsing |
| `factorio-ai-companion/` | RCON protocol implementation, MCP server pattern, Lua mod structure, chat bridge |
| `factorio-learning-environment/` | Comprehensive Factorio tool API design, game action patterns |

Key reference files:
- Rig Tool trait: `rig/rig/rig-core/src/tool/mod.rs`
- Rig agent builder: `rig/rig/rig-core/src/agent/builder.rs`
- Rig Anthropic provider: `rig/rig/rig-core/src/providers/anthropic/client.rs`
- Python RCON + Lua queries: `factorio_llm/src/factorio_tools.py`
- TypeScript RCON protocol: `factorio-ai-companion/src/rcon/client.ts`
- Lua mod structure: `factorio-ai-companion/factorio-mod/control.lua`

## Factorio RCON notes
- Factorio must run as multiplayer host (Multiplayer → Host New Game) for RCON to work
- Default port: 27015, configured in Factorio's config.ini
- Commands: `/c <lua>` for Lua execution, `rcon.print()` for return values
- Factorio 2.x uses DOT syntax (not colon) for API method calls
- Using `/c` commands disables achievements for the save
