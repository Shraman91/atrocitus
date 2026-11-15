# Atrocitus Phase 3 Walkthrough: Core Combat & Resources Framework

Phase 3 establishes the full foundational combat loop and resource systems for *Atrocitus*. The Ashen Colosseum now acts as a complete combat proving ground with 5 fully playable class combat kits, 4 distinct AI enemies, 2.5D height band collision filtering, invulnerability frame timing, real monospace UI combat HUD, floating damage text, trauma-based screen shake, and the classic Souls death and bloodstain retrieval loop.

---

## 1. Summary of Changes & Features Implemented

### 1.1 Data-Driven Combat Configurations (`data/combat.ron`)
- Externalized all player attributes (HP, souls, mana, regen, channel timings), all 5 class ability parameters (damages, reaches, spreads, projectile speeds, cooldowns, knockbacks), and all 4 enemy archetypes (HP, souls, speeds, reaches, attack damages, cooldowns) into `data/combat.ron`.
- Built robust deserialization with default fallback so balance adjustments require no recompilation.

### 1.2 2.5D Height Bands & Frame-Data Attack System (`src/game/combat.rs`)
- **Vertical Height Bands**: `Low`, `Mid`, `High`, `All`.
  - `High` aerial targets (e.g. Carrion Fly) are completely immune to low/ground attacks and standard horizontal slashes; they can only be struck by high attacks (Knight Combo Finisher `↑High`, Hunter Flintlock `↑High`, Mage Cinder Nova `*All`) or when they dive down to bite.
  - `Low` crawlers and puddles hit low targets; ground worms can be hit by low and mid attacks, but ground acid puddles only damage low-band targets.
  - `Mid` standard standing height intersects mid and high cleaves.
  - `All` represents full 2.5D radial explosions (Mage Cinder Nova).
- **Hitbox Shapes**: `Arc` (cleaves with directional spread), `Circle` (AoE explosions), `Aabb` (boxes), and `Point/Ray` (projectiles).
- **Spatial Hash Grid Broadphase**: Zero-allocation 32px cell spatial hash grid handles broadphase collision queries without $O(n^2)$ overhead.
- **True Invulnerability Frames (i-frames)**:
  - Dodge roll i-frames active during frames 2 through 14 of the 23-frame roll cycle ($\sim 0.033\text{s}$ to $0.231\text{s}$).
  - Post-damage invulnerability window ($0.6\text{s}$) with high-frequency sprite blinking.
- **Hit-Stop & Screen Shake**: 2–4 frames of crunch hit-stop on solid impacts, coupled with trauma-based camera screen shake.

### 1.3 5 Class Combat Move Sets
- **Knight**:
  - `LMB`: 3-hit combo: Step 1 (Low slash, 2 dmg), Step 2 (Mid slash, 3 dmg), Step 3 (Rising heavy slash `↑High`, 5 dmg, 45 knockback).
  - `RMB`: Frontal Shield Block held: halves movement speed, absorbs 80% damage from frontal attacks ($\pm 70^\circ$), staggers light attackers with spark deflection.
- **Mage**:
  - `LMB`: Ember Bolt: fast cyan-fire projectile (Mid band, 4 dmg, 5 mana cost).
  - `RMB`: Cinder Nova: gathers flame aura for 0.4s charge, detonates in a 42px radial AoE explosion (All bands, 8 dmg, 60 knockback, 25 mana cost, 2.0s cooldown).
  - `E` (Hold on Corpse): Siphon Corpse: hold `E` near any defeated enemy corpse for 0.8s to channel dark purple siphon stream, destroying the corpse and restoring 40 mana.
- **Assassin**:
  - `LMB`: Rapid 3-hit dagger flurry (Mid band, fast 0.06s startup).
  - `RMB`: Shadowstep: instant 45px teleport-dash towards cursor with smoke dissipation; grants 0.5s backstab window dealing **3.0x backstab critical damage**.
- **Hunter**:
  - `LMB`: Dual blade sweep: wide 120-degree arc slash in Mid band.
  - `RMB`: Pistol-Sword Flintlock Shot: high-velocity silver bullet in `↑High` band (6 dmg, 200px range, hits flying Carrion Fly, 25px recoil on player).
