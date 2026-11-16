use super::enemy_sprites;
use super::palette::Palette;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::game::classes::ClassKind;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Sprite {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::TRANSPARENT; width * height],
        }
    }

    pub fn load_png<P: AsRef<Path>>(path: P) -> Result<Self, image::ImageError> {
        let img = image::open(path)?.to_rgba8();
        let (width, height) = img.dimensions();
        let mut pixels = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                let p = img.get_pixel(x, y);
                pixels.push(Color::rgba(p[0], p[1], p[2], p[3]));
            }
        }

        Ok(Self {
            width: width as usize,
            height: height as usize,
            pixels,
        })
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Color {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            Color::TRANSPARENT
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, col: Color) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = col;
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, col: Color) {
        let x_end = (x + w).min(self.width);
        let y_end = (y + h).min(self.height);
        for cy in y..y_end {
            let row = cy * self.width;
            for cx in x..x_end {
                self.pixels[row + cx] = col;
            }
        }
    }

    /// Adds a 1-pixel dark outline around all non-transparent pixels bordering transparency
    pub fn add_outline(&mut self, outline_col: Color) {
        let mut to_outline = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.pixels[y * self.width + x].a == 0 {
                    let has_opaque_neighbor = (x > 0 && self.pixels[y * self.width + (x - 1)].a > 0)
                        || (x + 1 < self.width && self.pixels[y * self.width + (x + 1)].a > 0)
                        || (y > 0 && self.pixels[(y - 1) * self.width + x].a > 0)
                        || (y + 1 < self.height && self.pixels[(y + 1) * self.width + x].a > 0);
                    if has_opaque_neighbor {
                        to_outline.push((x, y));
                    }
                }
            }
        }
        for (ox, oy) in to_outline {
            self.pixels[oy * self.width + ox] = outline_col;
        }
    }

    /// Adds a cool rim light on the leftmost opaque edge of each row
    pub fn add_cool_rim(&mut self, rim_col: Color) {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let col = self.pixels[idx];
                if col.a > 0 && col != Palette::OUTLINE_DARK {
                    self.pixels[idx] = rim_col;
                    break;
                }
            }
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, x: i32, y: i32, flip_x: bool) {
        let w = self.width as i32;
        let h = self.height as i32;

        let start_y = y.max(0);
        let end_y = (y + h).min(fb.height as i32);
        let start_x = x.max(0);
        let end_x = (x + w).min(fb.width as i32);

        for cy in start_y..end_y {
            let sy = (cy - y) as usize;
            let fb_row = cy as usize * fb.width;
            let sprite_row = sy * self.width;

            for cx in start_x..end_x {
                let sx = if flip_x {
                    self.width - 1 - (cx - x) as usize
                } else {
                    (cx - x) as usize
                };

                let col = self.pixels[sprite_row + sx];
                if col.a > 0 {
                    let fb_idx = fb_row + cx as usize;
                    fb.pixels[fb_idx] = col.blend_over(fb.pixels[fb_idx]);
                }
            }
        }
    }

    pub fn draw_alpha(&self, fb: &mut Framebuffer, x: i32, y: i32, flip_x: bool, alpha: u8) {
        if alpha == 0 {
            return;
        }
        let w = self.width as i32;
        let h = self.height as i32;

        let start_y = y.max(0);
        let end_y = (y + h).min(fb.height as i32);
        let start_x = x.max(0);
        let end_x = (x + w).min(fb.width as i32);
        let a_factor = alpha as f32 / 255.0;

        for cy in start_y..end_y {
            let sy = (cy - y) as usize;
            let fb_row = cy as usize * fb.width;
            let sprite_row = sy * self.width;

            for cx in start_x..end_x {
                let sx = if flip_x {
                    self.width - 1 - (cx - x) as usize
                } else {
                    (cx - x) as usize
                };

                let mut col = self.pixels[sprite_row + sx];
                if col.a > 0 {
                    col.a = (col.a as f32 * a_factor).round() as u8;
                    let fb_idx = fb_row + cx as usize;
                    fb.pixels[fb_idx] = col.blend_over(fb.pixels[fb_idx]);
                }
            }
        }
    }

    /// Blits (draws) this sprite onto another destination Sprite with alpha blending
    pub fn blit(&self, dest: &mut Sprite, x: i32, y: i32, flip_x: bool) {
        let w = self.width as i32;
        let h = self.height as i32;
        let start_y = y.max(0);
        let end_y = (y + h).min(dest.height as i32);
        let start_x = x.max(0);
        let end_x = (x + w).min(dest.width as i32);

        for cy in start_y..end_y {
            let sy = (cy - y) as usize;
            let dest_row = cy as usize * dest.width;
            let src_row = sy * self.width;

            for cx in start_x..end_x {
                let sx = if flip_x {
                    self.width - 1 - (cx - x) as usize
                } else {
                    (cx - x) as usize
                };

                let col = self.pixels[src_row + sx];
                if col.a > 0 {
                    let dest_idx = dest_row + cx as usize;
                    dest.pixels[dest_idx] = col.blend_over(dest.pixels[dest_idx]);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction4 {
    South,
    North,
    East,
    SouthEast,
}

#[derive(Clone)]
pub struct ClassGearSet {
    pub head: [Sprite; 4],
    pub torso: [Sprite; 4],
    pub cape: [Option<Sprite>; 4],
    pub main_hand: [Option<Sprite>; 4],
    pub off_hand: [Option<Sprite>; 4],
    pub accessory: [Option<Sprite>; 4],
    pub lmb_frames: [[Sprite; 4]; 4],
    pub rmb_frames: [[Sprite; 4]; 4],
    pub hit_frame: [Sprite; 4],
    pub death_frame: Sprite,
}

pub struct SpriteBank {
    // Player Base Body (Wanderer)
    pub body_idle: [[Sprite; 4]; 4],  // 4 directions x 4 breathing frames
    pub body_walk: [[Sprite; 6]; 4],  // 4 directions x 6 walk frames
    pub body_roll: [[Sprite; 7]; 4],  // 4 directions x 7 roll frames with smear/dust
    pub body_lmb: [[Sprite; 4]; 4],   // 4 directions x 4 attack frames
    pub body_rmb: [[Sprite; 4]; 4],   // 4 directions x 4 secondary attack frames
    pub body_hit: [Sprite; 4],        // 4 directions hit reaction
    pub body_death: Sprite,           // death collapsed frame

    // Class Gear Sets
    pub mage_gear: ClassGearSet,
    pub knight_gear: ClassGearSet,
    pub assassin_gear: ClassGearSet,
    pub hunter_gear: ClassGearSet,
    pub alchemist_gear: ClassGearSet,

    // Bonfire & NPCs (4-frame animations)
    pub bonfire_anim: [Sprite; 4],
    pub firekeeper_anim: [Sprite; 4],
    pub firekeeper_portrait: Sprite,
    pub blacksmith_anim: [Sprite; 4],
    pub adventurer_anim: [Sprite; 4],
    pub witch_anim: [Sprite; 4],

    // The Ashen Colosseum Props
    pub colosseum_arch: Sprite,
    pub broken_column: Sprite,
    pub gate_of_trials: Sprite,
    pub weapon_rack: Sprite,
    pub training_dummy: Sprite,
    pub spectator_stand: Sprite,
    pub tattered_banner: [Sprite; 2],
    pub bone_pile: Sprite,
    pub headless_gladiator: Sprite,
    pub hanging_cage: Sprite,
    pub wall_torch: [Sprite; 2],
    pub fallen_arch_rubble: Sprite,

    // First Area (Gothic Chapel, Autumn Woods, Gloomwood Deep, Dread Keep)
    pub chapel_altar: Sprite,
    pub chapel_pew: Sprite,
    pub grave_stone: Sprite,
    pub rotting_corpse: Sprite,
    pub chapel_arch: Sprite,
    pub autumn_tree: Sprite,
    pub fallen_log: Sprite,
    pub broken_cart: Sprite,
    pub wooden_fence: Sprite,
    pub gnarled_tree: Sprite,
    pub glowing_mushrooms: [Sprite; 2],
    pub root_briar: Sprite,
    pub rotten_stump: Sprite,
    pub castle_pillar: Sprite,
    pub gargoyle_statue: Sprite,
    pub castle_brazier: [Sprite; 2],
    pub castle_banner: [Sprite; 2],
    pub fog_wall_gate: Sprite,

    // Enemies (Phase 3/4)
    // 1. Acid Sludge
    pub sludge_idle: [Sprite; 4],
    pub sludge_walk: [Sprite; 4],
    pub sludge_vomit_windup: Sprite,
    pub sludge_vomit_spit: Sprite,
    pub sludge_hit: Sprite,
    pub sludge_death: [Sprite; 3],

    // 2. Bogged Skeleton
    pub skeleton_idle: [[Sprite; 4]; 4],
    pub skeleton_walk: [[Sprite; 4]; 4],
    pub skeleton_draw: [[Sprite; 3]; 4],
    pub skeleton_release: [Sprite; 4],
    pub skeleton_hit: [Sprite; 4],
    pub skeleton_death: Sprite,

    // 3. Carrion Fly
    pub fly_anim: [Sprite; 2],
    pub fly_shadow: Sprite,
    pub fly_attack: [Sprite; 2],
    pub fly_hit: Sprite,
    pub fly_death: Sprite,

    // 4. Corpse Worm
    pub worm_anim: [Sprite; 4],
    pub worm_lunge: [Sprite; 2],
    pub worm_hit: Sprite,
    pub worm_death: Sprite,
}

impl SpriteBank {
    pub fn build() -> Self {
        Self {
            body_idle: [
                [
                    generate_base_body_idle(Direction4::South, 0),
                    generate_base_body_idle(Direction4::South, 1),
                    generate_base_body_idle(Direction4::South, 2),
                    generate_base_body_idle(Direction4::South, 3),
                ],
                [
                    generate_base_body_idle(Direction4::North, 0),
                    generate_base_body_idle(Direction4::North, 1),
                    generate_base_body_idle(Direction4::North, 2),
                    generate_base_body_idle(Direction4::North, 3),
                ],
                [
                    generate_base_body_idle(Direction4::East, 0),
                    generate_base_body_idle(Direction4::East, 1),
                    generate_base_body_idle(Direction4::East, 2),
                    generate_base_body_idle(Direction4::East, 3),
                ],
                [
                    generate_base_body_idle(Direction4::SouthEast, 0),
                    generate_base_body_idle(Direction4::SouthEast, 1),
                    generate_base_body_idle(Direction4::SouthEast, 2),
                    generate_base_body_idle(Direction4::SouthEast, 3),
                ],
            ],

            body_walk: [
                [
                    generate_base_body_walk(Direction4::South, 0),
                    generate_base_body_walk(Direction4::South, 1),
                    generate_base_body_walk(Direction4::South, 2),
                    generate_base_body_walk(Direction4::South, 3),
                    generate_base_body_walk(Direction4::South, 4),
                    generate_base_body_walk(Direction4::South, 5),
                ],
                [
                    generate_base_body_walk(Direction4::North, 0),
                    generate_base_body_walk(Direction4::North, 1),
                    generate_base_body_walk(Direction4::North, 2),
                    generate_base_body_walk(Direction4::North, 3),
                    generate_base_body_walk(Direction4::North, 4),
                    generate_base_body_walk(Direction4::North, 5),
                ],
                [
                    generate_base_body_walk(Direction4::East, 0),
                    generate_base_body_walk(Direction4::East, 1),
                    generate_base_body_walk(Direction4::East, 2),
                    generate_base_body_walk(Direction4::East, 3),
                    generate_base_body_walk(Direction4::East, 4),
                    generate_base_body_walk(Direction4::East, 5),
                ],
                [
                    generate_base_body_walk(Direction4::SouthEast, 0),
                    generate_base_body_walk(Direction4::SouthEast, 1),
                    generate_base_body_walk(Direction4::SouthEast, 2),
                    generate_base_body_walk(Direction4::SouthEast, 3),
                    generate_base_body_walk(Direction4::SouthEast, 4),
                    generate_base_body_walk(Direction4::SouthEast, 5),
                ],
            ],

            body_roll: [
                [
                    generate_base_body_roll(Direction4::South, 0),
                    generate_base_body_roll(Direction4::South, 1),
                    generate_base_body_roll(Direction4::South, 2),
                    generate_base_body_roll(Direction4::South, 3),
                    generate_base_body_roll(Direction4::South, 4),
                    generate_base_body_roll(Direction4::South, 5),
                    generate_base_body_roll(Direction4::South, 6),
                ],
                [
                    generate_base_body_roll(Direction4::North, 0),
                    generate_base_body_roll(Direction4::North, 1),
                    generate_base_body_roll(Direction4::North, 2),
                    generate_base_body_roll(Direction4::North, 3),
                    generate_base_body_roll(Direction4::North, 4),
                    generate_base_body_roll(Direction4::North, 5),
                    generate_base_body_roll(Direction4::North, 6),
                ],
                [
                    generate_base_body_roll(Direction4::East, 0),
                    generate_base_body_roll(Direction4::East, 1),
                    generate_base_body_roll(Direction4::East, 2),
                    generate_base_body_roll(Direction4::East, 3),
                    generate_base_body_roll(Direction4::East, 4),
                    generate_base_body_roll(Direction4::East, 5),
                    generate_base_body_roll(Direction4::East, 6),
                ],
                [
                    generate_base_body_roll(Direction4::SouthEast, 0),
                    generate_base_body_roll(Direction4::SouthEast, 1),
                    generate_base_body_roll(Direction4::SouthEast, 2),
                    generate_base_body_roll(Direction4::SouthEast, 3),
                    generate_base_body_roll(Direction4::SouthEast, 4),
                    generate_base_body_roll(Direction4::SouthEast, 5),
                    generate_base_body_roll(Direction4::SouthEast, 6),
                ],
            ],

            body_lmb: [
                [
                    generate_base_body_lmb(Direction4::South, 0),
                    generate_base_body_lmb(Direction4::South, 1),
                    generate_base_body_lmb(Direction4::South, 2),
                    generate_base_body_lmb(Direction4::South, 3),
                ],
                [
                    generate_base_body_lmb(Direction4::North, 0),
                    generate_base_body_lmb(Direction4::North, 1),
                    generate_base_body_lmb(Direction4::North, 2),
                    generate_base_body_lmb(Direction4::North, 3),
                ],
                [
                    generate_base_body_lmb(Direction4::East, 0),
                    generate_base_body_lmb(Direction4::East, 1),
                    generate_base_body_lmb(Direction4::East, 2),
                    generate_base_body_lmb(Direction4::East, 3),
                ],
                [
                    generate_base_body_lmb(Direction4::SouthEast, 0),
                    generate_base_body_lmb(Direction4::SouthEast, 1),
                    generate_base_body_lmb(Direction4::SouthEast, 2),
                    generate_base_body_lmb(Direction4::SouthEast, 3),
                ],
            ],

            body_rmb: [
                [
                    generate_base_body_rmb(Direction4::South, 0),
                    generate_base_body_rmb(Direction4::South, 1),
                    generate_base_body_rmb(Direction4::South, 2),
                    generate_base_body_rmb(Direction4::South, 3),
                ],
                [
                    generate_base_body_rmb(Direction4::North, 0),
                    generate_base_body_rmb(Direction4::North, 1),
                    generate_base_body_rmb(Direction4::North, 2),
                    generate_base_body_rmb(Direction4::North, 3),
                ],
                [
                    generate_base_body_rmb(Direction4::East, 0),
                    generate_base_body_rmb(Direction4::East, 1),
                    generate_base_body_rmb(Direction4::East, 2),
                    generate_base_body_rmb(Direction4::East, 3),
                ],
                [
                    generate_base_body_rmb(Direction4::SouthEast, 0),
                    generate_base_body_rmb(Direction4::SouthEast, 1),
                    generate_base_body_rmb(Direction4::SouthEast, 2),
                    generate_base_body_rmb(Direction4::SouthEast, 3),
                ],
            ],

            body_hit: [
                generate_base_body_hit(Direction4::South),
                generate_base_body_hit(Direction4::North),
                generate_base_body_hit(Direction4::East),
                generate_base_body_hit(Direction4::SouthEast),
            ],

            body_death: generate_base_body_death(),

            mage_gear: generate_mage_gear_set(),
            knight_gear: generate_knight_gear_set(),
            assassin_gear: generate_assassin_gear_set(),
            hunter_gear: generate_hunter_gear_set(),
            alchemist_gear: generate_alchemist_gear_set(),

            bonfire_anim: [
                generate_bonfire_frame(0),
                generate_bonfire_frame(1),
                generate_bonfire_frame(2),
                generate_bonfire_frame(3),
            ],
            firekeeper_anim: [
                generate_firekeeper_frame(0),
                generate_firekeeper_frame(1),
                generate_firekeeper_frame(2),
                generate_firekeeper_frame(3),
            ],
            firekeeper_portrait: generate_firekeeper_portrait(),
            blacksmith_anim: [
                generate_blacksmith_frame(0),
                generate_blacksmith_frame(1),
                generate_blacksmith_frame(2),
                generate_blacksmith_frame(3),
            ],
            adventurer_anim: [
                generate_adventurer_npc_frame(0),
                generate_adventurer_npc_frame(1),
                generate_adventurer_npc_frame(2),
                generate_adventurer_npc_frame(3),
            ],
            witch_anim: [
                generate_witch_frame(0),
                generate_witch_frame(1),
                generate_witch_frame(2),
                generate_witch_frame(3),
            ],

            colosseum_arch: generate_colosseum_arch(),
            broken_column: generate_broken_column(),
            gate_of_trials: generate_gate_of_trials(),
            weapon_rack: generate_weapon_rack(),
            training_dummy: generate_training_dummy(),
            spectator_stand: generate_spectator_stand(),
            tattered_banner: [
                generate_tattered_banner(0),
                generate_tattered_banner(1),
            ],
            bone_pile: generate_bone_pile(),
            headless_gladiator: generate_headless_gladiator(),
            hanging_cage: generate_hanging_cage(),
            wall_torch: [
                generate_wall_torch(0),
                generate_wall_torch(1),
            ],
            fallen_arch_rubble: generate_fallen_arch_rubble(),

            chapel_altar: generate_chapel_altar(),
            chapel_pew: generate_chapel_pew(),
            grave_stone: generate_grave_stone(),
            rotting_corpse: generate_rotting_corpse(),
            chapel_arch: generate_chapel_arch(),
            autumn_tree: generate_autumn_tree(),
            fallen_log: generate_fallen_log(),
            broken_cart: generate_broken_cart(),
            wooden_fence: generate_wooden_fence(),
            gnarled_tree: generate_gnarled_tree(),
            glowing_mushrooms: [
                generate_glowing_mushrooms(0),
                generate_glowing_mushrooms(1),
            ],
            root_briar: generate_root_briar(),
            rotten_stump: generate_rotten_stump(),
            castle_pillar: generate_castle_pillar(),
            gargoyle_statue: generate_gargoyle_statue(),
            castle_brazier: [
                generate_castle_brazier(0),
                generate_castle_brazier(1),
            ],
            castle_banner: [
                generate_castle_banner(0),
                generate_castle_banner(1),
            ],
            fog_wall_gate: generate_fog_wall_gate(),

            sludge_idle: [
                generate_sludge_idle(0),
                generate_sludge_idle(1),
                generate_sludge_idle(2),
                generate_sludge_idle(3),
            ],
            sludge_walk: [
                generate_sludge_walk(0),
                generate_sludge_walk(1),
                generate_sludge_walk(2),
                generate_sludge_walk(3),
            ],
            sludge_vomit_windup: generate_sludge_windup(),
            sludge_vomit_spit: generate_sludge_spit(),
            sludge_hit: generate_sludge_hit(),
            sludge_death: [
                generate_sludge_death(0),
                generate_sludge_death(1),
                generate_sludge_death(2),
            ],

            skeleton_idle: core::array::from_fn(|_| [
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(0),
            ]),
            skeleton_walk: core::array::from_fn(|_| [
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(1),
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(1),
            ]),
            skeleton_draw: core::array::from_fn(|_| [
                enemy_sprites::SKELETON.to_sprite(0),
                enemy_sprites::SKELETON.to_sprite(1),
                enemy_sprites::SKELETON.to_sprite(2),
            ]),
            skeleton_release: core::array::from_fn(|_| enemy_sprites::SKELETON.to_sprite(2)),
            skeleton_hit: core::array::from_fn(|_| enemy_sprites::SKELETON.to_sprite(1)),
            skeleton_death: generate_skeleton_death(),

            fly_anim: [
                enemy_sprites::FLY.to_sprite(0),
                enemy_sprites::FLY.to_sprite(1),
            ],
            fly_shadow: generate_fly_shadow(),
            fly_attack: [
                enemy_sprites::FLY.to_sprite(0),
                enemy_sprites::FLY.to_sprite(1),
            ],
            fly_hit: enemy_sprites::FLY.to_sprite(1),
            fly_death: enemy_sprites::FLY.to_sprite(0),

            worm_anim: [
                enemy_sprites::WORM.to_sprite(0),
                enemy_sprites::WORM.to_sprite(1),
                enemy_sprites::WORM.to_sprite(2),
                enemy_sprites::WORM.to_sprite(1),
            ],
            worm_lunge: [
                enemy_sprites::WORM.to_sprite(1),
                enemy_sprites::WORM.to_sprite(2),
            ],
            worm_hit: enemy_sprites::WORM.to_sprite(1),
            worm_death: enemy_sprites::WORM.to_sprite(2),
        }
    }

    pub fn get_class_gear(&self, class: ClassKind) -> Option<&ClassGearSet> {
        match class {
            ClassKind::Mage => Some(&self.mage_gear),
            ClassKind::Knight => Some(&self.knight_gear),
            ClassKind::Assassin => Some(&self.assassin_gear),
            ClassKind::Hunter => Some(&self.hunter_gear),
            ClassKind::Alchemist => Some(&self.alchemist_gear),
            ClassKind::Wanderer => None,
        }
    }
}

// -------------------------------------------------------------
// Base Body Painter & Generators (Ashen Wanderer, 18x30)
// -------------------------------------------------------------
#[allow(clippy::too_many_arguments)]
fn paint_humanoid_base(
    sp: &mut Sprite,
    dir: Direction4,
    head_y_off: i32,
    l_arm_pos: i32,
    r_arm_pos: i32,
    l_leg_off: (i32, i32),
    r_leg_off: (i32, i32),
    scarf_lift: i32,
) {
    let skin_shadow = Palette::SKIN_SHADOW;
    let skin_mid = Palette::SKIN_MID;
    let skin_warm = Palette::SKIN_WARM_HIGHLIGHT;

    let cloth_dark = Palette::CLOTH_DARK;
    let cloth_mid = Palette::CLOTH_MID;
    let cloth_light = Palette::CLOTH_LIGHT;

    let scarf_dark = Color::rgb(110, 115, 130);
    let scarf_mid = Color::rgb(165, 175, 190);
    let scarf_light = Palette::ASH_WHITE;

    let leather_dark = Palette::LEATHER_DARK;
    let leather_mid = Palette::LEATHER_MID;
    let pants_dark = Color::rgb(24, 26, 32);
    let pants_mid = Color::rgb(42, 46, 56);
    let boots_dark = Palette::IRON_DARK;
    let boots_mid = Color::rgb(50, 44, 38);

    let hy = (4 + head_y_off).max(1) as usize;

    // Head & Hair/Cowl
    match dir {
        Direction4::South => {
            // Hair / cowl top
            sp.fill_rect(7, hy, 4, 1, scarf_dark);
            sp.fill_rect(6, hy + 1, 6, 1, scarf_mid);
            // Face
            sp.fill_rect(7, hy + 2, 4, 3, skin_mid);
            sp.set_pixel(7, hy + 2, skin_shadow);
            sp.set_pixel(10, hy + 2, skin_warm);
            // Shadowed eyes
            sp.set_pixel(8, hy + 3, skin_shadow);
            sp.set_pixel(10, hy + 3, skin_shadow);
            // Scarf around neck
            sp.fill_rect(6, hy + 5, 6, 2, scarf_mid);
            sp.set_pixel(7, hy + 5, scarf_light);
            // Trailing scarf knot
            let knot_len = (3 - scarf_lift).max(1) as usize;
            sp.fill_rect(9, hy + 7, 2, knot_len, scarf_light);
            sp.set_pixel(10, hy + 7 + knot_len - 1, scarf_mid);
        }
        Direction4::North => {
            // Back of cowl / head
            sp.fill_rect(7, hy, 4, 1, scarf_dark);
            sp.fill_rect(6, hy + 1, 6, 4, scarf_dark);
            sp.fill_rect(7, hy + 2, 4, 3, scarf_mid);
            // Scarf wrap
            sp.fill_rect(6, hy + 5, 6, 2, scarf_mid);
        }
        Direction4::East => {
            sp.fill_rect(8, hy, 3, 1, scarf_dark);
            sp.fill_rect(7, hy + 1, 5, 2, scarf_mid);
            // Profile face
            sp.fill_rect(8, hy + 2, 4, 3, skin_mid);
            sp.set_pixel(11, hy + 3, skin_warm);
            // Scarf neck & knot trailing behind
            sp.fill_rect(7, hy + 5, 5, 2, scarf_mid);
            let kx = (5 - scarf_lift).max(3) as usize;
            sp.fill_rect(kx, hy + 5, 2, 2, scarf_light);
        }
        Direction4::SouthEast => {
            sp.fill_rect(7, hy, 4, 1, scarf_dark);
            sp.fill_rect(6, hy + 1, 6, 2, scarf_mid);
            sp.fill_rect(7, hy + 2, 5, 3, skin_mid);
            sp.set_pixel(11, hy + 3, skin_warm);
            sp.fill_rect(6, hy + 5, 6, 2, scarf_mid);
            sp.fill_rect(6, hy + 7, 2, (3 - scarf_lift).max(1) as usize, scarf_light);
        }
    }

    // Torso / Jerkin (rows hy+7..hy+14)
    let ty = hy + 7;
    for y in ty..(ty + 7).min(20) {
        let x_start = match dir {
            Direction4::East => 6,
            _ => 5,
        };
        for x in x_start..13 {
            let col = if x < 7 {
                cloth_dark
            } else if x > 10 {
                cloth_light
            } else {
                cloth_mid
            };
            sp.set_pixel(x, y, col);
        }
    }

    // Traveler straps & belt
    sp.set_pixel(7, ty + 1, leather_mid);
    sp.set_pixel(7, ty + 2, leather_mid);
    sp.set_pixel(11, ty + 1, leather_mid);
    sp.set_pixel(11, ty + 2, leather_mid);
    // Leather belt with brass buckle
    let by = (ty + 6).min(20);
    sp.fill_rect(5, by, 8, 1, leather_dark);
    sp.set_pixel(9, by, Palette::GOLD_TRIM);

    // Traveler backpack on back (visible in North, East, SouthEast)
    if dir != Direction4::South {
        let bx = match dir {
            Direction4::North => 7,
            _ => 4,
        };
        sp.fill_rect(bx, ty + 1, 3, 4, leather_dark);
        sp.fill_rect(bx, ty, 3, 1, Color::rgb(140, 145, 160)); // bedroll on top
    }

    // Arms
    let lay = ((ty + 1) as i32 + l_arm_pos).clamp(6, 20) as usize;
    let ray = ((ty + 1) as i32 + r_arm_pos).clamp(6, 20) as usize;
    match dir {
        Direction4::East => {
            sp.fill_rect(9, ray, 2, 5, cloth_mid);
            sp.fill_rect(9, ray + 5, 2, 1, skin_mid);
        }
        _ => {
            sp.fill_rect(4, lay, 2, 5, cloth_dark);
            sp.fill_rect(4, lay + 5, 2, 1, skin_mid);
            sp.fill_rect(12, ray, 2, 5, cloth_light);
            sp.fill_rect(12, ray + 5, 2, 1, Palette::SKIN_LIGHT);
        }
    }

    // Legs / Trousers (rows 21..25)
    let lx = (6 + l_leg_off.0).clamp(3, 12) as usize;
    let ly = (21 + l_leg_off.1).clamp(18, 25) as usize;
    let rx = (10 + r_leg_off.0).clamp(5, 14) as usize;
    let ry = (21 + r_leg_off.1).clamp(18, 25) as usize;

    sp.fill_rect(lx, ly, 2, (25 - ly).max(1), pants_dark);
    sp.fill_rect(rx, ry, 2, (25 - ry).max(1), pants_mid);

    // Sturdy Boots (rows 25..28)
    sp.fill_rect(lx, 25, 3, 3, boots_dark);
    sp.fill_rect(rx, 25, 3, 3, boots_mid);
    sp.set_pixel(lx + 1, 25, leather_mid); // boot cuff
    sp.set_pixel(rx + 1, 25, leather_mid);

    // Contact shadow at feet
    for x in 4..14 {
        sp.set_pixel(x, 28, Color::rgba(0, 0, 0, 80));
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
}

fn generate_base_body_idle(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let head_off = if frame == 1 || frame == 2 { -1 } else { 0 };
    let scarf_lift = if frame == 2 { 1 } else { 0 };
    paint_humanoid_base(&mut sp, dir, head_off, 0, 0, (0, 0), (0, 0), scarf_lift);
    sp
}

fn generate_base_body_walk(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let (head_off, l_arm, r_arm, l_leg, r_leg) = match frame % 6 {
        0 => (0, -1, 1, (1, 0), (-1, -1)),
        1 => (-1, 0, 0, (1, 0), (0, 0)),
        2 => (0, 1, -1, (0, 0), (1, 0)),
        3 => (0, 1, -1, (-1, -1), (1, 0)),
        4 => (-1, 0, 0, (0, 0), (1, 0)),
        _ => (0, -1, 1, (1, 0), (0, 0)),
    };
    paint_humanoid_base(&mut sp, dir, head_off, l_arm, r_arm, l_leg, r_leg, 0);
    sp
}

fn generate_base_body_roll(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let dark = Palette::CLOTH_DARK;
    let mid = Palette::CLOTH_MID;
    let scarf = Palette::ASH_WHITE;
    let dust = Color::rgba(210, 215, 230, 180);

    match frame % 7 {
        0 => {
            paint_humanoid_base(&mut sp, dir, 3, 2, 2, (0, 2), (0, 2), 0);
        }
        1 => {
            sp.fill_rect(5, 14, 8, 5, mid);
            sp.fill_rect(3, 15, 3, 3, dark);
            sp.fill_rect(12, 13, 3, 3, Palette::SKIN_MID);
            sp.fill_rect(2, 17, 3, 2, scarf);
            sp.set_pixel(1, 18, dust);
        }
        2 => {
            sp.fill_rect(5, 12, 8, 8, mid);
            sp.fill_rect(6, 13, 6, 6, dark);
            sp.fill_rect(4, 15, 3, 2, scarf);
            sp.set_pixel(2, 15, dust);
            sp.set_pixel(1, 16, dust);
        }
        3 => {
            sp.fill_rect(5, 10, 8, 8, mid);
            sp.fill_rect(6, 8, 4, 3, Palette::IRON_DARK);
            sp.fill_rect(7, 17, 4, 2, scarf);
            sp.set_pixel(3, 11, dust);
            sp.set_pixel(14, 11, dust);
        }
        4 => {
            sp.fill_rect(4, 16, 9, 6, mid);
            sp.fill_rect(12, 17, 4, 4, dark);
            sp.fill_rect(2, 19, 3, 2, dust);
            sp.fill_rect(1, 20, 2, 2, dust);
        }
        5 => {
            sp.fill_rect(4, 18, 10, 5, mid);
            sp.fill_rect(13, 19, 3, 3, Palette::SKIN_MID);
            sp.fill_rect(2, 20, 3, 2, dust);
            sp.fill_rect(1, 21, 2, 1, dust);
        }
        _ => {
            paint_humanoid_base(&mut sp, dir, 1, 1, 1, (0, 1), (0, 1), 0);
        }
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_base_body_lmb(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let slash_col = Color::rgb(215, 235, 255);
    match frame % 4 {
        0 => {
            paint_humanoid_base(&mut sp, dir, 0, -2, 2, (0, 0), (0, 0), 0);
        }
        1 => {
            paint_humanoid_base(&mut sp, dir, 0, 1, -2, (1, 0), (-1, 0), 1);
            sp.fill_rect(11, 10, 4, 2, slash_col);
            sp.set_pixel(15, 12, slash_col);
            sp.set_pixel(16, 13, slash_col);
            sp.set_pixel(15, 14, Palette::MAGE_CYAN);
        }
        2 => {
            paint_humanoid_base(&mut sp, dir, 0, 2, -1, (1, 0), (-1, 0), 0);
            sp.set_pixel(16, 14, slash_col);
            sp.set_pixel(15, 15, Palette::MAGE_CYAN);
        }
        _ => {
            paint_humanoid_base(&mut sp, dir, 0, 0, 0, (0, 0), (0, 0), 0);
        }
    }
    sp
}

fn generate_base_body_rmb(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let spark = Palette::BONFIRE_YELLOW;
    match frame % 4 {
        0 => {
            paint_humanoid_base(&mut sp, dir, 1, -3, -3, (0, 1), (0, 1), 0);
        }
        1 => {
            paint_humanoid_base(&mut sp, dir, 0, 2, 2, (2, 0), (-1, 0), 1);
            sp.set_pixel(15, 16, spark);
            sp.set_pixel(16, 15, Palette::BONFIRE_CORE);
            sp.set_pixel(17, 16, Palette::BONFIRE_ORANGE);
        }
        2 => {
            paint_humanoid_base(&mut sp, dir, 0, 1, 1, (1, 0), (0, 0), 0);
            sp.set_pixel(16, 17, spark);
        }
        _ => {
            paint_humanoid_base(&mut sp, dir, 0, 0, 0, (0, 0), (0, 0), 0);
        }
    }
    sp
}

fn generate_base_body_hit(dir: Direction4) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    paint_humanoid_base(&mut sp, dir, -1, -3, 3, (-1, 0), (1, 0), 1);
    sp.set_pixel(9, 14, Color::rgb(255, 60, 60));
    sp.set_pixel(10, 13, Color::rgb(255, 180, 180));
    sp
}

fn generate_base_body_death() -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let dark = Palette::CLOTH_DARK;
    let mid = Palette::CLOTH_MID;
    let skin = Palette::SKIN_SHADOW;
    let scarf = Palette::ASH_WHITE;

    sp.fill_rect(2, 24, 14, 4, dark);
    sp.fill_rect(4, 23, 8, 3, mid);
    sp.fill_rect(12, 24, 3, 2, skin);
    sp.fill_rect(1, 26, 4, 2, scarf);
    for x in 3..15 {
        sp.set_pixel(x, 28, Palette::BLOOD_STAIN);
    }
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

// -------------------------------------------------------------
// Class 1: MAGE (Witch Hat, Twilight Robe, Staff Orb)
// -------------------------------------------------------------
fn generate_mage_gear_set() -> ClassGearSet {
    let mut head: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut torso: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut cape: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut main_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));

    let hat_dark = Color::rgb(22, 16, 38);
    let hat_mid = Color::rgb(42, 32, 70);
    let hat_rim = Color::rgb(65, 50, 105);
    let jewel = Palette::MAGE_CYAN;
    let robe_dark = Color::rgb(24, 20, 42);
    let robe_mid = Color::rgb(40, 32, 68);
    let gold = Palette::GOLD_TRIM;
    let staff_wood = Color::rgb(65, 42, 26);
    let orb_cyan = Palette::MAGE_CYAN;
    let orb_core = Color::rgb(215, 245, 255);

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        let sp_h = &mut head[d_idx];
        sp_h.fill_rect(3, 6, 12, 1, hat_rim);
        sp_h.fill_rect(4, 5, 10, 1, hat_mid);
        sp_h.fill_rect(6, 3, 5, 2, hat_dark);
        sp_h.fill_rect(7, 1, 3, 2, hat_dark);
        sp_h.set_pixel(8, 0, hat_rim);
        sp_h.set_pixel(9, 5, jewel);
        sp_h.add_outline(Palette::OUTLINE_DARK);

        let sp_t = &mut torso[d_idx];
        sp_t.fill_rect(5, 11, 8, 14, robe_mid);
        sp_t.fill_rect(4, 14, 2, 7, robe_dark);
        sp_t.fill_rect(12, 14, 2, 7, robe_dark);
        sp_t.fill_rect(5, 24, 8, 1, gold);
        sp_t.set_pixel(9, 13, gold);
        sp_t.set_pixel(9, 15, gold);
        sp_t.add_outline(Palette::OUTLINE_DARK);

        let sp_c = &mut cape[d_idx];
        sp_c.fill_rect(4, 12, 10, 14, hat_dark);
        sp_c.set_pixel(6, 16, Palette::MOON_PALE);
        sp_c.set_pixel(11, 19, Palette::MOON_PALE);
        sp_c.add_outline(Palette::OUTLINE_DARK);

        let sp_m = &mut main_hand[d_idx];
        let sx = match d_idx {
            1 => 3,
            2 => 13,
            _ => 14,
        };
        sp_m.fill_rect(sx, 5, 2, 21, staff_wood);
        sp_m.fill_rect(sx - 1, 3, 4, 3, orb_cyan);
        sp_m.set_pixel(sx, 4, orb_core);
        sp_m.add_outline(Palette::OUTLINE_DARK);
    }

    let mut lmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut rmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut hit_frame: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let death_frame = generate_base_body_death();

    for d in 0..4 {
        let dir = match d {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        for f in 0..4 {
            let mut sp_l = generate_base_body_lmb(dir, f);
            head[d].blit(&mut sp_l, 0, 0, false);
            torso[d].blit(&mut sp_l, 0, 0, false);
            if f == 1 || f == 2 {
                sp_l.fill_rect(12, 8, 4, 2, orb_core);
                sp_l.set_pixel(16, 10, orb_cyan);
            }
            lmb_frames[d][f] = sp_l;

            let mut sp_r = generate_base_body_rmb(dir, f);
            head[d].blit(&mut sp_r, 0, 0, false);
            torso[d].blit(&mut sp_r, 0, 0, false);
            if f == 1 {
                sp_r.fill_rect(6, 12, 6, 6, Color::rgba(88, 166, 255, 120));
                sp_r.set_pixel(9, 15, orb_core);
            }
            rmb_frames[d][f] = sp_r;
        }
        let mut sp_h = generate_base_body_hit(dir);
        head[d].blit(&mut sp_h, 0, 0, false);
        torso[d].blit(&mut sp_h, 0, 0, false);
        hit_frame[d] = sp_h;
    }

    ClassGearSet {
        head,
        torso,
        cape: [Some(cape[0].clone()), Some(cape[1].clone()), Some(cape[2].clone()), Some(cape[3].clone())],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [None, None, None, None],
        accessory: [None, None, None, None],
        lmb_frames,
        rmb_frames,
        hit_frame,
        death_frame,
    }
}

// -------------------------------------------------------------
// Class 2: KNIGHT (Winged Greathelm, Plate Armor, Sword & Shield)
// -------------------------------------------------------------
fn generate_knight_gear_set() -> ClassGearSet {
    let mut head: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut torso: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut cape: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut main_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut off_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));

    let steel = Palette::STEEL_GLINT;
    let iron_mid = Palette::IRON_MID;
    let iron_dark = Palette::IRON_DARK;
    let gold = Palette::KNIGHT_GOLD;
    let crimson = Palette::CAPE_CRIMSON;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        let sp_h = &mut head[d_idx];
        sp_h.fill_rect(6, 3, 6, 6, iron_mid);
        sp_h.fill_rect(7, 2, 4, 1, steel);
        sp_h.fill_rect(4, 1, 2, 4, steel);
        sp_h.fill_rect(12, 1, 2, 4, steel);
        if d_idx != 1 {
            sp_h.fill_rect(7, 5, 4, 1, gold);
        }
        sp_h.add_outline(Palette::OUTLINE_DARK);

        let sp_t = &mut torso[d_idx];
        sp_t.fill_rect(5, 10, 8, 12, iron_mid);
        sp_t.fill_rect(4, 10, 2, 5, steel);
        sp_t.fill_rect(12, 10, 2, 5, steel);
        sp_t.fill_rect(7, 10, 4, 13, crimson);
        sp_t.set_pixel(8, 13, gold);
        sp_t.set_pixel(9, 13, gold);
        sp_t.set_pixel(8, 14, gold);
        sp_t.set_pixel(9, 14, gold);
        sp_t.add_outline(Palette::OUTLINE_DARK);

        let sp_c = &mut cape[d_idx];
        sp_c.fill_rect(4, 11, 10, 15, crimson);
        sp_c.fill_rect(4, 25, 10, 1, gold);
        sp_c.add_outline(Palette::OUTLINE_DARK);

        let sp_m = &mut main_hand[d_idx];
        let sx = match d_idx {
            1 => 3,
            _ => 14,
        };
        sp_m.fill_rect(sx, 7, 2, 17, steel);
        sp_m.fill_rect(sx - 1, 13, 4, 1, gold);
        sp_m.set_pixel(sx, 14, iron_dark);
        sp_m.set_pixel(sx, 15, gold);
        sp_m.add_outline(Palette::OUTLINE_DARK);

        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 13,
            _ => 2,
        };
        sp_o.fill_rect(ox, 11, 5, 8, iron_mid);
        sp_o.fill_rect(ox + 1, 12, 3, 6, crimson);
        sp_o.set_pixel(ox + 2, 14, gold);
        sp_o.add_outline(Palette::OUTLINE_DARK);
    }

    let mut lmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut rmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut hit_frame: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let death_frame = generate_base_body_death();

    for d in 0..4 {
        let dir = match d {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        for f in 0..4 {
            let mut sp_l = generate_base_body_lmb(dir, f);
            head[d].blit(&mut sp_l, 0, 0, false);
            torso[d].blit(&mut sp_l, 0, 0, false);
            if f == 1 {
                sp_l.fill_rect(12, 6, 5, 3, steel);
                sp_l.fill_rect(14, 9, 3, 4, steel);
            }
            lmb_frames[d][f] = sp_l;

            let mut sp_r = generate_base_body_rmb(dir, f);
            head[d].blit(&mut sp_r, 0, 0, false);
            torso[d].blit(&mut sp_r, 0, 0, false);
            if f == 1 {
                sp_r.fill_rect(14, 11, 3, 6, gold);
            }
            rmb_frames[d][f] = sp_r;
        }
        let mut sp_h = generate_base_body_hit(dir);
        head[d].blit(&mut sp_h, 0, 0, false);
        torso[d].blit(&mut sp_h, 0, 0, false);
        hit_frame[d] = sp_h;
    }

    ClassGearSet {
        head,
        torso,
        cape: [Some(cape[0].clone()), Some(cape[1].clone()), Some(cape[2].clone()), Some(cape[3].clone())],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
        lmb_frames,
        rmb_frames,
        hit_frame,
        death_frame,
    }
}

// -------------------------------------------------------------
// Class 3: ASSASSIN (Shadow Cowl, Red Sash, Dual Daggers)
// -------------------------------------------------------------
fn generate_assassin_gear_set() -> ClassGearSet {
    let mut head: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut torso: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut main_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut off_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));

    let shadow_dark = Color::rgb(14, 15, 20);
    let cowl_mid = Color::rgb(28, 30, 38);
    let red_glow = Palette::ASSASSIN_RED;
    let dagger_steel = Palette::STEEL_GLINT;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        let sp_h = &mut head[d_idx];
        sp_h.fill_rect(6, 2, 6, 7, cowl_mid);
        sp_h.fill_rect(7, 4, 4, 3, shadow_dark);
        if d_idx != 1 {
            sp_h.set_pixel(8, 5, red_glow);
            sp_h.set_pixel(10, 5, red_glow);
        }
        sp_h.add_outline(Palette::OUTLINE_DARK);

        let sp_t = &mut torso[d_idx];
        sp_t.fill_rect(5, 10, 8, 12, shadow_dark);
        sp_t.fill_rect(6, 17, 6, 2, red_glow);
        sp_t.add_outline(Palette::OUTLINE_DARK);

        let sp_m = &mut main_hand[d_idx];
        let mx = match d_idx {
            1 => 3,
            _ => 14,
        };
        sp_m.fill_rect(mx, 12, 2, 8, dagger_steel);
        sp_m.set_pixel(mx, 19, red_glow);
        sp_m.add_outline(Palette::OUTLINE_DARK);

        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 14,
            _ => 3,
        };
        sp_o.fill_rect(ox, 13, 2, 7, dagger_steel);
        sp_o.add_outline(Palette::OUTLINE_DARK);
    }

    let mut lmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut rmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut hit_frame: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let death_frame = generate_base_body_death();

    for d in 0..4 {
        let dir = match d {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        for f in 0..4 {
            let mut sp_l = generate_base_body_lmb(dir, f);
            head[d].blit(&mut sp_l, 0, 0, false);
            torso[d].blit(&mut sp_l, 0, 0, false);
            if f == 1 {
                sp_l.fill_rect(12, 10, 5, 2, red_glow);
                sp_l.fill_rect(13, 13, 4, 2, dagger_steel);
            }
            lmb_frames[d][f] = sp_l;

            let mut sp_r = generate_base_body_rmb(dir, f);
            head[d].blit(&mut sp_r, 0, 0, false);
            torso[d].blit(&mut sp_r, 0, 0, false);
            if f == 1 {
                sp_r.fill_rect(12, 9, 5, 7, Color::rgba(179, 45, 46, 140));
            }
            rmb_frames[d][f] = sp_r;
        }
        let mut sp_h = generate_base_body_hit(dir);
        head[d].blit(&mut sp_h, 0, 0, false);
        torso[d].blit(&mut sp_h, 0, 0, false);
        hit_frame[d] = sp_h;
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
        lmb_frames,
        rmb_frames,
        hit_frame,
        death_frame,
    }
}

