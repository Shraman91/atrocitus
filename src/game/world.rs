use crate::assets::palette::Palette;
use crate::assets::sprites::{Sprite, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::lighting::ShadowCaster;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    StoneArch,
    StonePillar,
    Gravestone,
    SwordInGround,
    DeadTree,
    FogGate,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub ground_pixels: Vec<Color>,
    pub props: Vec<PropInstance>,
    pub colliders: Vec<Collider>,
}

impl World {
    pub fn build(width: usize, height: usize) -> Self {
        let mut ground_pixels = vec![Palette::STONE_DARK; width * height];
        let mut props = Vec::with_capacity(32);
        let mut colliders = Vec::with_capacity(40);

        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;
        let plateau_rx = width as f32 * 0.42;
        let plateau_ry = height as f32 * 0.38;

        // Generate circular stone shrine plateau
        for y in 0..height {
            let dy = (y as f32 - cy) / plateau_ry;
            let row_offset = y * width;

            for x in 0..width {
                let dx = (x as f32 - cx) / plateau_rx;
                let dist2 = dx * dx + dy * dy;

                let col = if dist2 < 0.85 {
                    // Plateau interior flagstones
                    let tile_x = x / 12;
                    let tile_y = y / 8;
                    let is_seam = (x % 12 == 0) || (y % 8 == 0);
                    let crack = ((x * 17 + y * 31) % 43) == 0;

                    if is_seam || crack {
                        Palette::STONE_CRACK
                    } else if (tile_x + tile_y) % 2 == 0 {
                        Palette::STONE_MID
                    } else {
                        Palette::STONE_DARK
                    }
                } else if dist2 < 1.0 {
                    // Chiseled plateau rim
                    Palette::STONE_LIGHT
                } else if dist2 < 1.08 {
                    // Cliff edge shadow
                    Palette::VOID_BLACK
                } else {
                    // Abyss void
                    Palette::VOID_BLACK
                };

                ground_pixels[row_offset + x] = col;
            }
        }

        // Inner raised stone dais around central area
        for y in (cy as usize - 14)..=(cy as usize + 14) {
            let dy = (y as f32 - cy) / 14.0;
            let row_offset = y * width;
            for x in (cx as usize - 22)..=(cx as usize + 22) {
                let dx = (x as f32 - cx) / 22.0;
                if dx * dx + dy * dy <= 1.0 {
                    let is_rim = (dx * dx + dy * dy) > 0.82;
                    ground_pixels[row_offset + x] = if is_rim {
                        Palette::STONE_LIGHT
                    } else {
                        Palette::STONE_MID
                    };
                }
            }
        }

        // Add Perimeter colliders around the edge of the plateau
        let num_rim_colliders = 24;
        for i in 0..num_rim_colliders {
            let angle = (i as f32 / num_rim_colliders as f32) * std::f32::consts::TAU;
            let rx = cx + angle.cos() * (plateau_rx * 1.02);
            let ry = cy + angle.sin() * (plateau_ry * 1.02);
            colliders.push(Collider::new(rx - 8.0, ry - 6.0, 16.0, 12.0));
        }

        // Placement of Props & Colliders
        // 1. Broken Gothic Arches (North edge & Entrance)
        props.push(PropInstance {
            x: cx as i32 - 70,
            y: cy as i32 - 45,
            base_y: cy as i32 - 15,
            sprite_index: PropType::StoneArch,
        });
        colliders.push(Collider::new(cx - 68.0, cy - 18.0, 6.0, 8.0));
        colliders.push(Collider::new(cx - 48.0, cy - 18.0, 6.0, 8.0));

        props.push(PropInstance {
            x: cx as i32 + 42,
            y: cy as i32 - 45,
            base_y: cy as i32 - 15,
            sprite_index: PropType::StoneArch,
        });
        colliders.push(Collider::new(cx + 44.0, cy - 18.0, 6.0, 8.0));
        colliders.push(Collider::new(cx + 64.0, cy - 18.0, 6.0, 8.0));

        // 2. Stone Pillars
        let pillar_coords = [
            (cx as i32 - 40, cy as i32 + 25),
            (cx as i32 + 35, cy as i32 + 25),
            (cx as i32 - 80, cy as i32 + 10),
            (cx as i32 + 75, cy as i32 + 10),
            (cx as i32, cy as i32 - 48),
        ];
        for &(px, py) in &pillar_coords {
            props.push(PropInstance {
                x: px,
                y: py - 20,
                base_y: py + 2,
                sprite_index: PropType::StonePillar,
            });
            colliders.push(Collider::new(px as f32 + 1.0, py as f32 - 2.0, 8.0, 6.0));
        }

        // 3. Dead twisted trees
        props.push(PropInstance {
            x: cx as i32 - 85,
            y: cy as i32 - 38,
            base_y: cy as i32 - 10,
            sprite_index: PropType::DeadTree,
        });
        colliders.push(Collider::new(cx - 76.0, cy - 14.0, 8.0, 6.0));

        props.push(PropInstance {
            x: cx as i32 + 68,
            y: cy as i32 - 38,
            base_y: cy as i32 - 10,
            sprite_index: PropType::DeadTree,
        });
        colliders.push(Collider::new(cx + 78.0, cy - 14.0, 8.0, 6.0));

        // 4. Fog Gate Archway (North-East exit)
        props.push(PropInstance {
            x: cx as i32 + 8,
            y: cy as i32 - 58,
            base_y: cy as i32 - 32,
            sprite_index: PropType::FogGate,
        });
        colliders.push(Collider::new(cx + 10.0, cy - 36.0, 20.0, 6.0));

        // 5. Gravestones & Swords in ground around perimeter
        let graves = [
            (cx as i32 - 60, cy as i32 + 32),
            (cx as i32 - 50, cy as i32 + 36),
            (cx as i32 + 55, cy as i32 + 32),
            (cx as i32 + 65, cy as i32 + 35),
            (cx as i32 - 25, cy as i32 + 38),
            (cx as i32 + 20, cy as i32 + 38),
        ];
        for &(gx, gy) in &graves {
            props.push(PropInstance {
                x: gx,
                y: gy - 12,
                base_y: gy + 1,
                sprite_index: PropType::Gravestone,
            });
            colliders.push(Collider::new(gx as f32 + 1.0, gy as f32 - 2.0, 6.0, 4.0));
        }

        let swords = [
            (cx as i32 - 28, cy as i32 - 12),
            (cx as i32 + 26, cy as i32 - 12),
            (cx as i32 - 18, cy as i32 + 20),
            (cx as i32 + 18, cy as i32 + 20),
            (cx as i32 - 42, cy as i32 + 6),
            (cx as i32 + 42, cy as i32 + 6),
        ];
        for &(sx, sy) in &swords {
            props.push(PropInstance {
                x: sx,
                y: sy - 14,
                base_y: sy + 1,
                sprite_index: PropType::SwordInGround,
            });
        }

        Self {
            width,
            height,
            ground_pixels,
            props,
            colliders,
        }
    }

