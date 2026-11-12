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

pub struct BlacksmithScene {
    selected_tab: usize,
    selected_option: usize,
    time: f32,
    feedback_message: Option<(&'static str, f32)>,
    term_cols: usize,
    term_rows: usize,
}

impl Default for BlacksmithScene {
    fn default() -> Self {
        Self::new()
    }
}

impl BlacksmithScene {
    pub fn new() -> Self {
        Self {
            selected_tab: 0,
            selected_option: 0,
            time: 0.0,
            feedback_message: None,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for BlacksmithScene {
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

        // Tab navigation (A/D, Left/Right)
        if input.just_pressed(VirtualKey::A) || input.just_pressed(VirtualKey::Left) {
            if self.selected_tab > 0 {
                self.selected_tab -= 1;
            } else {
                self.selected_tab = 2;
            }
            self.selected_option = 0;
        }
        if input.just_pressed(VirtualKey::D) || input.just_pressed(VirtualKey::Right) {
            if self.selected_tab < 2 {
                self.selected_tab += 1;
            } else {
                self.selected_tab = 0;
            }
            self.selected_option = 0;
        }

        // Option navigation (W/S, Up/Down)
        if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
            if self.selected_option > 0 {
                self.selected_option -= 1;
            } else {
                self.selected_option = 2;
            }
        }
        if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
            if self.selected_option < 2 {
                self.selected_option += 1;
            } else {
                self.selected_option = 0;
            }
        }

        // Action confirmation
        if input.just_pressed(VirtualKey::Enter)
            || input.just_pressed(VirtualKey::Space)
            || input.mouse_left_just_pressed
        {
            match self.selected_tab {
                0 => self.feedback_message = Some(("Weapon tempered with Ashen Cinders! (+3 ATK)", 2.5)),
                1 => self.feedback_message = Some(("Armor reinforced with Titanite Shards! (+5 DEF)", 2.5)),
                2 => self.feedback_message = Some(("Durability fully restored at the anvil!", 2.5)),
                _ => {}
            }
        }

        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let pw = (ui.cols.saturating_sub(12)).min(84);
        let ph = (ui.rows.saturating_sub(8)).min(22);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, "FORGEMASTER VALEN - THE ANVIL", PanelStyle::double());

        let save = SaveData::load();
        let class_name = if let Some(c) = save.chosen_class {
            c.display_name()
        } else {
            "UNARMORED ADVENTURER"
        };

        // Tabs
        let tabs = [" [REINFORCE WEAPON] ", " [TEMPER ARMOR] ", " [REPAIR DURABILITY] "];
        let mut tab_col = px + 4;
        let tab_row = py + 2;

        for (idx, &tab) in tabs.iter().enumerate() {
            let is_sel = idx == self.selected_tab;
            let col = if is_sel { Palette::BONFIRE_RED } else { Palette::TEXT_MUTED };
            let bg = if is_sel { Some(Color::rgb(28, 18, 20)) } else { Some(Color::rgb(14, 16, 22)) };
            if is_sel {
                ui.print_bold(tab_col, tab_row, tab, col, bg);
            } else {
                ui.print(tab_col, tab_row, tab, col, bg);
            }
            tab_col += tab.len() + 2;
        }

        // Active Loadout Info
        let bg = Some(Color::rgb(14, 16, 22));
        ui.print(px + 4, py + 4, &format!("Current Mantle: {}", class_name), Palette::TEXT_GOLD, bg);

        // Options according to tab
        let options = match self.selected_tab {
            0 => [
                ("Basic Reinforcement (+1 Sharpness)", "Requires: 1 Titanite Shard, 100 Souls"),
                ("Heavy Cinder Forging (+3 Power)", "Requires: 2 Titanite Shards, 250 Souls"),
                ("Occult Infusion (Arcane Damage)", "Requires: 1 Dark Gem, 500 Souls"),
            ],
            1 => [
                ("Stitch Leather & Mail (+2 Defense)", "Requires: 1 Cloth Scraps, 80 Souls"),
                ("Iron Plating Layer (+5 Poise)", "Requires: 2 Iron Ingots, 220 Souls"),
                ("Ember Weave (+10 Fire Resist)", "Requires: 1 Bonfire Ash, 400 Souls"),
            ],
            _ => [
                ("Sharpen Dented Edges", "Cost: 30 Souls"),
                ("Mend Torn Garments", "Cost: 50 Souls"),
                ("Complete Overhaul (Restores 100%)", "Cost: 120 Souls"),
            ],
        };

        for (i, &(opt, req)) in options.iter().enumerate() {
            let opt_row = py + 7 + i * 3;
            let is_sel = i == self.selected_option;
            let (prefix, text_col, bold) = if is_sel {
                ("▶ ", Palette::BONFIRE_ORANGE, true)
            } else {
                ("  ", Palette::TEXT_BRIGHT, false)
            };

            let line = format!("{}{}", prefix, opt);
            if bold {
                ui.print_bold(px + 4, opt_row, &line, text_col, bg);
            } else {
                ui.print(px + 4, opt_row, &line, text_col, bg);
            }
            ui.print(px + 8, opt_row + 1, req, Palette::TEXT_MUTED, bg);
        }

        // Feedback toast
        if let Some((msg, _)) = self.feedback_message {
            ui.print_bold(px + 4, py + ph - 3, msg, Palette::BONFIRE_YELLOW, bg);
        }

        // Footer
        ui.print_centered(
            py + ph + 1,
            "[A/D] Tabs  |  [W/S] Select  |  [ENTER/CLICK] Forge  |  [ESC] Leave",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