// -------------------------------------------------------------
// Class 4: HUNTER (Tricorn Hat, Buff Duster, Pistol Swords)
// -------------------------------------------------------------
fn generate_hunter_gear_set() -> ClassGearSet {
    let mut head: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut torso: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut main_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut off_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));

    let leather = Color::rgb(45, 36, 26);
    let duster_buff = Color::rgb(75, 60, 44);
    let brass = Palette::HUNTER_BRASS;
    let blade = Palette::STEEL_GLINT;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        let sp_h = &mut head[d_idx];
        sp_h.fill_rect(4, 4, 10, 2, leather);
        sp_h.fill_rect(6, 2, 6, 2, leather);
        sp_h.set_pixel(11, 2, brass);
        sp_h.add_outline(Palette::OUTLINE_DARK);

        let sp_t = &mut torso[d_idx];
        sp_t.fill_rect(4, 10, 10, 15, duster_buff);
        sp_t.fill_rect(7, 10, 1, 15, brass);
        sp_t.fill_rect(10, 10, 1, 15, brass);
        sp_t.add_outline(Palette::OUTLINE_DARK);

        let sp_m = &mut main_hand[d_idx];
        let mx = match d_idx {
            1 => 3,
            _ => 14,
        };
        sp_m.fill_rect(mx, 8, 2, 14, blade);
        sp_m.set_pixel(mx + 1, 11, brass);
        sp_m.set_pixel(mx + 1, 12, brass);
        sp_m.add_outline(Palette::OUTLINE_DARK);

        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 14,
            _ => 3,
        };
        sp_o.fill_rect(ox, 10, 2, 12, blade);
        sp_o.set_pixel(ox - 1, 12, brass);
        sp_o.add_outline(Palette::OUTLINE_DARK);
    }

    let mut lmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut rmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut hit_frame: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let death_frame = generate_base_body_death();

    for d in 0..4 {
        let dir = match d {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        for f in 0..4 {
            let mut sp_l = generate_base_body_lmb(dir, f);
            head[d].blit(&mut sp_l, 0, 0, false);
            torso[d].blit(&mut sp_l, 0, 0, false);
            lmb_frames[d][f] = sp_l;

            let mut sp_r = generate_base_body_rmb(dir, f);
            head[d].blit(&mut sp_r, 0, 0, false);
            torso[d].blit(&mut sp_r, 0, 0, false);
            if f == 1 {
                sp_r.fill_rect(14, 8, 3, 3, Palette::BONFIRE_YELLOW);
                sp_r.set_pixel(15, 9, Palette::BONFIRE_CORE);
                sp_r.fill_rect(15, 6, 2, 2, Color::rgba(220, 225, 235, 180));
            }
            rmb_frames[d][f] = sp_r;
        }
        let mut sp_h = generate_base_body_hit(dir);
        head[d].blit(&mut sp_h, 0, 0, false);
        torso[d].blit(&mut sp_h, 0, 0, false);
        hit_frame[d] = sp_h;
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
        lmb_frames,
        rmb_frames,
        hit_frame,
        death_frame,
    }
}

