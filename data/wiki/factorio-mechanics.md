# Factorio Mechanics Deep Dive
## Low-Level Game Engine Reference for Optimization-Focused Play

---

## 1. TICK SYSTEM & SIMULATION

### Core Constants
```
TICKS_PER_SECOND = 60
TICK_DURATION = 16.667ms (1/60 second)
```

### Simulation Model
- All game logic is deterministic and tick-based
- Every entity updates once per tick in a defined order
- Recipe times shown in UI are in **seconds**, not ticks
- `actual_ticks = recipe_time_seconds × 60`

### UPS vs FPS
- **UPS** (Updates Per Second): Simulation speed, soft-capped at 60
- **FPS** (Frames Per Second): Rendering speed, independent of UPS
- When CPU can't complete tick calculations in 16.67ms, UPS drops
- Game slows down proportionally (30 UPS = half speed)

### Debug: Press F4 → enable "show-fps" to monitor

---

## 2. BELT PHYSICS

### Internal Position System
Belts use fixed-point arithmetic with **256 positions per tile**:
```
POSITIONS_PER_TILE = 256
ITEM_SPACING = 64 positions (items are 64/256 = 0.25 tiles apart)
MAX_ITEMS_PER_TILE = 4 (per lane)
```

### Belt Speeds (positions per tick)
| Belt Type   | Positions/Tick | Tiles/Tick | Items/Second |
|-------------|----------------|------------|--------------|
| Yellow      | 8              | 0.03125    | 15           |
| Red         | 16             | 0.0625     | 30           |
| Blue        | 24             | 0.09375    | 45           |
| Turbo (SA)  | 32             | 0.125      | 60           |

### Items Per Second Formula
```
items_per_second = (belt_speed_positions_per_tick × 60) / 64
```

### Turn Lane Positions
- **Straight segment**: 256 positions
- **Outer turn lane**: 295 positions (items slow down)
- **Inner turn lane**: 106 positions (items speed up)

### Critical Insight: Item Positioning
- Items exist on exactly ONE tile at any moment (no in-between states)
- Inserters can only pick up items from the tile they're "on"
- Compression = items spaced exactly 64 positions apart

### Belt Throughput Table (items/second per lane)
| Belt    | Per Lane | Full Belt (2 lanes) |
|---------|----------|---------------------|
| Yellow  | 7.5      | 15                  |
| Red     | 15       | 30                  |
| Blue    | 22.5     | 45                  |
| Turbo   | 30       | 60                  |

---

## 3. INSERTER MECHANICS

### Rotation Speed & Timing
Inserters rotate at a fixed **degrees per tick** or **turns per tick**.

| Inserter       | Rotation Speed | Turns/Tick | Chest→Chest Ticks |
|----------------|----------------|------------|-------------------|
| Burner         | 106°/s         | 0.00491    | 102               |
| Regular        | 149°/s         | 0.00694    | 72                |
| Long-handed    | 214°/s         | 0.00992    | 52                |
| Fast           | 864°/s         | 0.04       | 26                |
| Stack          | 864°/s         | 0.04       | 26                |
| Bulk (SA)      | 864°/s         | 0.04       | 26                |

### Chest-to-Chest Cycle Time Formula
```
half_turn_ticks = floor(0.5 / rotation_speed_turns_per_tick)
cycle_ticks = half_turn_ticks × 2 + 2
```
The `+2` accounts for pickup and drop actions.

### Important: Tick Quantization
If rotation speed would give an odd number of ticks per full turn, it's **truncated to the next lowest even number**. Inserters must complete half-turns in whole ticks.

Example: Fast inserter at 0.04 turns/tick → 25 ticks theoretical → actually 24 ticks (2×12).

### Belt Pickup Timing Variables
Pickup time depends on:
1. **Lane**: Near side ~8.7% faster than far side
2. **Belt direction**: Affects when items enter pickup zone
3. **Inserter orientation**: North-facing ~4% slower than East/West

### Drop Position
Items dropped on belts land **64 positions from the start of the next tile** (not centered).

### Items Per Second (chest-to-chest)
```
items_per_second = (hand_size × 60) / cycle_ticks
```

| Inserter       | Hand Size | Cycle Ticks | Items/sec (1 stack) |
|----------------|-----------|-------------|---------------------|
| Burner         | 1         | 102         | 0.59                |
| Regular        | 1         | 72          | 0.83                |
| Long-handed    | 1         | 52          | 1.15                |
| Fast           | 1         | 26          | 2.31                |
| Stack (12 cap) | 12        | 26          | 27.69               |
| Bulk (SA, 12)  | 12        | 26          | 27.69               |

---

## 4. CIRCUIT NETWORK

