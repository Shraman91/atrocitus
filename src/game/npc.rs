use super::world::Collider;
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::lighting::{PointLight, ShadowCaster};
use crate::engine::text::{FontSize, TextRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Firekeeper,
    Blacksmith,
    UnknownAdventurer,
    ShadyWitch,
}

pub struct Npc {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub anim_timer: f32,
    pub anim_frame: usize,
}

impl Npc {
    pub fn new(kind: NpcKind, x: f32, y: f32) -> Self {
        Self {
            kind,
            x,
            y,
            anim_timer: 0.0,
            anim_frame: 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            NpcKind::Firekeeper => "THE VEILED FIREKEEPER",
            NpcKind::Blacksmith => "BLACKSMITH VALEN",
            NpcKind::UnknownAdventurer => "UNKNOWN ADVENTURER",
            NpcKind::ShadyWitch => "SHADY WITCH MORGANA",
        }
    }

    pub fn action_prompt(&self) -> &'static str {
        match self.kind {
            NpcKind::Firekeeper => "[E / Click] CLAIM MANTLE",
            NpcKind::Blacksmith => "[E / Click] FORGE & UPGRADE",
            NpcKind::UnknownAdventurer => "[E / Click] LISTEN TO TALE",
            NpcKind::ShadyWitch => "[E / Click] ARCANE SHOP",
        }
    }

    pub fn accent_color(&self) -> Color {
        match self.kind {
            NpcKind::Firekeeper => Palette::BONFIRE_ORANGE,
            NpcKind::Blacksmith => Palette::BONFIRE_RED,
            NpcKind::UnknownAdventurer => Palette::MAGE_CYAN,
            NpcKind::ShadyWitch => Color::rgb(190, 80, 245),
        }
    }

    pub fn get_collider(&self) -> Collider {
        let (w, h) = match self.kind {
            NpcKind::Blacksmith => (16.0, 10.0),
            _ => (14.0, 8.0),
        };
        Collider::new(self.x - w * 0.5, self.y - h * 0.5, w, h)
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        let (w, h) = match self.kind {
            NpcKind::Blacksmith => (16.0, 24.0),
            _ => (12.0, 20.0),
        };
        ShadowCaster {
            x: self.x,
            y: self.y,
            width: w,
            height: h,
        }
    }

    pub fn get_light(&self, time: f32) -> PointLight {
        let flicker = ((time * 7.5).sin() * 0.08 + (time * 19.0).cos() * 0.04) + 0.9;
        match self.kind {
            NpcKind::Firekeeper => PointLight {
                x: self.x + 5.0,
                y: self.y - 6.0,
                radius: 36.0 * flicker,
                intensity: 0.85 * flicker,
                color: Palette::BONFIRE_YELLOW,
                flicker_amount: 0.08,
            },
            NpcKind::Blacksmith => PointLight {
                x: self.x + 4.0,
                y: self.y - 8.0,
                radius: 42.0 * flicker,
                intensity: 0.90 * flicker,
                color: Color::rgb(255, 90, 30), // deep forge coals glow
                flicker_amount: 0.12,
            },
            NpcKind::UnknownAdventurer => PointLight {
                x: self.x,
                y: self.y - 10.0,
                radius: 26.0,
                intensity: 0.40,
                color: Palette::MAGE_CYAN,
                flicker_amount: 0.03,
            },
            NpcKind::ShadyWitch => PointLight {
                x: self.x + 5.0,
                y: self.y - 8.0,
                radius: 34.0 * flicker,
                intensity: 0.75 * flicker,
                color: Color::rgb(180, 70, 235), // violet cauldron flame
                flicker_amount: 0.07,
            },
        }
    }

    pub fn is_player_in_range(&self, px: f32, py: f32) -> bool {
        let dx = px - self.x;
        let dy = (py - self.y) * 1.3;
        (dx * dx + dy * dy).sqrt() < 28.0
    }

    pub fn is_mouse_hovered(&self, mx: f32, my: f32) -> bool {
        let dx = (mx - self.x).abs();
        let dy = (my - (self.y - 12.0)).abs();
        dx < 14.0 && dy < 16.0
    }

    pub fn update(&mut self, dt: f32) {
        let speed = match self.kind {
            NpcKind::Blacksmith => 2.5, // hammer rhythm
            _ => 1.5,
        };
        self.anim_timer += dt * speed;
        self.anim_frame = (self.anim_timer as usize) % 4;
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        let (sprite, draw_x, draw_y, shadow_rx) = match self.kind {
            NpcKind::Firekeeper => (
                &bank.firekeeper_anim[self.anim_frame],
                (self.x - 9.0).round() as i32 - cam_x,
                (self.y - 28.0).round() as i32 - cam_y,
                7,
            ),
            NpcKind::Blacksmith => (
                &bank.blacksmith_anim[self.anim_frame],
                (self.x - 11.0).round() as i32 - cam_x,
                (self.y - 32.0).round() as i32 - cam_y,
                9,
            ),
            NpcKind::UnknownAdventurer => (
                &bank.adventurer_anim[self.anim_frame],
                (self.x - 9.0).round() as i32 - cam_x,
                (self.y - 28.0).round() as i32 - cam_y,
                7,
            ),
            NpcKind::ShadyWitch => (
                &bank.witch_anim[self.anim_frame],
                (self.x - 9.0).round() as i32 - cam_x,
                (self.y - 28.0).round() as i32 - cam_y,
                7,
            ),
        };

        // Contact shadow ellipse under feet
        let foot_x = (self.x - cam_x as f32).round() as i32;
        let foot_y = (self.y - cam_y as f32).round() as i32;
        fb.draw_ellipse(foot_x, foot_y, shadow_rx, 2, Color::rgba(4, 5, 10, 130));

        // Draw character sprite
        sprite.draw(fb, draw_x, draw_y, false);

        // Overhead pulsing marker glyph
        let accent = self.accent_color();
        let pulse = ((self.anim_timer * 3.0).sin() * 0.25 + 0.75).clamp(0.0, 1.0);
        let head_y = draw_y - 4;
        let head_x = foot_x;
        fb.set_pixel_add(head_x, head_y, accent.scale(pulse));
        fb.set_pixel_add(head_x - 1, head_y + 1, accent.scale(pulse * 0.7));
        fb.set_pixel_add(head_x + 1, head_y + 1, accent.scale(pulse * 0.7));
        fb.set_pixel_add(head_x, head_y + 2, accent.scale(pulse));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_ui(
        &self,
        fb: &mut Framebuffer,
        cam_x: i32,
        cam_y: i32,
        px: f32,
        py: f32,
        mx: f32,
        my: f32,
    ) {
        let in_range = self.is_player_in_range(px, py);
        let hovered = self.is_mouse_hovered(mx, my);

        if in_range || hovered {
            let scr_x = (self.x - cam_x as f32).round() as i32;
            let scr_y = (self.y - cam_y as f32).round() as i32;

            let prompt = self.action_prompt();
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = scr_x - prompt_w / 2;
            let by = scr_y - 36;

            fb.draw_rect(bx - 3, by - 2, prompt_w + 6, 9, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 3, by - 2, prompt_w + 6, 9, self.accent_color(), false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_GOLD);

            // Nameplate above prompt if hovered
            if hovered {
                let name = self.name();
                let name_w = TextRenderer::measure_width(name, FontSize::Tiny3x5);
                let nbx = scr_x - name_w / 2;
                let nby = by - 8;
                fb.draw_rect(nbx - 2, nby - 1, name_w + 4, 7, Palette::VOID_BLACK, true);
                TextRenderer::draw_text(fb, nbx, nby, name, FontSize::Tiny3x5, self.accent_color());
            }
        }
    }
}
