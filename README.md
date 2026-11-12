# Atrocitus

> A dark-fantasy, souls-like action RPG built in Rust with a custom dual-layer TrueColor terminal rendering engine.

---

## ⚔️ Overview

**Atrocitus** delivers a rich, atmospheric dark-fantasy experience rendered entirely in the terminal using ANSI 24-bit TrueColor. It features a unique **dual-layer rendering architecture**:
1. **Half-Block Pixel Layer (`▀`)**: Operates at double vertical resolution ($W \times 2H$) for rich 2.5D graphics, dynamic lighting, contact shadow ellipses, layered sprites, and atmospheric post-processing.
2. **Real Monospace Character UI Layer (`UiCanvas`)**: Draws crisp, native 1-cell monospace terminal characters over the pixel canvas with box-drawing borders (`─ │ ┌ ┐ └ ┘`), `unicode-width` word-wrapping, padding, and 2-cell screen anchoring.

### Key Systems
- **Fluid WASD Movement & Mouse Aiming**: WASD provides smooth acceleration ($\sim 0.12\text{s}$) and deceleration ($\sim 0.08\text{s}$) with separate X/Y wall collision sliding. The mouse continuously aims player facing with 8-direction hysteresis buffer.
- **Eased Dodge Roll**: Right-Click or Space triggers an ease-out roll toward the mouse cursor covering $\sim 65\text{px}$ over $0.45\text{s}$, with invulnerability glow, motion blur ghost trails, $0.8\text{s}$ cooldown, and input buffering during the final $100\text{ms}$.
- **The Ashen Colosseum Hub & 4 Interactive NPCs**:
  - **The Veiled Firekeeper**: Choose your class / mantle (Knight, Mage, Assassin, Hunter, Alchemist) with warm amber aura.
  - **Blacksmith Valen**: Forging & upgrade station with hammer animation and glowing forge coals.
  - **Unknown Adventurer**: Cryptic world lore and warnings with teal moonlight.
  - **Shady Witch Morgana**: Arcane shop selling spells and dark elixirs with violet cauldron light.
- **Overlays & Progression**:
  - **Map (`M`)**: Real-time fog-of-war tracking, entity markers, and directional player arrow.
  - **Backpack (`B`)**: 5×4 inventory grid, equipped gear sidebar, item rarities, tooltips, and `F5` debug loot.
  - **Spellbook (`K`)**: Arcane codex with class incantations.
  - **Sprite Gallery (`F4`)**: Fullscreen 1x and 3x scale viewer with direction and animation toggles.
  - **Layout Debug Overlay (`F2`)**: Visualizes all UI layout bounding boxes with colored bracket corners.

---

## 🕹️ Controls

| Input | Action |
|---|---|
| **W, A, S, D** | Smooth Character Movement (with X/Y wall sliding) |
| **Mouse Cursor** | Aim Facing Direction (Continuous 8-directional hysteresis) |
| **Space** / **Right-Click** | Eased Dodge Roll toward Cursor (i-frame glow + ghost trails) |
| **E** / **Left-Click** | Interact with NPCs, Bonfire, or examine Gate |
| **M** | Toggle Colosseum Map (Fog-of-war & entity radar) |
| **B** | Toggle Backpack / Inventory (5×4 grid & gear inspection) |
| **K** | Toggle Spellbook (Mage incantations & catalysts) |
| **F1** or **H** | Toggle Controls Hint HUD |
| **F2** | Toggle UI Layout Debug Bounding Boxes |
| **F3** | Toggle Debug Overlay (Coordinates, Camera, Colliders) |
| **F4** | Open Fullscreen Sprite Gallery (1x & 3x scale viewer) |
| **F5** | Debug: Populate Backpack with 8 Test Items |
| **F9** | Debug: Reset Class / Mantle |
| **Esc** | Pause Menu / Back / Close Overlays |

---

## ⚙️ Terminal Requirements