// -------------------------------------------------------------
// Class 5: ALCHEMIST (Plague Beak Cowl, Vials, Arcane Eye)
// -------------------------------------------------------------
fn generate_alchemist_gear_set() -> ClassGearSet {
    let mut head: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut torso: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut accessory: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let mut off_hand: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));

    let leather = Color::rgb(42, 32, 24);
    let apron_dark = Color::rgb(28, 45, 38);
    let teal_glow = Palette::ALCHEMIST_TEAL;
    let eye_core = Color::rgb(190, 255, 240);
    let vial_red = Color::rgb(225, 45, 45);

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        let sp_h = &mut head[d_idx];
        sp_h.fill_rect(6, 2, 6, 7, leather);
        if d_idx != 1 {
            sp_h.fill_rect(8, 4, 2, 2, teal_glow);
            sp_h.set_pixel(9, 5, eye_core);
            sp_h.fill_rect(9, 7, 2, 3, Color::rgb(180, 165, 140));
        }
        sp_h.add_outline(Palette::OUTLINE_DARK);

        let sp_t = &mut torso[d_idx];
        sp_t.fill_rect(5, 10, 8, 14, apron_dark);
        sp_t.set_pixel(6, 17, teal_glow);
        sp_t.set_pixel(8, 17, vial_red);
        sp_t.set_pixel(11, 17, teal_glow);
        sp_t.add_outline(Palette::OUTLINE_DARK);

        let sp_a = &mut accessory[d_idx];
        let ex = match d_idx {
            1 => 4,
            2 => 14,
            _ => 15,
        };
        let ey = 9;
        sp_a.fill_rect(ex - 1, ey - 1, 3, 3, Color::rgb(180, 140, 70));
        sp_a.set_pixel(ex, ey, eye_core);
        sp_a.set_pixel(ex, ey - 1, teal_glow);
        sp_a.set_pixel(ex, ey + 1, teal_glow);
        sp_a.add_outline(Palette::OUTLINE_DARK);

        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 13,
            _ => 3,
        };
        sp_o.fill_rect(ox, 15, 3, 3, teal_glow);
        sp_o.set_pixel(ox + 1, 14, Color::rgb(190, 190, 210));
        sp_o.add_outline(Palette::OUTLINE_DARK);
    }

    let mut lmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut rmb_frames: [[Sprite; 4]; 4] = core::array::from_fn(|_| core::array::from_fn(|_| Sprite::new(18, 30)));
    let mut hit_frame: [Sprite; 4] = core::array::from_fn(|_| Sprite::new(18, 30));
    let death_frame = generate_base_body_death();

    for d in 0..4 {
        let dir = match d {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        for f in 0..4 {
            let mut sp_l = generate_base_body_lmb(dir, f);
            head[d].blit(&mut sp_l, 0, 0, false);
            torso[d].blit(&mut sp_l, 0, 0, false);
            if f == 1 {
                sp_l.fill_rect(13, 8, 5, 2, eye_core);
            }
            lmb_frames[d][f] = sp_l;

            let mut sp_r = generate_base_body_rmb(dir, f);
            head[d].blit(&mut sp_r, 0, 0, false);
            torso[d].blit(&mut sp_r, 0, 0, false);
            if f == 1 {
                sp_r.fill_rect(13, 12, 4, 4, teal_glow);
                sp_r.set_pixel(15, 14, eye_core);
            }
            rmb_frames[d][f] = sp_r;
        }
        let mut sp_h = generate_base_body_hit(dir);
        head[d].blit(&mut sp_h, 0, 0, false);
        torso[d].blit(&mut sp_h, 0, 0, false);
        hit_frame[d] = sp_h;
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [None, None, None, None],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [Some(accessory[0].clone()), Some(accessory[1].clone()), Some(accessory[2].clone()), Some(accessory[3].clone())],
        lmb_frames,
        rmb_frames,
        hit_frame,
        death_frame,
    }
}