### Signal Propagation
```
COMBINATOR_DELAY = 1 tick
WIRE_DELAY = 0 ticks (instant across any distance)
```

### Update Order (per tick)
1. All entities read their input signals (from previous tick's network state)
2. All entities compute their outputs
3. Network values update as sum of all connected outputs

### Key Implications
- Combinator chains add 1 tick delay per combinator
- Inserters reading circuits have 1-tick lag before responding
- SR latches need 2+ tick pulses to flip reliably

### Pulse Detection (1-tick pulse generator)
```
Signal change detection:
- Connect signal to arithmetic combinator: EACH × -1 → EACH
- Sum original signal with combinator output via different colored wire
- Result is non-zero for exactly 1 tick when signal changes
```

### Signal Value Range
```
MIN_VALUE = -2,147,483,648
MAX_VALUE = 2,147,483,647
(32-bit signed integer)
```

---

## 5. CRAFTING & PRODUCTION

### Core Formula
```
actual_craft_time = recipe_time / crafting_speed
items_per_second = (items_per_craft × crafting_speed) / recipe_time
```

### Crafting Speeds
| Machine              | Crafting Speed |
|----------------------|----------------|
| Player (hand)        | 1.0            |
| Assembler 1          | 0.5            |
| Assembler 2          | 0.75           |
| Assembler 3          | 1.25           |
| Chemical Plant       | 1.0            |
| Oil Refinery         | 1.0            |
| Stone Furnace        | 1.0            |
| Steel Furnace        | 2.0            |
| Electric Furnace     | 2.0            |
| Foundry (SA)         | 4.0            |
| Electromagnetic (SA) | 2.0            |
| Biochamber (SA)      | 2.0            |
| Cryogenic (SA)       | 2.0            |

### Example Calculation
Green circuits: 0.5s recipe, 1 output
- In Assembler 1: `1 / (0.5 / 0.5) = 1.0 items/sec`
- In Assembler 2: `1 / (0.5 / 0.75) = 1.5 items/sec`
- In Assembler 3: `1 / (0.5 / 1.25) = 2.5 items/sec`

### Machines Needed Formula
```
machines_needed = (target_items_per_second × recipe_time) / (crafting_speed × items_per_craft)
```

---

## 6. MINING

### Mining Formula
```
output_rate = (mining_power - mining_hardness) × mining_speed / mining_time × productivity_bonus
```

Note: Mining hardness was removed in 1.0. Simplified formula:
```
output_rate = mining_speed / mining_time × (1 + productivity_bonus)
```

### Electric Mining Drill Stats
- Base mining speed: 0.5/s
- Mining time: 1s (for most ores)
- Base output: **0.5 ore/second** per drill

### Mining Productivity Research
- **+10% per level** (additive)
- No speed penalty
- Infinite research (keeps going)

### Resource Drain
| Drill Type    | Drain Rate |
|---------------|------------|
| Burner        | 100%       |
| Electric      | 100%       |
| Big (SA)      | 50%        |

Quality reduces drain by **1/6 per quality level**.

### Drill Ratios for Belt Saturation
| Belt   | Items/sec | Drills Needed (base) |
|--------|-----------|----------------------|
| Yellow | 15        | 30                   |
| Red    | 30        | 60                   |
| Blue   | 45        | 90                   |

With +100% mining productivity: halve these numbers.

---

## 7. FLUID DYNAMICS

### Fluid System Constants
```
PIPE_CAPACITY = 100 units
STORAGE_TANK_CAPACITY = 25,000 units
PUMP_OUTPUT = 1,200 units/second (theoretical max)
MAX_FLOW_PER_TICK = 100 units/connection
MAX_FLOW_PER_SECOND = 6,000 units (theoretical)
```

### Flow Formula (simplified)
```
flow_per_tick = (pressure_A - pressure_B) × 0.4 + momentum
pressure = (current_level / max_capacity) × (max_pressure - zero_pressure) + zero_pressure
```

### Practical Throughput
Flow decreases **exponentially** with pipe length:
| Distance (tiles) | Approx. Flow (units/s) |
|------------------|------------------------|
| 1-2              | ~6,000                 |
| 12               | ~1,285                 |
| 50               | ~600                   |
| 200              | ~200                   |

### Underground Pipes
- Count as **full distance** (no throughput bonus since 0.17)
- Still only 2 entities for UPS

### Pump Behavior
- Forces connected pipe to 100% fill
- Blocks backflow completely
- Outputs at maximum "pressure"
- Use pumps every 12-17 pipes for long runs

### Quirks
- **Build order affects flow**: entities updated in placement order
- **Preferred directions exist**: flow can differ based on orientation
- **Tank rotation matters**: affects which output gets priority

### Best Practices
1. Use pumps liberally
2. Keep pipe runs short
3. Use trains/barrels for long distance
4. Parallelize instead of longer pipes

---

## 8. POWER NETWORK

### Priority Order (highest to lowest)
1. **Solar panels**: Always output maximum or match demand
2. **Lightning collectors** (Space Age)
3. **Steam engines/turbines/fusion**: Share remaining load equally
4. **Accumulators**: Only discharge when other sources insufficient

### Key Metrics
- **Satisfaction** = consumption / demand (should be 100%)
- **Production** = output / capacity (should NOT be 100%)

### Accumulator Specs
- Capacity: 5 MJ
- Max discharge: 300 kW
- Full discharge time: 16.67 seconds

### Flicker Prevention Formula
```
accumulators_needed = peak_consumption / 300kW
```

### Drain vs Consumption
- **Drain**: Constant power draw when entity is connected (always on)
- **Consumption**: Power used when entity is active

Example: Radar has 300kW drain (always) + additional consumption when scanning.

### Power Switch Trick
Use circuit network to disconnect beacons/machines when not needed to save the significant drain.

### No Direct Circuit Access
- Cannot read satisfaction % directly
- Workaround: Monitor accumulator charge level
  - Below 100% = insufficient production

---

## 9. TRAIN PHYSICS

### Acceleration Formula (per tick)
```python
# Step 1: Apply friction
speed = max(0, abs(speed) - friction_force / weight)

# Step 2: Apply locomotive force
speed = speed + (10 × locos_in_direction × fuel_accel_bonus / weight)

# Step 3: Apply air resistance
speed = speed × (1 - air_resistance / (weight / 1000))

# Step 4: Cap to maximum
speed = min(speed, 1.2 × fuel_top_speed_multiplier)
```

### Rolling Stock Properties
| Type          | Weight | Friction | Braking Force | Air Resistance |
|---------------|--------|----------|---------------|----------------|
| Locomotive    | 2000   | 0.5      | 10            | 0.0075         |
| Cargo Wagon   | 1000   | 0.5      | 3             | 0.01           |
| Fluid Wagon   | 1000   | 0.5      | 3             | 0.01           |
| Artillery    | 4000   | 0.5      | 6             | 0.015          |

### Fuel Bonuses
| Fuel          | Acceleration | Top Speed |
|---------------|--------------|-----------|
| Wood          | +0%          | +0%       |
| Coal          | +0%          | +0%       |
| Solid Fuel    | +20%         | +5%       |
| Rocket Fuel   | +80%         | +15%      |
| Nuclear Fuel  | +150%        | +15%      |

### Maximum Speeds
```
base_max_speed = 1.2 tiles/tick = 72 tiles/second
with_nuclear_fuel = 1.2 × 1.15 = 1.38 tiles/tick = 82.8 tiles/second
```

### Braking Distance Formula
```
braking_force = Σ(rolling_stock_braking × research_bonus + friction)
distance = 0.5 × mass × velocity² / braking_force
```

### Example Braking Distance
50 locomotives, nuclear fuel, max speed (82.8 tiles/sec):
- No research: **181 tiles**
- Max research (×2): **93 tiles**

### Signal Reservation
Trains reserve their entire braking distance ahead, turning signals yellow.

---

## 10. MODULE MATHEMATICS

### Module Effects
| Module Type   | Speed  | Productivity | Energy  | Pollution |
|---------------|--------|--------------|---------|-----------|
| Speed 1       | +20%   | -            | +50%    | -         |
| Speed 2       | +30%   | -            | +60%    | -         |
| Speed 3       | +50%   | -            | +70%    | -         |
| Productivity 1| -5%    | +4%          | +40%    | +5%       |
| Productivity 2| -10%   | +6%          | +60%    | +7.5%     |
| Productivity 3| -15%   | +10%         | +80%    | +10%      |
| Efficiency 1  | -      | -            | -30%    | -         |
| Efficiency 2  | -      | -            | -40%    | -         |
| Efficiency 3  | -      | -            | -50%    | -         |

### Stacking Rules
**All bonuses within a category are ADDITIVE**:
```
total_speed = base_speed × (1 + Σ speed_bonuses - Σ speed_penalties)
total_productivity = 1 + Σ productivity_bonuses
total_energy = base_energy × (1 + Σ energy_modifiers)
```

### Beacon Transmission
```
transmission_strength = distribution_efficiency / √(num_beacons)
```

| Beacon Quality | Distribution Efficiency |
|----------------|-------------------------|
| Normal         | 1.5                     |
| Legendary      | 2.5                     |

Example: 8 normal beacons with 2× Speed 3 each:
```
per_beacon = 2 × 50% × (1.5 / √8) = 53% speed bonus per beacon
total = 8 × 53% = +424% speed from beacons
```

### Productivity Stacking (multiplicative across steps)
```
final_output = base_output × (1 + prod_step1) × (1 + prod_step2) × ...
```

| Steps | With Prod 3 (+10% each) |
|-------|-------------------------|
| 1     | 1.10× output            |
| 2     | 1.21× output            |
| 3     | 1.33× output            |
| 4     | 1.46× output            |

### Optimal Module Strategy

**Assemblers/Furnaces:**
- 4× Productivity 3 + Speed beacons

**At 12+ beacons per machine:**
- Prod 3 setup exceeds pure Speed 3 in items/sec

**Mining drills:**
- Speed beacons only worthwhile at 13+ beacons (rare)

**Labs/Rocket Silo:**
- ALWAYS max productivity modules

### Effective Output Formula
```
items_per_second = (items_per_craft × (1 + prod_bonus)) / (recipe_time / effective_speed)
effective_speed = base_speed × (1 + speed_mods - prod_penalties)
```

### Energy Efficiency
```
energy_per_item = (base_energy × (1 + energy_modifiers)) / (crafting_speed × productivity)
```

Note: Beacons always draw 480kW regardless of modules installed.

---

## 11. POLLUTION

### Pollution Formula
```
pollution = base_pollution × (1 + energy_modifier) × (1 + pollution_modifier)
```

- Energy consumption increases pollution proportionally
- Productivity modules add explicit pollution multiplier
- These stack **multiplicatively**

### Pollution Absorption
Trees, water, and biters absorb pollution. Biter evolution is driven by:
1. Time passed
2. Pollution generated
3. Spawner kills

---

## 12. CALCULATION WORKFLOW

### Step-by-Step Ratio Calculation
1. **Define target**: X items/second (or X items/minute ÷ 60)
2. **Calculate machine output**:
   ```
   output = (items_per_craft × crafting_speed × (1 + productivity)) / recipe_time
   ```
3. **Machines needed**: `target / output_per_machine`
4. **Input requirements**: `machines × inputs_per_craft × crafts_per_second`
5. **Verify belt capacity**: Compare to belt throughput limits
6. **Check power**: Sum all machine consumption + drain
7. **Iterate**: Add modules/beacons, recalculate

### Quick Reference Ratios (vanilla, no modules)
| Product            | Ratio                    |
|--------------------|--------------------------|
| Green circuits     | 3 wire : 2 circuit asm   |
| Red circuits       | 6 wire : 8 plastic : 10 red circuit asm |
| Blue circuits      | 2 green : 2 red : 1 blue (approximately) |
| Gears              | 1 gear asm : 10 users    |
| Steel              | 1 steel furnace : 5 iron furnaces |
| Red science        | 5 gear : 10 red science asm |

---

## 13. USEFUL TOOLS

### In-Game
- F4 debug menu (UPS, entity counts, etc.)
- Production statistics (P key)
- Electric network info (hover over power pole)

### External
- **Kirk McDonald Calculator**: https://kirkmcdonald.github.io/
- **Factorio Cheat Sheet**: https://factoriocheatsheet.com/
- **Rate Calculator mod** (in-game, Ctrl+N with Max Rate Calculator)

### Console Commands (for testing)
```lua
/c game.speed = 0.1  -- Slow motion
/c game.player.force.manual_mining_speed_modifier = 1000  -- Fast mining
/c game.player.force.laboratory_speed_modifier = 10  -- Fast research
```

---

## 14. KEY INSIGHTS FOR OPTIMIZATION

1. **Everything is deterministic**: Same inputs always produce same outputs
2. **Tick-based means discrete**: All timing in multiples of 1/60 second
3. **Inserter timing dominates throughput**: Belt speed rarely the bottleneck
4. **Fluids are complex**: Pressure differential + momentum, not simple flow
5. **Power priority is automatic**: No circuit control needed for basic operation
6. **Productivity compounds**: Use it everywhere allowed for massive gains
7. **Beacons have diminishing returns**: √n scaling means early beacons best
8. **UPS is your enemy at scale**: Direct insertion, train limits, bot limits

### The Meta-Rule
```
Optimal factory = maximize throughput per UPS tick
```

This means:
- Fewer entities doing more work each
- Direct insertion over belts where possible
- Chests as buffers, not belts
- Trains over bots for long-distance bulk
- Productivity modules always (free output, no extra entities)

---

*Document compiled from Factorio Wiki, official forums, and community research. Values verified against game version 2.0.*
