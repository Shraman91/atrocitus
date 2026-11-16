# Atrocitus Implementation Progress & Checklist

Tracking document for all phases of development. Updated after every phase to preserve context and state.

---

## Overall Status
- **Current Phase**: Phase 4 COMPLETED (First Area: The Dilapidated Forest & The Dread Keep with 4 Parts connected by Shimmering Fog Walls). Ready for Phase 5 (Polish & Balance / Boss Encounter) upon user command.
- **Engine Baseline**: Verified at 120x40, 160x50, 200x60; 60 FPS fixed timestep; dual-layer renderer; panic-safe terminal guard. All 25 unit/integration tests passing. Zero clippy warnings. Release binary compiled.

---

## Phase Checklist

### Phase 1: Improve the Starting Area ("The Ashen Colosseum")
- [x] **Data-Driven Configuration (`data/colosseum.ron`)**
  - [x] Externalize arena dimensions, light sources, torch positions, prop layouts, and dialogue strings.
  - [x] Serde loader in `src/game/world.rs` so tuning does not require recompiling.
- [x] **Richer Arena Ground & Tile Art**
  - [x] Layered stone textures with 3-4 tone ramps and hue-shifted shadows (cool) and warm highlights.
  - [x] Cracked flagstones, sand patches, bone piles, rubble, moss growth, puddles reflecting firelight.
- [x] **Raised Central Dais & Bonfire Architecture**
  - [x] Raised stone dais with carved concentric steps leading to the bonfire.
  - [x] Coiled sword sprite, animated flame, rising ember particles, and warm light pool.
- [x] **Expanded Colosseum Prop & Structural Set**
  - [x] 2nd tier of stands with arched facades and spectator silhouettes.
  - [x] Wall torches acting as secondary warm light sources.
  - [x] Statues of headless gladiators, weapon racks, and iron cages.
  - [x] Broken columns, fallen arches, and swaying tattered banners.
- [x] **Walkable Layout & Lighting Pass**
  - [x] Ensure generous walking paths around all props; no collision traps.
  - [x] Clearly lit standing spots and interaction room for all 4 NPCs.
  - [x] Visible paved path to the Gate of Trials with `[E] Examine Gate` / `[E] Enter` prompt.
  - [x] Ambient light floor ($\ge 35\%$), soft blue-moonlight, radial bonfire light with harmonic flicker, per-torch lights, light shafts through broken roof, drifting ash & fog. No pitch-black zones.
- [x] **Control Scheme Alignment**
  - [x] Set `Space` as the only dodge roll key (unbind RMB from roll).
  - [x] Reserve `LMB` for primary attack and `RMB` for class secondary.
  - [x] Update README, HUD controls hint, and pause menu text.
- [x] **Verification**
  - [x] Tested and verified at 120x40, 160x50, 200x60.
  - [x] `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --release`.
  - [x] Steady 60 FPS performance.

---

### Phase 2: Character, NPC and Enemy Sprite Quality Pass
- [x] **Player Sprite System**
  - [x] Adventurer base (travel clothes, backpack, scarf, 1px dark outline, cool rim highlight) in 8 directions (4 bases + flip_x) at 18x30 proportions.
  - [x] Idle (4-frame breathing cycle), walk (6-frame cycle), roll (7-frame dedicated cycle with smear/dust).
- [x] **Class Overlays & Animation Frames**
  - [x] Mage (witch hat, twilight celestial robe, cyan glowing orb staff), Knight (winged plate, tabard, cape, shield, longsword), Assassin (hooded cowl, red silk sash, dual daggers), Hunter (tricorn hat, buff duster, flintlock pistol-swords), Alchemist (plague cowl, goggles, belt vials, floating arcane eye).
  - [x] Attack frames for LMB and RMB (4 action frames per direction).
  - [x] Hit reaction frame and death frame for each class gear set.
- [x] **NPC Quality Pass**
  - [x] Firekeeper (veiled, swayed iron lantern with pulsing ember, 4-frame idle).
  - [x] Blacksmith (broad, apron, hammer, anvil, strike spark burst and recoil smoke, 4-frame idle).
  - [x] Unknown Adventurer (hooded, masked, blade sharpening with 4-point glint flash, 4-frame idle).
  - [x] Shady Witch (hunched, crooked hat, bubbling flask with sparkle pop, 4-frame idle).
- [x] **Enemy Sprite Set (Phase 3 Preparation)**
  - [x] Sludge (~26px blobby acid ooze with squash/stretch idle, squelch walk, swelling vomit wind-up, spit frame, hit, death melt).
  - [x] Skeleton (16x25 bogged archer, `src/assets/enemy_sprites.rs`: 0 = idle with bow, 1 = walk, 2 = bow drawn with telegraph poison arrow).
  - [x] Fly (12x9 carrion fly, `src/assets/enemy_sprites.rs`: 0 = wings up, 1 = wings down with ground shadow).
  - [x] Worm (12x8 segmented horror grub, `src/assets/enemy_sprites.rs`: 3-frame undulating wiggle cycle).
  - [x] Added `CharSprite` system with palette mapping and emissive pixel flags (`e` and `v`) for dark glowing eyes and poison tips.