// The Ashen Colosseum Props
// -------------------------------------------------------------
fn generate_colosseum_arch() -> Sprite {
    let w = 32;
    let h = 36;
    let mut sp = Sprite::new(w, h);
    let st_dark = Palette::STONE_COOL_DEEP;
    let st_mid = Palette::CAVEA_STONE_MID;
    let st_light = Palette::CAVEA_STONE_LIGHT;
    let st_highlight = Palette::STONE_WARM_LIGHT;
    let moss = Palette::MOSS_GREEN_MID;

    // Two tiers of Roman arches with hue-shifted masonry
    for y in 0..36 {
        for x in 0..32 {
            let is_outer_pillar = !(6..26).contains(&x);
            let is_inner_pillar = (14..=17).contains(&x);
            let is_pillar = is_outer_pillar || is_inner_pillar;
            let is_arch_lip = y == 0 || y == 1 || y == 17 || y == 18;

            if is_arch_lip {
                sp.pixels[y * w + x] = if x % 3 == 0 { st_highlight } else { st_light };
            } else if is_pillar {
                let col = if (x + y) % 6 == 0 {
                    st_highlight
                } else if (x + y) % 3 == 0 {
                    st_light
                } else {
                    st_mid
                };
                sp.pixels[y * w + x] = col;
            } else {
                // Interior dark arch void
                sp.pixels[y * w + x] = st_dark;
            }
        }
    }

    // Moss / Ivy clinging to outer pillars and arch lips
    for y in 2..16 {
        if y % 3 == 0 {
            sp.pixels[y * w + 5] = moss;
            sp.pixels[(y + 1) * w + 6] = Palette::MOSS_GREEN_DARK;
            sp.pixels[y * w + 26] = moss;
        }
    }

    sp
}

fn generate_broken_column() -> Sprite {
    let w = 12;
    let h = 26;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::STONE_COOL_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::STONE_WARM_LIGHT;
    let moss = Palette::MOSS_GREEN_MID;

    // 1px dark silhouette outline base
    for x in 0..12 {
        sp.pixels[25 * w + x] = dark;
    }

    // Base capital
    for x in 1..11 {
        sp.pixels[23 * w + x] = mid;
        sp.pixels[24 * w + x] = light;
    }
    // Fluted shaft with shattered top
    for y in 6..23 {
        for x in 2..10 {
            let col = match (x - 2) % 4 {
                0 => dark,
                1 => mid,
                2 => light,
                _ => mid,
            };
            sp.pixels[y * w + x] = col;
        }
    }
    // Broken jagged top with highlights
    for x in 2..8 {
        let top_y = 6 - (x % 3);
        sp.pixels[top_y * w + x] = light;
        sp.pixels[(top_y + 1) * w + x] = mid;
    }

    // Moss patch at cracked base
    sp.pixels[22 * w + 3] = moss;
    sp.pixels[23 * w + 3] = moss;
    sp.pixels[22 * w + 4] = Palette::MOSS_GREEN_LIGHT;

    sp
}

fn generate_gate_of_trials() -> Sprite {
    let w = 32;
    let h = 34;
    let mut sp = Sprite::new(w, h);
    let stone = Palette::CAVEA_STONE_MID;
    let iron = Palette::IRON_DARK;
    let rust = Color::rgb(85, 45, 30);

    // Archway surround
    for y in 0..34 {
        for x in 0..32 {
            if !(4..=27).contains(&x) || y < 4 {
                sp.pixels[y * w + x] = stone;
            } else {
                // Iron Portcullis Grate
                let is_bar_x = x % 4 == 0;
                let is_bar_y = y % 5 == 0;
                if is_bar_x || is_bar_y {
                    sp.pixels[y * w + x] = if (x + y) % 3 == 0 { rust } else { iron };
                } else {
                    sp.pixels[y * w + x] = Palette::VOID_BLACK;
                }
            }
        }
    }
    sp
}

fn generate_weapon_rack() -> Sprite {
    let w = 18;
    let h = 20;
    let mut sp = Sprite::new(w, h);
    let wood = Color::rgb(55, 40, 25);
    let steel = Palette::STEEL_GLINT;

    // Wooden A-frame rack
    for y in 8..20 {
        sp.pixels[y * w + 2] = wood;
        sp.pixels[y * w + 15] = wood;
    }
    for x in 2..16 {
        sp.pixels[10 * w + x] = wood;
        sp.pixels[16 * w + x] = wood;
    }
    // Racked spears and gladii
    for (i, &sx) in [5, 8, 11, 13].iter().enumerate() {
        let top = 2 + (i % 2) * 2;
        for y in top..18 {
            sp.pixels[y * w + sx] = steel;
        }
    }
    sp
}

fn generate_training_dummy() -> Sprite {
    let w = 12;
    let h = 20;
    let mut sp = Sprite::new(w, h);
    let wood = Color::rgb(60, 44, 30);
    let sack = Color::rgb(110, 95, 70);
    let cut = Color::rgb(35, 12, 12);

    // Central pole
    for y in 10..20 {
        sp.pixels[y * w + 5] = wood;
        sp.pixels[y * w + 6] = wood;
    }
    // Crossbar arms
    for x in 1..11 {
        sp.pixels[8 * w + x] = wood;
    }
    // Burlap torso
    for y in 5..14 {
        for x in 3..9 {
            sp.pixels[y * w + x] = sack;
        }
    }
    // Straw head
    for y in 1..5 {
        for x in 4..8 {
            sp.pixels[y * w + x] = sack;
        }
    }
    // Slash marks
    sp.pixels[7 * w + 5] = cut;
    sp.pixels[8 * w + 6] = cut;
    sp.pixels[9 * w + 7] = cut;
    sp
}

fn generate_spectator_stand() -> Sprite {
    let w = 28;
    let h = 18;
    let mut sp = Sprite::new(w, h);
    let stone = Palette::CAVEA_STONE_DARK;
    let silhouette = Palette::SPECTATOR_SILHOUETTE;
    let eye = Palette::SPECTATOR_EYE_GLOW;

    // Tiered stone steps
    for y in 0..18 {
        for x in 0..28 {
            sp.pixels[y * w + x] = stone;
        }
    }

    // 3 Hooded hollow spectators in rows
    let spectators = [(4, 2), (13, 2), (21, 2), (8, 9), (17, 9)];
    for &(sx, sy) in &spectators {
        for dy in 0..5 {
            for dx in 0..4 {
                sp.pixels[(sy + dy) * w + (sx + dx)] = silhouette;
            }
        }
        // Faint glowing eyes
        sp.pixels[(sy + 2) * w + sx + 1] = eye;
        sp.pixels[(sy + 2) * w + sx + 2] = eye;
    }

    sp
}

fn generate_tattered_banner(frame: usize) -> Sprite {
    let w = 14;
    let h = 30;
    let mut sp = Sprite::new(w, h);
    let pole = Color::rgb(55, 45, 35);
    let banner_red = Color::rgb(85, 20, 20);
    let gold_rune = Palette::GOLD_TRIM;

    // Pole
    for y in 0..30 {
        sp.pixels[y * w + 2] = pole;
    }
    // Crossbar
    for x in 2..13 {
        sp.pixels[3 * w + x] = pole;
    }

    // Swaying cloth
    let sway = if frame == 0 { 0 } else { 1 };
    for y in 4..25 {
        let x_end = 12 - (y / 10) + sway;
        for x in 3..x_end {
            sp.pixels[y * w + x] = if (x + y) % 6 == 0 { gold_rune } else { banner_red };
        }
    }
    sp
}

fn generate_bone_pile() -> Sprite {
    let w = 12;
    let h = 10;
    let mut sp = Sprite::new(w, h);
    let bone = Palette::BONE_WHITE;
    let blood = Palette::BLOOD_STAIN;

    for y in 4..10 {
        for x in 2..10 {
            if (x + y) % 2 == 0 {
                sp.pixels[y * w + x] = bone;
            } else {
                sp.pixels[y * w + x] = blood;
            }
        }
    }
    // Skull hint
    sp.pixels[3 * w + 5] = bone;
    sp.pixels[3 * w + 6] = bone;
    sp.pixels[4 * w + 5] = Palette::VOID_BLACK; // eye socket
    sp.pixels[4 * w + 6] = Palette::VOID_BLACK;
    sp
}

