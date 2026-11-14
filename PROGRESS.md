# Atrocitus Implementation Progress & Checklist

Tracking document for all phases of development. Updated after every phase to preserve context and state.

---

## Overall Status
- **Current Phase**: Phase 2 COMPLETED. Ready for Phase 3 (Core Combat + Resources Framework) upon user command.
- **Engine Baseline**: Verified at 120x40, 160x50, 200x60; 60 FPS fixed timestep; dual-layer renderer; panic-safe terminal guard. All 13 unit/integration tests passing. Zero clippy warnings. Release binary compiled.

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
- [ ] **Frame-Data Attack System**
  - [ ] Startup, active, recovery frames, damage, knockback, hit-stop (2-4 frames), screen shake.
  - [ ] Hitbox shapes (arc, cone, circle, projectile) with Height Bands (Low, Mid, High).
  - [ ] Spatial hash for efficient collision queries (up to ~40 live entities).
  - [ ] Roll i-frames and post-damage invulnerability frames.
- [ ] **2.5D Height System**
  - [ ] Ground position $(x, y)$ and height $z$. Flying enemies at High band with ground shadow.
- [ ] **Player Combat HUD**
  - [ ] 5 Hearts with half-heart granularity.
  - [ ] Souls counter (defaults to 100).
  - [ ] Mage-only mana bar.
  - [ ] Status effect icons (Poison, Acid).
  - [ ] Class ability icons with cooldown rings and "↑" High-band indicators.
- [ ] **Souls & Death Loop**
  - [ ] Enemies drop souls; player drops soul bloodstain on death.
  - [ ] "YOU DIED" banner, respawn at last bonfire, retrieve or lose souls.
- [ ] **Corpse Consumption & Status Effects**
  - [ ] Poison DoT and Acid puddle hazard zones.
  - [ ] Mage corpse consumption (hold `E` for 0.8s) to restore mana with particle stream.
- [ ] **5 Class Move Implementations**
  - [ ] Knight: 3-hit combo (Low, Low, High rising slash) + frontal shield block.
  - [ ] Mage: Ember Bolt (fast projectile) + Cinder Nova (charged AoE all bands).
  - [ ] Assassin: 3-hit combo (Low, Low, High uppercut) + Shadowstep blink (3x backstab).
  - [ ] Hunter: Pistol-sword flintlock (ranged High) + blade slash combo.
  - [ ] Alchemist: Arced vial throw (acid/fire DoT) + Eye Rite slowing arcane zone.
- [ ] **Combat Debug Tools**
  - [ ] `F6`: Toggle hitboxes, hurtboxes, and height band overlays.
  - [ ] `F7`: Enemy spawner at mouse cursor.

---

### Phase 4: The Dilapidated Forest (First Area) + Enemies
- [ ] **Area Transition & Structure (`data/areas/forest.ron`)**
  - [ ] Gate of Trials in Colosseum -> Loading screen -> Dilapidated Forest.
  - [ ] Bonfire checkpoint at entrance, winding path with 5 encounter zones, clearing bonfire, boss fog gate.
  - [ ] Map (`M`) new region reveal with fog-of-war.
- [ ] **Forest Environment Art**
  - [ ] Dead twisted trees, rotted logs, mossy walls, broken wagon, murky puddles, hanging roots.
  - [ ] Glowing fungus / mushrooms as light sources, falling leaves, fireflies, green-teal ambient.
- [ ] **4 AI Enemy Implementations (`data/enemies.ron`)**
  - [ ] Sludge: Acid vomit cone (3 hearts) + acid puddle DoT.
  - [ ] Skeleton: Kiting archer with telegraphed green glow shot + Poison effect.
  - [ ] Fly: Erratic High-band flight; dive-bite; immune to Low/ground attacks.
  - [ ] Worm: Swarming fast crawler (6-12 count flocking), bite damage with immunity window.
- [ ] **Zone Encounters & Respawn Logic**
  - [ ] Zone 1 (worms) -> Zone 2 (sludge) -> Zone 3 (skeletons) -> Zone 4 (flies) -> Zone 5 (clearing guard).
  - [ ] Bonfire rest respawns all area mobs.

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