- [x] **Sprite Gallery (`F4`) & Controls Polish**
  - [x] Fix sprite visibility in Gallery: punch out UI cells over stage rect with `ui.clear_rect()`, rendering half-block TrueColor sprites directly over a stone floor on `Framebuffer`.
  - [x] Fix Class Select preview visibility: punch out UI cells over card interior so rotating class character preview renders cleanly without UI box interference.
  - [x] Fix Windows keyboard release lag: implement `GetAsyncKeyState` physical key release query so WASD stops instantly (0ms hang instead of 500ms timeout).
  - [x] Fix mouse world coordinates: accurately project screen mouse cursor into world space using camera offset (`input.mouse_world_at()`) for pixel-accurate aiming, rolling, and NPC clicks.
  - [x] Snappy movement: increased walk speed to 56.0 and run to 86.0; reduced acceleration to 0.07s and stop time to 0.04s.
  - [x] Responsive dodge roll: duration 0.38s, cooldown 0.48s, with 0.15s input buffering window.
  - [x] Added `test_gallery_and_class_select_sprites_not_obscured_by_ui` integration test. All 14 tests passing.

---

### Phase 3: Core Combat + Resources Framework
- [x] **Frame-Data Attack System**
  - [x] Startup, active, recovery frames, damage, knockback, hit-stop (2-4 frames), trauma-based screen shake.
  - [x] Hitbox shapes (Arc cleave, Circle AoE, AABB, Projectile) with Height Bands (Low, Mid, High, All).
  - [x] Spatial hash grid (32px cells) for zero-allocation broadphase collision queries.
  - [x] Roll invulnerability frames (frames 2 through 14 of 23-frame roll) and post-damage i-frames with sprite blink.
- [x] **2.5D Height System**
  - [x] Ground position $(x, y)$ and height $z$. Flying enemies (Carrion Fly) at High band with ground shadow.
  - [x] Strict height-band collision checks: Low-only attacks cannot hit High flying enemies; High attacks hit Mid/High; All hits all.
- [x] **Player Combat HUD & Resources**
  - [x] 5 Hearts with half-heart granularity (10 HP total) in UI layer with damage white-flash.
  - [x] Souls counter (`✦ 100 SOULS`).
  - [x] Mage-only mana bar (`MANA [████████░░░░] 60/100`) with regen.
  - [x] Status effect indicators: Poison DoT (`[☣ POISON 4.2s]`), Acid burn (`[⚡ ACID 2.8s]`).
  - [x] Class ability HUD with move descriptions and `[↑High]` height band tags.
- [x] **Souls & Death Loop**
  - [x] "YOU DIED" death screen and bloodstain dropping at player death location.
  - [x] Bonfire respawn with full HP and cleared status effects; retrieving bloodstain restores souls with `+X SOULS RETRIEVED` banner.
  - [x] Defeated enemies yield souls with floating "+X" text and leave persistent corpses.
- [x] **Corpse Consumption & Hazard Zones**
  - [x] Poison DoT and Acid puddle hazard zones dealing periodic tick damage.
  - [x] Mage corpse consumption: holding `E` for 0.8s channels dark purple siphon stream, consumes corpse, and restores 40 mana.
- [x] **5 Playable Class Move Sets**
  - [x] Knight: 3-hit combo (Low, Mid, High rising slash) + frontal shield block (`RMB`, 80% damage reduction, staggered attacker).
  - [x] Mage: Ember Bolt (`LMB`, Mid band, 5 mana) + Cinder Nova (`RMB`, radial explosion, All bands, 25 mana) + Corpse Siphon (`E`).
  - [x] Assassin: Rapid 3-hit dagger flurry (`LMB`) + Shadowstep (`RMB`, 45px teleport-dash with 3x backstab crit window).
  - [x] Hunter: Twin blade sweep (`LMB`, wide arc Mid band) + Flintlock Shot (`RMB`, silver bullet High band, hits flying enemies, recoil).
  - [x] Alchemist: Acid Vial Throw (`LMB`, lobbed arc, creates Low acid puddle for 3.5s) + Eye Rite (`RMB`, slows enemies by 50% for 4s).
- [x] **Combat Debug Tools**
  - [x] `F6`: Toggle hitbox, hurtbox, and height-band visualization (green hurtbox, blue High, yellow Low, red active attacks).
  - [x] `F7`: Enemy spawner at cursor position cycling Sludge -> Skeleton -> Fly -> Worm.
