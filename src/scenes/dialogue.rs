use super::class_select::ClassSelectScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::{word_wrap, Rect};
use crate::ui::UiCanvas;

pub struct DialogueScene {
    lines: Vec<&'static str>,
    current_line: usize,
    char_timer: f32,
    chars_revealed: usize,
    time: f32,
    letterbox_anim: f32,
}

impl Default for DialogueScene {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueScene {
    pub fn new() -> Self {
        Self {
            lines: vec![
                "Welcome, Ashen One, to the ruined Colosseum of the Ancients.",
                "The embers of your soul are scattered... cold and formless.",
                "Before the Gate of Trials opens, you must claim your mantle.",
                "Choose your destiny, wanderer. What power will you wield?",
            ],
            current_line: 0,
            char_timer: 0.0,
            chars_revealed: 0,
            time: 0.0,
            letterbox_anim: 0.0,
        }
    }
}

impl Scene for DialogueScene {
    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;
        if self.letterbox_anim < 1.0 {
            self.letterbox_anim = (self.letterbox_anim + dt * 3.0).min(1.0);
        }

        let full_text = self.lines[self.current_line];
        if self.chars_revealed < full_text.len() {
            self.char_timer += dt * 38.0;
            self.chars_revealed = (self.char_timer as usize).min(full_text.len());
        }

        let wants_advance = input.just_pressed(VirtualKey::Space)
            || input.just_pressed(VirtualKey::Enter)
            || input.just_pressed(VirtualKey::E)
            || input.mouse_left_just_pressed;

        let wants_exit = input.just_pressed(VirtualKey::Escape);

        if wants_exit {
            return SceneTransition::Pop;
        }

        if wants_advance {
            if self.chars_revealed < full_text.len() {
                // Instantly reveal current line
                self.chars_revealed = full_text.len();
            } else if self.current_line + 1 < self.lines.len() {
                // Next line
                self.current_line += 1;
                self.chars_revealed = 0;
                self.char_timer = 0.0;
            } else {
                // Transition to Class Selection
                return SceneTransition::Swap(Box::new(ClassSelectScene::new()));
            }
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let w = fb.width as i32;
        let h = fb.height as i32;

        // Cinematic Letterbox black bars on Framebuffer
        let bar_height = (h as f32 * 0.12 * self.letterbox_anim).round() as i32;
        fb.draw_rect(0, 0, w, bar_height, Palette::VOID_BLACK, true);
        fb.draw_rect(0, h - bar_height, w, bar_height, Palette::VOID_BLACK, true);

        // Lower Dialogue Card using UiCanvas
        let panel_h = 9;
        let panel_y = ui.rows.saturating_sub(panel_h + 2);
        let panel_w = (ui.cols.saturating_sub(8)).min(100);
        let panel_x = (ui.cols.saturating_sub(panel_w)) / 2;

        let panel_rect = Rect::new(panel_x as i32, panel_y as i32, panel_w as i32, panel_h as i32);
        ui.draw_panel(panel_rect, "THE VEILED FIREKEEPER", PanelStyle::rounded());

        // Dialogue Body Text with typewriter reveal using word_wrap
        let full_text = self.lines[self.current_line];
        let visible_text = &full_text[..self.chars_revealed];

        let text_max_w = panel_w.saturating_sub(6);
        let wrapped_lines = word_wrap(visible_text, text_max_w);

        let bg_color = Some(Color::rgb(14, 16, 22));
        for (idx, line) in wrapped_lines.iter().enumerate() {
            if idx < (panel_h - 4) {
                ui.print(panel_x + 3, panel_y + 2 + idx, line, Palette::TEXT_BRIGHT, bg_color);
            }
        }

        // Action Prompts at bottom right of the panel
        let prompt = if self.current_line + 1 == self.lines.len() && self.chars_revealed == full_text.len() {
            "[SPACE / CLICK] CHOOSE CLASS  |  [ESC] LEAVE"
        } else {
            "[SPACE / CLICK] CONTINUE  |  [ESC] LEAVE"
        };
        let prompt_col = (panel_x + panel_w).saturating_sub(prompt.len() + 3);
        ui.print(prompt_col, panel_y + panel_h - 2, prompt, Palette::TEXT_GOLD, bg_color);
    }
}
