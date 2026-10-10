# Atrocitus

> A dark-fantasy, souls-like action RPG built in Rust with a custom TrueColor terminal rendering engine.

---

## ⚔️ Overview

**Atrocitus** delivers a rich, atmospheric dark-fantasy experience rendered entirely in the terminal using ANSI 24-bit TrueColor and high-density half-block pixel graphics (`▀`), achieving double the vertical resolution of standard terminal cells.

This initial release introduces:
1. **Title Screen**: Ruined gothic spire silhouettes with slow parallax, ember particle storms, and glowing atmospheric menus.
2. **Staged Loading Screen**: Real staged asset generation, rotating dark-fantasy lore lines, and an animated flickering bonfire progress bar.
3. **The Ashen Sanctum (Lobby Hub)**: An interactive 2.5D third-person hub featuring:
   - Controllable knight with 8-direction movement, footstep dust, and walk/run animations.
   - Central animated Bonfire with coiled sword, dynamic radial point lighting, soft ground shadows, and interactive resting sequence.
   - Atmospheric post-processing: drifting ground fog, film grain, and edge vignette.
   - Depth-sorted environment props (broken arches, stone pillars, gravestones, swords in ground, dead trees, and a fog gate).
   - NPC silhouettes (Blacksmith with anvil and hooded Firekeeper) with proximity `[E]` talk interactions.

---

## 🕹️ Controls

| Key | Action |
|---|---|
| **W, A, S, D** or **Arrow Keys** | 8-Directional Movement / Menu Navigation |
| **Shift** (Hold) | Run (Speed boost) |
| **E** | Interact (Rest at Bonfire / Talk to NPCs / Inspect Fog Gate) |
| **Enter / Space** | Confirm / Select Menu Option |
| **Esc** | Pause Game / Return / Back |
| **F3** | Toggle Debug Overlay (Coordinates, Camera, Facing direction) |
| **F1** or **H** | Toggle HUD Controls Hint |

---

## ⚙️ Terminal Requirements

- **Color Support**: 24-bit TrueColor terminal (`COLORTERM=truecolor` or modern emulator).
  - Recommended terminals: **Windows Terminal**, **WezTerm**, **Alacritty**, **Ghostty**, **iTerm2**, or **VSCode Terminal**.
- **Minimum Resolution**: **120 columns × 40 rows** (corresponds to a **120×80** logical pixel canvas).
  - *If your window is smaller, a live warning screen prompts you to enlarge the window, resuming automatically once resized.*

---

## 🚀 Building & Running

### Prerequisites
- Rust stable (2024 edition or newer)
- Cargo

### Run in Release Mode (Recommended for optimal 60 FPS performance)
```bash
cargo run --release
```

### Run Tests & Verification
```bash
cargo test
cargo clippy
```

---

## 🎨 Swapping in Custom PNG Art

Atrocitus includes a procedural pixel-art engine, but also supports loading standard PNG images from the `assets/` directory without changing game logic:

1. Place your PNG file inside `assets/` (e.g., `assets/custom_knight.png`).
2. Load and draw it in code using:
```rust
use crate::assets::Sprite;

let sprite = Sprite::load_png("assets/custom_knight.png")?;
sprite.draw(&mut framebuffer, screen_x, screen_y, false);
```

---

## 🏛️ Architecture

```
src/
├── main.rs                 # Game loop, TrueColor verification, resize handling
├── engine/
│   ├── color.rs            # 8-bit RGBA color math, blending, lerp, and ANSI writing
│   ├── framebuffer.rs      # 2D logical pixel grid & drawing primitives
│   ├── renderer.rs         # Diff-based terminal renderer using half-blocks ('▀')
│   ├── input.rs            # Kitty keyboard enhancement protocol + fallback held tracking
│   ├── timing.rs           # 60 Hz fixed-timestep clock & FPS monitor
│   ├── lighting.rs         # Dynamic 2.5D point lights, falloff & projected ground shadows
│   ├── postprocess.rs      # Vignette, drifting fog simulation, and film grain
│   └── text.rs             # 3x5 and 5x7 bitmap pixel font rasterizers
├── assets/
│   ├── palette.rs          # Ashen dark-fantasy Souls-like color palette
│   ├── font_data.rs        # Bitmap glyph lookup tables
│   └── sprites.rs          # Procedural sprite bank & PNG converter
├── game/
│   ├── camera.rs           # Smooth 2.5D follow camera with boundary clamping
│   ├── world.rs            # Sanctum plateau terrain, colliders & props
│   ├── player.rs           # 8-direction knight controller & animation state
│   ├── bonfire.rs          # Central bonfire entity, point light & rest banner
│   ├── npc.rs              # Blacksmith & Firekeeper NPCs with dialogue bubbles
│   └── particles.rs        # Zero-allocation particle pool (embers, ash, dust)
└── scenes/
    ├── mod.rs              # Scene trait & cross-fade state machine
    ├── title.rs            # Title screen with parallax spires & logo glow
    ├── loading.rs          # Staged initialization, progress bar & lore tips
    └── lobby.rs            # The Ashen Sanctum interactive hub scene
```

---

## 🔮 Roadmap / Next Steps

- **Combat System**: Light/heavy attacks, weapon swings with trail particles, stamina management, and hitboxes.
- **Enemies & AI**: Wandering hollows, aggro ranges, attack windups, and poise break mechanics.
- **Inventory & Equipment**: Weapons, armor sets, Estus flask healing item, and consumable souls.
- **World Expansion**: Transition through the Fog Gate into procedural or tilemap dungeon zones (e.g. *The Sunken Catacombs*, *High Wall of Loth*).
- **Save / Load**: Serialization of player progress, unlocked shortcuts, and resting bonfire checkpoints via `serde`.
