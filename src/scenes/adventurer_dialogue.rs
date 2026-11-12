use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::{word_wrap, Rect};
use crate::ui::UiCanvas;

pub struct AdventurerDialogueScene {
    lines: Vec<&'static str>,
    current_line: usize,
    char_timer: f32,
    chars_revealed: usize,
    time: f32,
}

impl Default for AdventurerDialogueScene {
    fn default() -> Self {
        Self::new()
    }
}

impl AdventurerDialogueScene {
    pub fn new() -> Self {
        Self {
            lines: vec![
                "Heh... another hollow washed up on these blood-soaked sands.",
                "Tell me, wanderer, do you hear the bells tolling? Three times for kings who never returned.",
                "The coiled blade in the center... it doesn't give life. It merely delays the ash.",
                "Seek the Gate of Trials if you must. But know that no blade leaves these ruins unstained.",
                "Take my advice: sharpen your steel, watch the shadows, and trust neither witch nor firekeeper.",
            ],
            current_line: 0,
            char_timer: 0.0,
            chars_revealed: 0,
            time: 0.0,
        }
    }
}

impl Scene for AdventurerDialogueScene {
    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;

        let full_text = self.lines[self.current_line];
        if self.chars_revealed < full_text.len() {
            self.char_timer += dt * 36.0;
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
                self.chars_revealed = full_text.len();
            } else if self.current_line + 1 < self.lines.len() {
                self.current_line += 1;
                self.chars_revealed = 0;
                self.char_timer = 0.0;
            } else {
                return SceneTransition::Pop;
            }
        }

        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let panel_h = 9;
        let panel_y = ui.rows.saturating_sub(panel_h + 2);
        let panel_w = (ui.cols.saturating_sub(8)).min(100);
        let panel_x = (ui.cols.saturating_sub(panel_w)) / 2;

        let panel_rect = Rect::new(panel_x as i32, panel_y as i32, panel_w as i32, panel_h as i32);
        let style = PanelStyle {
            border: crate::ui::canvas::BorderKind::Rounded,
            border_color: Palette::MAGE_CYAN,
            title_color: Palette::MAGE_CYAN,
            bg_color: Some(Color::rgb(14, 16, 22)),
        };
        ui.draw_panel(panel_rect, "UNKNOWN ADVENTURER", style);

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

        let prompt = if self.current_line + 1 == self.lines.len() && self.chars_revealed == full_text.len() {
            "[SPACE / CLICK] FAREWELL  |  [ESC] LEAVE"
        } else {
            "[SPACE / CLICK] CONTINUE  |  [ESC] LEAVE"
        };
        let prompt_col = (panel_x + panel_w).saturating_sub(prompt.len() + 3);
        ui.print(prompt_col, panel_y + panel_h - 2, prompt, Palette::MAGE_CYAN, bg_color);
    }
}
