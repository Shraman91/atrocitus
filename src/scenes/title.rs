use super::loading::LoadingScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::classes::SaveData;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;
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
    embers: Vec<(f32, f32, f32, f32, f32)>, // (x, y, vx, vy, life)
    term_cols: usize,
    term_rows: usize,
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
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for TitleScene {
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

        // Update ember particles
        let mut rng = rand::rng();
        for ember in &mut self.embers {
            ember.4 -= dt;
            if ember.4 <= 0.0 {
                ember.0 = rng.random_range(0.0..self.term_cols as f32);
                ember.1 = (self.term_rows * 2).saturating_sub(4) as f32;
                ember.2 = rng.random_range(-2.0..2.0);
                ember.3 = rng.random_range(-12.0..-4.0);
                ember.4 = rng.random_range(2.0..4.0);
            } else {
                ember.0 += ember.2 * dt;
                ember.1 += ember.3 * dt;
                ember.0 += ((ember.1 * 0.1).sin() * 2.0) * dt;
            }
        }

        let save = SaveData::load();
        let has_save = save.chosen_class.is_some();

        match self.state {
            TitleState::PressAnyKey => {
                if input.just_pressed(VirtualKey::Enter)
                    || input.just_pressed(VirtualKey::Space)
                    || input.mouse_left_just_pressed
                    || input.any_key_just_pressed
                {
                    self.state = TitleState::Menu;
                }
            }
            TitleState::Menu => {
                // Keyboard Navigation
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

                // Mouse Navigation
                let start_row = self.term_rows / 2 + 2;
                for i in 0..4 {
                    let item_row = start_row + i * 2;
                    if input.mouse_cell.1 == item_row {
                        self.selected_item = i;
                        if input.mouse_left_just_pressed {
                            match self.selected_item {
                                0 => return SceneTransition::Switch(Box::new(LoadingScene::new())),
                                1 => {
                                    if has_save {
                                        return SceneTransition::Switch(Box::new(LoadingScene::new()));
                                    }
                                }
                                2 => self.state = TitleState::SettingsModal,
                                3 => return SceneTransition::Quit,
                                _ => {}
                            }
                        }
                    }
                }

                if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
                    match self.selected_item {
                        0 => return SceneTransition::Switch(Box::new(LoadingScene::new())),
                        1 => {
                            if has_save {
                                return SceneTransition::Switch(Box::new(LoadingScene::new()));
                            }
                        }
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
                    || input.mouse_left_just_pressed
                {
                    self.state = TitleState::Menu;
                }
            }
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
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

            let u2 = x as f32 - self.time * 2.2;
            let peak2 = (u2 * 0.07).sin() * 8.0 + (u2 * 0.16).cos() * 4.0;
            let sy2 = (horizon_y as f32 - 14.0 + peak2).round() as i32;
            for y in sy2..horizon_y {
                fb.set_pixel_blend(x as i32, y, spire_col2);
            }
        }

        // Draw floating glowing embers
        for ember in &self.embers {
            let ex = ember.0.round() as i32;
            let ey = ember.1.round() as i32;
            let alpha = (ember.4 / 3.0).clamp(0.0, 1.0);
            let col = if ember.4 > 2.0 {
                Palette::BONFIRE_YELLOW.scale(alpha)
            } else if ember.4 > 1.0 {
                Palette::BONFIRE_ORANGE.scale(alpha)
            } else {
                Palette::BONFIRE_RED.scale(alpha)
            };
            fb.set_pixel_add(ex, ey, col);
        }

        // Centered stylized block logo fitting within 60% of terminal width
        let logo_lines = [
            "▄▀█  ▀█▀  █▀▄  ▄▀▄  ▄▀▀  █  ▀█▀  █ █  ▄▀▀",
            "█▀█   █   █▀▄  █ █  █    █   █   █ █  ▀▀▄",
            "▀ ▀   ▀   ▀ ▀  ▀▀▀  ▀▀▀  ▀   ▀   ▀▀▀  ▀▀▀",
        ];
        let glow = ((self.time * 2.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
        let gold_glow = Palette::GOLD_TRIM.scale(glow);
        let logo_start_row = (ui.rows / 2).saturating_sub(8);

        for (idx, line) in logo_lines.iter().enumerate() {
            ui.print_centered(logo_start_row + idx, line, gold_glow, None);
        }

        ui.print_centered(
            logo_start_row + 4,
            "COLOSSEUM OF ASH & CINDERS",
            Palette::TEXT_MUTED,
            None,
        );

        let save = SaveData::load();
        let has_save = save.chosen_class.is_some();

        // Render current state UI with real terminal character glyphs
        match self.state {
            TitleState::PressAnyKey => {
                let flash = ((self.time * 4.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                if flash > 0.3 {
                    ui.print_centered(
                        ui.rows / 2 + 3,
                        "[ PRESS ANY KEY OR CLICK TO ENTER ]",
                        Palette::TEXT_GOLD,
                        None,
                    );
                }
            }
            TitleState::Menu => {
                let menu_items = [
                    ("NEW GAME", true),
                    ("CONTINUE", has_save),
                    ("SETTINGS", true),
                    ("QUIT", true),
                ];

                let start_row = ui.rows / 2 + 2;
                for (i, &(label, enabled)) in menu_items.iter().enumerate() {
                    let row = start_row + i * 2;
                    let is_selected = i == self.selected_item;

                    let (prefix, suffix, color, bold) = if is_selected {
                        let select_glow = ((self.time * 6.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
                        ("▶ ", " ◀", Palette::TEXT_GOLD.scale(select_glow), true)
                    } else if !enabled {
                        ("  ", "  ", Palette::TEXT_DARK, false)
                    } else {
                        ("  ", "  ", Palette::TEXT_NORMAL, false)
                    };

                    let full_label = format!("{}{}{}", prefix, label, suffix);
                    let col = (ui.cols.saturating_sub(full_label.len())) / 2;
                    if bold {
                        ui.print_bold(col, row, &full_label, color, None);
                    } else {
                        ui.print(col, row, &full_label, color, None);
                    }
                }
            }
            TitleState::SettingsModal => {
                let mw = 48;
                let mh = 12;
                let mx = (ui.cols.saturating_sub(mw)) / 2;
                let my = (ui.rows.saturating_sub(mh)) / 2;

                let modal_rect = Rect::new(mx as i32, my as i32, mw as i32, mh as i32);
                ui.draw_panel(modal_rect, "SETTINGS", PanelStyle::double());

                let panel_bg = Some(Color::rgb(14, 16, 22));
                ui.print_centered(my + 3, "TRUECOLOR: ENABLED (24-bit ANSI)", Palette::TEXT_BRIGHT, panel_bg);
                ui.print_centered(my + 5, "MOUSE MODE: SGR EXTENDED PRECISION", Palette::TEXT_MUTED, panel_bg);
                ui.print_centered(my + 7, "CONTROLS: WASD MOVE | MOUSE AIM", Palette::TEXT_NORMAL, panel_bg);
                ui.print_centered(my + 9, "[ESC / CLICK] CLOSE", Palette::TEXT_GOLD, panel_bg);
            }
        }
    }
}
