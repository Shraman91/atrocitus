use super::lobby::LobbyScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::InputState;
use crate::engine::postprocess::PostProcessor;
use crate::engine::text::{FontSize, TextRenderer};
use crate::game::world::World;

const LORE_TIPS: &[&str] = &[
    "Embers remember what the living forget.",
    "In the deep below, even steel turns to ash.",
    "A flame unkept seeks fuel in the soul.",
    "Rest at the coiled blade to restore weary flesh.",
    "The bells have ceased their tolling across the ruins.",
];

const STAGES: &[&str] = &[
    "Forging sprite tables & glyphs...",
    "Constructing The Ashen Sanctum...",
    "Kindling the sacred bonfire...",
    "Harmonizing light & shadow...",
    "Awakening the Champion of Ash...",
];

pub struct LoadingScene {
    time: f32,
    stage_index: usize,
    lore_index: usize,
    lore_timer: f32,
    bonfire_frame: usize,
    bonfire_timer: f32,
    min_time: f32,
    completed: bool,
    world_cache: Option<World>,
}

impl Default for LoadingScene {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadingScene {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            stage_index: 0,
            lore_index: 0,
            lore_timer: 0.0,
            bonfire_frame: 0,
            bonfire_timer: 0.0,
            min_time: 2.6,
            completed: false,
            world_cache: None,
        }
    }
}

impl Scene for LoadingScene {
    fn update(
        &mut self,
        dt: f32,
        _input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;

        // Bonfire animation loop
        self.bonfire_timer += dt * 10.0;
        self.bonfire_frame = (self.bonfire_timer as usize) % 4;

        // Lore tips cycling
        self.lore_timer += dt;
        if self.lore_timer > 3.0 {
            self.lore_timer = 0.0;
            self.lore_index = (self.lore_index + 1) % LORE_TIPS.len();
        }

        // Staged initialization progress
        let stage_progress = (self.time / self.min_time).clamp(0.0, 1.0);
        self.stage_index = ((stage_progress * (STAGES.len() as f32 - 0.01)) as usize).min(STAGES.len() - 1);

        // Perform real world generation at stage 1 if not yet built
        if self.stage_index >= 1 && self.world_cache.is_none() {
            self.world_cache = Some(World::build(240, 160));
        }

        if self.time >= self.min_time && !self.completed {
            self.completed = true;
            let world = self.world_cache.take().unwrap_or_else(|| World::build(240, 160));
            return SceneTransition::Switch(Box::new(LobbyScene::new(world)));
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, bank: &SpriteBank) {
        fb.clear(Palette::VOID_BLACK);

        let center_x = fb.width as i32 / 2;
        let center_y = fb.height as i32 / 2;

        // Flickering bonfire in center
        let bf_x = center_x - 9;
        let bf_y = center_y - 14;
        bank.bonfire_anim[self.bonfire_frame].draw(fb, bf_x, bf_y, false);

        // Bonfire glow circle
        let glow_r = 16 + (self.bonfire_frame % 2) as i32;
        fb.draw_circle(center_x, center_y - 4, glow_r, Color::rgba(255, 140, 20, 25), false);

        // Thin Ember Progress Bar
        let bar_w = 70;
        let bar_x = center_x - bar_w / 2;
        let bar_y = center_y + 16;
        let progress = (self.time / self.min_time).clamp(0.0, 1.0);
        let fill_w = (progress * bar_w as f32).round() as i32;

        // Bar frame
        fb.draw_rect(bar_x - 2, bar_y - 1, bar_w + 4, 4, Palette::STONE_DARK, false);
        // Bar background
        fb.draw_rect(bar_x, bar_y, bar_w, 2, Palette::VOID_BLACK, true);
        // Bar fill
        if fill_w > 0 {
            fb.draw_rect(bar_x, bar_y, fill_w, 2, Palette::BONFIRE_ORANGE, true);
            // Glowing leading ember
            fb.set_pixel(bar_x + fill_w - 1, bar_y, Palette::BONFIRE_CORE);
            fb.set_pixel(bar_x + fill_w - 1, bar_y + 1, Palette::BONFIRE_CORE);
        }

        // Current Stage Text
        let stage_text = STAGES[self.stage_index];
        TextRenderer::draw_text_centered(
            fb,
            bar_y + 7,
            stage_text,
            FontSize::Tiny3x5,
            Palette::TEXT_GOLD,
        );

        // Rotating Lore Tip at bottom
        let lore_text = LORE_TIPS[self.lore_index];
        TextRenderer::draw_text_centered(
            fb,
            fb.height as i32 - 12,
            lore_text,
            FontSize::Tiny3x5,
            Palette::TEXT_MUTED,
        );

        // Post-processing
        PostProcessor::apply_vignette(fb, 0.5);
    }
}