- **Color Support**: 24-bit TrueColor terminal (`COLORTERM=truecolor` or modern emulator).
  - Recommended terminals: **Windows Terminal**, **WezTerm**, **Alacritty**, **Ghostty**, **iTerm2**, or **VSCode Terminal**.
- **Minimum Resolution**: **120 columns × 40 rows** (corresponds to a **120×80** logical pixel canvas).
  - *If your window is smaller, a live warning screen prompts you to enlarge the window, resuming automatically once resized.*
  - *Fully tested and optimized for 120×40, 160×50, and 200×60.*

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
cargo clippy --all-targets -- -D warnings
```

---

## 🏛️ Architecture

```
src/
├── main.rs                 # 60Hz loop, Kitty protocol + repeat fallback, dual-layer rendering
├── ui/
│   ├── mod.rs              # UI subsystem exports
│   ├── layout.rs           # Rect, Margin, Padding, word_wrap, text_width, truncation
│   └── canvas.rs           # UiCanvas, UiCell, border styles, debug rect overlays
├── engine/
│   ├── color.rs            # 8-bit RGBA color math, blending, lerp, and ANSI writing
│   ├── framebuffer.rs      # 2D logical pixel grid, half-block primitives, draw_ellipse
│   ├── renderer.rs         # DiffRenderer composing pixel Framebuffer and UiCanvas text
│   ├── input.rs            # Mouse tracking, held key timing, and VirtualKey mappings
│   ├── timing.rs           # 60 Hz fixed-timestep clock & FPS monitor
│   ├── lighting.rs         # 2.5D radial point lights, soft shadows & elevated light floor
│   ├── postprocess.rs      # Drifting ground fog simulation, film grain, and vignette
│   └── text.rs             # 3x5 and 5x7 bitmap pixel font rasterizers
├── assets/
│   ├── palette.rs          # Ashen dark-fantasy Souls-like color palette
│   ├── font_data.rs        # Bitmap glyph lookup tables
│   └── sprites.rs          # Layered character gear, dark outlines, rim lighting & animations
├── game/
│   ├── camera.rs           # Smooth 2.5D follow camera with boundary clamping
│   ├── classes.rs          # Class attributes, gear loadouts & RON save persistence
│   ├── equipment.rs        # Layered equipment compositing for character sprites
│   ├── inventory.rs        # 5x4 grid inventory system, rarities, tooltips & F5 debug items
│   ├── map.rs              # MapGrid fog-of-war discovery system
│   ├── npc.rs              # 4 Colosseum NPCs (Firekeeper, Blacksmith, Adventurer, Witch)
│   ├── particles.rs        # Zero-allocation particle pool (embers, ash, dust)
│   ├── player.rs           # Velocity movement, wall sliding, mouse aim, eased dodge roll
│   └── world.rs            # Colosseum arena ground, parallax sky, colliders & props
└── scenes/
    ├── mod.rs              # Scene trait & cross-fade state machine + resolution tests
    ├── title.rs            # Centered 41-char block logo, native UI menus & settings modal
    ├── loading.rs          # Asset preload, real-char progress bar & rotating lore tips
    ├── lobby.rs            # The Ashen Colosseum hub with all 4 NPCs & ambient effects
    ├── class_select.rs     # Responsive 5-column class cards & mantle selection
    ├── dialogue.rs         # Firekeeper dialogue with word-wrapping and choice prompt
    ├── blacksmith.rs       # Blacksmith forge & upgrade station
    ├── adventurer_dialogue.rs # Unknown Adventurer cryptic lore dialogue
    ├── witch_shop.rs       # Shady Witch arcane shop & dark elixirs
    ├── map.rs              # Fullscreen arena map with fog-of-war & entity markers
    ├── backpack.rs         # 5x4 grid inventory overlay with paper-doll equipment
    ├── spellbook.rs        # Mage spellbook codex (mapped to 'K')
    └── gallery.rs          # Fullscreen 1x and 3x sprite gallery viewer ('F4')
```
