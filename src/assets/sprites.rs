use super::palette::Palette;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
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

    pub fn from_ascii(rows: &[&str], char_map: &[(char, Color)]) -> Self {
        let height = rows.len();
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let mut pixels = vec![Color::TRANSPARENT; width * height];

        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                let color = char_map
                    .iter()
                    .find(|(mapped_c, _)| *mapped_c == c)
                    .map(|(_, col)| *col)
                    .unwrap_or(Color::TRANSPARENT);
                pixels[y * width + x] = color;
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    /// Loads a sprite from PNG image file in the filesystem.
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

/// Pre-generated sprite collection for characters, environment props, and title logo.
pub struct SpriteBank {
    pub knight_idle: [Sprite; 4],      // S, N, E, SE
    pub knight_walk: [[Sprite; 4]; 4], // 4 directions x 4 walk frames
    pub bonfire_anim: [Sprite; 4],
    pub blacksmith_anim: [Sprite; 2],
    pub firekeeper_anim: [Sprite; 2],
    pub stone_arch: Sprite,
    pub stone_pillar: Sprite,
    pub gravestone: Sprite,
    pub sword_in_ground: Sprite,
    pub dead_tree: Sprite,
    pub fog_gate: Sprite,
}

impl SpriteBank {
    pub fn build() -> Self {
        Self {
            knight_idle: [
                generate_knight_sprite(Direction4::South, 0, false),
                generate_knight_sprite(Direction4::North, 0, false),
                generate_knight_sprite(Direction4::East, 0, false),
                generate_knight_sprite(Direction4::SouthEast, 0, false),
            ],
            knight_walk: [
                [
                    generate_knight_sprite(Direction4::South, 0, true),
                    generate_knight_sprite(Direction4::South, 1, true),
                    generate_knight_sprite(Direction4::South, 2, true),
                    generate_knight_sprite(Direction4::South, 3, true),
                ],
                [
                    generate_knight_sprite(Direction4::North, 0, true),
                    generate_knight_sprite(Direction4::North, 1, true),
                    generate_knight_sprite(Direction4::North, 2, true),
                    generate_knight_sprite(Direction4::North, 3, true),
                ],
                [
                    generate_knight_sprite(Direction4::East, 0, true),
                    generate_knight_sprite(Direction4::East, 1, true),
                    generate_knight_sprite(Direction4::East, 2, true),
                    generate_knight_sprite(Direction4::East, 3, true),
                ],
                [
                    generate_knight_sprite(Direction4::SouthEast, 0, true),
                    generate_knight_sprite(Direction4::SouthEast, 1, true),
                    generate_knight_sprite(Direction4::SouthEast, 2, true),
                    generate_knight_sprite(Direction4::SouthEast, 3, true),
                ],
            ],
            bonfire_anim: [
                generate_bonfire_frame(0),
                generate_bonfire_frame(1),
                generate_bonfire_frame(2),
                generate_bonfire_frame(3),
            ],
            blacksmith_anim: [
                generate_blacksmith_frame(0),
                generate_blacksmith_frame(1),
            ],
            firekeeper_anim: [
                generate_firekeeper_frame(0),
                generate_firekeeper_frame(1),
            ],
            stone_arch: generate_stone_arch(),
            stone_pillar: generate_stone_pillar(),
            gravestone: generate_gravestone(),
            sword_in_ground: generate_sword_in_ground(),
            dead_tree: generate_dead_tree(),
            fog_gate: generate_fog_gate(),
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

// Procedural Sprite Generators

fn generate_knight_sprite(dir: Direction4, frame: usize, walking: bool) -> Sprite {
    let w = 16;
    let h = 22;
    let mut sp = Sprite::new(w, h);

    // Color definitions
    let h_iron = Palette::STEEL_GLINT;
    let m_iron = Palette::ARMOR_MID;
    let d_iron = Palette::ARMOR_SHADOW;
    let cape = Palette::CAPE_CRIMSON;
    let d_cape = Palette::CAPE_DARK;
    let visor = Palette::BONFIRE_YELLOW;
    let shadow = Color::rgba(0, 0, 0, 100);

    // Cast soft shadow at feet
    for dx in 3..13 {
        sp.pixels[21 * w + dx] = shadow;
    }

    let leg_offset = if walking {
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
            // Helm
            for y in 2..6 {
                for x in 6..10 {
                    sp.pixels[y * w + x] = m_iron;
                }
            }
            sp.pixels[2 * w + 7] = h_iron;
            sp.pixels[2 * w + 8] = h_iron;
            // Visor slit
            sp.pixels[4 * w + 7] = visor;
            sp.pixels[4 * w + 8] = visor;

            // Shoulders & Chest
            for x in 4..12 {
                sp.pixels[6 * w + x] = h_iron;
                sp.pixels[7 * w + x] = m_iron;
                sp.pixels[8 * w + x] = m_iron;
                sp.pixels[9 * w + x] = d_iron;
            }
            // Belt & Tassets
            for x in 5..11 {
                sp.pixels[10 * w + x] = Palette::GOLD_TRIM;
                sp.pixels[11 * w + x] = d_iron;
                sp.pixels[12 * w + x] = m_iron;
            }

            // Tattered Cape sides
            sp.pixels[7 * w + 3] = cape;
            sp.pixels[8 * w + 3] = cape;
            sp.pixels[9 * w + 3] = d_cape;
            sp.pixels[7 * w + 12] = cape;
            sp.pixels[8 * w + 12] = cape;
            sp.pixels[9 * w + 12] = d_cape;

            // Legs
            let l1 = (13..19).map(|y| (y, 5 + leg_offset.0));
            let l2 = (13..19).map(|y| (y, 9 + leg_offset.1));
            for (y, x) in l1.chain(l2) {
                if (0..15).contains(&x) && y < 21 {
                    sp.pixels[y * w + x as usize] = m_iron;
                    sp.pixels[y * w + (x + 1) as usize] = d_iron;
                }
            }
            // Boots
            sp.pixels[20 * w + (5 + leg_offset.0).clamp(0, 14) as usize] = d_iron;
            sp.pixels[20 * w + (6 + leg_offset.0).clamp(0, 14) as usize] = d_iron;
            sp.pixels[20 * w + (9 + leg_offset.1).clamp(0, 14) as usize] = d_iron;
            sp.pixels[20 * w + (10 + leg_offset.1).clamp(0, 14) as usize] = d_iron;
        }
        Direction4::North => {
            // Helm back
            for y in 2..6 {
                for x in 6..10 {
                    sp.pixels[y * w + x] = d_iron;
                }
            }
            sp.pixels[2 * w + 7] = m_iron;
            sp.pixels[2 * w + 8] = m_iron;

            // Full Tattered Cape back
            for y in 6..14 {
                for x in 4..12 {
                    sp.pixels[y * w + x] = if (x + y + frame).is_multiple_of(5) {
                        d_cape
                    } else {
                        cape
                    };
                }
            }
            // Cape tattered ends
            sp.pixels[14 * w + 4] = cape;
            sp.pixels[14 * w + 6] = cape;
            sp.pixels[14 * w + 9] = cape;
            sp.pixels[14 * w + 11] = cape;

            // Legs underneath
            for y in 15..20 {
                let x1 = (5 + leg_offset.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_offset.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = d_iron;
                sp.pixels[y * w + x2] = d_iron;
            }
        }
        Direction4::East => {
            // Side helm
            for y in 2..6 {
                for x in 7..11 {
                    sp.pixels[y * w + x] = m_iron;
                }
            }
            sp.pixels[4 * w + 10] = visor;

            // Torso & Pauldron
            for y in 6..12 {
                for x in 6..11 {
                    sp.pixels[y * w + x] = if x == 9 { h_iron } else { m_iron };
                }
            }
            // Cape flowing behind
            for y in 7..14 {
                let cx = 4.max(6 - (y - 7) / 2);
                sp.pixels[y * w + cx] = cape;
                sp.pixels[y * w + cx + 1] = d_cape;
            }

            // Greatsword slung on back
            for i in 0..12 {
                let sy = 3 + i;
                let sx = 5 - i / 4;
                if sx < w {
                    sp.pixels[sy * w + sx] = Palette::STEEL_GLINT;
                }
            }

            // Legs
            for y in 13..20 {
                let x1 = (7 + leg_offset.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_offset.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = m_iron;
                sp.pixels[y * w + x2] = d_iron;
            }
            sp.pixels[20 * w + (8 + leg_offset.0).clamp(0, 14) as usize] = d_iron;
            sp.pixels[20 * w + (10 + leg_offset.1).clamp(0, 14) as usize] = d_iron;
        }
        Direction4::SouthEast => {
            // Angle helm
            for y in 2..6 {
                for x in 6..11 {
                    sp.pixels[y * w + x] = m_iron;
                }
            }
            sp.pixels[4 * w + 9] = visor;
            sp.pixels[4 * w + 10] = visor;

            // Torso
            for y in 6..12 {
                for x in 5..12 {
                    sp.pixels[y * w + x] = m_iron;
                }
            }
            // Cape corner
            for y in 8..14 {
                sp.pixels[y * w + 4] = cape;
                sp.pixels[y * w + 5] = d_cape;
            }

            // Legs
            for y in 13..20 {
                let x1 = (6 + leg_offset.0).clamp(0, 14) as usize;
                let x2 = (9 + leg_offset.1).clamp(0, 14) as usize;
                sp.pixels[y * w + x1] = m_iron;
                sp.pixels[y * w + x2] = d_iron;
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
    let bone = Palette::ASH_WHITE;
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
    // Crossguard
    sp.pixels[4 * w + 8] = hilt_col;
    sp.pixels[4 * w + 9] = hilt_col;
    sp.pixels[4 * w + 10] = hilt_col;
    sp.pixels[4 * w + 11] = hilt_col;
    // Pommel
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

fn generate_blacksmith_frame(frame: usize) -> Sprite {
    let w = 20;
    let h = 20;
    let mut sp = Sprite::new(w, h);
    let hood = Palette::ARMOR_SHADOW;
    let apron = Palette::ARMOR_MID;
    let skin = Palette::STONE_LIGHT;
    let anvil = Palette::IRON_DARK;
    let steel = Palette::STEEL_GLINT;

    // Anvil (right side)
    for y in 10..18 {
        for x in 12..18 {
            sp.pixels[y * w + x] = anvil;
        }
    }
    // Anvil top horn
    sp.pixels[10 * w + 11] = steel;
    sp.pixels[10 * w + 18] = anvil;

    // Seated hooded figure (left side)
    for y in 4..8 {
        for x in 4..9 {
            sp.pixels[y * w + x] = hood;
        }
    }
    sp.pixels[6 * w + 7] = skin; // beard / face hint

    // Body / Apron
    for y in 8..16 {
        for x in 3..10 {
            sp.pixels[y * w + x] = apron;
        }
    }

    // Arm with Hammer
    let arm_y = if frame == 0 { 8 } else { 10 };
    for x in 9..14 {
        sp.pixels[arm_y * w + x] = skin;
    }
    // Hammer head
    let ham_y = if frame == 0 { 7 } else { 9 };
    sp.pixels[ham_y * w + 13] = steel;
    sp.pixels[(ham_y + 1) * w + 13] = steel;

    sp
}

fn generate_firekeeper_frame(frame: usize) -> Sprite {
    let w = 14;
    let h = 22;
    let mut sp = Sprite::new(w, h);
    let robe_dark = Palette::ABYSS;
    let robe_mid = Palette::STONE_DARK;
    let robe_gold = Palette::GOLD_TRIM;
    let crown = Palette::ASH_WHITE;
    let hair = Palette::MOON_PALE;

    let breathe = if frame == 0 { 0 } else { 1 };

    // Crown / Blindfold
    for y in 2..5 {
        for x in 5..9 {
            sp.pixels[y * w + x] = crown;
        }
    }
    // Pale hair flowing down
    for y in 5..12 {
        sp.pixels[y * w + 4] = hair;
        sp.pixels[y * w + 9] = hair;
    }

    // Flowing Robes
    for y in (5 + breathe)..20 {
        let expand = (y - 5) / 4;
        let x_start = (5 - expand).max(2);
        let x_end = (8 + expand).min(11);
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

    sp
}

fn generate_stone_arch() -> Sprite {
    let w = 28;
    let h = 32;
    let mut sp = Sprite::new(w, h);
    let st_dark = Palette::STONE_DARK;
    let st_mid = Palette::STONE_MID;
    let st_light = Palette::STONE_LIGHT;

    // Left pillar
    for y in 8..32 {
        for x in 2..7 {
            let col = if x == 2 { st_dark } else if x == 6 { st_light } else { st_mid };
            sp.pixels[y * w + x] = col;
        }
    }
    // Right pillar
    for y in 8..32 {
        for x in 21..26 {
            let col = if x == 21 { st_light } else if x == 25 { st_dark } else { st_mid };
            sp.pixels[y * w + x] = col;
        }
    }
    // Arch curve
    for y in 0..10 {
        for x in 2..26 {
            let center_dist = (x as i32 - 14).abs();
            let expected_y = 8 - (12 - center_dist) / 2;
            if (y as i32 - expected_y).abs() <= 2 {
                let col = if y % 2 == 0 { st_mid } else { st_light };
                sp.pixels[y * w + x] = col;
            }
        }
    }

    sp
}

fn generate_stone_pillar() -> Sprite {
    let w = 10;
    let h = 24;
    let mut sp = Sprite::new(w, h);
    let st_dark = Palette::STONE_DARK;
    let st_mid = Palette::STONE_MID;
    let st_light = Palette::STONE_LIGHT;

    // Base capital
    for x in 1..9 {
        sp.pixels[x] = st_light;
        sp.pixels[w + x] = st_mid;
        sp.pixels[22 * w + x] = st_mid;
        sp.pixels[23 * w + x] = st_dark;
    }

    // Fluted column
    for y in 2..22 {
        for x in 2..8 {
            let col = match x {
                2 => st_dark,
                3 | 5 => st_light,
                4 | 6 => st_mid,
                _ => st_dark,
            };
            sp.pixels[y * w + x] = col;
        }
    }

    sp
}

fn generate_gravestone() -> Sprite {
    let w = 8;
    let h = 14;
    let mut sp = Sprite::new(w, h);
    let dark = Palette::STONE_DARK;
    let mid = Palette::STONE_MID;
    let light = Palette::STONE_LIGHT;

    for y in 2..14 {
        for x in 1..7 {
            let col = if x == 1 { light } else if x == 6 { dark } else { mid };
            sp.pixels[y * w + x] = col;
        }
    }
    // Rounded arch top
    for x in 2..6 {
        sp.pixels[x] = light;
        sp.pixels[w + x] = mid;
    }
    // Carved cross rune
    sp.pixels[4 * w + 3] = dark;
    sp.pixels[5 * w + 2] = dark;
    sp.pixels[5 * w + 3] = dark;
    sp.pixels[5 * w + 4] = dark;
    sp.pixels[6 * w + 3] = dark;
    sp.pixels[7 * w + 3] = dark;

    sp
}

fn generate_sword_in_ground() -> Sprite {
    let w = 8;
    let h = 16;
    let mut sp = Sprite::new(w, h);
    let steel = Palette::STEEL_GLINT;
    let hilt = Palette::GOLD_TRIM;

    // Pommel & Hilt
    sp.pixels[3] = hilt;
    sp.pixels[w + 3] = hilt;
    // Crossguard
    for x in 1..6 {
        sp.pixels[2 * w + x] = hilt;
    }
    // Blade angled into ground
    for y in 3..14 {
        let x = 3 + (y - 3) / 6;
        sp.pixels[y * w + x] = steel;
    }

    sp
}

fn generate_dead_tree() -> Sprite {
    let w = 24;
    let h = 30;
    let mut sp = Sprite::new(w, h);
    let bark_dark = Palette::VOID_BLACK;
    let bark_mid = Palette::STONE_DARK;

    // Trunk
    for y in 10..30 {
        let taper = (30 - y) / 7;
        let tx = 11 + (y % 2);
        for x in (tx - 2 + taper)..=(tx + 2 - taper) {
            sp.pixels[y * w + x] = if x % 2 == 0 { bark_dark } else { bark_mid };
        }
    }

    // Twisted Branches
    let branches = [
        (12, 14, 5, 6),
        (13, 12, 19, 4),
        (8, 8, 3, 2),
        (16, 7, 22, 2),
        (12, 6, 12, 0),
    ];
    for &(x0, y0, x1, y1) in &branches {
        let steps = 10;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let bx = (x0 as f32 * (1.0 - t) + x1 as f32 * t).round() as usize;
            let by = (y0 as f32 * (1.0 - t) + y1 as f32 * t).round() as usize;
            if bx < w && by < h {
                sp.pixels[by * w + bx] = bark_dark;
            }
        }
    }

    sp
}

fn generate_fog_gate() -> Sprite {
    let w = 24;
    let h = 28;
    let mut sp = Sprite::new(w, h);
    let st_dark = Palette::STONE_DARK;
    let st_light = Palette::STONE_LIGHT;
    let fog = Palette::FOG_LIGHT;

    // Archway frame
    for y in 2..28 {
        sp.pixels[y * w + 2] = st_dark;
        sp.pixels[y * w + 3] = st_light;
        sp.pixels[y * w + 20] = st_light;
        sp.pixels[y * w + 21] = st_dark;
    }
    for x in 2..22 {
        sp.pixels[w + x] = st_light;
        sp.pixels[2 * w + x] = st_dark;
    }

    // Swirling misty fog plane inside
    for y in 4..27 {
        for x in 4..20 {
            let alpha = (((x * y * 7) % 60) + 120) as u8;
            sp.pixels[y * w + x] = Color::rgba(fog.r, fog.g, fog.b, alpha);
        }
    }

    sp
}