    /// Checks if a bounding box collides with any world barrier or prop.
    pub fn check_collision(&self, box_collider: &Collider) -> bool {
        // Keep within world boundaries
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

    /// Draws the parallax background sky, distant moon, and silhouette mountains/ruins.
    pub fn draw_parallax_sky(&self, fb: &mut Framebuffer, cam_x: f32, _cam_y: f32) {
        let sky_top = Palette::VOID_BLACK;
        let sky_bot = Palette::DEEP_NIGHT;
        let horizon_y = fb.height as i32 / 2;

        // Gradient sky
        fb.draw_gradient_v(0, 0, fb.width as i32, horizon_y, sky_top, sky_bot);

        // Distant Pale Moon
        let moon_x = (fb.width as f32 * 0.75 - cam_x * 0.05).round() as i32;
        let moon_y = 12;
        fb.draw_circle(moon_x, moon_y, 7, Palette::MOON_PALE, true);
        fb.draw_circle(moon_x, moon_y, 10, Palette::MOON_HAZE, false);

        // Layer 1: Distant mountain / ruined spires silhouette (slowest parallax)
        let spire_col1 = Palette::ABYSS;
        for col in 0..fb.width {
            let world_u = col as f32 + cam_x * 0.1;
            let peak1 = (world_u * 0.04).sin() * 8.0 + (world_u * 0.09).cos() * 4.0;
            let peak_y = (horizon_y as f32 - 16.0 + peak1).round() as i32;
            for y in peak_y..horizon_y {
                fb.set_pixel_blend(col as i32, y, spire_col1);
            }
        }

        // Layer 2: Mid-distance castle ramparts silhouette
        let spire_col2 = Color::rgb(14, 18, 26);
        for col in 0..fb.width {
            let world_u = col as f32 + cam_x * 0.25;
            let peak2 = (world_u * 0.06).sin() * 6.0 + (world_u * 0.15).cos() * 3.0;
            let peak_y = (horizon_y as f32 - 8.0 + peak2).round() as i32;
            for y in peak_y..horizon_y {
                fb.set_pixel_blend(col as i32, y, spire_col2);
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
                PropType::StonePillar => (10.0, 24.0),
                PropType::StoneArch => (26.0, 30.0),
                PropType::DeadTree => (18.0, 28.0),
                PropType::Gravestone => (8.0, 12.0),
                PropType::FogGate => (22.0, 26.0),
                PropType::SwordInGround => (6.0, 12.0),
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
    pub fn get_prop_sprite<'a>(&self, bank: &'a SpriteBank, prop_type: PropType) -> &'a Sprite {
        match prop_type {
            PropType::StoneArch => &bank.stone_arch,
            PropType::StonePillar => &bank.stone_pillar,
            PropType::Gravestone => &bank.gravestone,
            PropType::SwordInGround => &bank.sword_in_ground,
            PropType::DeadTree => &bank.dead_tree,
            PropType::FogGate => &bank.fog_gate,
        }
    }
}
