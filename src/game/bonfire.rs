use super::particles::ParticleSystem;
use super::world::Collider;
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::{PointLight, ShadowCaster};
use crate::engine::text::{FontSize, TextRenderer};
use rand::RngExt;

pub struct Bonfire {
    pub x: f32,
    pub y: f32,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub ember_spawn_timer: f32,
    pub is_resting: bool,
    pub rest_timer: f32,
    pub rest_banner_timer: f32,
}

impl Bonfire {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            anim_timer: 0.0,
            anim_frame: 0,
            ember_spawn_timer: 0.0,
            is_resting: false,
            rest_timer: 0.0,
            rest_banner_timer: 0.0,
        }
    }

    pub fn get_collider(&self) -> Collider {
        Collider::new(self.x - 7.0, self.y - 4.0, 14.0, 8.0)
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        ShadowCaster {
            x: self.x,
            y: self.y,
            width: 14.0,
            height: 16.0,
        }
    }

    pub fn is_player_in_range(&self, px: f32, py: f32) -> bool {
        let dx = px - self.x;
        let dy = (py - self.y) * 1.5; // vertical perspective
        (dx * dx + dy * dy).sqrt() < 24.0
    }

    pub fn get_light(&self) -> PointLight {
        let resting_boost = if self.is_resting { 1.3 } else { 1.0 };
        PointLight {
            x: self.x,
            y: self.y - 4.0,
            radius: 88.0 * resting_boost,
            intensity: 1.65 * resting_boost,
            color: Palette::BONFIRE_YELLOW,
            flicker_amount: 0.08,
        }
    }

    pub fn update(
        &mut self,
        dt: f32,
        px: f32,
        py: f32,
        input: &InputState,
        particles: &mut ParticleSystem,
    ) {
        // Flame animation loop
        self.anim_timer += dt * 10.0;
        self.anim_frame = (self.anim_timer as usize) % 4;

        // Continuous rising bonfire embers
        self.ember_spawn_timer += dt;
        if self.ember_spawn_timer > 0.06 {
            self.ember_spawn_timer = 0.0;
            let mut rng = rand::rng();
            let ox = rng.random_range(-5.0..5.0);
            let oy = rng.random_range(-4.0..2.0);
            let vx = rng.random_range(-3.0..3.0);
            let vy = rng.random_range(-12.0..-6.0);
            let life = rng.random_range(1.0..2.2);
            particles.spawn_ember(self.x + ox, self.y + oy - 6.0, vx, vy, life);
        }

        // Handle rest interaction
        if self.is_player_in_range(px, py) && input.just_pressed(VirtualKey::E) {
            self.is_resting = true;
            self.rest_timer = 1.8;
            self.rest_banner_timer = 3.5;
            particles.spawn_burst(self.x, self.y - 6.0, 40);
        }

        if self.is_resting {
            self.rest_timer -= dt;
            if self.rest_timer <= 0.0 {
                self.is_resting = false;
            }
        }

        if self.rest_banner_timer > 0.0 {
            self.rest_banner_timer -= dt;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        let draw_x = (self.x - 9.0).round() as i32 - cam_x;
        let draw_y = (self.y - 18.0).round() as i32 - cam_y;
        bank.bonfire_anim[self.anim_frame].draw(fb, draw_x, draw_y, false);
    }

    pub fn draw_ui(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32, px: f32, py: f32) {
        // "[E] Rest at bonfire" prompt
        if self.is_player_in_range(px, py) && !self.is_resting && self.rest_banner_timer <= 0.0 {
            let prompt = "[E] REST AT BONFIRE";
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = (self.x - cam_x as f32).round() as i32 - prompt_w / 2;
            let by = (self.y - cam_y as f32).round() as i32 - 28;

            fb.draw_rect(bx - 3, by - 2, prompt_w + 6, 9, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 3, by - 2, prompt_w + 6, 9, Palette::UI_BORDER, false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_GOLD);
        }

        // "BONFIRE RESTED" dramatic announcement banner
        if self.rest_banner_timer > 0.0 {
            let banner = "BONFIRE RESTED";
            let banner_w = TextRenderer::measure_width(banner, FontSize::Standard5x7);
            let center_x = (fb.width as i32 - banner_w) / 2;
            let center_y = fb.height as i32 / 3;

            // Translucent banner backdrop
            let alpha = if self.rest_banner_timer > 3.0 {
                (3.5 - self.rest_banner_timer) / 0.5
            } else if self.rest_banner_timer < 0.8 {
                self.rest_banner_timer / 0.8
            } else {
                1.0
            };

            let bg_alpha = (alpha * 180.0) as u8;
            let bar_y = center_y - 4;
            fb.draw_rect(0, bar_y, fb.width as i32, 17, Color::rgba(5, 6, 10, bg_alpha), true);
            fb.draw_line(0, bar_y, fb.width as i32 - 1, bar_y, Color::rgba(255, 140, 20, (alpha * 150.0) as u8));
            fb.draw_line(0, bar_y + 16, fb.width as i32 - 1, bar_y + 16, Color::rgba(255, 140, 20, (alpha * 150.0) as u8));

            let text_col = Palette::TEXT_GOLD.scale(alpha);
            TextRenderer::draw_text(fb, center_x, center_y, banner, FontSize::Standard5x7, text_col);
        }
    }
}
