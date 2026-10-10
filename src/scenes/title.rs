use super::loading::LoadingScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::postprocess::PostProcessor;
use crate::engine::text::{FontSize, TextRenderer};
use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TitleState {
    PressAnyKey,
    Menu,
    SettingsModal,
}

pub struct TitleScene {
    state: TitleState,
    selected_item: usize,
    time: f32,
    embers: Vec<(f32, f32, f32, f32, f32)>, // x, y, vx, vy, life
}

impl Default for TitleScene {
    fn default() -> Self {
        Self::new()
    }
}

impl TitleScene {
    pub fn new() -> Self {
        let mut embers = Vec::with_capacity(60);
        let mut rng = rand::rng();
        for _ in 0..50 {
            embers.push((
                rng.random_range(0.0..140.0),
                rng.random_range(0.0..90.0),
                rng.random_range(-2.0..2.0),
                rng.random_range(-10.0..-4.0),
                rng.random_range(0.5..3.0),
            ));
        }

        Self {
            state: TitleState::PressAnyKey,
            selected_item: 0,
            time: 0.0,
            embers,
        }
    }

    fn draw_logo(&self, fb: &mut Framebuffer, center_y: i32) {
        // Large stylized ATROCITUS logo title
        let logo_text = "A T R O C I T U S";
        let logo_w = TextRenderer::measure_width(logo_text, FontSize::Standard5x7);
        let start_x = (fb.width as i32 - logo_w) / 2;

        let glow = ((self.time * 2.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
        let logo_col = Palette::BONFIRE_YELLOW.lerp(Palette::BONFIRE_CORE, glow);
        let shadow_col = Palette::BONFIRE_RED.scale(0.6);

        // Shadow / Glow outline
        TextRenderer::draw_text(fb, start_x + 1, center_y + 1, logo_text, FontSize::Standard5x7, shadow_col);
        TextRenderer::draw_text(fb, start_x, center_y, logo_text, FontSize::Standard5x7, logo_col);

        // Decorative horizontal rule with diamond in center
        let line_w = (logo_w + 30).min(fb.width as i32 - 10);
        let line_x = (fb.width as i32 - line_w) / 2;
        let line_y = center_y + 10;
        let line_col = Palette::TEXT_MUTED.scale(0.7);

        fb.draw_line(line_x, line_y, line_x + line_w / 2 - 4, line_y, line_col);
        fb.draw_line(line_x + line_w / 2 + 4, line_y, line_x + line_w, line_y, line_col);
        fb.set_pixel(line_x + line_w / 2, line_y, Palette::BONFIRE_ORANGE);

        // Tagline
        let tag = "A CURSE UPON THE ASHES";
        let tag_w = TextRenderer::measure_width(tag, FontSize::Tiny3x5);
        let tag_x = (fb.width as i32 - tag_w) / 2;
        TextRenderer::draw_text(fb, tag_x, line_y + 4, tag, FontSize::Tiny3x5, Palette::TEXT_MUTED);
    }
}

impl Scene for TitleScene {
    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;

        // Update ember particles
        let mut rng = rand::rng();
        for ember in &mut self.embers {
            ember.4 -= dt;
            if ember.4 <= 0.0 {
                ember.0 = rng.random_range(0.0..140.0);
                ember.1 = 85.0;
                ember.2 = rng.random_range(-2.0..2.0);
                ember.3 = rng.random_range(-12.0..-4.0);
                ember.4 = rng.random_range(2.0..4.0);
            } else {
                ember.0 += ember.2 * dt;
                ember.1 += ember.3 * dt;
                ember.0 += ((ember.1 * 0.1).sin() * 2.0) * dt;
            }
        }

        match self.state {
            TitleState::PressAnyKey => {
                if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
                    return SceneTransition::Switch(Box::new(LoadingScene::new()));
                } else if input.any_key_just_pressed {
                    self.state = TitleState::Menu;
                }
            }
            TitleState::Menu => {
                if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
                    if self.selected_item > 0 {
                        self.selected_item -= 1;
                    } else {
                        self.selected_item = 3;
                    }
                }
                if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
                    if self.selected_item < 3 {
                        self.selected_item += 1;
                    } else {
                        self.selected_item = 0;
                    }
                }
                if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
                    match self.selected_item {
                        0 => return SceneTransition::Switch(Box::new(LoadingScene::new())),
                        1 => {} // Continue disabled
                        2 => self.state = TitleState::SettingsModal,
                        3 => return SceneTransition::Quit,
                        _ => {}
                    }
                }
            }
            TitleState::SettingsModal => {
                if input.just_pressed(VirtualKey::Escape)
                    || input.just_pressed(VirtualKey::Enter)
                    || input.just_pressed(VirtualKey::Space)
                {
                    self.state = TitleState::Menu;
                }
            }
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, _bank: &SpriteBank) {
        // Deep background gradient
        fb.draw_gradient_v(0, 0, fb.width as i32, fb.height as i32, Palette::VOID_BLACK, Palette::DEEP_NIGHT);

        // Distant Spire silhouettes drifting slowly
        let spire_col1 = Palette::ABYSS;
        let spire_col2 = Color::rgb(15, 18, 26);
        let horizon_y = fb.height as i32 - 14;

        for x in 0..fb.width {
            let u1 = x as f32 + self.time * 1.5;
            let peak1 = (u1 * 0.05).sin() * 12.0 + (u1 * 0.12).cos() * 6.0;
            let sy1 = (horizon_y as f32 - 25.0 + peak1).round() as i32;
            for y in sy1..horizon_y {
                fb.set_pixel_blend(x as i32, y, spire_col1);
            }

            let u2 = x as f32 + self.time * 3.0;
            let peak2 = (u2 * 0.08).sin() * 8.0 + (u2 * 0.18).cos() * 4.0;
            let sy2 = (horizon_y as f32 - 15.0 + peak2).round() as i32;
            for y in sy2..fb.height as i32 {
                fb.set_pixel_blend(x as i32, y, spire_col2);
            }
        }

        // Rising ember particles
        for ember in &self.embers {
            let sx = ember.0.round() as i32;
            let sy = ember.1.round() as i32;
            if fb.in_bounds(sx, sy) {
                let col = if ember.4 > 2.0 {
                    Palette::BONFIRE_YELLOW
                } else if ember.4 > 1.0 {
                    Palette::BONFIRE_ORANGE
                } else {
                    Palette::BONFIRE_RED
                };
                fb.set_pixel_add(sx, sy, col);
            }
        }

        // Draw Stylized ATROCITUS Title
        let logo_y = fb.height as i32 / 4;
        self.draw_logo(fb, logo_y);

        // State rendering
        match self.state {
            TitleState::PressAnyKey => {
                let blink = (self.time * 3.0).sin() > 0.0;
                if blink {
                    let prompt = "PRESS ANY KEY TO AWAKEN";
                    TextRenderer::draw_text_centered(
                        fb,
                        fb.height as i32 / 2 + 18,
                        prompt,
                        FontSize::Standard5x7,
                        Palette::TEXT_GOLD,
                    );
                }
            }
            TitleState::Menu => {
                let menu_items = [
                    ("NEW GAME", true),
                    ("CONTINUE", false),
                    ("SETTINGS", true),
                    ("QUIT", true),
                ];

                let start_y = fb.height as i32 / 2 + 10;
                for (i, &(label, enabled)) in menu_items.iter().enumerate() {
                    let y = start_y + i as i32 * 9;
                    let is_selected = i == self.selected_item;

                    let (prefix, suffix, color) = if is_selected {
                        let glow = ((self.time * 6.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
                        ("> ", " <", Palette::TEXT_GOLD.scale(glow))
                    } else if !enabled {
                        ("  ", "  ", Palette::TEXT_DARK)
                    } else {
                        ("  ", "  ", Palette::TEXT_NORMAL)
                    };

                    let full_label = format!("{}{}{}", prefix, label, suffix);
                    TextRenderer::draw_text_centered(
                        fb,
                        y,
                        &full_label,
                        FontSize::Standard5x7,
                        color,
                    );
                }
            }
            TitleState::SettingsModal => {
                let modal_w = 70;
                let modal_h = 32;
                let mx = (fb.width as i32 - modal_w) / 2;
                let my = (fb.height as i32 - modal_h) / 2;

                fb.draw_rect(mx, my, modal_w, modal_h, Palette::UI_PANEL_BG, true);
                fb.draw_rect(mx, my, modal_w, modal_h, Palette::UI_BORDER, false);

                TextRenderer::draw_text_centered(
                    fb,
                    my + 4,
                    "SETTINGS",
                    FontSize::Standard5x7,
                    Palette::TEXT_GOLD,
                );
                TextRenderer::draw_text_centered(
                    fb,
                    my + 13,
                    "TRUECOLOR: ENABLED",
                    FontSize::Tiny3x5,
                    Palette::TEXT_NORMAL,
                );
                TextRenderer::draw_text_centered(
                    fb,
                    my + 19,
                    "TARGET FPS: 60 HZ",
                    FontSize::Tiny3x5,
                    Palette::TEXT_NORMAL,
                );
                TextRenderer::draw_text_centered(
                    fb,
                    my + 25,
                    "[ESC / ENTER] BACK",
                    FontSize::Tiny3x5,
                    Palette::TEXT_MUTED,
                );
            }
        }

        // Post-processing
        PostProcessor::apply_drifting_fog(fb, self.time);
        PostProcessor::apply_vignette(fb, 0.45);
    }
}