- **Alchemist**:
  - `LMB`: Acid Vial Lob: arced projectile that shatters on ground into a 3.5s Low-band Acid Puddle dealing 2 DPS to ground enemies.
  - `RMB`: Eye Rite: channels floating arcane eye forward 40px, creating a 36px distortion field slowing enemies by 50% for 4.0s.

### 1.4 Enemy Archetypes & AI (`src/game/enemy.rs`)
- **Acid Sludge**: Low band, squelches toward player, vomits acid projectile cone and leaves acid puddles on the ground.
- **Bogged Archer (Skeleton)**: Mid band, kites away if player is within 45px, draws bow with green emissive telegraph, fires poisoned arrows applying Poison DoT (1 dmg / 1.5s for 6s).
- **Carrion Fly**: High band ($z = 14\text{px}$), hovers with erratic sinusoidal flight, immune to ground slashes, dive-bites player lowering to Mid band before ascending.
- **Corpse Worm**: Low band, fast swarming ground crawler with animated wiggle frames, lunges forward when within 22px.
- **Persistent Corpses**: Defeated enemies remain on the arena floor, allowing Mage corpse siphon.

### 1.5 Player Combat HUD & Souls Death Loop (`src/scenes/lobby.rs`)
- **5 Hearts with Half-Heart Granularity**: 10 HP total rendered with real terminal characters (`♥`, `♡`, `·`), flashing white on damage.
- **Souls Counter**: `✦ 100 SOULS` in amber text, dropping to 0 on death.
- **Mage Mana Bar**: `MANA [████████░░░░] 60/100` displayed under hearts for Mage mantle with continuous slow regen.
- **Status Effect Indicators**: Real-time badges for `[☣ POISON 4.2s]` and `[⚡ ACID 2.8s]`.
- **Ability Move Hints**: Contextual control guides showing move names, cooldowns, and `[↑High]` height band tags.
- **"YOU DIED" & Bloodstain Loop**:
  - Screen desaturates and displays centered red `YOU DIED` banner upon reaching 0 HP.
  - Drops a Bloodstain containing all lost souls at death location.
  - Respawns player at the bonfire after 2.5s with full health and cleared status effects.
  - Walking back to the bloodstain retrieves all lost souls with golden burst particles and centered `✦ +X SOULS RETRIEVED ✦` banner. Dying before retrieval overwrites the bloodstain.

### 1.6 Combat Debug Tools
- `F6`: Toggles Hitbox, Hurtbox, and Height Band overlay (green hurtbox, blue High band, yellow Low band, red active cleaves/projectiles/puddles).
- `F7`: Cursor Enemy Spawner cycling Sludge $\to$ Skeleton $\to$ Fly $\to$ Worm directly at world mouse position.

---

## 2. Verification Results

| Test / Command | Result | Notes |
|---|---|---|
| `cargo test` | **24 passed, 0 failed** | Unit tests for Height Bands, Spatial Hash, RON combat loading, i-frame intervals, Knight block reduction, death & respawn loop, and multi-resolution rendering (120x40, 160x50, 200x60). |
| `cargo clippy --all-targets -- -D warnings` | **0 warnings** | Completely clean code style across all targets. |
| `cargo build --release` | **Success** | Optimized release binary compiled in target/release. |
| Fixed 60 FPS Timestep | **Verified** | Zero per-frame allocations in hot collision and rendering paths. |
| Terminal Guard & Panic Hook | **Verified** | Safe terminal restore preserved. |

---

## 3. How to Test in the Colosseum

1. Run the game:
   ```powershell
   cargo run --release
   ```
2. Press `Enter` to enter the Ashen Colosseum.
3. Test Combat & Abilities:
   - Press `F7` at any mouse position to spawn enemies (Sludge $\to$ Skeleton $\to$ Fly $\to$ Worm).
   - Press `F6` to toggle the debug hitbox and height-band visualization.
   - Attack with `LMB` (primary attack/combo) and `RMB` (class secondary).
   - Test `Space` to dodge roll through incoming arrows/vomit (observe i-frames).
   - Switch classes at the Firekeeper (`[E] Speak` $\to$ Class Select) to test all 5 move sets.
   - For Mage: hold `E` near a defeated corpse to siphon 40 mana.
   - Take fatal damage to verify the `YOU DIED` death screen, bonfire respawn, and bloodstain soul retrieval.