fn generate_headless_gladiator() -> Sprite {
    let w = 16;
    let h = 28;
    let mut sp = Sprite::new(w, h);

    let outline = Color::rgb(10, 10, 14);
    let plinth_dark = Palette::STONE_COOL_DARK;
    let plinth_mid = Palette::STONE_MID;
    let plinth_light = Palette::STONE_LIGHT;
    let bronze_dark = Palette::BRONZE_STATUE_DARK;
    let bronze_mid = Palette::BRONZE_STATUE_MID;
    let bronze_light = Palette::BRONZE_STATUE_LIGHT;
    let moss = Palette::MOSS_GREEN_MID;

    // 1. Plinth / Stone Pedestal (bottom 6 rows)
    for y in 22..28 {
        for x in 1..15 {
            let col = if y == 22 {
                plinth_light
            } else if y == 27 || x == 1 || x == 14 {
                plinth_dark
            } else if (x + y) % 3 == 0 {
                plinth_light
            } else {
                plinth_mid
            };
            sp.pixels[y * w + x] = col;
        }
    }
    // Moss patch on plinth
    sp.pixels[26 * w + 2] = moss;
    sp.pixels[25 * w + 3] = moss;

    // 2. Dark outline for statue silhouette
    for y in 4..22 {
        for x in 3..13 {
            sp.pixels[y * w + x] = outline;
        }
    }

    // 3. Gladiator legs & armored greaves
    for y in 15..22 {
        for x in 4..7 {
            sp.pixels[y * w + x] = bronze_dark;
        }
        for x in 9..12 {
            sp.pixels[y * w + x] = bronze_mid;
        }
    }

    // 4. Gladiator armored tunic & chest plate
    for y in 6..15 {
        for x in 4..12 {
            let tone = if x < 6 {
                bronze_dark
            } else if x < 9 {
                bronze_mid
            } else {
                bronze_light
            };
            sp.pixels[y * w + x] = tone;
        }
    }

    // 5. Left arm holding round buckler shield
    for y in 8..15 {
        for x in 2..5 {
            sp.pixels[y * w + x] = if (x + y) % 2 == 0 { bronze_light } else { bronze_mid };
        }
    }

    // 6. Right arm holding broken gladius hilt
    sp.pixels[9 * w + 12] = bronze_mid;
    sp.pixels[10 * w + 13] = bronze_light;
    sp.pixels[11 * w + 13] = Palette::IRON_LIGHT;

    // 7. Severed jagged neck at top
    for x in 6..10 {
        sp.pixels[4 * w + x] = Palette::STONE_COOL_DEEP;
        sp.pixels[5 * w + x] = bronze_dark;
    }

    // Cool rim light on left side
    for y in 6..22 {
        sp.pixels[y * w + 3] = Palette::ARMOR_HIGHLIGHT;
    }

    sp
}

fn generate_hanging_cage() -> Sprite {
    let w = 14;
    let h = 24;
    let mut sp = Sprite::new(w, h);

    let iron_dark = Palette::CAGE_IRON_DARK;
    let iron_bar = Palette::CAGE_IRON_BAR;
    let bone = Palette::BONE_WHITE;

    // Hanging chain / rope from ceiling
    for y in 0..5 {
        sp.pixels[y * w + 6] = iron_dark;
        sp.pixels[y * w + 7] = if y % 2 == 0 { iron_bar } else { iron_dark };
    }

    // Cage roof dome
    for x in 3..11 {
        sp.pixels[5 * w + x] = iron_bar;
    }
    for x in 2..12 {
        sp.pixels[6 * w + x] = iron_dark;
    }

    // Vertical bars & interior void
    for y in 7..19 {
        sp.pixels[y * w + 2] = iron_bar;
        sp.pixels[y * w + 5] = iron_bar;
        sp.pixels[y * w + 8] = iron_bar;
        sp.pixels[y * w + 11] = iron_bar;
    }

    // Skeletal remains inside
    sp.pixels[12 * w + 6] = bone;
    sp.pixels[12 * w + 7] = bone;
    sp.pixels[13 * w + 6] = Palette::VOID_BLACK; // Eye socket
    sp.pixels[13 * w + 7] = Palette::VOID_BLACK;
    sp.pixels[14 * w + 6] = bone;
    sp.pixels[16 * w + 6] = bone;
    sp.pixels[16 * w + 7] = bone;

    // Cage base rim
    for x in 2..12 {
        sp.pixels[19 * w + x] = iron_dark;
        sp.pixels[20 * w + x] = iron_bar;
    }

    // Bottom hook / spike
    sp.pixels[21 * w + 6] = iron_dark;
    sp.pixels[22 * w + 6] = iron_dark;

    sp
}

fn generate_wall_torch(frame: usize) -> Sprite {
    let w = 8;
    let h = 14;
    let mut sp = Sprite::new(w, h);

    let iron = Palette::IRON_DARK;
    let wood = Palette::TORCH_WOOD;
    let (core, flame, ember) = (Palette::TORCH_CORE, Palette::TORCH_FLAME, Palette::TORCH_EMBER);

    // Wall bracket mounting plate
    for y in 7..12 {
        sp.pixels[y * w + 1] = iron;
    }
    // Angled sconce arm
    sp.pixels[9 * w + 2] = iron;
    sp.pixels[8 * w + 3] = iron;
    sp.pixels[7 * w + 4] = iron;

    // Wooden torch shaft
    for y in 5..10 {
        sp.pixels[y * w + 4] = wood;
    }

    // Flickering torch flame (2 frames)
    if frame == 0 {
        sp.pixels[4 * w + 4] = core;
        sp.pixels[3 * w + 4] = core;
        sp.pixels[4 * w + 3] = flame;
        sp.pixels[4 * w + 5] = flame;
        sp.pixels[2 * w + 4] = flame;
        sp.pixels[2 * w + 3] = ember;
        sp.pixels[w + 4] = ember;
    } else {
        sp.pixels[4 * w + 4] = core;
        sp.pixels[3 * w + 4] = core;
        sp.pixels[4 * w + 3] = flame;
        sp.pixels[3 * w + 5] = flame;
        sp.pixels[2 * w + 4] = flame;
        sp.pixels[2 * w + 5] = ember;
        sp.pixels[w + 3] = ember;
    }

    sp
}

