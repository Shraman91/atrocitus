use crate::assets::palette::Palette;
use crate::assets::sprites::{Sprite, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::lighting::{PointLight, ShadowCaster};
use crate::engine::pathfinding::NavGrid;

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Collider {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn intersects(&self, other: &Collider) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }
}

pub struct PropInstance {
    pub x: i32,
    pub y: i32,
    pub base_y: i32,
    pub sprite_index: PropType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PropType {
    // Colosseum Props
    ColosseumArch,
    BrokenColumn,
    GateOfTrials,
    WeaponRack,
    TrainingDummy,
    SpectatorStand,
    TatteredBanner,
    BonePile,
    HeadlessGladiator,
    HangingCage,
    WallTorch,
    FallenArchRubble,

    // First Area Props (Part 1: Chapel)
    ChapelAltar,
    ChapelPew,
    GraveStone,
    RottingCorpse,
    ChapelArch,

    // First Area Props (Part 2: Autumn Woods)
    AutumnTree,
    FallenLog,
    BrokenCart,
    WoodenFence,

    // First Area Props (Part 3: Deep Forest)
    GnarledTree,
    GlowingMushrooms,
    RootBriar,
    RottenStump,

    // First Area Props (Part 4: Castle Boss Area)
    CastlePillar,
    GargoyleStatue,
    CastleBrazier,
    CastleBanner,

    // Fog Wall
    FogWallGate,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TorchConfig {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub intensity: f32,
    pub color: (u8, u8, u8),
    pub flicker: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightShaftConfig {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub width: f32,
    pub alpha: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PropConfig {
    pub x: i32,
    pub y: i32,
    pub prop_type: PropType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColosseumConfig {
    pub width: usize,
    pub height: usize,
    pub ambient_color: (u8, u8, u8),
    pub dais_radius_x: f32,
    pub dais_radius_y: f32,
    pub step_count: usize,
    pub gate_x: f32,
    pub gate_y: f32,
    pub torches: Vec<TorchConfig>,
    pub light_shafts: Vec<LightShaftConfig>,
    pub props: Vec<PropConfig>,
}

impl ColosseumConfig {
    pub fn load() -> Self {
        if let Some(cfg) = std::fs::read_to_string("data/colosseum.ron")
            .ok()
            .and_then(|content| ron::from_str::<ColosseumConfig>(&content).ok())
        {
            return cfg;
        }
        Self::fallback_default()
    }

    pub fn fallback_default() -> Self {
        Self {
            width: 280,
            height: 190,
            ambient_color: (44, 50, 68),
            dais_radius_x: 24.0,
            dais_radius_y: 16.0,
            step_count: 3,
            gate_x: 124.0,
            gate_y: 18.0,
            torches: vec![
                TorchConfig {
                    x: 104.0,
                    y: 40.0,
                    radius: 46.0,
                    intensity: 0.85,
                    color: (255, 160, 45),
                    flicker: 0.10,
                },
                TorchConfig {
                    x: 176.0,
                    y: 40.0,
                    radius: 46.0,
                    intensity: 0.85,
                    color: (255, 160, 45),
                    flicker: 0.10,
                },
            ],
            light_shafts: vec![
                LightShaftConfig {
                    start_x: 75.0,
                    start_y: 0.0,
                    end_x: 110.0,
                    end_y: 130.0,
                    width: 22.0,
                    alpha: 24,
                },
                LightShaftConfig {
                    start_x: 155.0,
                    start_y: 0.0,
                    end_x: 185.0,
                    end_y: 140.0,
                    width: 26.0,
                    alpha: 28,
                },
            ],
            props: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ForestPartId {
    Chapel,
    AutumnWoods,
    DeepForest,
    CastleKeep,
}

impl ForestPartId {
    pub fn display_title(&self) -> &'static str {
        match self {
            ForestPartId::Chapel => "THE DESECRATED CHAPEL",
            ForestPartId::AutumnWoods => "THE AUTUMNAL WOODS",
            ForestPartId::DeepForest => "THE GLOOMWOOD DEEP",
            ForestPartId::CastleKeep => "THE DREAD KEEP",
        }
    }

    pub fn next(&self) -> Option<ForestPartId> {
        match self {
            ForestPartId::Chapel => Some(ForestPartId::AutumnWoods),
            ForestPartId::AutumnWoods => Some(ForestPartId::DeepForest),
            ForestPartId::DeepForest => Some(ForestPartId::CastleKeep),
            ForestPartId::CastleKeep => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FogWallConfig {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub target_part: ForestPartId,
    pub target_spawn: (f32, f32),
    pub prompt: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnemySpawnConfig {
    pub kind: crate::game::enemy::EnemyKind,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForestPartConfig {
    pub id: ForestPartId,
    pub name: String,
    pub subtitle: String,
    pub width: usize,
    pub height: usize,
    pub ambient_color: (u8, u8, u8),
    pub bonfire: Option<BonfireConfig>,
    pub player_spawn: (f32, f32),
    pub colosseum_gate: Option<(f32, f32)>,
    pub fog_wall: Option<FogWallConfig>,
    pub return_fog_wall: Option<FogWallConfig>,
    pub stage_center: Option<(f32, f32)>,
    pub stage_radius: Option<f32>,
    pub torches: Vec<TorchConfig>,
    pub props: Vec<PropConfig>,
    pub enemy_spawns: Vec<EnemySpawnConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BonfireConfig {
    pub x: f32,
    pub y: f32,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForestAreaConfig {
    pub title: String,
    pub parts: Vec<ForestPartConfig>,
}

impl ForestAreaConfig {
    pub fn load() -> Self {
        if let Some(cfg) = std::fs::read_to_string("data/areas/forest.ron")
            .ok()
            .and_then(|content| ron::from_str::<ForestAreaConfig>(&content).ok())
        {
            return cfg;
        }
        Self::fallback_default()
    }

    pub fn get_part(&self, id: ForestPartId) -> Option<&ForestPartConfig> {
        self.parts.iter().find(|p| p.id == id)
    }

    pub fn fallback_default() -> Self {
        Self {
            title: "The Dilapidated Forest & The Dread Keep".to_string(),
            parts: vec![
                ForestPartConfig {
                    id: ForestPartId::Chapel,
                    name: "The Desecrated Chapel".to_string(),
                    subtitle: "Resting place of forgotten saints, now swarming with carrion and rot".to_string(),
                    width: 240,
                    height: 180,
                    ambient_color: (32, 34, 46),
                    bonfire: Some(BonfireConfig { x: 120.0, y: 142.0, name: "Chapel Courtyard Bonfire".to_string() }),
                    player_spawn: (120.0, 130.0),
                    colosseum_gate: Some((120.0, 168.0)),
                    fog_wall: Some(FogWallConfig {
                        x: 120.0,
                        y: 24.0,
                        width: 34.0,
                        target_part: ForestPartId::AutumnWoods,
                        target_spawn: (130.0, 158.0),
                        prompt: "Traverse Fog Wall to The Autumnal Woods".to_string(),
                    }),
                    return_fog_wall: None,
                    stage_center: None,
                    stage_radius: None,
                    torches: vec![
                        TorchConfig { x: 80.0, y: 52.0, radius: 44.0, intensity: 0.85, color: (255, 150, 40), flicker: 0.08 },
                        TorchConfig { x: 160.0, y: 52.0, radius: 44.0, intensity: 0.85, color: (255, 150, 40), flicker: 0.08 },
                    ],
                    props: vec![],
                    enemy_spawns: vec![],
                },
            ],
        }
    }
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub ground_pixels: Vec<Color>,
    pub props: Vec<PropInstance>,
    pub colliders: Vec<Collider>,
    pub nav_grid: NavGrid,
    pub torch_lights: Vec<PointLight>,
    pub light_shafts: Vec<LightShaftConfig>,
}

impl World {
    pub fn build(width: usize, height: usize) -> Self {
        let config = ColosseumConfig::load();
        let eff_w = config.width.max(width);
        let eff_h = config.height.max(height);

        let mut ground_pixels = vec![Palette::VOID_BLACK; eff_w * eff_h];
        let mut props = Vec::with_capacity(config.props.len() + 16);
        let mut colliders = Vec::with_capacity(config.props.len() + 32);
        let mut nav_grid = NavGrid::new(eff_w as f32, eff_h as f32, 4.0);

        let cx = eff_w as f32 * 0.5;
        let cy = eff_h as f32 * 0.5;
        let arena_rx = eff_w as f32 * 0.44;
        let arena_ry = eff_h as f32 * 0.38;

        // 1. Generate Rich Elliptical Colosseum Floor with multi-tone shading
        for y in 0..eff_h {
            let dy = (y as f32 - cy) / arena_ry;
            let row_offset = y * eff_w;

            for x in 0..eff_w {
                let dx = (x as f32 - cx) / arena_rx;
                let dist2 = dx * dx + dy * dy;

                let col = if dist2 < 0.85 {
                    // Interior Arena Floor
                    let noise = ((x * 19 + y * 37) ^ ((x >> 2) + (y >> 2))) % 79;
                    let blood = ((x * 23 + y * 47) % 97) == 0 && (dist2 > 0.28 && dist2 < 0.72);
                    let flagstone = ((x / 14 + y / 10) % 5 == 0) && (x % 14 > 1 && y % 10 > 1);

                    // Puddles reflecting firelight near perimeter
                    let is_puddle = ((x as i32 - (cx as i32 - 45)).pow(2)
                        + (y as i32 - (cy as i32 + 25)).pow(2) * 2) < 32
                        || ((x as i32 - (cx as i32 + 52)).pow(2)
                            + (y as i32 - (cy as i32 - 18)).pow(2) * 2) < 26;

                    if is_puddle {
                        if (x + y) % 3 == 0 {
                            Palette::PUDDLE_REFLECT
                        } else {
                            Palette::PUDDLE_WATER
                        }
                    } else if blood {
                        Palette::BLOOD_STAIN
                    } else if flagstone {
                        if (x + y) % 3 == 0 {
                            Palette::STONE_WARM_MID
                        } else if (x + y) % 2 == 0 {
                            Palette::CAVEA_STONE_MID
                        } else {
                            Palette::CAVEA_STONE_DARK
                        }
                    } else if noise < 22 {
                        Palette::ARENA_SAND_LIGHT
                    } else if noise < 55 {
                        Palette::ARENA_SAND_MID
                    } else {
                        Palette::ARENA_SAND_DARK
                    }
                } else if dist2 < 0.95 {
                    // Chiseled Roman stone rim with moss growth
                    let is_moss = (x * 7 + y * 13) % 29 == 0;
                    if is_moss {
                        Palette::MOSS_GREEN_MID
                    } else if (x / 8 + y / 8) % 2 == 0 {
                        Palette::CAVEA_STONE_LIGHT
                    } else {
                        Palette::CAVEA_STONE_MID
                    }
                } else if dist2 < 1.05 {
                    // Cavea tiers base shadow (cool blue-black)
                    Palette::STONE_COOL_DARK
                } else {
                    // Outer darkness / spectator void
                    Palette::VOID_BLACK
                };

                ground_pixels[row_offset + x] = col;
            }
        }

        // 2. Visible Paved Cobblestone Path from Bonfire North to Gate of Trials
        let path_half_w = 7;
        let gate_top_y = config.gate_y as usize;
        let bonfire_y = cy as usize;
        for y in gate_top_y..=bonfire_y {
            let row_offset = y * eff_w;
            for x in (cx as usize - path_half_w)..=(cx as usize + path_half_w) {
                let dist_from_center = (x as i32 - cx as i32).abs();
                if dist_from_center == path_half_w as i32 {
                    // Curb stones with warm highlight
                    ground_pixels[row_offset + x] = Palette::STONE_WARM_LIGHT;
                } else if dist_from_center == path_half_w as i32 - 1 {
                    ground_pixels[row_offset + x] = Palette::STONE_WARM_MID;
                } else {
                    // Paved dark flagstone interior
                    ground_pixels[row_offset + x] = if (x + y) % 2 == 0 {
                        Palette::CAVEA_STONE_MID
                    } else {
                        Palette::CAVEA_STONE_DARK
                    };
                }
            }
        }

        // 3. Central Raised Stone Dais with 3 Concentric Steps
        let dais_rx = config.dais_radius_x;
        let dais_ry = config.dais_radius_y;
        let dais_min_y = (cy - dais_ry - 4.0).max(0.0) as usize;
        let dais_max_y = (cy + dais_ry + 4.0).min(eff_h as f32 - 1.0) as usize;
        let dais_min_x = (cx - dais_rx - 4.0).max(0.0) as usize;
        let dais_max_x = (cx + dais_rx + 4.0).min(eff_w as f32 - 1.0) as usize;

        for y in dais_min_y..=dais_max_y {
            let dy = (y as f32 - cy) / dais_ry;
            let row_offset = y * eff_w;
            for x in dais_min_x..=dais_max_x {
                let dx = (x as f32 - cx) / dais_rx;
                let d2 = dx * dx + dy * dy;

                if d2 <= 1.05 {
                    let col = if d2 > 0.85 {
                        // Step 3 (Outer step riser)
                        if (x + y) % 2 == 0 {
                            Palette::STONE_WARM_LIGHT
                        } else {
                            Palette::STONE_COOL_DARK
                        }
                    } else if d2 > 0.60 {
                        // Step 2 (Middle tread)
                        if (x + y) % 3 == 0 {
                            Palette::STONE_WARM_MID
                        } else {
                            Palette::CAVEA_STONE_MID
                        }
                    } else if d2 > 0.25 {
                        // Step 1 (Inner dais platform)
                        if (x + y) % 4 == 0 {
                            Palette::STONE_CRACK
                        } else {
                            Palette::CAVEA_STONE_LIGHT
                        }
                    } else {
                        // Center Bonfire Hearth (Charred ashes & cracked slate)
                        if (x + y) % 3 == 0 {
                            Palette::STONE_CRACK
                        } else if (x + y) % 2 == 0 {
                            Palette::STONE_DARK
                        } else {
                            Palette::ASH_WHITE
                        }
                    };
                    ground_pixels[row_offset + x] = col;
                }
            }
        }

        // 4. Perimeter Rim Colliders
        let num_rim_colliders = 28;
        for i in 0..num_rim_colliders {
            let angle = (i as f32 / num_rim_colliders as f32) * std::f32::consts::TAU;
            let rx = cx + angle.cos() * (arena_rx * 0.96);
            let ry = cy + angle.sin() * (arena_ry * 0.96);
            let col = Collider::new(rx - 8.0, ry - 6.0, 16.0, 12.0);
            colliders.push(col);
        }

        // 5. Instantiate Props from Config with non-trapping footprints
        for p in &config.props {
            let (w, _h, base_off, col_w, col_h, has_col) = match p.prop_type {
                PropType::GateOfTrials => (32, 34, 32, 28.0, 12.0, true),
                PropType::ColosseumArch => (32, 36, 34, 24.0, 12.0, true),
                PropType::BrokenColumn => (12, 26, 24, 8.0, 6.0, true),
                PropType::WeaponRack => (18, 20, 18, 14.0, 8.0, true),
                PropType::TrainingDummy => (12, 20, 18, 8.0, 8.0, true),
                PropType::SpectatorStand => (28, 18, 16, 24.0, 10.0, true),
                PropType::TatteredBanner => (14, 30, 28, 4.0, 6.0, true),
                PropType::BonePile => (12, 10, 8, 0.0, 0.0, false), // Walkable!
                PropType::HeadlessGladiator => (16, 28, 26, 10.0, 8.0, true),
                PropType::HangingCage => (14, 24, 20, 8.0, 6.0, true),
                PropType::WallTorch => (8, 14, 12, 0.0, 0.0, false), // Mounted on wall!
                PropType::FallenArchRubble => (20, 12, 10, 16.0, 8.0, true),
                // First Area (Chapel)
                PropType::ChapelAltar => (30, 22, 20, 24.0, 10.0, true),
                PropType::ChapelPew => (24, 16, 14, 20.0, 8.0, true),
                PropType::GraveStone => (16, 22, 20, 12.0, 6.0, true),
                PropType::RottingCorpse => (24, 14, 12, 0.0, 0.0, false), // Walkable!
                PropType::ChapelArch => (32, 38, 36, 26.0, 10.0, true),
                // First Area (Autumn Woods)
                PropType::AutumnTree => (34, 42, 40, 10.0, 8.0, true),
                PropType::FallenLog => (26, 14, 12, 22.0, 8.0, true),
                PropType::BrokenCart => (26, 20, 18, 20.0, 10.0, true),
                PropType::WoodenFence => (22, 16, 14, 20.0, 4.0, true),
                // First Area (Deep Forest)
                PropType::GnarledTree => (38, 46, 44, 14.0, 10.0, true),
                PropType::GlowingMushrooms => (20, 18, 16, 0.0, 0.0, false), // Walkable!
                PropType::RootBriar => (24, 18, 16, 18.0, 8.0, true),
                PropType::RottenStump => (18, 16, 14, 14.0, 8.0, true),
                // First Area (Castle Boss Area)
                PropType::CastlePillar => (20, 40, 38, 16.0, 10.0, true),
                PropType::GargoyleStatue => (20, 28, 26, 14.0, 8.0, true),
                PropType::CastleBrazier => (16, 26, 24, 10.0, 8.0, true),
                PropType::CastleBanner => (14, 34, 32, 0.0, 0.0, false),
                PropType::FogWallGate => (38, 36, 34, 0.0, 0.0, false),
            };

            let base_y = p.y + base_off;
            props.push(PropInstance {
                x: p.x,
                y: p.y,
                base_y,
                sprite_index: p.prop_type,
            });

            if has_col {
                let cx_box = p.x as f32 + (w as f32 - col_w) * 0.5;
                let cy_box = (base_y as f32) - col_h * 0.5;
                colliders.push(Collider::new(cx_box, cy_box, col_w, col_h));
            }
        }

        // Build NavGrid blocked cells from all colliders and arena perimeter
        for c in &colliders {
            nav_grid.set_blocked_rect(c.x, c.y, c.x + c.w, c.y + c.h);
        }

        for r in 0..nav_grid.rows {
            let wy = (r as f32 + 0.5) * nav_grid.cell_size;
            let dy = (wy - cy) / arena_ry;
            for c in 0..nav_grid.cols {
                let wx = (c as f32 + 0.5) * nav_grid.cell_size;
                let dx = (wx - cx) / arena_rx;
                if dx * dx + dy * dy >= 0.88 {
                    nav_grid.walkable[r * nav_grid.cols + c] = false;
                }
            }
        }

        // 6. Convert Torch Config to PointLights
        let mut torch_lights = Vec::with_capacity(config.torches.len());
        for t in &config.torches {
            torch_lights.push(PointLight {
                x: t.x,
                y: t.y,
                radius: t.radius,
                intensity: t.intensity,
                color: Color::rgb(t.color.0, t.color.1, t.color.2),
                flicker_amount: t.flicker,
            });
        }

        Self {
            width: eff_w,
            height: eff_h,
            ground_pixels,
            props,
            colliders,
            nav_grid,
            torch_lights,
            light_shafts: config.light_shafts,
        }
    }

    /// Builds a world area for one of the 4 parts of the Dilapidated Forest / Dread Keep.
    pub fn build_forest_part(part_cfg: &ForestPartConfig) -> Self {
        let eff_w = part_cfg.width;
        let eff_h = part_cfg.height;

        let mut ground_pixels = vec![Palette::VOID_BLACK; eff_w * eff_h];
        let mut props = Vec::with_capacity(part_cfg.props.len() + 16);
        let mut colliders = Vec::with_capacity(part_cfg.props.len() + 32);
        let mut nav_grid = NavGrid::new(eff_w as f32, eff_h as f32, 4.0);

        let cx = eff_w as f32 * 0.5;
        let cy = eff_h as f32 * 0.5;

        // Ground generation per part
        match part_cfg.id {
            ForestPartId::Chapel => {
                // Desecrated Chapel: Ruined gothic stone floor, nave flagstones, blood pools near corpses, mossy graveyard perimeter
                for y in 0..eff_h {
                    let row = y * eff_w;
                    for x in 0..eff_w {
                        let dx = (x as f32 - cx).abs();
                        let dy = (y as f32 - cy).abs();

                        let col = if dx < 75.0 && dy < 72.0 {
                            let flagstone = (x / 12 + y / 8) % 4 == 0 && (x % 12 > 1 && y % 8 > 1);
                            let crack = ((x * 17 + y * 31) % 67) == 0;
                            if crack {
                                Palette::STONE_CRACK
                            } else if flagstone {
                                if (x + y) % 3 == 0 {
                                    Palette::STONE_WARM_MID
                                } else {
                                    Palette::CAVEA_STONE_MID
                                }
                            } else if (x + y) % 2 == 0 {
                                Palette::CAVEA_STONE_DARK
                            } else {
                                Palette::STONE_COOL_DARK
                            }
                        } else if dx < 95.0 && dy < 85.0 {
                            // Overgrown churchyard grounds with moss and dirt
                            let moss = ((x * 19 + y * 23) % 19) == 0;
                            if moss {
                                Palette::MOSS_GREEN_MID
                            } else if (x + y) % 3 == 0 {
                                Palette::WEED_DARK
                            } else {
                                Palette::ARENA_SAND_DARK
                            }
                        } else {
                            // Void boundary
                            Palette::VOID_BLACK
                        };

                        ground_pixels[row + x] = col;
                    }
                }

                // Desecrated nave central aisle
                for y in 35..155 {
                    let row = y * eff_w;
                    for x in (cx as usize - 9)..=(cx as usize + 9) {
                        let dist = (x as i32 - cx as i32).abs();
                        if dist == 9 {
                            ground_pixels[row + x] = Palette::STONE_WARM_LIGHT;
                        } else {
                            ground_pixels[row + x] = if (x + y) % 2 == 0 {
                                Palette::CAVEA_STONE_LIGHT
                            } else {
                                Palette::CAVEA_STONE_MID
                            };
                        }
                    }
                }
            }

            ForestPartId::AutumnWoods => {
                // Autumnal Woods: golden rust, burnt orange leaves carpeted over dark loam, winding trail
                for y in 0..eff_h {
                    let row = y * eff_w;
                    for x in 0..eff_w {
                        let dx = (x as f32 - cx).abs();
                        let dy = (y as f32 - cy).abs();

                        let col = if dx < 105.0 && dy < 82.0 {
                            let leaf_noise = ((x * 23 + y * 41) ^ ((x >> 2) + (y >> 2))) % 97;
                            if leaf_noise < 20 {
                                Color::rgb(225, 135, 42) // golden amber
                            } else if leaf_noise < 45 {
                                Color::rgb(180, 75, 26)  // rust
                            } else if leaf_noise < 65 {
                                Color::rgb(130, 48, 20)  // burnt umber
                            } else if leaf_noise < 80 {
                                Palette::ARENA_SAND_MID  // dry trail soil
                            } else {
                                Palette::ARENA_SAND_DARK
                            }
                        } else {
                            Palette::VOID_BLACK
                        };

                        ground_pixels[row + x] = col;
                    }
                }

                // Winding forest trail through glade
                for y in 25..170 {
                    let trail_cx = (cx + ((y as f32 * 0.08).sin() * 14.0)).round() as usize;
                    let row = y * eff_w;
                    for x in trail_cx.saturating_sub(7)..=(trail_cx + 7).min(eff_w - 1) {
                        let dist = (x as i32 - trail_cx as i32).abs();
                        ground_pixels[row + x] = if dist >= 6 {
                            Color::rgb(140, 60, 25)
                        } else if (x + y) % 2 == 0 {
                            Palette::ARENA_SAND_LIGHT
                        } else {
                            Palette::ARENA_SAND_MID
                        };
                    }
                }
            }

            ForestPartId::DeepForest => {
                // Gloomwood Deep: dark teal-black peat bog, murky stagnant water pools, glowing lichen
                for y in 0..eff_h {
                    let row = y * eff_w;
                    for x in 0..eff_w {
                        let dx = (x as f32 - cx).abs();
                        let dy = (y as f32 - cy).abs();

                        let col = if dx < 118.0 && dy < 88.0 {
                            let peat_noise = ((x * 37 + y * 47) ^ (x >> 1)) % 89;
                            let root = ((x * 11 + y * 7) % 31) == 0;
                            let lichen = ((x * 13 + y * 29) % 43) == 0;

                            // Murky swamp puddles
                            let in_puddle = ((x as i32 - 110).pow(2) + (y as i32 - 130).pow(2) * 2) < 45
                                || ((x as i32 - 175).pow(2) + (y as i32 - 95).pow(2) * 2) < 40;

                            if in_puddle {
                                if (x + y) % 3 == 0 {
                                    Color::rgb(26, 42, 38)
                                } else {
                                    Color::rgb(18, 30, 26)
                                }
                            } else if lichen {
                                Color::rgb(42, 110, 80) // faint bioluminescent glow
                            } else if root {
                                Color::rgb(35, 45, 38)
                            } else if peat_noise < 40 {
                                Color::rgb(18, 24, 22)
                            } else if peat_noise < 75 {
                                Color::rgb(14, 18, 16)
                            } else {
                                Color::rgb(24, 32, 28)
                            }
                        } else {
                            Palette::VOID_BLACK
                        };

                        ground_pixels[row + x] = col;
                    }
                }
            }

            ForestPartId::CastleKeep => {
                // The Dread Keep: Gothic obsidian & slate paving, grand elevated central stage
                for y in 0..eff_h {
                    let row = y * eff_w;
                    for x in 0..eff_w {
                        let dx = (x as f32 - cx).abs();
                        let dy = (y as f32 - cy).abs();

                        let col = if dx < 105.0 && dy < 82.0 {
                            let diamond = (x / 8 + y / 8) % 2 == 0;
                            let flag = x % 16 > 1 && y % 12 > 1;
                            if flag && diamond {
                                Color::rgb(42, 45, 58)
                            } else if flag {
                                Color::rgb(28, 30, 40)
                            } else {
                                Color::rgb(18, 20, 26)
                            }
                        } else {
                            Palette::VOID_BLACK
                        };

                        ground_pixels[row + x] = col;
                    }
                }

                // Grand Central Boss Stage: elevated circular dais with steps and heraldic mosaic
                if let (Some(stage_pos), Some(stage_r)) = (part_cfg.stage_center, part_cfg.stage_radius) {
                    let sx = stage_pos.0;
                    let sy = stage_pos.1;
                    let sr_y = stage_r * 0.72; // perspective flatten

                    let min_y = (sy - sr_y - 4.0).max(0.0) as usize;
                    let max_y = (sy + sr_y + 4.0).min(eff_h as f32 - 1.0) as usize;
                    let min_x = (sx - stage_r - 4.0).max(0.0) as usize;
                    let max_x = (sx + stage_r + 4.0).min(eff_w as f32 - 1.0) as usize;

                    for y in min_y..=max_y {
                        let dy = (y as f32 - sy) / sr_y;
                        let row = y * eff_w;
                        for x in min_x..=max_x {
                            let dx = (x as f32 - sx) / stage_r;
                            let d2 = dx * dx + dy * dy;

                            if d2 <= 1.05 {
                                let col = if d2 > 0.88 {
                                    // Outer step riser
                                    if (x + y) % 2 == 0 {
                                        Palette::STONE_WARM_LIGHT
                                    } else {
                                        Palette::STONE_COOL_DARK
                                    }
                                } else if d2 > 0.65 {
                                    // Middle step tread
                                    if (x + y) % 3 == 0 {
                                        Palette::STONE_WARM_MID
                                    } else {
                                        Palette::CAVEA_STONE_MID
                                    }
                                } else if d2 > 0.22 {
                                    // Grand Stage Platform (polished heraldic stone)
                                    let pattern = (x / 6 + y / 6) % 2 == 0;
                                    if pattern {
                                        Color::rgb(60, 66, 88)
                                    } else {
                                        Color::rgb(38, 42, 56)
                                    }
                                } else {
                                    // Central Heraldic Sigil Star
                                    if (x + y) % 2 == 0 {
                                        Palette::GOLD_TRIM
                                    } else {
                                        Color::rgb(140, 30, 40)
                                    }
                                };
                                ground_pixels[row + x] = col;
                            }
                        }
                    }
                }
            }
        }

        // Perimeter boundary colliders with entrance and exit portals
        let margin = 14.0;
        let w_f = eff_w as f32;
        let h_f = eff_h as f32;

        // Left and Right border walls
        colliders.push(Collider::new(0.0, 0.0, margin, h_f));
        colliders.push(Collider::new(w_f - margin, 0.0, margin, h_f));

        // Top wall with gap for northern fog wall
        let fog_x = part_cfg.fog_wall.as_ref().map(|f| f.x).unwrap_or(cx);
        let top_gap_w = 44.0;
        colliders.push(Collider::new(margin, 0.0, (fog_x - top_gap_w * 0.5 - margin).max(0.0), margin));
        colliders.push(Collider::new(fog_x + top_gap_w * 0.5, 0.0, (w_f - margin - (fog_x + top_gap_w * 0.5)).max(0.0), margin));

        // Bottom wall with gap for entrance/return gate
        let return_x = part_cfg.return_fog_wall.as_ref().map(|f| f.x)
            .or_else(|| part_cfg.colosseum_gate.map(|g| g.0))
            .unwrap_or(cx);
        let bot_gap_w = 44.0;
        colliders.push(Collider::new(margin, h_f - margin, (return_x - bot_gap_w * 0.5 - margin).max(0.0), margin));
        colliders.push(Collider::new(return_x + bot_gap_w * 0.5, h_f - margin, (w_f - margin - (return_x + bot_gap_w * 0.5)).max(0.0), margin));

        // Instantiate Props
        for p in &part_cfg.props {
            let (w, _h, base_off, col_w, col_h, has_col) = match p.prop_type {
                PropType::GateOfTrials => (32, 34, 32, 28.0, 12.0, true),
                PropType::ColosseumArch => (32, 36, 34, 24.0, 12.0, true),
                PropType::BrokenColumn => (12, 26, 24, 8.0, 6.0, true),
                PropType::WeaponRack => (18, 20, 18, 14.0, 8.0, true),
                PropType::TrainingDummy => (12, 20, 18, 8.0, 8.0, true),
                PropType::SpectatorStand => (28, 18, 16, 24.0, 10.0, true),
                PropType::TatteredBanner => (14, 30, 28, 4.0, 6.0, true),
                PropType::BonePile => (12, 10, 8, 0.0, 0.0, false),
                PropType::HeadlessGladiator => (16, 28, 26, 10.0, 8.0, true),
                PropType::HangingCage => (14, 24, 20, 8.0, 6.0, true),
                PropType::WallTorch => (8, 14, 12, 0.0, 0.0, false),
                PropType::FallenArchRubble => (20, 12, 10, 16.0, 8.0, true),
                PropType::ChapelAltar => (30, 22, 20, 24.0, 10.0, true),
                PropType::ChapelPew => (24, 16, 14, 20.0, 8.0, true),
                PropType::GraveStone => (16, 22, 20, 12.0, 6.0, true),
                PropType::RottingCorpse => (24, 14, 12, 0.0, 0.0, false),
                PropType::ChapelArch => (32, 38, 36, 26.0, 10.0, true),
                PropType::AutumnTree => (34, 42, 40, 10.0, 8.0, true),
                PropType::FallenLog => (26, 14, 12, 22.0, 8.0, true),
                PropType::BrokenCart => (26, 20, 18, 20.0, 10.0, true),
                PropType::WoodenFence => (22, 16, 14, 20.0, 4.0, true),
                PropType::GnarledTree => (38, 46, 44, 14.0, 10.0, true),
                PropType::GlowingMushrooms => (20, 18, 16, 0.0, 0.0, false),
                PropType::RootBriar => (24, 18, 16, 18.0, 8.0, true),
                PropType::RottenStump => (18, 16, 14, 14.0, 8.0, true),
                PropType::CastlePillar => (20, 40, 38, 16.0, 10.0, true),
                PropType::GargoyleStatue => (20, 28, 26, 14.0, 8.0, true),
                PropType::CastleBrazier => (16, 26, 24, 10.0, 8.0, true),
                PropType::CastleBanner => (14, 34, 32, 0.0, 0.0, false),
                PropType::FogWallGate => (38, 36, 34, 0.0, 0.0, false),
            };

            let base_y = p.y + base_off;
            props.push(PropInstance {
                x: p.x,
                y: p.y,
                base_y,
                sprite_index: p.prop_type,
            });

            if has_col {
                let cx_box = p.x as f32 + (w as f32 - col_w) * 0.5;
                let cy_box = (base_y as f32) - col_h * 0.5;
                colliders.push(Collider::new(cx_box, cy_box, col_w, col_h));
            }
        }

        // Build NavGrid from colliders
        for c in &colliders {
            nav_grid.set_blocked_rect(c.x, c.y, c.x + c.w, c.y + c.h);
        }

        // Convert Torches to PointLights
        let mut torch_lights = Vec::with_capacity(part_cfg.torches.len() + 2);
        for t in &part_cfg.torches {
            torch_lights.push(PointLight {
                x: t.x,
                y: t.y,
                radius: t.radius,
                intensity: t.intensity,
                color: Color::rgb(t.color.0, t.color.1, t.color.2),
                flicker_amount: t.flicker,
            });
        }

        Self {
            width: eff_w,
            height: eff_h,
            ground_pixels,
            props,
            colliders,
            nav_grid,
            torch_lights,
            light_shafts: vec![],
        }
    }

    /// Checks if a bounding box collides with any world barrier or prop.
    pub fn check_collision(&self, box_collider: &Collider) -> bool {
        if box_collider.x < 10.0
            || box_collider.x + box_collider.w > self.width as f32 - 10.0
            || box_collider.y < 10.0
            || box_collider.y + box_collider.h > self.height as f32 - 10.0
        {
            return true;
        }

        for c in &self.colliders {
            if box_collider.intersects(c) {
                return true;
            }
        }
        false
    }

    /// Draws volumetric light shafts streaming down through the broken colosseum ceiling.
    pub fn draw_light_shafts(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32, time_secs: f32) {
        let pulse = (time_secs * 1.5).sin() * 0.15 + 0.85;
        let shaft_color = Color::rgb(180, 205, 245);

        for shaft in &self.light_shafts {
            let start_scr_x = shaft.start_x - cam_x as f32;
            let start_scr_y = shaft.start_y - cam_y as f32;
            let end_scr_x = shaft.end_x - cam_x as f32;
            let end_scr_y = shaft.end_y - cam_y as f32;

            let dy = end_scr_y - start_scr_y;
            if dy <= 0.0 {
                continue;
            }

            let y_start = (start_scr_y.max(0.0)).round() as i32;
            let y_end = (end_scr_y.min(fb.height as f32)).round() as i32;

            for y in y_start..y_end {
                let t = (y as f32 - start_scr_y) / dy;
                let center_x = start_scr_x + (end_scr_x - start_scr_x) * t;
                let half_w = (shaft.width * (0.6 + t * 0.4)) * 0.5;
                let alpha_base = (shaft.alpha as f32 * pulse * (1.0 - t * 0.35)) as u8;

                let x_min = ((center_x - half_w).max(0.0)).round() as i32;
                let x_max = ((center_x + half_w).min(fb.width as f32)).round() as i32;

                for x in x_min..x_max {
                    let dist_norm = ((x as f32 - center_x) / half_w).abs().min(1.0);
                    let edge_falloff = (1.0 - dist_norm * dist_norm).max(0.0);
                    let a = (alpha_base as f32 * edge_falloff) as u8;
                    if a > 0 {
                        fb.set_pixel_add(x, y, shaft_color.scale(a as f32 / 255.0));
                    }
                }
            }
        }
    }

    /// Draws the parallax background sky, distant red moon, and Colosseum ruins silhouette.
    pub fn draw_parallax_sky(&self, fb: &mut Framebuffer, cam_x: f32, _cam_y: f32) {
        let sky_top = Palette::VOID_BLACK;
        let sky_bot = Palette::DEEP_NIGHT;
        let horizon_y = fb.height as i32 / 2;

        // Gradient sky
        fb.draw_gradient_v(0, 0, fb.width as i32, horizon_y, sky_top, sky_bot);

        // Distant Ashen Moon (Blood-tinged)
        let moon_x = (fb.width as f32 * 0.75 - cam_x * 0.05).round() as i32;
        let moon_y = 12;
        fb.draw_circle(moon_x, moon_y, 7, Palette::MOON_PALE, true);
        fb.draw_circle(moon_x, moon_y, 10, Palette::MOON_HAZE, false);

        // Layer 1: Distant Colosseum outer ring & ruined arches silhouette (slow parallax)
        let ring_col = Palette::ABYSS;
        for col in 0..fb.width {
            let world_u = col as f32 + cam_x * 0.1;
            let arch_pattern = ((world_u * 0.12).sin() * 5.0).max(0.0);
            let wall_base = (world_u * 0.03).sin() * 3.0;
            let peak_y = (horizon_y as f32 - 16.0 - arch_pattern + wall_base).round() as i32;
            for y in peak_y..horizon_y {
                fb.set_pixel_blend(col as i32, y, ring_col);
            }
        }

        // Layer 2: Mid-distance Cavea arches & spectator pillars silhouette
        let cavea_col = Color::rgb(16, 18, 26);
        for col in 0..fb.width {
            let world_u = col as f32 + cam_x * 0.25;
            let col_rib = if (col + (cam_x * 0.25) as usize) % 18 < 3 { 6.0 } else { 0.0 };
            let peak2 = (world_u * 0.05).sin() * 4.0 + col_rib;
            let peak_y = (horizon_y as f32 - 10.0 + peak2).round() as i32;
            for y in peak_y..horizon_y {
                fb.set_pixel_blend(col as i32, y, cavea_col);
            }
        }
    }

    /// Draws the ground plane visible within the camera view.
    pub fn draw_ground(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32) {
        let min_x = cam_x.max(0);
        let max_x = (cam_x + fb.width as i32).min(self.width as i32);
        let min_y = cam_y.max(0);
        let max_y = (cam_y + fb.height as i32).min(self.height as i32);

        for y in min_y..max_y {
            let fb_y = y - cam_y;
            let world_row_offset = y as usize * self.width;
            let fb_row_offset = fb_y as usize * fb.width;

            for x in min_x..max_x {
                let fb_x = (x - cam_x) as usize;
                fb.pixels[fb_row_offset + fb_x] = self.ground_pixels[world_row_offset + x as usize];
            }
        }
    }

    /// Collects shadow casters from world props for the lighting pass.
    pub fn get_shadow_casters(&self) -> Vec<ShadowCaster> {
        let mut casters = Vec::with_capacity(self.props.len() + 4);
        for p in &self.props {
            let (w, h) = match p.sprite_index {
                PropType::ColosseumArch => (30.0, 32.0),
                PropType::BrokenColumn => (10.0, 22.0),
                PropType::GateOfTrials => (30.0, 32.0),
                PropType::WeaponRack => (16.0, 18.0),
                PropType::TrainingDummy => (12.0, 20.0),
                PropType::SpectatorStand => (26.0, 16.0),
                PropType::TatteredBanner => (6.0, 24.0),
                PropType::BonePile => (10.0, 8.0),
                PropType::HeadlessGladiator => (14.0, 22.0),
                PropType::HangingCage => (12.0, 18.0),
                PropType::WallTorch => (6.0, 10.0),
                PropType::FallenArchRubble => (18.0, 10.0),
                PropType::ChapelAltar => (28.0, 18.0),
                PropType::ChapelPew => (22.0, 14.0),
                PropType::GraveStone => (14.0, 18.0),
                PropType::RottingCorpse => (18.0, 8.0),
                PropType::ChapelArch => (28.0, 34.0),
                PropType::AutumnTree => (26.0, 38.0),
                PropType::FallenLog => (24.0, 10.0),
                PropType::BrokenCart => (22.0, 16.0),
                PropType::WoodenFence => (20.0, 12.0),
                PropType::GnarledTree => (32.0, 40.0),
                PropType::GlowingMushrooms => (16.0, 12.0),
                PropType::RootBriar => (20.0, 14.0),
                PropType::RottenStump => (16.0, 12.0),
                PropType::CastlePillar => (18.0, 36.0),
                PropType::GargoyleStatue => (18.0, 24.0),
                PropType::CastleBrazier => (14.0, 22.0),
                PropType::CastleBanner => (10.0, 28.0),
                PropType::FogWallGate => (34.0, 32.0),
            };
            casters.push(ShadowCaster {
                x: p.x as f32 + w * 0.5,
                y: p.base_y as f32,
                width: w,
                height: h,
            });
        }
        casters
    }

    /// Returns the sprite corresponding to a prop type.
    pub fn get_prop_sprite<'a>(
        &self,
        bank: &'a SpriteBank,
        prop_type: PropType,
        banner_frame: usize,
    ) -> &'a Sprite {
        match prop_type {
            PropType::ColosseumArch => &bank.colosseum_arch,
            PropType::BrokenColumn => &bank.broken_column,
            PropType::GateOfTrials => &bank.gate_of_trials,
            PropType::WeaponRack => &bank.weapon_rack,
            PropType::TrainingDummy => &bank.training_dummy,
            PropType::SpectatorStand => &bank.spectator_stand,
            PropType::TatteredBanner => &bank.tattered_banner[banner_frame % 2],
            PropType::BonePile => &bank.bone_pile,
            PropType::HeadlessGladiator => &bank.headless_gladiator,
            PropType::HangingCage => &bank.hanging_cage,
            PropType::WallTorch => &bank.wall_torch[banner_frame % 2],
            PropType::FallenArchRubble => &bank.fallen_arch_rubble,
            PropType::ChapelAltar => &bank.chapel_altar,
            PropType::ChapelPew => &bank.chapel_pew,
            PropType::GraveStone => &bank.grave_stone,
            PropType::RottingCorpse => &bank.rotting_corpse,
            PropType::ChapelArch => &bank.chapel_arch,
            PropType::AutumnTree => &bank.autumn_tree,
            PropType::FallenLog => &bank.fallen_log,
            PropType::BrokenCart => &bank.broken_cart,
            PropType::WoodenFence => &bank.wooden_fence,
            PropType::GnarledTree => &bank.gnarled_tree,
            PropType::GlowingMushrooms => &bank.glowing_mushrooms[banner_frame % 2],
            PropType::RootBriar => &bank.root_briar,
            PropType::RottenStump => &bank.rotten_stump,
            PropType::CastlePillar => &bank.castle_pillar,
            PropType::GargoyleStatue => &bank.gargoyle_statue,
            PropType::CastleBrazier => &bank.castle_brazier[banner_frame % 2],
            PropType::CastleBanner => &bank.castle_banner[banner_frame % 2],
            PropType::FogWallGate => &bank.fog_wall_gate,
        }
    }
}
