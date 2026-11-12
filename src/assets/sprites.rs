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
}

pub struct SpriteBank {
    pub body_idle: [Sprite; 4],      // S, N, E, SE
    pub body_walk: [[Sprite; 4]; 4], // 4 directions x 4 walk frames

    // Class Gear Sets
    pub mage_gear: ClassGearSet,
    pub knight_gear: ClassGearSet,
    pub assassin_gear: ClassGearSet,
    pub hunter_gear: ClassGearSet,
    pub alchemist_gear: ClassGearSet,

    // Bonfire & NPCs
    pub bonfire_anim: [Sprite; 4],
    pub firekeeper_anim: [Sprite; 2],
    pub firekeeper_portrait: Sprite,
    pub blacksmith_anim: [Sprite; 2],
    pub adventurer_anim: [Sprite; 2],
    pub witch_anim: [Sprite; 2],

    // The Ashen Colosseum Props
    pub colosseum_arch: Sprite,
    pub broken_column: Sprite,
    pub gate_of_trials: Sprite,
    pub weapon_rack: Sprite,
    pub training_dummy: Sprite,
    pub spectator_stand: Sprite,
    pub tattered_banner: [Sprite; 2],
    pub bone_pile: Sprite,
}

impl SpriteBank {
    pub fn build() -> Self {
        Self {
            body_idle: [
                generate_base_body(Direction4::South, 0, false),
                generate_base_body(Direction4::North, 0, false),
                generate_base_body(Direction4::East, 0, false),
                generate_base_body(Direction4::SouthEast, 0, false),
            ],
            body_walk: [
                [
                    generate_base_body(Direction4::South, 0, true),
                    generate_base_body(Direction4::South, 1, true),
                    generate_base_body(Direction4::South, 2, true),
                    generate_base_body(Direction4::South, 3, true),
                ],
                [
                    generate_base_body(Direction4::North, 0, true),
                    generate_base_body(Direction4::North, 1, true),
                    generate_base_body(Direction4::North, 2, true),
                    generate_base_body(Direction4::North, 3, true),
                ],
                [
                    generate_base_body(Direction4::East, 0, true),
                    generate_base_body(Direction4::East, 1, true),
                    generate_base_body(Direction4::East, 2, true),
                    generate_base_body(Direction4::East, 3, true),
                ],
                [
                    generate_base_body(Direction4::SouthEast, 0, true),
                    generate_base_body(Direction4::SouthEast, 1, true),
                    generate_base_body(Direction4::SouthEast, 2, true),
                    generate_base_body(Direction4::SouthEast, 3, true),
                ],
            ],

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
            ],
            firekeeper_portrait: generate_firekeeper_portrait(),
            blacksmith_anim: [
                generate_blacksmith_frame(0),
                generate_blacksmith_frame(1),
            ],
            adventurer_anim: [
                generate_adventurer_npc_frame(0),
                generate_adventurer_npc_frame(1),
            ],
            witch_anim: [
                generate_witch_frame(0),
                generate_witch_frame(1),
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
// Base Body Generator (Ashen Wanderer)
// -------------------------------------------------------------
fn generate_base_body(dir: Direction4, frame: usize, walking: bool) -> Sprite {
    let w = 16;
    let h = 22;
    let mut sp = Sprite::new(w, h);

    let skin = Palette::ASH_WHITE.lerp(Palette::STONE_LIGHT, 0.4);
    let linen = Palette::STONE_DARK;
    let boots = Palette::IRON_DARK;
    let shadow = Color::rgba(0, 0, 0, 90);

    // Cast soft shadow at feet
    for dx in 3..13 {
        sp.pixels[21 * w + dx] = shadow;
    }

    let leg_off = if walking {
        match frame % 4 {
            0 => (1, -1),
            1 => (0, 0),
            2 => (-1, 1),
            _ => (0, 0),
        }
    } else {
        (0, 0)
    };

    match dir {
        Direction4::South => {
            // Head
            for y in 2..6 {
                for x in 6..10 {
                    sp.pixels[y * w + x] = skin;
                }
            }
            // Torso / linen under-tunic
            for y in 6..12 {
                for x in 5..11 {
                    sp.pixels[y * w + x] = linen;
                }
            }
            // Arms
            for y in 7..12 {
                sp.pixels[y * w + 4] = skin;
                sp.pixels[y * w + 11] = skin;
            }
            // Legs
            for y in 12..19 {
                let x1 = (5 + leg_off.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_off.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = linen;
                sp.pixels[y * w + x2] = linen;
            }
            // Boots
            sp.pixels[20 * w + (5 + leg_off.0).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (6 + leg_off.0).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (9 + leg_off.1).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (10 + leg_off.1).clamp(0, 14) as usize] = boots;
        }
        Direction4::North => {
            // Back of head
            for y in 2..6 {
                for x in 6..10 {
                    sp.pixels[y * w + x] = skin;
                }
            }
            // Back tunic
            for y in 6..12 {
                for x in 5..11 {
                    sp.pixels[y * w + x] = linen;
                }
            }
            for y in 12..19 {
                let x1 = (5 + leg_off.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_off.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = linen;
                sp.pixels[y * w + x2] = linen;
            }
            sp.pixels[20 * w + (5 + leg_off.0).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (9 + leg_off.1).clamp(0, 14) as usize] = boots;
        }
        Direction4::East => {
            // Side head
            for y in 2..6 {
                for x in 7..11 {
                    sp.pixels[y * w + x] = skin;
                }
            }
            // Side tunic
            for y in 6..12 {
                for x in 6..10 {
                    sp.pixels[y * w + x] = linen;
                }
            }
            // Near arm
            for y in 7..12 {
                sp.pixels[y * w + 8] = skin;
            }
            // Legs
            for y in 12..19 {
                let x1 = (6 + leg_off.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_off.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = linen;
                sp.pixels[y * w + x2] = linen;
            }
            sp.pixels[20 * w + (7 + leg_off.0).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (10 + leg_off.1).clamp(0, 14) as usize] = boots;
        }
        Direction4::SouthEast => {
            for y in 2..6 {
                for x in 6..11 {
                    sp.pixels[y * w + x] = skin;
                }
            }
            for y in 6..12 {
                for x in 5..11 {
                    sp.pixels[y * w + x] = linen;
                }
            }
            for y in 12..19 {
                let x1 = (5 + leg_off.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_off.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = linen;
                sp.pixels[y * w + x2] = linen;
            }
            sp.pixels[20 * w + (6 + leg_off.0).clamp(0, 14) as usize] = boots;
            sp.pixels[20 * w + (9 + leg_off.1).clamp(0, 14) as usize] = boots;
        }
    }

    sp
}

// -------------------------------------------------------------
// Class 1: MAGE Gear (Witch Hat, Twilight Robes, Gnarled Staff)
// -------------------------------------------------------------
fn generate_mage_gear_set() -> ClassGearSet {
    let mut head = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut torso = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut main_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut cape = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];

    let hat_dark = Color::rgb(22, 18, 38);
    let hat_rim = Color::rgb(45, 36, 75);
    let robe_color = Color::rgb(32, 28, 54);
    let robe_gold = Palette::GOLD_TRIM;
    let staff_wood = Color::rgb(70, 48, 32);
    let orb_cyan = Palette::MAGE_CYAN;
    let orb_glow = Color::rgb(180, 230, 255);

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        // Pointed Witch Hat
        let sp_h = &mut head[d_idx];
        // Brim
        for x in 3..13 {
            sp_h.pixels[4 * 16 + x] = hat_rim;
        }
        // Cone peak tilting back
        for y in 1..4 {
            let cx = 8 - (3 - y);
            sp_h.pixels[y * 16 + cx] = hat_dark;
            sp_h.pixels[y * 16 + cx + 1] = hat_dark;
        }
        sp_h.pixels[6] = hat_rim; // tip

        // Twilight Robe
        let sp_t = &mut torso[d_idx];
        for y in 6..18 {
            let spread = (y - 6) / 4;
            let x0 = (5 - spread).max(3);
            let x1 = (10 + spread).min(12);
            for x in x0..=x1 {
                sp_t.pixels[y * 16 + x] = if y == 17 { robe_gold } else { robe_color };
            }
        }

        // Starry Cape
        let sp_c = &mut cape[d_idx];
        for y in 8..19 {
            sp_c.pixels[y * 16 + 3] = hat_dark;
            sp_c.pixels[y * 16 + 12] = hat_dark;
        }

        // Gnarled Staff with glowing cyan orb
        let sp_m = &mut main_hand[d_idx];
        let sx = match d_idx {
            1 => 3,  // North
            2 => 11, // East
            _ => 12, // South
        };
        for y in 3..19 {
            sp_m.pixels[y * 16 + sx] = staff_wood;
        }
        // Glowing Orb atop staff
        sp_m.pixels[16 + sx] = orb_glow;
        sp_m.pixels[2 * 16 + sx] = orb_cyan;
        if sx > 0 { sp_m.pixels[2 * 16 + sx - 1] = orb_cyan; }
        if sx < 15 { sp_m.pixels[2 * 16 + sx + 1] = orb_cyan; }
    }

    ClassGearSet {
        head,
        torso,
        cape: [Some(cape[0].clone()), Some(cape[1].clone()), Some(cape[2].clone()), Some(cape[3].clone())],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [None, None, None, None],
        accessory: [None, None, None, None],
    }
}

// -------------------------------------------------------------
// Class 2: KNIGHT Gear (Great Helm, Plate, Longsword, Shield)
// -------------------------------------------------------------
fn generate_knight_gear_set() -> ClassGearSet {
    let mut head = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut torso = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut cape = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut main_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut off_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];

    let steel = Palette::STEEL_GLINT;
    let armor_mid = Palette::ARMOR_MID;
    let gold = Palette::KNIGHT_GOLD;
    let crimson = Palette::CAPE_CRIMSON;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        // Helm
        let sp_h = &mut head[d_idx];
        for y in 2..6 {
            for x in 6..10 {
                sp_h.pixels[y * 16 + x] = armor_mid;
            }
        }
        sp_h.pixels[2 * 16 + 7] = steel;
        sp_h.pixels[4 * 16 + 7] = gold; // visor slit
        sp_h.pixels[4 * 16 + 8] = gold;

        // Plate Torso & Tabard
        let sp_t = &mut torso[d_idx];
        for y in 6..14 {
            for x in 4..12 {
                sp_t.pixels[y * 16 + x] = if x == 7 || x == 8 { gold } else { armor_mid };
            }
        }

        // Tattered Cape
        let sp_c = &mut cape[d_idx];
        for y in 7..18 {
            sp_c.pixels[y * 16 + 3] = crimson;
            sp_c.pixels[y * 16 + 12] = crimson;
        }

        // Longsword
        let sp_m = &mut main_hand[d_idx];
        let sx = match d_idx {
            1 => 4,
            _ => 12,
        };
        for y in 5..18 {
            sp_m.pixels[y * 16 + sx] = steel;
        }
        sp_m.pixels[6 * 16 + sx - 1] = gold;
        sp_m.pixels[6 * 16 + sx + 1] = gold;

        // Heater Shield
        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 11,
            _ => 3,
        };
        for y in 8..15 {
            for x in (ox - 1)..=(ox + 2) {
                if x < 16 {
                    sp_o.pixels[y * 16 + x] = if y == 11 { gold } else { steel };
                }
            }
        }
    }

    ClassGearSet {
        head,
        torso,
        cape: [Some(cape[0].clone()), Some(cape[1].clone()), Some(cape[2].clone()), Some(cape[3].clone())],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
    }
}

// -------------------------------------------------------------
// Class 3: ASSASSIN Gear (Shadow Cowl, Stealth Robe, Dual Daggers)
// -------------------------------------------------------------
fn generate_assassin_gear_set() -> ClassGearSet {
    let mut head = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut torso = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut main_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut off_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];

    let shadow_black = Color::rgb(14, 15, 20);
    let cowl_grey = Color::rgb(28, 30, 38);
    let dagger_steel = Palette::STEEL_GLINT;
    let blood_glint = Palette::ASSASSIN_RED;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        // Shadow Cowl obscuring face
        let sp_h = &mut head[d_idx];
        for y in 1..6 {
            for x in 5..11 {
                sp_h.pixels[y * 16 + x] = cowl_grey;
            }
        }
        sp_h.pixels[4 * 16 + 7] = blood_glint; // glowing eye glint

        // Stealth Robe / Leather tunic
        let sp_t = &mut torso[d_idx];
        for y in 6..14 {
            for x in 5..11 {
                sp_t.pixels[y * 16 + x] = shadow_black;
            }
        }
        sp_t.pixels[10 * 16 + 8] = Color::rgb(55, 30, 30); // crimson sash

        // Main Dagger
        let sp_m = &mut main_hand[d_idx];
        let mx = match d_idx {
            1 => 3,
            _ => 12,
        };
        for y in 10..17 {
            sp_m.pixels[y * 16 + mx] = dagger_steel;
        }
        sp_m.pixels[16 * 16 + mx] = blood_glint;

        // Offhand Stiletto
        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 12,
            _ => 3,
        };
        for y in 10..16 {
            sp_o.pixels[y * 16 + ox] = dagger_steel;
        }
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
    }
}