fn generate_fallen_arch_rubble() -> Sprite {
    let w = 20;
    let h = 12;
    let mut sp = Sprite::new(w, h);

    let dark = Palette::STONE_COOL_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::STONE_WARM_LIGHT;
    let moss = Palette::MOSS_GREEN_MID;

    // Rubble heap silhouette
    for y in 4..12 {
        for x in 1..19 {
            if y == 11 || x == 1 || x == 18 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 4 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Fractured wedge voussoir blocks
    for y in 2..8 {
        for x in 5..10 {
            sp.pixels[y * w + x] = if (x + y) % 2 == 0 { light } else { mid };
        }
    }
    for y in 3..9 {
        for x in 11..16 {
            sp.pixels[y * w + x] = if x % 2 == 0 { mid } else { dark };
        }
    }

    // Moss creeping on stones
    sp.pixels[6 * w + 4] = moss;
    sp.pixels[7 * w + 4] = moss;
    sp.pixels[8 * w + 15] = moss;
    sp.pixels[9 * w + 16] = Palette::MOSS_GREEN_LIGHT;

    sp
}

fn generate_chapel_altar() -> Sprite {
    let w = 30;
    let h = 22;
    let mut sp = Sprite::new(w, h);
    let stone_dark = Palette::STONE_COOL_DARK;
    let stone_mid = Palette::CAVEA_STONE_MID;
    let stone_light = Palette::STONE_WARM_LIGHT;
    let cloth = Palette::ASH_WHITE;
    let blood = Palette::BLOOD_STAIN;
    let gold = Palette::GOLD_TRIM;

    // Altar stone base plinth (steps)
    for y in 14..22 {
        for x in 2..28 {
            if y == 21 || x == 2 || x == 27 {
                sp.pixels[y * w + x] = stone_dark;
            } else if y >= 18 {
                sp.pixels[y * w + x] = if (x + y) % 3 == 0 { stone_light } else { stone_mid };
            } else {
                sp.pixels[y * w + x] = stone_mid;
            }
        }
    }

    // Carved gothic arch niches in plinth
    for y in 15..19 {
        for x in [6, 7, 14, 15, 22, 23] {
            sp.pixels[y * w + x] = stone_dark;
        }
    }

    // Altar top slab
    for y in 8..14 {
        for x in 1..29 {
            if y == 8 || y == 13 || x == 1 || x == 28 {
                sp.pixels[y * w + x] = stone_dark;
            } else {
                sp.pixels[y * w + x] = stone_mid;
            }
        }
    }

    // Desecrated Altar Cloth runner in center with blood stains
    for y in 7..16 {
        for x in 9..21 {
            if x == 9 || x == 20 {
                sp.pixels[y * w + x] = gold;
            } else if (y > 9 && (x + y) % 4 == 0) || (y == 11 && (11..18).contains(&x)) {
                sp.pixels[y * w + x] = blood;
            } else {
                sp.pixels[y * w + x] = cloth;
            }
        }
    }

    // Chalice and Candles on top
    // Chalice (center)
    sp.pixels[5 * w + 14] = gold;
    sp.pixels[5 * w + 15] = gold;
    sp.pixels[6 * w + 14] = gold;
    sp.pixels[6 * w + 15] = gold;
    sp.pixels[7 * w + 14] = gold;
    sp.pixels[7 * w + 15] = gold;

    // Left Candle with flame
    sp.pixels[3 * w + 5] = Palette::TORCH_FLAME;
    sp.pixels[4 * w + 5] = Palette::TORCH_CORE;
    sp.pixels[5 * w + 5] = cloth;
    sp.pixels[6 * w + 5] = cloth;
    sp.pixels[7 * w + 5] = stone_dark;

    // Right Candle with flame
    sp.pixels[3 * w + 24] = Palette::TORCH_FLAME;
    sp.pixels[4 * w + 24] = Palette::TORCH_CORE;
    sp.pixels[5 * w + 24] = cloth;
    sp.pixels[6 * w + 24] = cloth;
    sp.pixels[7 * w + 24] = stone_dark;

    sp
}

fn generate_chapel_pew() -> Sprite {
    let w = 24;
    let h = 16;
    let mut sp = Sprite::new(w, h);
    let wood_dark = Color::rgb(38, 25, 18);
    let wood_mid = Color::rgb(64, 44, 30);
    let wood_light = Color::rgb(95, 68, 48);
    let moss = Palette::MOSS_GREEN_MID;

    // End panels (carved trefoil gothic silhouette)
    for y in 4..16 {
        for x in [2, 3, 20, 21] {
            if y == 15 || y == 4 || x == 2 || x == 21 {
                sp.pixels[y * w + x] = wood_dark;
            } else {
                sp.pixels[y * w + x] = wood_mid;
            }
        }
    }
    sp.pixels[3 * w + 3] = wood_light;
    sp.pixels[3 * w + 20] = wood_light;

    // Bench seat and backrest planks
    for y in 6..12 {
        for x in 4..20 {
            if y == 6 || y == 11 {
                sp.pixels[y * w + x] = wood_dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = wood_light;
            } else {
                sp.pixels[y * w + x] = wood_mid;
            }
        }
    }

    // Fractured wood splinter gap
    sp.pixels[8 * w + 11] = wood_dark;
    sp.pixels[9 * w + 11] = wood_dark;
    sp.pixels[9 * w + 12] = Palette::VOID_BLACK;

    // Moss on foot
    sp.pixels[14 * w + 4] = moss;
    sp.pixels[15 * w + 4] = moss;
    sp.pixels[15 * w + 5] = Palette::MOSS_GREEN_LIGHT;

    sp
}

fn generate_grave_stone() -> Sprite {
    let w = 16;
    let h = 22;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::STONE_COOL_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::STONE_WARM_LIGHT;
    let crack = Palette::STONE_CRACK;
    let moss = Palette::MOSS_GREEN_MID;

    // Arched tombstone outline
    for y in 4..22 {
        for x in 2..14 {
            let in_arch = if y < 8 {
                (4..=11).contains(&x) || (y >= 6 && (3..=12).contains(&x))
            } else {
                true
            };

            if in_arch {
                if y == 21 || x == 2 || x == 13 || (y == 4 && (5..=10).contains(&x)) {
                    sp.pixels[y * w + x] = dark;
                } else if (x + y) % 4 == 0 {
                    sp.pixels[y * w + x] = light;
                } else {
                    sp.pixels[y * w + x] = mid;
                }
            }
        }
    }

    // Inset carved cross
    for y in 8..15 {
        sp.pixels[y * w + 8] = crack;
    }
    for x in 6..11 {
        sp.pixels[10 * w + x] = crack;
    }

    // Weathered diagonal crack
    sp.pixels[15 * w + 9] = crack;
    sp.pixels[16 * w + 10] = crack;
    sp.pixels[17 * w + 11] = crack;

    // Moss at ground line
    sp.pixels[20 * w + 3] = moss;
    sp.pixels[21 * w + 3] = moss;
    sp.pixels[21 * w + 4] = Palette::MOSS_GREEN_LIGHT;
    sp.pixels[20 * w + 12] = moss;
    sp.pixels[21 * w + 12] = moss;

    sp
}

fn generate_rotting_corpse() -> Sprite {
    let w = 24;
    let h = 14;
    let mut sp = Sprite::new(w, h);
    let blood_dark = Palette::BLOOD_STAIN;
    let blood_mid = Color::rgb(72, 16, 20);
    let cloth_dark = Color::rgb(42, 38, 48);
    let cloth_mid = Color::rgb(65, 58, 70);
    let bone = Palette::BONE_WHITE;
    let rot = Color::rgb(48, 58, 40);

    // Dried blood pool spreading on flagstones
    for y in 5..14 {
        for x in 1..23 {
            let dx = (x as i32 - 12).abs();
            let dy = (y as i32 - 9).abs();
            if dx * dx + dy * dy * 3 < 70 {
                sp.pixels[y * w + x] = if (x + y) % 2 == 0 { blood_dark } else { blood_mid };
            }
        }
    }

    // Prone torso in tattered tunic
    for y in 4..10 {
        for x in 7..17 {
            if y == 4 || y == 9 || x == 7 || x == 16 {
                sp.pixels[y * w + x] = cloth_dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = rot;
            } else {
                sp.pixels[y * w + x] = cloth_mid;
            }
        }
    }

    // Exposed ribcage bones
    sp.pixels[5 * w + 10] = bone;
    sp.pixels[6 * w + 10] = bone;
    sp.pixels[5 * w + 13] = bone;
    sp.pixels[6 * w + 13] = bone;

    // Fallen skull on left
    for y in 5..9 {
        for x in 3..7 {
            sp.pixels[y * w + x] = bone;
        }
    }
    sp.pixels[6 * w + 4] = Palette::VOID_BLACK; // eye socket
    sp.pixels[6 * w + 6] = Palette::VOID_BLACK;
    sp.pixels[8 * w + 5] = Palette::VOID_BLACK; // jaw crack

    // Splayed legs on right
    sp.pixels[7 * w + 18] = cloth_dark;
    sp.pixels[7 * w + 19] = cloth_mid;
    sp.pixels[8 * w + 20] = bone;
    sp.pixels[9 * w + 18] = cloth_dark;
    sp.pixels[10 * w + 19] = bone;

    sp
}

fn generate_chapel_arch() -> Sprite {
    let w = 32;
    let h = 38;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::STONE_COOL_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::STONE_COOL_LIGHT;
    let moss = Palette::MOSS_GREEN_MID;

    // Twin stone piers on left and right
    for y in 0..38 {
        for x in 0..6 {
            if x == 0 || x == 5 || y == 37 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
        for x in 26..32 {
            if x == 26 || x == 31 || y == 37 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Pointed gothic arch curve overhead
    for x in 6..26 {
        let dist_from_center = (x as i32 - 16).abs();
        let arch_y = (dist_from_center * 14) / 10;
        for y in (arch_y as usize)..=(arch_y as usize + 6).min(37) {
            if y == arch_y as usize || y == arch_y as usize + 6 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 2 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Gothic trefoil cusps hanging in arch
    sp.pixels[9 * w + 15] = light;
    sp.pixels[9 * w + 16] = light;
    sp.pixels[10 * w + 15] = dark;
    sp.pixels[10 * w + 16] = dark;

    // Moss on pier bases
    sp.pixels[35 * w + 3] = moss;
    sp.pixels[36 * w + 3] = moss;
    sp.pixels[35 * w + 28] = moss;

    sp
}

fn generate_autumn_tree() -> Sprite {
    let w = 34;
    let h = 42;
    let mut sp = Sprite::new(w, h);
    let bark_dark = Color::rgb(35, 30, 32);
    let bark_mid = Color::rgb(145, 150, 158); // pale silver birch
    let bark_light = Color::rgb(195, 200, 208);
    let knot = Color::rgb(48, 38, 36);

    // Autumn leaf palette
    let leaf_amber = Color::rgb(240, 175, 55);
    let leaf_orange = Color::rgb(215, 115, 35);
    let leaf_rust = Color::rgb(175, 65, 25);
    let leaf_dark = Color::rgb(105, 38, 20);

    // Slender birch trunk
    for y in 14..42 {
        let cx = 17;
        let trunk_w = if y > 36 { 3 } else { 2 };
        for x in (cx - trunk_w)..=(cx + trunk_w) {
            if x == cx - trunk_w || x == cx + trunk_w || y == 41 {
                sp.pixels[y * w + x] = bark_dark;
            } else if (y % 6 == 0) && x == cx {
                sp.pixels[y * w + x] = knot;
            } else if x == cx - trunk_w + 1 {
                sp.pixels[y * w + x] = bark_light;
            } else {
                sp.pixels[y * w + x] = bark_mid;
            }
        }
    }

    // Spreading branches
    for (bx, by) in [(13, 20), (12, 19), (11, 18), (21, 22), (22, 21), (23, 20)] {
        sp.pixels[by * w + bx] = bark_dark;
    }

    // Autumn foliage clusters (golden rust fall canopy)
    for y in 2..22 {
        for x in 3..31 {
            let dx = (x as i32 - 17).abs();
            let dy = (y as i32 - 12).abs();
            let d = dx * dx + dy * dy * 2;
            if d < 130 {
                let noise = ((x * 13 + y * 29) ^ (x * 7)) % 7;
                let col = if d > 105 || noise == 0 {
                    leaf_dark
                } else if noise <= 2 {
                    leaf_rust
                } else if noise <= 4 {
                    leaf_orange
                } else {
                    leaf_amber
                };
                sp.pixels[y * w + x] = col;
            }
        }
    }

    sp
}

fn generate_fallen_log() -> Sprite {
    let w = 26;
    let h = 14;
    let mut sp = Sprite::new(w, h);
    let bark_dark = Color::rgb(34, 24, 18);
    let bark_mid = Color::rgb(58, 42, 30);
    let bark_light = Color::rgb(88, 64, 46);
    let moss = Palette::MOSS_GREEN_MID;
    let shroom = Color::rgb(220, 145, 55);

    // Horizontal log body
    for y in 4..14 {
        for x in 2..24 {
            if y == 4 || y == 13 || x == 2 || x == 23 {
                sp.pixels[y * w + x] = bark_dark;
            } else if y <= 7 {
                sp.pixels[y * w + x] = if (x + y) % 3 == 0 { bark_light } else { bark_mid };
            } else {
                sp.pixels[y * w + x] = bark_mid;
            }
        }
    }

    // Hollow decayed end on left
    for y in 6..12 {
        for x in 3..6 {
            sp.pixels[y * w + x] = Palette::VOID_BLACK;
        }
    }

    // Moss along upper rim
    sp.pixels[5 * w + 8] = moss;
    sp.pixels[5 * w + 9] = moss;
    sp.pixels[5 * w + 15] = Palette::MOSS_GREEN_LIGHT;
    sp.pixels[5 * w + 16] = moss;

    // Small wild bracket mushrooms
    sp.pixels[7 * w + 18] = shroom;
    sp.pixels[7 * w + 19] = shroom;
    sp.pixels[8 * w + 19] = bark_dark;

    sp
}

fn generate_broken_cart() -> Sprite {
    let w = 26;
    let h = 20;
    let mut sp = Sprite::new(w, h);
    let wood_dark = Color::rgb(42, 28, 20);
    let wood_mid = Color::rgb(72, 50, 36);
    let wood_light = Color::rgb(105, 75, 54);
    let iron = Palette::IRON_MID;

    // Wooden cart body (tilted / wrecked)
    for y in 6..14 {
        for x in 4..20 {
            if y == 6 || y == 13 || x == 4 || x == 19 {
                sp.pixels[y * w + x] = wood_dark;
            } else if (x + y) % 2 == 0 {
                sp.pixels[y * w + x] = wood_light;
            } else {
                sp.pixels[y * w + x] = wood_mid;
            }
        }
    }

    // Shattered spoked wheel lying on side
    for y in 10..19 {
        for x in 16..25 {
            let dx = x as i32 - 20;
            let dy = y as i32 - 14;
            let d = dx * dx + dy * dy;
            if (12..=18).contains(&d) {
                sp.pixels[y * w + x] = iron; // iron wheel rim
            } else if d <= 3 || (dx.abs() == 0 && d <= 12) || (dy.abs() == 0 && d <= 12) {
                sp.pixels[y * w + x] = wood_dark; // spokes and hub
            }
        }
    }

    // Left cart axle post resting on ground
    for y in 14..19 {
        sp.pixels[y * w + 6] = wood_dark;
        sp.pixels[y * w + 7] = wood_mid;
    }

    sp
}

fn generate_wooden_fence() -> Sprite {
    let w = 22;
    let h = 16;
    let mut sp = Sprite::new(w, h);
    let wood_dark = Color::rgb(36, 26, 18);
    let wood_mid = Color::rgb(62, 45, 32);
    let wood_light = Color::rgb(90, 68, 50);
    let moss = Palette::MOSS_GREEN_MID;

    // Vertical posts at x=3,4 and x=18,19
    for y in 2..16 {
        for x in [3, 4, 18, 19] {
            if y == 2 || y == 15 || x == 3 || x == 18 {
                sp.pixels[y * w + x] = wood_dark;
            } else {
                sp.pixels[y * w + x] = wood_mid;
            }
        }
    }

    // Horizontal split rails
    for x in 4..19 {
        // Upper rail
        sp.pixels[5 * w + x] = wood_dark;
        sp.pixels[6 * w + x] = wood_light;
        sp.pixels[7 * w + x] = wood_dark;

        // Lower rail
        sp.pixels[10 * w + x] = wood_dark;
        sp.pixels[11 * w + x] = wood_mid;
        sp.pixels[12 * w + x] = wood_dark;
    }

    sp.pixels[14 * w + 5] = moss;
    sp.pixels[15 * w + 5] = moss;

    sp
}

fn generate_gnarled_tree() -> Sprite {
    let w = 38;
    let h = 46;
    let mut sp = Sprite::new(w, h);
    let dark = Color::rgb(22, 24, 28);
    let mid = Color::rgb(42, 45, 52);
    let light = Color::rgb(65, 70, 80);
    let moss = Color::rgb(35, 58, 42);

    // Thick twisted trunk
    for y in 16..46 {
        let cx = 19;
        let base_expand = if y > 38 { (y - 38) as i32 } else { 0 };
        let trunk_w = 4 + base_expand;
        for x in ((cx - trunk_w) as usize)..=((cx + trunk_w) as usize) {
            if x == (cx - trunk_w) as usize || x == (cx + trunk_w) as usize || y == 45 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Serpentine root flares
    for y in 38..46 {
        sp.pixels[y * w + 11] = dark;
        sp.pixels[y * w + 12] = mid;
        sp.pixels[y * w + 26] = mid;
        sp.pixels[y * w + 27] = dark;
    }

    // Heavy gnarled canopy & twisted boughs
    for y in 2..24 {
        for x in 2..36 {
            let dx = (x as i32 - 19).abs();
            let dy = (y as i32 - 13).abs();
            let d = dx * dx + dy * dy * 2;
            if d < 180 {
                let is_edge = d > 145;
                if is_edge {
                    sp.pixels[y * w + x] = dark;
                } else if (x * 7 + y * 13) % 5 == 0 {
                    sp.pixels[y * w + x] = moss;
                } else if (x + y) % 3 == 0 {
                    sp.pixels[y * w + x] = light;
                } else {
                    sp.pixels[y * w + x] = mid;
                }
            }
        }
    }

    // Hanging Spanish moss tendrils
    for y in 24..32 {
        sp.pixels[y * w + 8] = moss;
        sp.pixels[y * w + 14] = moss;
        sp.pixels[y * w + 24] = moss;
        sp.pixels[y * w + 30] = moss;
    }

    sp
}

fn generate_glowing_mushrooms(frame: usize) -> Sprite {
    let w = 20;
    let h = 18;
    let mut sp = Sprite::new(w, h);
    let stalk_dark = Color::rgb(35, 42, 48);
    let stalk_mid = Color::rgb(75, 88, 98);
    let cap_cyan = Color::rgb(65, 230, 175);
    let cap_teal = Color::rgb(40, 180, 240);
    let cap_glow = Color::rgb(140, 255, 220);

    // Stalks
    for y in 8..18 {
        // Large center mushroom stalk
        sp.pixels[y * w + 10] = stalk_dark;
        sp.pixels[y * w + 11] = stalk_mid;

        // Left small stalk
        if y >= 11 {
            sp.pixels[y * w + 5] = stalk_dark;
            sp.pixels[y * w + 6] = stalk_mid;
        }

        // Right small stalk
        if y >= 12 {
            sp.pixels[y * w + 15] = stalk_dark;
            sp.pixels[y * w + 16] = stalk_mid;
        }
    }

    // Center Large Cap
    for y in 3..9 {
        for x in 6..16 {
            let dx = (x as i32 - 11).abs();
            let dy = (y as i32 - 6).abs();
            if dx * dx + dy * dy * 2 <= 22 {
                sp.pixels[y * w + x] = if (x + y + frame).is_multiple_of(3) { cap_glow } else { cap_cyan };
            }
        }
    }

    // Left Small Cap
    for y in 7..12 {
        for x in 3..9 {
            let dx = (x as i32 - 6).abs();
            if dx <= 2 && y <= 10 {
                sp.pixels[y * w + x] = if (x + frame).is_multiple_of(2) { cap_glow } else { cap_teal };
            }
        }
    }

    // Right Small Cap
    for y in 8..13 {
        for x in 13..19 {
            let dx = (x as i32 - 16).abs();
            if dx <= 2 && y <= 11 {
                sp.pixels[y * w + x] = if (y + frame).is_multiple_of(2) { cap_glow } else { cap_cyan };
            }
        }
    }

    sp
}

fn generate_root_briar() -> Sprite {
    let w = 24;
    let h = 18;
    let mut sp = Sprite::new(w, h);
    let dark = Color::rgb(28, 22, 20);
    let mid = Color::rgb(55, 42, 34);
    let light = Color::rgb(82, 64, 50);
    let thorn = Color::rgb(110, 40, 35);

    // Tangled root loops
    for y in 4..18 {
        for x in 2..22 {
            let pattern = ((x as i32 * 5 - y as i32 * 3).abs() % 7) == 0
                || ((x as i32 * 3 + y as i32 * 4) % 9) == 0;
            if pattern && y >= 6 {
                sp.pixels[y * w + x] = if (x + y) % 3 == 0 { light } else { mid };
            }
        }
    }

    // Outline and base shadow
    for x in 2..22 {
        sp.pixels[17 * w + x] = dark;
    }

    // Sharp thorns
    for (tx, ty) in [(6, 8), (11, 5), (15, 9), (18, 6)] {
        sp.pixels[ty * w + tx] = thorn;
    }

    sp
}

fn generate_rotten_stump() -> Sprite {
    let w = 18;
    let h = 16;
    let mut sp = Sprite::new(w, h);
    let dark = Color::rgb(30, 24, 22);
    let mid = Color::rgb(54, 42, 35);
    let light = Color::rgb(80, 65, 52);
    let shroom = Color::rgb(210, 140, 60);

    // Fluted stump trunk
    for y in 4..16 {
        for x in 2..16 {
            let is_edge = x == 2 || x == 15 || y == 15;
            if is_edge {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Hollow center cavity
    for y in 4..8 {
        for x in 6..12 {
            sp.pixels[y * w + x] = Palette::VOID_BLACK;
        }
    }

    // Bracket shelf mushrooms
    sp.pixels[9 * w + 14] = shroom;
    sp.pixels[9 * w + 15] = shroom;
    sp.pixels[12 * w + 3] = shroom;

    sp
}

fn generate_castle_pillar() -> Sprite {
    let w = 20;
    let h = 40;
    let mut sp = Sprite::new(w, h);
    let dark = Color::rgb(26, 28, 36);
    let mid = Color::rgb(52, 56, 70);
    let light = Color::rgb(88, 95, 118);
    let gold = Palette::GOLD_TRIM;

    // Base plinth
    for y in 34..40 {
        for x in 1..19 {
            if y == 39 || x == 1 || x == 18 {
                sp.pixels[y * w + x] = dark;
            } else if y == 34 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Fluted vertical shaft
    for y in 8..34 {
        for x in 3..17 {
            if x == 3 || x == 16 {
                sp.pixels[y * w + x] = dark;
            } else if x % 3 == 0 {
                sp.pixels[y * w + x] = dark; // fluting shadow line
            } else if x % 3 == 1 {
                sp.pixels[y * w + x] = light; // fluting highlight
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Ornate gothic capital at top
    for y in 0..8 {
        for x in 1..19 {
            if y == 0 || y == 7 || x == 1 || x == 18 {
                sp.pixels[y * w + x] = dark;
            } else if y == 4 {
                sp.pixels[y * w + x] = gold;
            } else {
                sp.pixels[y * w + x] = light;
            }
        }
    }

    sp
}

fn generate_gargoyle_statue() -> Sprite {
    let w = 20;
    let h = 28;
    let mut sp = Sprite::new(w, h);
    let dark = Color::rgb(24, 26, 32);
    let mid = Color::rgb(55, 60, 72);
    let light = Color::rgb(90, 98, 116);
    let eye = Color::rgb(255, 60, 50);

    // Pedestal plinth
    for y in 22..28 {
        for x in 3..17 {
            if y == 27 || x == 3 || x == 16 {
                sp.pixels[y * w + x] = dark;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Crouching gargoyle body & talons
    for y in 10..22 {
        for x in 6..14 {
            if x == 6 || x == 13 {
                sp.pixels[y * w + x] = dark;
            } else if (x + y) % 2 == 0 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Horned head
    for y in 4..10 {
        for x in 7..13 {
            sp.pixels[y * w + x] = mid;
        }
    }
    // Horns
    sp.pixels[2 * w + 6] = dark;
    sp.pixels[3 * w + 7] = light;
    sp.pixels[2 * w + 13] = dark;
    sp.pixels[3 * w + 12] = light;

    // Glowing red eye
    sp.pixels[6 * w + 9] = eye;
    sp.pixels[6 * w + 11] = eye;

    // Bat wings splayed on sides
    for y in 6..18 {
        sp.pixels[y * w + 3] = dark;
        sp.pixels[y * w + 4] = light;
        sp.pixels[y * w + 15] = light;
        sp.pixels[y * w + 16] = dark;
    }

    sp
}

fn generate_castle_brazier(frame: usize) -> Sprite {
    let w = 16;
    let h = 26;
    let mut sp = Sprite::new(w, h);
    let iron_dark = Color::rgb(22, 24, 28);
    let iron_mid = Palette::IRON_MID;
    let iron_light = Palette::IRON_LIGHT;

    let flame_core = Palette::BONFIRE_CORE;
    let flame_violet = Color::rgb(190, 85, 255);
    let flame_amber = Palette::BONFIRE_ORANGE;

    // Tripod iron legs and base
    for y in 14..26 {
        sp.pixels[y * w + 4] = iron_dark;
        sp.pixels[y * w + 11] = iron_dark;
        if y >= 20 {
            sp.pixels[y * w + 2] = iron_dark;
            sp.pixels[y * w + 13] = iron_dark;
        }
    }

    // Iron brazier bowl
    for y in 10..14 {
        for x in 2..14 {
            if y == 13 || x == 2 || x == 13 {
                sp.pixels[y * w + x] = iron_dark;
            } else {
                sp.pixels[y * w + x] = if (x + y) % 2 == 0 { iron_light } else { iron_mid };
            }
        }
    }

    // Roaring violet & amber fire animation across 2 frames
    for y in (2 + frame % 2)..10 {
        for x in 4..12 {
            let dx = (x as i32 - 7).abs();
            let dy = (y as i32 - 6).abs();
            if dx + dy < 5 {
                sp.pixels[y * w + x] = if dx == 0 && dy <= 1 {
                    flame_core
                } else if (x + y + frame).is_multiple_of(2) {
                    flame_violet
                } else {
                    flame_amber
                };
            }
        }
    }

    sp
}

fn generate_castle_banner(frame: usize) -> Sprite {
    let w = 14;
    let h = 34;
    let mut sp = Sprite::new(w, h);
    let iron = Palette::IRON_MID;
    let velvet_dark = Color::rgb(55, 12, 18);
    let velvet_mid = Color::rgb(115, 22, 32);
    let gold = Palette::GOLD_TRIM;

    // Iron wall bracket and spearhead finial
    for x in 1..13 {
        sp.pixels[2 * w + x] = iron;
    }
    sp.pixels[w + 1] = iron;
    sp.pixels[w + 12] = iron;

    // Hanging crimson velvet banner with subtle sway
    let sway = if frame % 2 == 1 { 1 } else { 0 };
    for y in 3..30 {
        let x_off = if y > 20 { sway } else { 0 };
        for x in (2 + x_off)..(12 + x_off) {
            if x == 2 + x_off || x == 11 + x_off {
                sp.pixels[y * w + x] = gold;
            } else if (x + y) % 3 == 0 {
                sp.pixels[y * w + x] = velvet_dark;
            } else {
                sp.pixels[y * w + x] = velvet_mid;
            }
        }
    }

    // Gold imperial crown/dragon crest in center
    for y in 10..16 {
        for x in (5 + sway)..(9 + sway) {
            sp.pixels[y * w + x] = gold;
        }
    }

    // Tattered scalloped bottom tails
    for (tx, ty) in [(3 + sway, 31), (4 + sway, 32), (7 + sway, 31), (9 + sway, 32), (10 + sway, 31)] {
        sp.pixels[ty * w + tx] = velvet_mid;
    }

    sp
}

fn generate_fog_wall_gate() -> Sprite {
    let w = 38;
    let h = 36;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::STONE_COOL_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::STONE_COOL_LIGHT;

    // Left fluted gothic pillar (width 5)
    for y in 0..36 {
        for x in 0..5 {
            if x == 0 || x == 4 || y == 35 {
                sp.pixels[y * w + x] = dark;
            } else if x == 2 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Right fluted gothic pillar (width 5)
    for y in 0..36 {
        for x in 33..38 {
            if x == 33 || x == 37 || y == 35 {
                sp.pixels[y * w + x] = dark;
            } else if x == 35 {
                sp.pixels[y * w + x] = light;
            } else {
                sp.pixels[y * w + x] = mid;
            }
        }
    }

    // Overhead pointed gothic lintel arch connecting pillars
    for x in 4..34 {
        let dist_from_center = (x as i32 - 19).abs();
        let arch_y = (dist_from_center * 8) / 15;
        for y in (arch_y as usize)..=(arch_y as usize + 5).min(35) {
            if y == arch_y as usize || y == arch_y as usize + 5 {
                sp.pixels[y * w + x] = dark;
            } else {
                sp.pixels[y * w + x] = if (x + y) % 2 == 0 { light } else { mid };
            }
        }
    }

    sp
}

fn generate_firekeeper_portrait() -> Sprite {
    let w = 32;
    let h = 32;
    let mut sp = Sprite::new(w, h);
    let skin = Palette::ASH_WHITE;
    let hair = Palette::MOON_PALE;
    let crown = Color::rgb(215, 220, 235);
    let blindfold = Palette::CAVEA_STONE_DARK;
    let robe = Color::rgb(20, 18, 28);
    let gold = Palette::GOLD_TRIM;
    let lantern_glow = Palette::BONFIRE_YELLOW;

    // Robe collar & shoulders
    for y in 20..32 {
        for x in 4..28 {
            sp.pixels[y * w + x] = if x == 15 || x == 16 { gold } else { robe };
        }
    }

    // Pale flowing hair
    for y in 6..24 {
        for x in 7..25 {
            sp.pixels[y * w + x] = hair;
        }
    }

    // Face
    for y in 10..20 {
        for x in 11..21 {
            sp.pixels[y * w + x] = skin;
        }
    }

    // Veiled Blindfold & Filigree Crown
    for y in 11..15 {
        for x in 10..22 {
            sp.pixels[y * w + x] = blindfold;
        }
    }
    for x in 9..23 {
        sp.pixels[10 * w + x] = crown;
        sp.pixels[12 * w + x] = crown;
    }

    // Lantern light in lower corner
    for dy in -3..=3 {
        for dx in -3..=3 {
            let lx = 6 + dx;
            let ly = 24 + dy;
            if (0..32).contains(&lx) && (0..32).contains(&ly) && (dx * dx + dy * dy) <= 8 {
                sp.pixels[ly as usize * w + lx as usize] = lantern_glow;
            }
        }
    }

    sp
}

fn generate_bonfire_frame(frame: usize) -> Sprite {
    let w = 18;
    let h = 20;
    let mut sp = Sprite::new(w, h);

    // Ash and bone base
    let ash = Palette::STONE_MID;
    let bone = Palette::BONE_WHITE;
    for y in 15..19 {
        for x in 3..15 {
            let dist = ((x as i32 - 9).pow(2) + (y as i32 - 16).pow(2) * 2) as f32;
            if dist < 32.0 {
                sp.pixels[y * w + x] = if (x + y) % 3 == 0 { bone } else { ash };
            }
        }
    }

    // Coiled Sword stuck upright in center
    let sword_col = Palette::COILED_SWORD;
    let hilt_col = Palette::GOLD_TRIM;
    for y in 4..16 {
        let twist = if y % 2 == 0 { 0 } else { 1 };
        sp.pixels[y * w + 9 + twist] = sword_col;
    }
    // Crossguard & pommel
    sp.pixels[4 * w + 8] = hilt_col;
    sp.pixels[4 * w + 9] = hilt_col;
    sp.pixels[4 * w + 10] = hilt_col;
    sp.pixels[4 * w + 11] = hilt_col;
    sp.pixels[2 * w + 9] = hilt_col;
    sp.pixels[3 * w + 9] = hilt_col;

    // Animated Fire Flames around the sword
    let (f_core, f_yel, f_ora, f_red) = (
        Palette::BONFIRE_CORE,
        Palette::BONFIRE_YELLOW,
        Palette::BONFIRE_ORANGE,
        Palette::BONFIRE_RED,
    );

    let offsets = match frame % 4 {
        0 => [(8, 14, f_core), (9, 13, f_core), (10, 14, f_yel), (8, 10, f_yel), (9, 7, f_ora), (10, 8, f_ora), (8, 5, f_red), (7, 12, f_red), (11, 12, f_red)],
        1 => [(9, 14, f_core), (10, 13, f_core), (8, 13, f_yel), (9, 9, f_yel), (8, 7, f_ora), (10, 6, f_ora), (10, 4, f_red), (7, 11, f_red), (11, 13, f_red)],
        2 => [(8, 14, f_core), (9, 13, f_core), (9, 11, f_yel), (10, 10, f_yel), (9, 8, f_ora), (8, 6, f_ora), (8, 4, f_red), (6, 12, f_red), (12, 11, f_red)],
        _ => [(9, 14, f_core), (8, 13, f_core), (10, 13, f_yel), (8, 9, f_yel), (10, 8, f_ora), (9, 6, f_ora), (9, 4, f_red), (7, 13, f_red), (11, 10, f_red)],
    };

    for &(x, y, col) in &offsets {
        if x < w && y < h {
            sp.pixels[y * w + x] = col;
            if x + 1 < w {
                sp.pixels[y * w + x + 1] = col.blend_over(sp.pixels[y * w + x + 1]);
            }
        }
    }

    sp
}



// -------------------------------------------------------------
// NPC Generators (4-Frame Idle Cycles)
// -------------------------------------------------------------

// -------------------------------------------------------------
// NPC Generators (4-Frame Idle Cycles)
// -------------------------------------------------------------
fn generate_firekeeper_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let veil = Palette::ASH_WHITE;
    let hair = Palette::MOON_PALE;
    let robe_dark = Palette::ABYSS;
    let robe_mid = Palette::STONE_DARK;
    let gold = Palette::GOLD_TRIM;
    let lantern_core = Palette::BONFIRE_CORE;
    let lantern_glow = Palette::BONFIRE_YELLOW;

    let breathe = if frame == 1 || frame == 2 { 1 } else { 0 };

    // Head, Veil & Hair
    sp.fill_rect(7, 3 + breathe, 4, 5, hair);
    sp.fill_rect(6, 5 + breathe, 6, 2, veil);
    sp.fill_rect(6, 2 + breathe, 6, 1, gold);

    // Flowing Robes
    for y in (8 + breathe)..27 {
        let expand = (y - (8 + breathe)) / 5;
        let x0 = (6 - expand).max(2);
        let x1 = (11 + expand).min(15);
        for x in x0..=x1 {
            let col = if x == x0 || x == x1 {
                gold
            } else if (x + y) % 2 == 0 {
                robe_dark
            } else {
                robe_mid
            };
            sp.set_pixel(x, y, col);
        }
    }

    // Swaying Iron Lantern
    let lx = match frame % 4 {
        1 => 13,
        3 => 15,
        _ => 14,
    };
    let ly = 16 + breathe;
    sp.fill_rect(lx, ly, 3, 4, Palette::IRON_DARK);
    sp.fill_rect(lx + 1, ly + 1, 1, 2, lantern_glow);
    if frame == 2 {
        sp.set_pixel(lx + 1, ly + 1, lantern_core);
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

fn generate_blacksmith_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(22, 34);
    let skin = Palette::SKIN_MID;
    let apron = Palette::LEATHER_MID;
    let iron_dark = Palette::IRON_DARK;
    let iron_mid = Palette::IRON_MID;
    let iron_bright = Palette::IRON_LIGHT;
    let spark = Palette::BONFIRE_YELLOW;

    // Muscular Head & Grizzled Beard
    sp.fill_rect(7, 4, 6, 6, skin);
    sp.fill_rect(7, 7, 6, 4, Color::rgb(42, 38, 36));

    // Torso & Heavy Apron
    sp.fill_rect(5, 10, 10, 14, apron);
    sp.fill_rect(4, 10, 2, 7, skin);
    sp.fill_rect(14, 10, 2, 7, skin);

    // Sturdy Legs & Boots
    sp.fill_rect(5, 24, 4, 8, iron_dark);
    sp.fill_rect(11, 24, 4, 8, iron_dark);

    // Anvil in front (x: 13..21, y: 22..30)
    sp.fill_rect(13, 22, 8, 8, iron_mid);
    sp.fill_rect(14, 21, 5, 1, Palette::BONFIRE_RED);

    // Hammer Animation:
    match frame % 4 {
        0 => {
            sp.fill_rect(16, 17, 2, 5, Color::rgb(65, 45, 25));
            sp.fill_rect(15, 20, 4, 2, iron_bright);
        }
        1 => {
            sp.fill_rect(17, 5, 2, 7, Color::rgb(65, 45, 25));
            sp.fill_rect(16, 4, 4, 3, iron_bright);
        }
        2 => {
            sp.fill_rect(15, 17, 2, 5, Color::rgb(65, 45, 25));
            sp.fill_rect(14, 20, 5, 2, iron_bright);
            // Sparks flying
            sp.set_pixel(13, 19, spark);
            sp.set_pixel(15, 17, Palette::BONFIRE_CORE);
            sp.set_pixel(18, 18, spark);
            sp.set_pixel(19, 16, Palette::BONFIRE_ORANGE);
            sp.set_pixel(16, 15, spark);
            sp.set_pixel(14, 14, Palette::BONFIRE_CORE);
            sp.set_pixel(17, 13, Palette::BONFIRE_ORANGE);
        }
        _ => {
            sp.fill_rect(16, 14, 2, 6, Color::rgb(65, 45, 25));
            sp.fill_rect(15, 13, 4, 2, iron_bright);
            sp.set_pixel(16, 19, Palette::ASH_WHITE);
            sp.set_pixel(17, 18, Palette::ASH_WHITE);
        }
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

fn generate_adventurer_npc_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let cowl = Color::rgb(35, 42, 52);
    let tunic = Color::rgb(55, 65, 78);
    let teal = Palette::MAGE_CYAN;
    let steel = Palette::IRON_LIGHT;

    // Hooded Cowl
    sp.fill_rect(5, 3, 8, 7, cowl);
    sp.set_pixel(8, 6, teal);

    // Travel Tunic with teal stitching
    sp.fill_rect(4, 10, 10, 10, tunic);
    sp.fill_rect(8, 10, 1, 10, teal);

    // Legs & Boots
    sp.fill_rect(5, 20, 3, 8, Palette::IRON_DARK);
    sp.fill_rect(10, 20, 3, 8, Palette::IRON_DARK);

    // Gladius Blade
    sp.fill_rect(13, 11, 2, 11, steel);
    sp.fill_rect(12, 13, 4, 1, Palette::GOLD_TRIM);

    // Whetstone & Blade Glint
    let wy = match frame % 4 {
        0 => 14,
        1 => 18,
        _ => 21,
    };
    sp.fill_rect(15, wy, 2, 2, Color::rgb(140, 145, 160));

    if frame == 2 {
        let gx = 13;
        let gy = 22;
        sp.set_pixel(gx, gy, Color::rgb(240, 250, 255));
        sp.set_pixel(gx - 1, gy, steel);
        sp.set_pixel(gx + 1, gy, steel);
        sp.set_pixel(gx, gy - 1, steel);
        sp.set_pixel(gx, gy + 1, steel);
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

fn generate_witch_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    let hat = Color::rgb(32, 18, 48);
    let robe_dark = Color::rgb(50, 18, 70);
    let robe_mid = Color::rgb(75, 28, 105);
    let violet = Color::rgb(215, 80, 255);
    let candle_flame = Palette::BONFIRE_YELLOW;

    // Crooked Hat with Candle
    sp.fill_rect(3, 8, 12, 2, hat);
    sp.fill_rect(5, 5, 7, 3, hat);
    sp.fill_rect(7, 2, 4, 3, hat);
    sp.set_pixel(9, 1, hat);

    // Candle perched on hat
    sp.fill_rect(4, 5, 1, 3, Palette::ASH_WHITE);
    sp.set_pixel(4, 4, candle_flame);
    if frame == 3 {
        sp.set_pixel(4, 3, Palette::BONFIRE_CORE);
    }

    // Flowing Violet Robes
    for y in 10..27 {
        let expand = (y - 10) / 4;
        let x0 = (5 - expand).max(2);
        let x1 = (11 + expand).min(15);
        for x in x0..=x1 {
            sp.set_pixel(x, y, if (x + y) % 2 == 0 { robe_dark } else { robe_mid });
        }
    }

    // Boiling Flask
    sp.fill_rect(13, 16, 3, 4, Color::rgb(180, 190, 215));
    sp.fill_rect(13, 17, 3, 3, violet);

    match frame % 4 {
        1 => {
            sp.set_pixel(14, 15, violet);
        }
        2 => {
            sp.set_pixel(14, 13, Color::rgb(245, 180, 255));
            sp.set_pixel(13, 12, violet);
            sp.set_pixel(16, 14, violet);
        }
        _ => {}
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

// -------------------------------------------------------------
// Enemy 1: ACID SLUDGE (26x20)
// -------------------------------------------------------------
fn generate_sludge_idle(frame: usize) -> Sprite {
    let mut sp = Sprite::new(26, 20);
    let moss = Palette::SLUDGE_MOSS_DARK;
    let mid = Palette::SLUDGE_ACID_MID;
    let yellow = Palette::SLUDGE_ACID_YELLOW;
    let core = Palette::SLUDGE_CORE;

    let (dy, dh, dw) = match frame % 4 {
        1 => (-2, 2, -2),
        2 => (2, -2, 2),
        _ => (0, 0, 0),
    };

    let y_start = (6 + dy).max(2) as usize;
    let y_end = (18 + dh).min(19) as usize;
    let x_pad = (3 - dw).max(1) as usize;

    for y in y_start..=y_end {
        let span = ((y - y_start) as f32 / (y_end - y_start) as f32).sin();
        let half_w = (span * (10.0 + dw as f32)).round() as usize;
        let x0 = 13_usize.saturating_sub(half_w).max(x_pad);
        let x1 = (13 + half_w).min(25 - x_pad);

        for x in x0..=x1 {
            let col = if x == x0 || x == x1 || y == y_start {
                yellow
            } else if y > y_end - 2 {
                moss
            } else {
                mid
            };
            sp.set_pixel(x, y, col);
        }
    }

    let cy = (11 + dy).clamp(4, 16) as usize;
    sp.fill_rect(11, cy, 4, 3, core);
    sp.fill_rect(12, cy - 1, 2, 5, core);

    if frame == 2 {
        sp.fill_rect(10, 14, 6, 2, Palette::VOID_BLACK);
        sp.set_pixel(11, 16, yellow);
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_sludge_walk(frame: usize) -> Sprite {
    generate_sludge_idle((frame + 1) % 4)
}

fn generate_sludge_windup() -> Sprite {
    let mut sp = Sprite::new(26, 20);
    sp.fill_rect(4, 4, 18, 14, Palette::SLUDGE_ACID_MID);
    sp.fill_rect(7, 6, 12, 10, Palette::SLUDGE_ACID_YELLOW);
    sp.fill_rect(9, 8, 8, 6, Palette::SLUDGE_CORE);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_sludge_spit() -> Sprite {
    let mut sp = Sprite::new(26, 20);
    sp.fill_rect(2, 8, 16, 10, Palette::SLUDGE_ACID_MID);
    sp.fill_rect(14, 10, 4, 4, Palette::VOID_BLACK);
    sp.fill_rect(20, 11, 4, 3, Palette::SLUDGE_ACID_YELLOW);
    sp.set_pixel(21, 12, Palette::SLUDGE_CORE);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_sludge_hit() -> Sprite {
    let mut sp = Sprite::new(26, 20);
    sp.fill_rect(2, 10, 20, 8, Palette::SLUDGE_MOSS_DARK);
    sp.fill_rect(4, 12, 16, 5, Palette::SLUDGE_ACID_MID);
    sp.set_pixel(12, 13, Color::rgb(255, 80, 80));
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_sludge_death(frame: usize) -> Sprite {
    let mut sp = Sprite::new(26, 20);
    let yellow = Palette::SLUDGE_ACID_YELLOW;
    let moss = Palette::SLUDGE_MOSS_DARK;
    match frame % 3 {
        0 => {
            sp.fill_rect(4, 12, 18, 6, moss);
            sp.fill_rect(8, 10, 10, 4, Palette::SLUDGE_CORE);
        }
        1 => {
            sp.fill_rect(3, 16, 20, 3, yellow);
            sp.fill_rect(6, 17, 14, 2, moss);
        }
        _ => {
            sp.fill_rect(2, 18, 22, 1, moss);
            sp.set_pixel(7, 18, yellow);
            sp.set_pixel(14, 18, yellow);
            sp.set_pixel(19, 18, yellow);
        }
    }
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

// -------------------------------------------------------------
// Enemy 2: BOGGED SKELETON ARCHER (18x30)
// -------------------------------------------------------------
fn paint_skeleton_base(sp: &mut Sprite, dir: Direction4, frame: usize) {
    let bone = Palette::BONE_WHITE;
    let bone_dark = Palette::BOG_BONE_MOSS;
    let mush_red = Palette::MUSHROOM_RED;
    let mush_pale = Palette::MUSHROOM_PALE;
    let eye_poison = Palette::POISON_ARROW_GREEN;

    let bob = if frame % 2 == 1 { 1 } else { 0 };

    sp.fill_rect(7, 4 + bob, 4, 4, bone);
    sp.fill_rect(8, 8 + bob, 2, 1, bone);
    if dir != Direction4::North {
        sp.set_pixel(7, 5 + bob, eye_poison);
        sp.set_pixel(10, 5 + bob, eye_poison);
    }
    sp.set_pixel(6, 3 + bob, mush_red);
    sp.set_pixel(7, 3 + bob, mush_red);
    sp.set_pixel(7, 2 + bob, mush_pale);

    sp.fill_rect(8, 9 + bob, 2, 7, bone);
    sp.fill_rect(6, 10 + bob, 6, 1, bone_dark);
    sp.fill_rect(6, 12 + bob, 6, 1, bone);
    sp.fill_rect(7, 14 + bob, 4, 1, bone_dark);
    sp.set_pixel(5, 9 + bob, mush_red);

    sp.fill_rect(5, 10 + bob, 1, 6, bone);
    sp.fill_rect(12, 10 + bob, 1, 6, bone);
    sp.fill_rect(7, 17 + bob, 1, 8, bone);
    sp.fill_rect(10, 17 + bob, 1, 8, bone);
}

fn generate_skeleton_idle(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    paint_skeleton_base(&mut sp, dir, frame);
    let bx = match dir {
        Direction4::East => 12,
        _ => 4,
    };
    sp.fill_rect(bx, 8, 1, 14, Palette::LEATHER_MID);
    sp.set_pixel(bx + 1, 7, Palette::BONE_WHITE);
    sp.set_pixel(bx + 1, 22, Palette::BONE_WHITE);

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

fn generate_skeleton_walk(dir: Direction4, frame: usize) -> Sprite {
    generate_skeleton_idle(dir, frame)
}

fn generate_skeleton_draw(dir: Direction4, frame: usize) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    paint_skeleton_base(&mut sp, dir, 0);

    sp.fill_rect(13, 7, 1, 15, Palette::LEATHER_MID);
    sp.set_pixel(14, 6, Palette::BONE_WHITE);
    sp.set_pixel(14, 22, Palette::BONE_WHITE);

    let arrow_x = match frame % 3 {
        0 => 12,
        1 => 10,
        _ => 8,
    };
    sp.fill_rect(arrow_x, 14, 6, 1, Palette::IRON_LIGHT);
    sp.set_pixel(14, 14, Palette::POISON_ARROW_GREEN);
    if frame == 2 {
        sp.set_pixel(14, 15, Palette::POISON_ARROW_GREEN);
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp.add_cool_rim(Palette::RIM_LIGHT_COOL);
    sp
}

fn generate_skeleton_release(dir: Direction4) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    paint_skeleton_base(&mut sp, dir, 0);
    sp.fill_rect(13, 7, 1, 15, Palette::LEATHER_MID);
    sp.fill_rect(14, 14, 4, 1, Palette::POISON_ARROW_GREEN);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_skeleton_hit(dir: Direction4) -> Sprite {
    let mut sp = Sprite::new(18, 30);
    paint_skeleton_base(&mut sp, dir, 1);
    sp.set_pixel(9, 11, Color::rgb(255, 60, 60));
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_skeleton_death() -> Sprite {
    let mut sp = Sprite::new(16, 25);
    let bone = Palette::BONE_WHITE;
    sp.fill_rect(3, 20, 10, 3, bone);
    sp.fill_rect(5, 18, 5, 3, bone);
    sp.set_pixel(4, 17, Palette::MUSHROOM_RED);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

// -------------------------------------------------------------
// Enemy 3: CARRION FLY (12x12 with Ground Shadow)
// -------------------------------------------------------------
fn generate_fly_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(12, 12);
    let chitin = Palette::FLY_CHITIN_DARK;
    let eye = Palette::FLY_EYE_TOXIC;
    let wing = Palette::FLY_WING_TRANSLUCENT;

    let y_off = if frame == 0 { 0 } else { 1 };

    sp.fill_rect(5, 5 + y_off, 3, 4, chitin);
    sp.set_pixel(7, 5 + y_off, eye);
    sp.set_pixel(7, 6 + y_off, eye);

    if frame == 0 {
        sp.fill_rect(2, 2, 3, 4, wing);
        sp.fill_rect(6, 2, 3, 4, wing);
    } else {
        sp.fill_rect(1, 6, 4, 2, wing);
        sp.fill_rect(7, 6, 4, 2, wing);
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_fly_shadow() -> Sprite {
    let mut sp = Sprite::new(12, 4);
    for x in 3..9 {
        sp.set_pixel(x, 1, Color::rgba(4, 5, 8, 120));
        sp.set_pixel(x, 2, Color::rgba(4, 5, 8, 120));
    }
    sp
}

fn generate_fly_attack(frame: usize) -> Sprite {
    let mut sp = generate_fly_frame(frame);
    sp.fill_rect(8, 6, 2, 2, Palette::FLY_EYE_TOXIC);
    sp
}

fn generate_fly_hit() -> Sprite {
    let mut sp = Sprite::new(12, 12);
    sp.fill_rect(4, 6, 4, 4, Palette::FLY_CHITIN_DARK);
    sp.set_pixel(6, 7, Color::rgb(255, 60, 60));
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_fly_death() -> Sprite {
    let mut sp = Sprite::new(12, 12);
    sp.fill_rect(4, 8, 4, 3, Palette::FLY_CHITIN_DARK);
    sp.set_pixel(5, 7, Palette::FLY_CHITIN_MID);
    sp.set_pixel(7, 7, Palette::FLY_CHITIN_MID);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

// -------------------------------------------------------------
// Enemy 4: CORPSE WORM (14x10)
// -------------------------------------------------------------
fn generate_worm_frame(frame: usize) -> Sprite {
    let mut sp = Sprite::new(14, 10);
    let flesh = Palette::WORM_PALE_FLESH;
    let shadow = Palette::WORM_FLESH_SHADOW;
    let teeth = Palette::WORM_RING_TEETH;

    match frame % 4 {
        0 => {
            sp.fill_rect(2, 4, 8, 3, flesh);
            sp.fill_rect(8, 3, 4, 4, shadow);
            sp.set_pixel(11, 4, teeth);
        }
        1 => {
            sp.fill_rect(1, 4, 10, 3, flesh);
            sp.fill_rect(9, 3, 4, 4, shadow);
            sp.set_pixel(12, 4, teeth);
        }
        2 => {
            sp.fill_rect(2, 4, 7, 3, flesh);
            sp.fill_rect(8, 2, 5, 6, shadow);
            sp.fill_rect(10, 3, 3, 4, teeth);
            sp.set_pixel(11, 4, Palette::VOID_BLACK);
            sp.set_pixel(11, 5, Palette::VOID_BLACK);
        }
        _ => {
            sp.fill_rect(3, 4, 7, 3, flesh);
            sp.fill_rect(8, 3, 4, 4, shadow);
            sp.set_pixel(11, 4, teeth);
        }
    }

    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_worm_lunge(frame: usize) -> Sprite {
    let mut sp = Sprite::new(14, 10);
    if frame == 0 {
        sp.fill_rect(1, 3, 6, 5, Palette::WORM_FLESH_SHADOW);
        sp.set_pixel(6, 4, Palette::WORM_RING_TEETH);
    } else {
        sp.fill_rect(1, 4, 6, 3, Palette::WORM_PALE_FLESH);
        sp.fill_rect(7, 2, 6, 6, Palette::WORM_FLESH_SHADOW);
        sp.fill_rect(10, 3, 4, 4, Palette::WORM_RING_TEETH);
        sp.set_pixel(11, 4, Palette::VOID_BLACK);
    }
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_worm_hit() -> Sprite {
    let mut sp = Sprite::new(14, 10);
    sp.fill_rect(3, 4, 8, 4, Palette::WORM_FLESH_SHADOW);
    sp.set_pixel(7, 5, Color::rgb(255, 60, 60));
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

fn generate_worm_death() -> Sprite {
    let mut sp = Sprite::new(14, 10);
    sp.fill_rect(2, 6, 10, 2, Palette::WORM_FLESH_SHADOW);
    sp.set_pixel(11, 6, Palette::WORM_RING_TEETH);
    sp.add_outline(Palette::OUTLINE_DARK);
    sp
}