- [x] **Data-Driven Configuration (`data/combat.ron`)**
  - [x] All player HP, mana, regen, class move damages, reaches, cooldowns, and enemy stats externalized with Ron deserialization and code fallbacks.

---

### Phase 4: The Dilapidated Forest & The Dread Keep (First Area)
- [x] **4-Part Gothic Area Architecture (`data/areas/forest.ron`)**
  - [x] Externalized data configuration for all 4 parts, ambient lighting colors, dimensions, bonfires, player spawn coordinates, prop lists, and enemy spawn tables.
  - [x] Serde loading with code fallback in `src/game/world.rs`.
- [x] **Part 1: The Desecrated Chapel**
  - [x] Gothic ruined chapel nave with cracked flagstones and crumbling arched walls.
  - [x] Scattered rotting corpses (siphonable by Mage for +40 mana), ancient carved pews, gothic altar, grave stones, and broken chapel arch.
  - [x] Mobs: Corpse Worms (burrowing vermin) and Carrion Flies (buzzing High-band swarmers).
  - [x] South Gate returning to Ashen Colosseum hub; forward Fog Wall leading to Autumn Woods.
  - [x] Checkpoint bonfire at the chapel threshold.
- [x] **Part 2: The Autumnal Woods**
  - [x] Transition zone into the woods with a lighter fall/autumn atmosphere (amber/rust ground foliage, golden birch trees).
  - [x] Fallen logs, broken peasant carts, weathered split-rail wooden fences.
  - [x] Fluttering autumn leaves ambient particle system (`spawn_leaf`).
  - [x] Mobs: Acid Sludges (corrosive ooze blobs) and Bogged Skeletons (kiting archers drawing poisoned arrows).
  - [x] Shimmering Fog Walls connecting back to Chapel and forward into Deep Forest.
- [x] **Part 3: The Gloomwood Deep**
  - [x] Dense, thick, oppressive forest with dark teal peat bogs and twisted canopy.
  - [x] Massive gnarled trees, tangled root briars, rotten hollow stumps.
  - [x] Animated bioluminescent glowing mushrooms (teal/cyan caps) as natural light sources.
  - [x] Drifting glowing spore particles (`spawn_spore`).
  - [x] All 4 mobs active in dense ambush encounters (Worms, Flies, Sludges, Skeletons).
  - [x] Shimmering Fog Walls connecting back to Autumn Woods and forward into Castle Keep.
- [x] **Part 4: The Dread Keep (Castle Boss Area)**
  - [x] Imposing gothic castle courtyard with obsidian flagstone flooring and towering battlements.
  - [x] Massive gothic castle pillars, grotesque gargoyle statues, animated iron braziers with violet-amber fire, and tattered wall banners.
  - [x] **Big elevated circular stage at the center** with concentric stone steps and heraldic runes, ready for boss encounters.
  - [x] Elite castle guard sentinels stationed on and around the perimeter of the stage.
  - [x] Return Fog Wall back into Deep Forest.
- [x] **Dark Souls-Style Shimmering Fog Walls**
  - [x] Rendered with undulating mist currents and soft golden-teal ethereal luminescence.
  - [x] Proximity detection and mouse hover interaction: `[Click / E] Traverse Fog Wall to <Destination>`.
  - [x] Smooth traversal transitions between connected parts with positioning at designated spawn points.
- [x] **Full Combat & Resource Integration**
  - [x] Bonfire checkpoint rest restores HP/mana and respawns area mobs.
  - [x] All class movesets (Knight 3-hit combo & block, Mage Bolt/Nova/Corpse siphon, Assassin flurry/shadowstep, Hunter twin blade/flintlock, Alchemist vial/eye) fully functional and dealing damage against all mobs.
  - [x] Souls accumulation and bloodstain dropping/retrieval on death.
  - [x] Map (`M`) tracking world coordinates across all 4 parts.
- [x] **Verification & Tests**
  - [x] Dedicated unit/integration test `test_area1_forest_four_parts_and_fog_walls` verifying all 4 parts, props, enemy tables, terrain generation, and multi-resolution rendering.
  - [x] All 25 tests pass. Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
  - [x] Release build compiled successfully (`cargo build --release`).

---

### Phase 5: Polish & Balance
- [ ] **Balance Pass via Data Files**
  - [ ] Tune speeds, damages, health values, soul rewards, cooldowns.
- [ ] **Tutorial & Feedback Systems**
  - [ ] Contextual dismissible tooltips (roll, block, corpse consume, high attacks).
  - [ ] Audio hook stubs.
  - [ ] Full game save/load (class, souls, bonfire checkpoint, explored map).
- [ ] **Documentation & Final Delivery**
  - [ ] Comprehensive README with controls and data file guide.
  - [ ] Final changelog and future roadmap.

