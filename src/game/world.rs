use crate::assets::palette::Palette;
use crate::assets::sprites::{Sprite, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::lighting::ShadowCaster;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropType {
    ColosseumArch,
    BrokenColumn,
    GateOfTrials,
    WeaponRack,
    TrainingDummy,
    SpectatorStand,
    TatteredBanner,
    BonePile,
}

pub struct World {
    pub width: usize,
    pub height: usize,
    pub ground_pixels: Vec<Color>,
    pub props: Vec<PropInstance>,
    pub colliders: Vec<Collider>,
    pub nav_grid: NavGrid,
}

impl World {
    pub fn build(width: usize, height: usize) -> Self {
        let mut ground_pixels = vec![Palette::VOID_BLACK; width * height];
        let mut props = Vec::with_capacity(36);
        let mut colliders = Vec::with_capacity(48);
        let mut nav_grid = NavGrid::new(width as f32, height as f32, 4.0);

        let cx = width as f32 * 0.5;
        let cy = height as f32 * 0.5;
        let arena_rx = width as f32 * 0.44;
        let arena_ry = height as f32 * 0.38;

        // Generate elliptical Ashen Colosseum floor
        for y in 0..height {
            let dy = (y as f32 - cy) / arena_ry;
            let row_offset = y * width;

            for x in 0..width {
                let dx = (x as f32 - cx) / arena_rx;
                let dist2 = dx * dx + dy * dy;

                let col = if dist2 < 0.85 {
                    // Sand arena floor with blood stains & broken stone pavers
                    let noise = ((x * 19 + y * 37) ^ ((x >> 2) + (y >> 2))) % 71;
                    let blood = ((x * 23 + y * 47) % 89) == 0 && (dist2 > 0.25 && dist2 < 0.75);
                    let flagstone = ((x / 14 + y / 10) % 5 == 0) && (x % 14 > 1 && y % 10 > 1);

                    if blood {
                        Palette::BLOOD_STAIN
                    } else if flagstone {
                        if (x + y) % 2 == 0 {
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
                    // Chiseled Roman stone rim
                    if (x / 8 + y / 8) % 2 == 0 {
                        Palette::CAVEA_STONE_LIGHT
                    } else {
                        Palette::CAVEA_STONE_MID
                    }
                } else if dist2 < 1.05 {
                    // Cavea tiers base shadow
                    Palette::CAVEA_STONE_DARK
                } else {
                    // Outer darkness / spectator void
                    Palette::VOID_BLACK
                };

                ground_pixels[row_offset + x] = col;
            }
        }

        // Central raised stone dais around bonfire
        for y in (cy as usize - 14)..=(cy as usize + 14) {
            let dy = (y as f32 - cy) / 14.0;
            let row_offset = y * width;
            for x in (cx as usize - 22)..=(cx as usize + 22) {
                let dx = (x as f32 - cx) / 22.0;
                let d2 = dx * dx + dy * dy;
                if d2 <= 1.0 {
                    let is_rim = d2 > 0.82;
                    ground_pixels[row_offset + x] = if is_rim {
                        Palette::CAVEA_STONE_LIGHT
                    } else if (x + y) % 3 == 0 {
                        Palette::STONE_CRACK
                    } else {
                        Palette::CAVEA_STONE_MID
                    };
                }
            }
        }

        // Perimeter colliders around the edge of the Colosseum arena
        let num_rim_colliders = 28;
        for i in 0..num_rim_colliders {
            let angle = (i as f32 / num_rim_colliders as f32) * std::f32::consts::TAU;
            let rx = cx + angle.cos() * (arena_rx * 0.96);
            let ry = cy + angle.sin() * (arena_ry * 0.96);
            let col = Collider::new(rx - 8.0, ry - 6.0, 16.0, 12.0);
            colliders.push(col);
        }

        // 1. Gate of Trials (North edge exit)
        let gate_x = cx as i32 - 16;
        let gate_y = (cy - arena_ry + 4.0) as i32;
        props.push(PropInstance {
            x: gate_x,
            y: gate_y,
            base_y: gate_y + 32,
            sprite_index: PropType::GateOfTrials,
        });
        colliders.push(Collider::new(gate_x as f32 + 2.0, gate_y as f32 + 20.0, 28.0, 12.0));

        // 2. Colosseum Arches (Upper cavea rim)
        let arch_coords = [
            (cx as i32 - 82, (cy - arena_ry * 0.65) as i32),
            (cx as i32 - 50, (cy - arena_ry * 0.88) as i32),
            (cx as i32 + 22, (cy - arena_ry * 0.88) as i32),
            (cx as i32 + 54, (cy - arena_ry * 0.65) as i32),
        ];
        for &(ax, ay) in &arch_coords {
            props.push(PropInstance {
                x: ax,
                y: ay,
                base_y: ay + 34,
                sprite_index: PropType::ColosseumArch,
            });
            colliders.push(Collider::new(ax as f32 + 4.0, ay as f32 + 22.0, 24.0, 12.0));
        }

        // 3. Spectator Stands (Tiers behind arches)
        let stands = [
            (cx as i32 - 105, (cy - arena_ry * 0.35) as i32),
            (cx as i32 + 80, (cy - arena_ry * 0.35) as i32),
        ];
        for &(sx, sy) in &stands {
            props.push(PropInstance {
                x: sx,
                y: sy,
                base_y: sy + 16,
                sprite_index: PropType::SpectatorStand,
            });
            colliders.push(Collider::new(sx as f32 + 2.0, sy as f32 + 8.0, 24.0, 10.0));
        }

        // 4. Broken Columns (Shattered Roman columns)
        let columns = [
            (cx as i32 - 62, cy as i32 + 24),
            (cx as i32 + 58, cy as i32 + 20),
            (cx as i32 - 84, cy as i32 + 5),
            (cx as i32 + 76, cy as i32 + 8),
            (cx as i32 - 32, cy as i32 + 36),
            (cx as i32 + 28, cy as i32 + 36),
        ];
        for &(col_x, col_y) in &columns {
            props.push(PropInstance {
                x: col_x,
                y: col_y - 20,
                base_y: col_y + 4,
                sprite_index: PropType::BrokenColumn,
            });
            colliders.push(Collider::new(col_x as f32 + 2.0, col_y as f32 - 2.0, 8.0, 8.0));
        }

        // 5. Weapon Racks
        let weapon_racks = [
            (cx as i32 - 42, cy as i32 - 18),
            (cx as i32 + 36, cy as i32 + 18),
        ];
        for &(wx, wy) in &weapon_racks {
            props.push(PropInstance {
                x: wx,
                y: wy - 16,
                base_y: wy + 2,
                sprite_index: PropType::WeaponRack,
            });
            colliders.push(Collider::new(wx as f32 + 2.0, wy as f32 - 4.0, 14.0, 8.0));
        }

        // 6. Training Dummy
        let dummy_pos = (cx as i32 - 58, cy as i32 - 16);
        props.push(PropInstance {
            x: dummy_pos.0,
            y: dummy_pos.1 - 20,
            base_y: dummy_pos.1 + 2,
            sprite_index: PropType::TrainingDummy,
        });
        colliders.push(Collider::new(dummy_pos.0 as f32 + 2.0, dummy_pos.1 as f32 - 4.0, 10.0, 8.0));

        // 7. Tattered Banners
        let banners = [
            (gate_x - 12, gate_y + 6),
            (gate_x + 36, gate_y + 6),
            (cx as i32 - 94, cy as i32 + 16),
            (cx as i32 + 88, cy as i32 + 16),
        ];
        for &(bx, by) in &banners {
            props.push(PropInstance {
                x: bx,
                y: by,
                base_y: by + 28,
                sprite_index: PropType::TatteredBanner,
            });
            colliders.push(Collider::new(bx as f32 + 2.0, by as f32 + 22.0, 4.0, 6.0));
        }

        // 8. Bone Piles
        let bone_piles = [
            (cx as i32 - 70, cy as i32 + 32),
            (cx as i32 + 64, cy as i32 + 30),
            (cx as i32 + 24, cy as i32 - 28),
            (cx as i32 - 44, cy as i32 + 34),
        ];
        for &(bx, by) in &bone_piles {
            props.push(PropInstance {
                x: bx,
                y: by - 6,
                base_y: by + 2,
                sprite_index: PropType::BonePile,
            });
        }

        // Build NavGrid blocked cells from all colliders and arena perimeter
        for c in &colliders {
            nav_grid.set_blocked_rect(c.x, c.y, c.x + c.w, c.y + c.h);
        }

        // Also block cells outside the arena ellipse
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

        Self {
            width,
            height,
            ground_pixels,
            props,
            colliders,
            nav_grid,
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
        }
    }
}
