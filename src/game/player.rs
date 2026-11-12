use super::classes::ClassKind;
use super::equipment::LayeredGear;
use super::particles::ParticleSystem;
use super::world::{Collider, World};
use crate::assets::sprites::{Direction4, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::ShadowCaster;

pub const PLAYER_WALK_SPEED: f32 = 44.0;
pub const PLAYER_RUN_SPEED: f32 = 64.0;
pub const ROLL_DURATION: f32 = 0.45;
pub const ROLL_COOLDOWN: f32 = 0.80;
pub const ROLL_DISTANCE: f32 = 65.0;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub dir4: Direction4,
    pub flip_x: bool,
    pub facing_angle: f32, // Continuous angle for 8-direction hysteresis
    pub is_moving: bool,
    pub is_running: bool,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub dust_timer: f32,
    pub equipped_class: Option<ClassKind>,
    pub gear: LayeredGear,

    // Dodge Roll state machine
    pub is_rolling: bool,
    pub roll_timer: f32,
    pub roll_cooldown: f32,
    pub roll_dir: (f32, f32),
    pub roll_buffered: bool,
    pub roll_ghosts: Vec<(f32, f32, f32, Direction4, bool)>, // (x, y, life, dir, flip)
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            dir4: Direction4::South,
            flip_x: false,
            facing_angle: std::f32::consts::FRAC_PI_2, // facing South initially
            is_moving: false,
            is_running: false,
            anim_timer: 0.0,
            anim_frame: 0,
            dust_timer: 0.0,
            equipped_class: None,
            gear: LayeredGear::empty(),
            is_rolling: false,
            roll_timer: 0.0,
            roll_cooldown: 0.0,
            roll_dir: (0.0, 1.0),
            roll_buffered: false,
            roll_ghosts: Vec::with_capacity(8),
        }
    }

    pub fn set_class(&mut self, class: Option<ClassKind>) {
        self.equipped_class = class;
    }

    pub fn get_collider(&self) -> Collider {
        Collider::new(self.x - 5.0, self.y - 3.0, 10.0, 6.0)
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        ShadowCaster {
            x: self.x,
            y: self.y,
            width: 12.0,
            height: 20.0,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        world: &World,
        particles: &mut ParticleSystem,
    ) {
        // 1. Decay ghost trails
        self.roll_ghosts.retain_mut(|(_, _, life, _, _)| {
            *life -= dt * 3.5;
            *life > 0.0
        });

        // 2. Roll Cooldown & Input Buffering
        if self.roll_cooldown > 0.0 {
            self.roll_cooldown -= dt;

            // In final 100ms of recovery, buffer roll input
            if self.roll_cooldown <= 0.10 {
                let wants_roll_buffer = input.mouse_right_just_pressed
                    || input.just_pressed(VirtualKey::Space);
                if wants_roll_buffer {
                    self.roll_buffered = true;
                }
            }
        }

        // 3. Aiming: Continuously face mouse cursor using 8-direction hysteresis
        if !self.is_rolling && input.mouse_ever_detected {
            let aim_dx = input.mouse_world_pos.0 - self.x;
            let aim_dy = input.mouse_world_pos.1 - self.y;
            let (new_dir, new_flip) = update_direction_hysteresis(
                aim_dx,
                aim_dy,
                &mut self.facing_angle,
            );
            self.dir4 = new_dir;
            self.flip_x = new_flip;
        }

        // 4. Trigger Dodge Roll
        let wants_roll = (input.mouse_right_just_pressed || input.just_pressed(VirtualKey::Space) || self.roll_buffered)
            && self.roll_cooldown <= 0.0;

        if wants_roll {
            self.roll_buffered = false;
            self.is_rolling = true;
            self.roll_timer = ROLL_DURATION;
            self.roll_cooldown = ROLL_COOLDOWN;

            // Aim direction towards mouse
            let aim_dx = input.mouse_world_pos.0 - self.x;
            let aim_dy = input.mouse_world_pos.1 - self.y;
            let aim_len = (aim_dx * aim_dx + aim_dy * aim_dy).sqrt().max(0.001);
            self.roll_dir = (aim_dx / aim_len, aim_dy / aim_len);

            // Lock facing direction to roll direction
            let (r_dir, r_flip) = update_direction_hysteresis(
                self.roll_dir.0,
                self.roll_dir.1,
                &mut self.facing_angle,
            );
            self.dir4 = r_dir;
            self.flip_x = r_flip;

            self.roll_ghosts.push((self.x, self.y, 1.0, self.dir4, self.flip_x));
            particles.spawn_dust(self.x, self.y);
        }

        // 5. Active Dodge Roll Execution (Ease-Out curve covering ~65px)
        if self.is_rolling {
            self.roll_timer -= dt;
            if self.roll_timer <= 0.0 {
                self.is_rolling = false;
                self.roll_timer = 0.0;
            } else {
                // Ease-out quadratic velocity: v(t) = v0 * (t_remaining / T)
                // Integrating over T gives total distance = 0.5 * v0 * T = ROLL_DISTANCE
                let v0 = (2.0 * ROLL_DISTANCE) / ROLL_DURATION;
                let current_roll_speed = v0 * (self.roll_timer / ROLL_DURATION);

                let move_x = self.roll_dir.0 * current_roll_speed * dt;
                let move_y = self.roll_dir.1 * current_roll_speed * dt;

                // Wall collision sliding
                let test_x_col = Collider::new(self.x + move_x - 5.0, self.y - 3.0, 10.0, 6.0);
                if !world.check_collision(&test_x_col) {
                    self.x += move_x;
                }
                let test_y_col = Collider::new(self.x - 5.0, self.y + move_y - 3.0, 10.0, 6.0);
                if !world.check_collision(&test_y_col) {
                    self.y += move_y;
                }

                // Animation & dust
                self.anim_timer += dt * 18.0;
                self.anim_frame = (self.anim_timer as usize) % 4;

                self.dust_timer += dt;
                if self.dust_timer > 0.06 {
                    self.dust_timer = 0.0;
                    self.roll_ghosts.push((self.x, self.y, 0.75, self.dir4, self.flip_x));
                    particles.spawn_dust(self.x, self.y);
                }
                return;
            }
        }

        // 6. Keyboard-Driven Movement with Smooth Acceleration & Deceleration
        let (key_dx, key_dy) = input.movement_vector();
        self.is_running = input.is_held(VirtualKey::Shift);

        let max_speed = if self.is_running {
            PLAYER_RUN_SPEED
        } else {
            PLAYER_WALK_SPEED
        };

        let target_vx = key_dx * max_speed;
        let target_vy = key_dy * max_speed;

        // Acceleration rate (~0.12s to full speed) and deceleration rate (~0.08s to stop)
        let accel_rate = max_speed / 0.12;
        let decel_rate = max_speed / 0.08;

        let rate_x = if target_vx.abs() > self.vx.abs() { accel_rate } else { decel_rate };
        let rate_y = if target_vy.abs() > self.vy.abs() { accel_rate } else { decel_rate };

        self.vx = approach(self.vx, target_vx, rate_x * dt);
        self.vy = approach(self.vy, target_vy, rate_y * dt);

        let current_speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        self.is_moving = current_speed > 1.0;

        if self.is_moving {
            let move_x = self.vx * dt;
            let move_y = self.vy * dt;

            // Separate X and Y collision sliding against colliders
            let test_x_col = Collider::new(self.x + move_x - 5.0, self.y - 3.0, 10.0, 6.0);
            if !world.check_collision(&test_x_col) {
                self.x += move_x;
            } else {
                self.vx = 0.0;
            }

            let test_y_col = Collider::new(self.x - 5.0, self.y + move_y - 3.0, 10.0, 6.0);
            if !world.check_collision(&test_y_col) {
                self.y += move_y;
            } else {
                self.vy = 0.0;
            }

            // Walk animation speed proportional to actual velocity
            let anim_speed_factor = (current_speed / PLAYER_WALK_SPEED).clamp(0.5, 2.0);
            self.anim_timer += dt * 10.0 * anim_speed_factor;
            self.anim_frame = (self.anim_timer as usize) % 4;

            self.dust_timer += dt;
            let dust_interval = if self.is_running { 0.15 } else { 0.25 };
            if self.dust_timer > dust_interval {
                self.dust_timer = 0.0;
                particles.spawn_dust(self.x + if self.flip_x { 2.0 } else { -2.0 }, self.y);
            }
        } else {
            self.anim_timer += dt * 3.0;
            self.anim_frame = 0;
            self.dust_timer = 0.0;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        let draw_x = (self.x - 8.0).round() as i32 - cam_x;
        let draw_y = (self.y - 20.0).round() as i32 - cam_y;

        // 1. Contact shadow ellipse under character feet
        let shadow_cx = draw_x + 8;
        let shadow_cy = draw_y + 20;
        fb.draw_ellipse(shadow_cx, shadow_cy, 6, 2, Color::rgba(5, 6, 12, 130));

        // 2. Draw dash roll motion blur ghost trails
        for &(gx, gy, glife, gdir, gflip) in &self.roll_ghosts {
            let g_sx = (gx - 8.0).round() as i32 - cam_x;
            let g_sy = (gy - 20.0).round() as i32 - cam_y;
            let g_dir_idx = match gdir {
                Direction4::South => 0,
                Direction4::North => 1,
                Direction4::East => 2,
                Direction4::SouthEast => 3,
            };
            let body = &bank.body_idle[g_dir_idx];
            let alpha = (glife * 110.0).clamp(0.0, 255.0) as u8;
            body.draw_alpha(fb, g_sx, g_sy, gflip, alpha);
        }

        let dir_idx = match self.dir4 {
            Direction4::South => 0,
            Direction4::North => 1,
            Direction4::East => 2,
            Direction4::SouthEast => 3,
        };

        // 3. Base Body sprite
        let body_sprite = if self.is_moving || self.is_rolling {
            &bank.body_walk[dir_idx][self.anim_frame]
        } else {
            &bank.body_idle[dir_idx]
        };

        // 4. Cosmetic i-frame glow during roll
        if self.is_rolling {
            let glow_alpha = ((self.roll_timer / ROLL_DURATION) * 70.0) as u8;
            let glow_col = Color::rgba(255, 230, 180, glow_alpha);
            fb.draw_circle(shadow_cx, shadow_cy - 10, 12, glow_col, false);
        }

        // 5. Layered gear composite rendering
        if let Some(gear_set) = self.equipped_class.and_then(|class| bank.get_class_gear(class)) {
            let gear_sprites: [Option<&crate::assets::sprites::Sprite>; 6] = [
                Some(&gear_set.head[dir_idx]),
                Some(&gear_set.torso[dir_idx]),
                gear_set.cape[dir_idx].as_ref(),
                gear_set.main_hand[dir_idx].as_ref(),
                gear_set.off_hand[dir_idx].as_ref(),
                gear_set.accessory[dir_idx].as_ref(),
            ];
            self.gear.draw_composite(
                fb,
                body_sprite,
                &gear_sprites,
                draw_x,
                draw_y,
                self.dir4,
                self.flip_x,
            );
            return;
        }

        // Unarmored Adventurer default
        let empty_gear: [Option<&crate::assets::sprites::Sprite>; 6] = [None, None, None, None, None, None];
        self.gear.draw_composite(
            fb,
            body_sprite,
            &empty_gear,
            draw_x,
            draw_y,
            self.dir4,
            self.flip_x,
        );
    }
}

/// Helper function to smoothly approach a target value with a max step
#[inline]
fn approach(current: f32, target: f32, max_step: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_step {
        target
    } else {
        current + diff.signum() * max_step
    }
}

/// 8-direction facing calculation with a 10-degree hysteresis buffer
fn update_direction_hysteresis(
    aim_dx: f32,
    aim_dy: f32,
    current_angle: &mut f32,
) -> (Direction4, bool) {
    if aim_dx.abs() < 0.001 && aim_dy.abs() < 0.001 {
        return (Direction4::South, false);
    }

    let target_angle = aim_dy.atan2(aim_dx);
    let mut diff = target_angle - *current_angle;
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }

    // 10 degrees hysteresis threshold (0.1745 radians)
    if diff.abs() > 0.1745 {
        *current_angle = target_angle;
    }

    let mut deg = current_angle.to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }

    // 8 sectors:
    // Sector 0: East (337.5..360, 0..22.5) -> East, false
    // Sector 1: SouthEast (22.5..67.5)     -> SouthEast, false
    // Sector 2: South (67.5..112.5)        -> South, false
    // Sector 3: SouthWest (112.5..157.5)   -> SouthEast, true
    // Sector 4: West (157.5..202.5)        -> East, true
    // Sector 5: NorthWest (202.5..247.5)   -> East, true
    // Sector 6: North (247.5..292.5)       -> North, false
    // Sector 7: NorthEast (292.5..337.5)   -> East, false
    match deg {
        d if (22.5..67.5).contains(&d) => (Direction4::SouthEast, false),
        d if (67.5..112.5).contains(&d) => (Direction4::South, false),
        d if (112.5..157.5).contains(&d) => (Direction4::SouthEast, true),
        d if (157.5..247.5).contains(&d) => (Direction4::East, true),
        d if (247.5..292.5).contains(&d) => (Direction4::North, false),
        _ => (Direction4::East, false),
    }
}