// -------------------------------------------------------------
// Class 4: HUNTER Gear (Tricorn Cap, Duster Coat, Dual Pistol Swords)
// -------------------------------------------------------------
fn generate_hunter_gear_set() -> ClassGearSet {
    let mut head = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut torso = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut main_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut off_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];

    let leather = Color::rgb(48, 38, 28);
    let coat_buff = Color::rgb(72, 58, 42);
    let brass = Palette::HUNTER_BRASS;
    let blade = Palette::STEEL_GLINT;

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        // Tricorn Cap
        let sp_h = &mut head[d_idx];
        for x in 3..13 {
            sp_h.pixels[3 * 16 + x] = leather;
        }
        for x in 5..11 {
            sp_h.pixels[2 * 16 + x] = leather;
        }
        sp_h.pixels[21] = brass; // small cockade/feather pin

        // Duster Coat
        let sp_t = &mut torso[d_idx];
        for y in 6..18 {
            for x in 4..12 {
                sp_t.pixels[y * 16 + x] = coat_buff;
            }
            sp_t.pixels[y * 16 + 8] = brass; // buttons
        }

        // Pistol Sword (Main Hand)
        let sp_m = &mut main_hand[d_idx];
        let mx = match d_idx {
            1 => 3,
            _ => 12,
        };
        for y in 7..18 {
            sp_m.pixels[y * 16 + mx] = blade;
        }
        // Flintlock barrel & hammer attached alongside blade
        sp_m.pixels[8 * 16 + mx + 1] = brass;
        sp_m.pixels[9 * 16 + mx + 1] = brass;
        sp_m.pixels[10 * 16 + mx + 1] = Color::rgb(30, 30, 30);

        // Pistol Sword (Off Hand)
        let sp_o = &mut off_hand[d_idx];
        let ox = match d_idx {
            1 => 12,
            _ => 3,
        };
        for y in 8..17 {
            sp_o.pixels[y * 16 + ox] = blade;
        }
        sp_o.pixels[9 * 16 + ox - 1] = brass;
        sp_o.pixels[10 * 16 + ox - 1] = Color::rgb(30, 30, 30);
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [Some(main_hand[0].clone()), Some(main_hand[1].clone()), Some(main_hand[2].clone()), Some(main_hand[3].clone())],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [None, None, None, None],
    }
}

