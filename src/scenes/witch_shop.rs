use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

pub struct WitchShopScene {
    selected_idx: usize,
    time: f32,
    feedback_message: Option<(&'static str, f32)>,
    term_cols: usize,
    term_rows: usize,
}

impl Default for WitchShopScene {
    fn default() -> Self {
        Self::new()
    }
}

impl WitchShopScene {
    pub fn new() -> Self {
        Self {
            selected_idx: 0,
            time: 0.0,
            feedback_message: None,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for WitchShopScene {
    fn on_resize(&mut self, w: usize, h: usize) {
        self.term_cols = w;
        self.term_rows = h / 2;
    }

    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;

        if let Some((_, ref mut timer)) = self.feedback_message {
            *timer -= dt;
            if *timer <= 0.0 {
                self.feedback_message = None;
            }
        }

        if input.just_pressed(VirtualKey::Escape) {
            return SceneTransition::Pop;
        }

        let num_items = 4;
        if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = num_items - 1;
            }
        }
        if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
            if self.selected_idx + 1 < num_items {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }

        if input.just_pressed(VirtualKey::Enter)
            || input.just_pressed(VirtualKey::Space)
            || input.mouse_left_just_pressed
        {
            let msgs = [
                "Acquired [Abyssal Dart]! Inscribed into forbidden memory.",
                "Acquired [Mist of Cinders]! Smoke clings to your cloak.",
                "Acquired [Bloodflame Surge]! Dangerous warmth fills your veins.",
                "Acquired [Flask of Pure Vitriol]! Corrosive acid stored safely.",
            ];
            self.feedback_message = Some((msgs[self.selected_idx], 2.5));
        }

        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let pw = (ui.cols.saturating_sub(12)).min(80);
        let ph = (ui.rows.saturating_sub(8)).min(20);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        let witch_accent = Color::rgb(190, 80, 245);
        let style = PanelStyle {
            border: crate::ui::canvas::BorderKind::Double,
            border_color: witch_accent,
            title_color: witch_accent,
            bg_color: Some(Color::rgb(14, 16, 22)),
        };
        ui.draw_panel(panel_rect, "SHADY WITCH MORGANA - FORBIDDEN ARCANA", style);

        let bg = Some(Color::rgb(14, 16, 22));
        ui.print(px + 4, py + 2, "“Seek power beyond the sacred flames? I have wares for the daring...”", Palette::TEXT_MUTED, bg);

        let items = [
            ("Abyssal Dart", "150 Souls", "Fires a swift bolt of deep dark magic that pierces guard."),
            ("Mist of Cinders", "300 Souls", "Blankets arena in scalding ash, obscuring player from foes."),
            ("Bloodflame Surge", "550 Souls", "Sacrifices minor health to unleash an erupting pillar of black fire."),
            ("Flask of Pure Vitriol", "200 Souls", "Potent alchemical solvent that strips armor and poison."),
        ];

        for (i, &(name, cost, desc)) in items.iter().enumerate() {
            let row = py + 4 + i * 3;
            let is_sel = i == self.selected_idx;

            let (prefix, name_col, bold) = if is_sel {
                ("▶ ", witch_accent, true)
            } else {
                ("  ", Palette::TEXT_BRIGHT, false)
            };

            let line = format!("{}{:<26}  Cost: {}", prefix, name, cost);
            if bold {
                ui.print_bold(px + 4, row, &line, name_col, bg);
            } else {
                ui.print(px + 4, row, &line, name_col, bg);
            }
            ui.print(px + 8, row + 1, desc, Palette::TEXT_MUTED, bg);
        }

        if let Some((msg, _)) = self.feedback_message {
            ui.print_bold(px + 4, py + ph - 2, msg, Palette::BONFIRE_YELLOW, bg);
        }

        ui.print_centered(
            py + ph + 1,
            "[W/S] Browse  |  [ENTER/CLICK] Barter  |  [ESC] Leave",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
