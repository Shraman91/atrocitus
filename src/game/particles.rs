use crate::assets::palette::Palette;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleType {
    Ember,
    Ash,
    Dust,
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max_life: f32,
    pub p_type: ParticleType,
    pub active: bool,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.0,
            max_life: 1.0,
            p_type: ParticleType::Ember,
            active: false,
        }
    }
}

pub struct ParticleSystem {
    pool: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: vec![Particle::default(); capacity],
        }
    }

    pub fn spawn_ember(&mut self, x: f32, y: f32, vx: f32, vy: f32, life: f32) {
        if let Some(p) = self.pool.iter_mut().find(|p| !p.active) {
            p.x = x;
            p.y = y;
            p.vx = vx;
            p.vy = vy;
            p.life = life;
            p.max_life = life;
            p.p_type = ParticleType::Ember;
            p.active = true;
        }
    }

    pub fn spawn_ash(&mut self, x: f32, y: f32) {
        if let Some(p) = self.pool.iter_mut().find(|p| !p.active) {
            let mut rng = rand::rng();
            p.x = x;
            p.y = y;
            p.vx = rng.random_range(-1.5..1.5);
            p.vy = rng.random_range(-2.5..-0.5);
            p.life = rng.random_range(2.0..5.0);
            p.max_life = p.life;
            p.p_type = ParticleType::Ash;
            p.active = true;
        }
    }

    pub fn spawn_dust(&mut self, x: f32, y: f32) {
        if let Some(p) = self.pool.iter_mut().find(|p| !p.active) {
            let mut rng = rand::rng();
            p.x = x;
            p.y = y;
            p.vx = rng.random_range(-1.0..1.0);
            p.vy = rng.random_range(-0.5..0.2);
            p.life = rng.random_range(0.3..0.6);
            p.max_life = p.life;
            p.p_type = ParticleType::Dust;
            p.active = true;
        }
    }

    pub fn spawn_burst(&mut self, cx: f32, cy: f32, count: usize) {
        let mut rng = rand::rng();
        for _ in 0..count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let speed = rng.random_range(8.0..28.0);
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed * 0.6; // perspective flatten
            self.spawn_ember(cx, cy, vx, vy, rng.random_range(0.8..1.8));
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in self.pool.iter_mut().filter(|p| p.active) {
            p.life -= dt;
            if p.life <= 0.0 {
                p.active = false;
                continue;
            }

            p.x += p.vx * dt;
            p.y += p.vy * dt;

            // Gentle wind and deceleration
            match p.p_type {
                ParticleType::Ember => {
                    p.vy -= 4.0 * dt; // rise upwards
                    p.vx += ((p.y * 0.1).sin() * 3.0) * dt;
                }
                ParticleType::Ash => {
                    p.vx += ((p.y * 0.05).cos() * 1.5) * dt;
                }
                ParticleType::Dust => {
                    p.vx *= 0.85;
                    p.vy *= 0.85;
                }
            }
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32) {
        for p in self.pool.iter().filter(|p| p.active) {
            let sx = (p.x - cam_x as f32).round() as i32;
            let sy = (p.y - cam_y as f32).round() as i32;

            if !fb.in_bounds(sx, sy) {
                continue;
            }

            let progress = (1.0 - (p.life / p.max_life)).clamp(0.0, 1.0);

            match p.p_type {
                ParticleType::Ember => {
                    let col = if progress < 0.3 {
                        Palette::BONFIRE_CORE
                    } else if progress < 0.7 {
                        Palette::BONFIRE_YELLOW.lerp(Palette::BONFIRE_ORANGE, (progress - 0.3) / 0.4)
                    } else {
                        Palette::BONFIRE_RED.lerp(Color::BLACK, (progress - 0.7) / 0.3)
                    };
                    fb.set_pixel_add(sx, sy, col);
                }
                ParticleType::Ash => {
                    let alpha = ((1.0 - progress) * 120.0) as u8;
                    let col = Color::rgba(Palette::ASH_WHITE.r, Palette::ASH_WHITE.g, Palette::ASH_WHITE.b, alpha);
                    fb.set_pixel_blend(sx, sy, col);
                }
                ParticleType::Dust => {
                    let alpha = ((1.0 - progress) * 80.0) as u8;
                    let col = Color::rgba(Palette::STONE_MID.r, Palette::STONE_MID.g, Palette::STONE_MID.b, alpha);
                    fb.set_pixel_blend(sx, sy, col);
                }
            }
        }
    }
}