// -------------------------------------------------------------
// Class 5: ALCHEMIST Gear (Goggles Cowl, Vials, Arcane Eye Emblem)
// -------------------------------------------------------------
fn generate_alchemist_gear_set() -> ClassGearSet {
    let mut head = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut torso = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut accessory = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];
    let mut off_hand = [Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22), Sprite::new(16, 22)];

    let robe_green = Color::rgb(24, 42, 36);
    let brass_lens = Palette::HUNTER_BRASS;
    let teal_glow = Palette::ALCHEMIST_TEAL;
    let eye_core = Color::rgb(180, 255, 230);
    let vial_red = Color::rgb(220, 50, 50);

    for (d_idx, _) in [Direction4::South, Direction4::North, Direction4::East, Direction4::SouthEast].iter().enumerate() {
        // Cowl with Ocular Goggles
        let sp_h = &mut head[d_idx];
        for y in 2..6 {
            for x in 5..11 {
                sp_h.pixels[y * 16 + x] = robe_green;
            }
        }
        sp_h.pixels[3 * 16 + 7] = brass_lens;
        sp_h.pixels[3 * 16 + 8] = teal_glow; // glowing ocular lens

        // Alchemist Coat with bandolier
        let sp_t = &mut torso[d_idx];
        for y in 6..16 {
            for x in 4..12 {
                sp_t.pixels[y * 16 + x] = robe_green;
            }
        }
        // Vials on belt
        sp_t.pixels[11 * 16 + 5] = teal_glow;
        sp_t.pixels[11 * 16 + 6] = vial_red;
        sp_t.pixels[11 * 16 + 9] = teal_glow;

        // Potion Satchel (Off Hand)
        let sp_o = &mut off_hand[d_idx];
        sp_o.pixels[12 * 16 + 3] = brass_lens;
        sp_o.pixels[13 * 16 + 3] = brass_lens;
        sp_o.pixels[13 * 16 + 4] = teal_glow;

        // Floating Arcane Eye Emblem (Accessory)
        let sp_a = &mut accessory[d_idx];
        let ex = match d_idx {
            1 => 4,  // North
            2 => 11, // East
            _ => 12, // South
        };
        let ey = 8;
        // Eye-shaped runic sigil
        sp_a.pixels[(ey - 1) * 16 + ex] = brass_lens;
        sp_a.pixels[ey * 16 + ex - 1] = teal_glow;
        sp_a.pixels[ey * 16 + ex] = eye_core;
        sp_a.pixels[ey * 16 + ex + 1] = teal_glow;
        sp_a.pixels[(ey + 1) * 16 + ex] = brass_lens;
    }

    ClassGearSet {
        head,
        torso,
        cape: [None, None, None, None],
        main_hand: [None, None, None, None],
        off_hand: [Some(off_hand[0].clone()), Some(off_hand[1].clone()), Some(off_hand[2].clone()), Some(off_hand[3].clone())],
        accessory: [Some(accessory[0].clone()), Some(accessory[1].clone()), Some(accessory[2].clone()), Some(accessory[3].clone())],
    }
}

