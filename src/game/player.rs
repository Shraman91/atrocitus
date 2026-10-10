use super::particles::ParticleSystem;
use super::world::{Collider, World};
use crate::assets::sprites::{Direction4, SpriteBank};
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::ShadowCaster;

pub const PLAYER_WALK_SPEED: f32 = 42.0;
pub const PLAYER_RUN_SPEED: f32 = 58.0;

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub dir4: Direction4,
    pub flip_x: bool,
    pub is_moving: bool,
    pub is_running: bool,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub dust_timer: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            dir4: Direction4::South,
            flip_x: false,
            is_moving: false,
            is_running: false,
            anim_timer: 0.0,
            anim_frame: 0,
            dust_timer: 0.0,
        }
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
        let (dx, dy) = input.movement_vector();
        self.is_moving = dx != 0.0 || dy != 0.0;
        self.is_running = input.is_held(VirtualKey::Shift);

        let speed = if self.is_running {
            PLAYER_RUN_SPEED
        } else {
            PLAYER_WALK_SPEED
        };

        if self.is_moving {
            // Determine facing direction
            if dy > 0.3 {
                if dx.abs() > 0.3 {
                    self.dir4 = Direction4::SouthEast;
                    self.flip_x = dx < 0.0;
                } else {
                    self.dir4 = Direction4::South;
                    self.flip_x = false;
                }
            } else if dy < -0.3 {
                if dx.abs() > 0.3 {
                    self.dir4 = Direction4::East;
                    self.flip_x = dx < 0.0;
                } else {
                    self.dir4 = Direction4::North;
                    self.flip_x = false;
                }
            } else if dx.abs() > 0.3 {
                self.dir4 = Direction4::East;
                self.flip_x = dx < 0.0;
            }

            // Movement with separate axis collision checking for smooth sliding along walls
            let move_x = dx * speed * dt;
            let move_y = dy * speed * dt;

            // Try moving X
            let test_x_collider = Collider::new(self.x + move_x - 5.0, self.y - 3.0, 10.0, 6.0);
            if !world.check_collision(&test_x_collider) {
                self.x += move_x;
            }

            // Try moving Y
            let test_y_collider = Collider::new(self.x - 5.0, self.y + move_y - 3.0, 10.0, 6.0);
            if !world.check_collision(&test_y_collider) {
                self.y += move_y;
            }

            // Update walk animation cycle
            self.anim_timer += dt * (if self.is_running { 14.0 } else { 10.0 });
            self.anim_frame = (self.anim_timer as usize) % 4;

            // Spawn footstep dust particles
            self.dust_timer += dt;
            if self.dust_timer > (if self.is_running { 0.12 } else { 0.20 }) {
                self.dust_timer = 0.0;
                particles.spawn_dust(self.x + if self.flip_x { 2.0 } else { -2.0 }, self.y);
            }
        } else {
            // Idle breathing animation
            self.anim_timer += dt * 3.0;
            self.anim_frame = 0;
            self.dust_timer = 0.0;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        let draw_x = (self.x - 8.0).round() as i32 - cam_x;
        let draw_y = (self.y - 20.0).round() as i32 - cam_y;

        let sprite = if self.is_moving {
            let dir_idx = match self.dir4 {
                Direction4::South => 0,
                Direction4::North => 1,
                Direction4::East => 2,
                Direction4::SouthEast => 3,
            };
            &bank.knight_walk[dir_idx][self.anim_frame]
        } else {
            let dir_idx = match self.dir4 {
                Direction4::South => 0,
                Direction4::North => 1,
                Direction4::East => 2,
                Direction4::SouthEast => 3,
            };
            &bank.knight_idle[dir_idx]
        };

        sprite.draw(fb, draw_x, draw_y, self.flip_x);
    }
}
