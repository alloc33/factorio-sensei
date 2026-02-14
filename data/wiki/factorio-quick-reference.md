# Factorio Quick Reference
> Numbers a pro has memorized. See `factorio-guide.md` for full formulas.

---

## Drills
| Drill | Output | Notes |
|-------|--------|-------|
| Burner | 0.25 ore/s | Half speed, needs fuel manually |
| Electric | 0.5 ore/s | The standard. Everything is based on this |
| Big Mining Drill (SA) | 2.5 ore/s | 5× faster, 50% resource drain. Other planets only |

**Belt saturation (electric drills, no modules):**
- Yellow belt (15/s) → 30 drills
- Red belt (30/s) → 60 drills
- Blue belt (45/s) → 90 drills

---

## Furnaces
| Furnace | Craft Speed | Iron plate/s | Steel plate/s |
|---------|-------------|-------------|---------------|
| Stone | 1.0 | 0.3125 | 0.0179 |
| Steel | 2.0 | 0.625 | 0.0357 |
| Electric | 2.0 | 0.625 | 0.0357 |

**Key ratio:** 1 electric drill (0.5/s) → feeds ~1.6 stone furnaces or ~0.8 steel furnaces

**Steel:** 5 iron plates → 1 steel, takes 16s at speed 1.0. Very slow. Plan big.

---

## Belts
| Belt | Per lane | Full belt (2 lanes) |
|------|----------|---------------------|
| Yellow | 7.5/s | **15/s** |
| Red | 15/s | **30/s** |
| Blue | 22.5/s | **45/s** |

These numbers are law. If your assemblers consume 16/s, one yellow belt is not enough.

---

## Inserters (items/s, chest→chest)
| Inserter | Items/s |
|----------|---------|
| Burner | 0.59 |
| Regular | 0.83 |
| Long-handed | 1.15 |
| Fast | 2.31 |
| Stack (cap 12) | 27.69 |

**Rule of thumb:** Regular inserters are fine until mid-game. Fast inserters when throughput matters. Stack inserters for bulk (ores, plates, ammo).

---

## Assemblers
| Machine | Craft Speed |
|---------|-------------|
| Assembler 1 | 0.5 |
| Assembler 2 | 0.75 |
| Assembler 3 | 1.25 |

**Output formula:** `items_per_second = craft_speed / recipe_time`

---

## The Ratios You Must Know

**Green circuits** (the most important ratio in the game):
- Recipe: 1 iron + 1.5 copper wire → 1 green circuit (0.5s)
- 3 copper wire assemblers : 2 green circuit assemblers
- This is sacred. Memorize it.

**Red science:** 5 gear assemblers : 10 red science assemblers

**Gears:** 1 gear assembler supports ~10 consumers (very efficient, don't overbuild)

**Steel smelting:** 1 steel furnace needs 5 iron furnaces feeding it

---

## Power

**Steam setup (early game):**
- 1 offshore pump = 1200 water/s
- 1 boiler consumes 60 water/s, outputs 60 steam/s
- 1 steam engine consumes 30 steam/s, produces 900kW
- **Ratio: 1 pump → 20 boilers → 40 steam engines = 36MW**

**Solar (mid-game):**
- 1 solar panel = 60kW (daytime only)
- Ratio: 21 solar panels : 25 accumulators (for 24h coverage)

---

## Think In Rates, Not Amounts

The single most important mindset shift:
- Don't think "I need 200 iron plates"
- Think "I need 2 iron plates **per second**, sustained"
- Then work backwards: how many furnaces? how many drills? what belt tier?

**The universal formula:**
```
machines_needed = (target_per_second × recipe_time) / craft_speed
```

Example: I need 3 green circuits/s
```
machines = (3 × 0.5) / 0.75 = 2 assembler 2s
```
Then figure out inputs: 2 machines × 3 iron/s + 2 machines × 4.5 copper/s = ...
and work backwards through the whole chain.

---

## Common Mistakes
- **Not enough smelting.** You always need more plates than you think.
- **Mixing belt lanes.** Keep iron and copper on separate belts or dedicated lanes.
- **Ignoring throughput.** One yellow belt of iron feeding 20 assemblers will starve them.
- **Building too tight.** Leave space. You will need to expand everything.
- **Not using the production stats (P key).** This tells you exactly where your bottleneck is.
