use super::world::Collider;
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::ShadowCaster;
use crate::engine::text::{FontSize, TextRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Blacksmith,
    Firekeeper,
}

pub struct Npc {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub dialogue_timer: f32,
    pub dialogue_text: &'static str,
}

impl Npc {
    pub fn new(kind: NpcKind, x: f32, y: f32) -> Self {
        Self {
            kind,
            x,
            y,
            anim_timer: 0.0,
            anim_frame: 0,
            dialogue_timer: 0.0,
            dialogue_text: "...",
        }
    }

    pub fn get_collider(&self) -> Collider {
        match self.kind {
            NpcKind::Blacksmith => Collider::new(self.x - 9.0, self.y - 4.0, 18.0, 8.0),
            NpcKind::Firekeeper => Collider::new(self.x - 6.0, self.y - 3.0, 12.0, 6.0),
        }
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        match self.kind {
            NpcKind::Blacksmith => ShadowCaster {
                x: self.x,
                y: self.y,
                width: 18.0,
                height: 18.0,
            },
            NpcKind::Firekeeper => ShadowCaster {
                x: self.x,
                y: self.y,
                width: 12.0,
                height: 20.0,
            },
        }
    }

    pub fn is_player_in_range(&self, px: f32, py: f32) -> bool {
        let dx = px - self.x;
        let dy = (py - self.y) * 1.4;
        (dx * dx + dy * dy).sqrt() < 22.0
    }

    pub fn update(&mut self, dt: f32, px: f32, py: f32, input: &InputState) {
        // Idle animation loop
        let speed = match self.kind {
            NpcKind::Blacksmith => 2.0,
            NpcKind::Firekeeper => 1.2,
        };
        self.anim_timer += dt * speed;
        self.anim_frame = (self.anim_timer as usize) % 2;

        // Interaction
        if self.is_player_in_range(px, py) && input.just_pressed(VirtualKey::E) {
            self.dialogue_timer = 3.0;
            self.dialogue_text = match self.kind {
                NpcKind::Blacksmith => "...",
                NpcKind::Firekeeper => "...",
            };
        }

        if self.dialogue_timer > 0.0 {
            self.dialogue_timer -= dt;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        match self.kind {
            NpcKind::Blacksmith => {
                let draw_x = (self.x - 10.0).round() as i32 - cam_x;
                let draw_y = (self.y - 18.0).round() as i32 - cam_y;
                bank.blacksmith_anim[self.anim_frame].draw(fb, draw_x, draw_y, false);
            }
            NpcKind::Firekeeper => {
                let draw_x = (self.x - 7.0).round() as i32 - cam_x;
                let draw_y = (self.y - 20.0).round() as i32 - cam_y;
                bank.firekeeper_anim[self.anim_frame].draw(fb, draw_x, draw_y, false);
            }
        }
    }

    pub fn draw_ui(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32, px: f32, py: f32) {
        let scr_x = (self.x - cam_x as f32).round() as i32;
        let scr_y = (self.y - cam_y as f32).round() as i32;

        if self.dialogue_timer > 0.0 {
            // Speech bubble
            TextRenderer::draw_speech_bubble(
                fb,
                scr_x,
                scr_y - 20,
                self.dialogue_text,
                Palette::TEXT_BRIGHT,
                Palette::UI_PANEL_BG,
                Palette::UI_BORDER,
            );
        } else if self.is_player_in_range(px, py) {
            // "[E] Talk" prompt
            let prompt = "[E] TALK";
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = scr_x - prompt_w / 2;
            let by = scr_y - 26;

            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_BORDER, false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_NORMAL);
        }
    }
}