// -------------------------------------------------------------
// The Ashen Colosseum Props
// -------------------------------------------------------------
fn generate_colosseum_arch() -> Sprite {
    let w = 32;
    let h = 36;
    let mut sp = Sprite::new(w, h);
    let st_dark = Palette::CAVEA_STONE_DARK;
    let st_mid = Palette::CAVEA_STONE_MID;
    let st_light = Palette::CAVEA_STONE_LIGHT;

    // Two tiers of Roman arches
    for y in 0..36 {
        for x in 0..32 {
            let is_pillar = !(6..=25).contains(&x) || (14..=17).contains(&x);
            let is_arch_lip = y == 0 || y == 1 || y == 17 || y == 18;

            if is_arch_lip {
                sp.pixels[y * w + x] = st_light;
            } else if is_pillar {
                sp.pixels[y * w + x] = if (x + y) % 5 == 0 { st_light } else { st_mid };
            } else {
                // Interior dark arch void
                sp.pixels[y * w + x] = st_dark;
            }
        }
    }
    sp
}

fn generate_broken_column() -> Sprite {
    let w = 12;
    let h = 26;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::CAVEA_STONE_DARK;
    let mid = Palette::CAVEA_STONE_MID;
    let light = Palette::CAVEA_STONE_LIGHT;

    // Base capital
    for x in 1..11 {
        sp.pixels[24 * w + x] = mid;
        sp.pixels[25 * w + x] = dark;
    }
    // Fluted shaft with shattered top
    for y in 6..24 {
        for x in 2..10 {
            sp.pixels[y * w + x] = if x % 2 == 0 { light } else { mid };
        }
    }
    // Broken jagged top
    for x in 2..8 {
        let top_y = 6 - (x % 3);
        sp.pixels[top_y * w + x] = light;
    }
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

fn generate_firekeeper_frame(frame: usize) -> Sprite {
    let w = 16;
    let h = 22;
    let mut sp = Sprite::new(w, h);
    let robe_dark = Palette::ABYSS;
    let robe_mid = Palette::STONE_DARK;
    let robe_gold = Palette::GOLD_TRIM;
    let crown = Palette::ASH_WHITE;
    let hair = Palette::MOON_PALE;
    let lantern = Palette::BONFIRE_YELLOW;

    let breathe = if frame == 0 { 0 } else { 1 };

    // Crown / Blindfold
    for y in 2..5 {
        for x in 5..10 {
            sp.pixels[y * w + x] = crown;
        }
    }
    // Hair flowing down
    for y in 5..12 {
        sp.pixels[y * w + 4] = hair;
        sp.pixels[y * w + 10] = hair;
    }

    // Flowing Robes
    for y in (5 + breathe)..20 {
        let expand = (y - 5) / 4;
        let x_start = (5 - expand).max(2);
        let x_end = (9 + expand).min(12);
        for x in x_start..=x_end {
            let col = if x == x_start || x == x_end {
                robe_gold
            } else if (x + y) % 2 == 0 {
                robe_dark
            } else {
                robe_mid
            };
            sp.pixels[y * w + x] = col;
        }
    }

    // Small Lantern in hand
    sp.pixels[13 * w + 12] = Palette::IRON_DARK;
    sp.pixels[14 * w + 12] = lantern;
    sp.pixels[15 * w + 12] = lantern;

    sp
}

fn generate_blacksmith_frame(frame: usize) -> Sprite {
    let w = 20;
    let h = 34;
    let mut sp = Sprite::new(w, h);

    let outline = Color::rgb(10, 10, 14);
    let rim = Color::rgb(145, 160, 195);
    let skin = Color::rgb(195, 145, 115);
    let apron = Color::rgb(85, 52, 32);
    let iron = Palette::IRON_MID;
    let iron_bright = Palette::IRON_LIGHT;
    let spark = Palette::BONFIRE_YELLOW;
    let pants = Palette::IRON_DARK;

    // Outline silhouette for bulk (height 34)
    for y in 4..32 {
        for x in 3..17 {
            sp.pixels[y * w + x] = outline;
        }
    }

    // Heavy Boots & Sturdy Legs
    for y in 22..32 {
        for x in 5..9 {
            sp.pixels[y * w + x] = pants;
        }
        for x in 11..15 {
            sp.pixels[y * w + x] = pants;
        }
    }

    // Leather Apron & Torso
    for y in 10..24 {
        for x in 5..15 {
            sp.pixels[y * w + x] = apron;
        }
    }

    // Bare muscular arms
    for y in 10..18 {
        sp.pixels[y * w + 4] = skin;
        sp.pixels[y * w + 15] = skin;
    }

    // Head, beard and brow
    for y in 5..10 {
        for x in 7..13 {
            sp.pixels[y * w + x] = skin;
        }
    }
    // Grizzled dark beard
    for y in 8..11 {
        for x in 7..13 {
            sp.pixels[y * w + x] = Color::rgb(45, 40, 38);
        }
    }

    // Cool rim light along left side
    for y in 6..28 {
        sp.pixels[y * w + 3] = rim;
    }

    // Anvil base in front
    for y in 20..28 {
        for x in 12..18 {
            sp.pixels[y * w + x] = iron;
        }
    }

    // Hammer animation: tap anvil on frame 1
    if frame == 0 {
        // Hammer raised
        for y in 6..12 {
            sp.pixels[y * w + 16] = Color::rgb(65, 45, 25); // handle
        }
        for x in 15..19 {
            sp.pixels[5 * w + x] = iron_bright;
            sp.pixels[6 * w + x] = iron_bright;
        }
    } else {
        // Hammer struck on anvil with glowing spark!
        for y in 14..20 {
            sp.pixels[y * w + 15] = Color::rgb(65, 45, 25);
        }
        for x in 13..17 {
            sp.pixels[20 * w + x] = iron_bright;
        }
        // Sparks flying
        sp.pixels[18 * w + 16] = spark;
        sp.pixels[17 * w + 18] = Palette::BONFIRE_CORE;
        sp.pixels[19 * w + 17] = Palette::BONFIRE_ORANGE;
    }

    sp
}

fn generate_adventurer_npc_frame(frame: usize) -> Sprite {
    let w = 18;
    let h = 30;
    let mut sp = Sprite::new(w, h);

    let outline = Color::rgb(10, 10, 14);
    let rim = Color::rgb(130, 155, 185);
    let cowl = Color::rgb(35, 40, 48);
    let tunic = Color::rgb(55, 62, 72);
    let teal_trim = Palette::MAGE_CYAN;
    let steel = Palette::IRON_LIGHT;
    let eye = Palette::MAGE_CYAN;

    // Dark outline perimeter
    for y in 4..28 {
        for x in 3..15 {
            sp.pixels[y * w + x] = outline;
        }
    }

    // Legs / Boots
    for y in 18..28 {
        for x in 5..8 {
            sp.pixels[y * w + x] = Palette::IRON_DARK;
        }
        for x in 9..12 {
            sp.pixels[y * w + x] = Palette::IRON_DARK;
        }
    }

    // Travel Tunic with teal stitching
    for y in 10..19 {
        for x in 4..13 {
            sp.pixels[y * w + x] = tunic;
        }
        sp.pixels[y * w + 8] = teal_trim;
    }

    // Hooded Cowl
    for y in 4..11 {
        for x in 5..13 {
            sp.pixels[y * w + x] = cowl;
        }
    }

    // Shadowed Face with faint teal glance
    sp.pixels[7 * w + 7] = eye;

    // Cool rim light on left perimeter
    for y in 5..26 {
        sp.pixels[y * w + 3] = rim;
    }

    // Sharpening blade animation
    let glint_off = if frame == 0 { 0 } else { 2 };
    for y in 12..20 {
        sp.pixels[y * w + 13] = steel;
    }
    // Whetstone / Glint
    sp.pixels[(14 + glint_off) * w + 14] = Color::rgb(220, 245, 255);

    sp
}

fn generate_witch_frame(frame: usize) -> Sprite {
    let w = 18;
    let h = 30;
    let mut sp = Sprite::new(w, h);

    let outline = Color::rgb(10, 10, 14);
    let rim = Color::rgb(160, 140, 195);
    let hat = Color::rgb(32, 18, 48);
    let robe = Color::rgb(55, 22, 75);
    let robe_inner = Color::rgb(85, 36, 115);
    let violet_glow = Color::rgb(215, 80, 255);

    // Dark outline
    for y in 2..28 {
        for x in 3..15 {
            sp.pixels[y * w + x] = outline;
        }
    }

    // Flowing violet robes
    for y in 12..28 {
        let expand = (y - 12) / 4;
        let x_start = (5 - expand).max(2);
        let x_end = (11 + expand).min(15);
        for x in x_start..=x_end {
            sp.pixels[y * w + x] = if (x + y) % 2 == 0 { robe } else { robe_inner };
        }
    }

    // Tall crooked witch hat
    sp.pixels[2 * w + 8] = hat;
    sp.pixels[3 * w + 8] = hat;
    sp.pixels[4 * w + 7] = hat;
    sp.pixels[4 * w + 8] = hat;
    for y in 5..8 {
        for x in 6..10 {
            sp.pixels[y * w + x] = hat;
        }
    }
    // Wide brim
    for x in 3..14 {
        sp.pixels[8 * w + x] = hat;
        sp.pixels[9 * w + x] = Color::rgb(70, 40, 95); // buckle / ribbon
    }

    // Left rim light
    for y in 4..27 {
        sp.pixels[y * w + 3] = rim;
    }

    // Bubbling vial in hand
    sp.pixels[16 * w + 13] = Color::rgb(200, 200, 220); // glass
    sp.pixels[17 * w + 13] = violet_glow;
    sp.pixels[18 * w + 13] = violet_glow;

    // Bubble rising on frame 1
    if frame == 1 {
        sp.pixels[14 * w + 13] = violet_glow;
        sp.pixels[13 * w + 14] = Color::rgb(240, 150, 255);
    }

    sp
}

