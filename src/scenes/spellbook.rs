use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::classes::SpellBook;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::{word_wrap, Rect};
use crate::ui::UiCanvas;

pub struct SpellbookScene {
    spellbook: SpellBook,
    current_tier_idx: usize,
    selected_spell_idx: usize,
    term_cols: usize,
    term_rows: usize,
}

impl Default for SpellbookScene {
    fn default() -> Self {
        Self::new()
    }
}

impl SpellbookScene {
    pub fn new() -> Self {
        Self {
            spellbook: SpellBook::load(),
            current_tier_idx: 0,
            selected_spell_idx: 0,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for SpellbookScene {
    fn on_resize(&mut self, w: usize, h: usize) {
        self.term_cols = w;
        self.term_rows = h / 2;
    }

    fn update(
        &mut self,
        _dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        if input.just_pressed(VirtualKey::Escape) || input.just_pressed(VirtualKey::K) {
            return SceneTransition::Pop;
        }

        let num_tiers = self.spellbook.tiers.len();
        if num_tiers == 0 {
            return SceneTransition::None;
        }

        // Tier switching (Left/Right, A/D)
        if input.just_pressed(VirtualKey::A) || input.just_pressed(VirtualKey::Left) {
            if self.current_tier_idx > 0 {
                self.current_tier_idx -= 1;
            } else {
                self.current_tier_idx = num_tiers - 1;
            }
            self.selected_spell_idx = 0;
        }
        if input.just_pressed(VirtualKey::D) || input.just_pressed(VirtualKey::Right) {
            if self.current_tier_idx + 1 < num_tiers {
                self.current_tier_idx += 1;
            } else {
                self.current_tier_idx = 0;
            }
            self.selected_spell_idx = 0;
        }

        // Spell navigation (Up/Down, W/S)
        let cur_tier = &self.spellbook.tiers[self.current_tier_idx];
        let num_spells = cur_tier.spells.len();

        if num_spells > 0 {
            if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
                if self.selected_spell_idx > 0 {
                    self.selected_spell_idx -= 1;
                } else {
                    self.selected_spell_idx = num_spells - 1;
                }
            }
            if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
                if self.selected_spell_idx + 1 < num_spells {
                    self.selected_spell_idx += 1;
                } else {
                    self.selected_spell_idx = 0;
                }
            }
        }

        // Mouse click on tabs or spell list
        let book_w = (self.term_cols.saturating_sub(10)).min(94);
        let book_h = (self.term_rows.saturating_sub(6)).min(26);
        let bx = (self.term_cols.saturating_sub(book_w)) / 2;
        let by = (self.term_rows.saturating_sub(book_h)) / 2;

        let mx = input.mouse_cell.0;
        let my = input.mouse_cell.1;

        if input.mouse_left_just_pressed {
            // Tab clicks
            for t_idx in 0..num_tiers {
                let tab_col = bx + 3 + t_idx * 9;
                let tab_row = by + 2;
                if my == tab_row && mx >= tab_col && mx < tab_col + 8 {
                    self.current_tier_idx = t_idx;
                    self.selected_spell_idx = 0;
                }
            }

            // Spell clicks on left page
            for s_idx in 0..num_spells {
                let spell_row = by + 6 + s_idx * 2;
                if my == spell_row && mx >= bx + 2 && mx < bx + book_w / 2 - 2 {
                    self.selected_spell_idx = s_idx;
                }
            }
        }

        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let book_w = (ui.cols.saturating_sub(10)).min(94);
        let book_h = (ui.rows.saturating_sub(6)).min(26);
        let bx = (ui.cols.saturating_sub(book_w)) / 2;
        let by = (ui.rows.saturating_sub(book_h)) / 2;

        let book_rect = Rect::new(bx as i32, by as i32, book_w as i32, book_h as i32);
        ui.draw_panel(book_rect, "ARCANE GRIMOIRE", PanelStyle::double());

        let half_w = book_w / 2;
        let spine_col = bx + half_w;

        // Draw central vertical spine divider
        for r in (by + 1)..(by + book_h - 1) {
            ui.print(spine_col, r, "│", Color::rgb(40, 44, 58), Some(Color::rgb(14, 16, 22)));
        }

        // Tier Tabs along top of Left Page
        for (t_idx, _) in self.spellbook.tiers.iter().enumerate() {
            let tab_col = bx + 3 + t_idx * 9;
            let tab_row = by + 2;
            let is_sel = t_idx == self.current_tier_idx;

            let tab_text = format!("[T-{}]", t_idx + 1);
            if is_sel {
                ui.print_bold(
                    tab_col,
                    tab_row,
                    &tab_text,
                    Palette::MAGE_CYAN,
                    Some(Color::rgb(22, 28, 40)),
                );
            } else {
                ui.print(
                    tab_col,
                    tab_row,
                    &tab_text,
                    Palette::TEXT_MUTED,
                    Some(Color::rgb(14, 16, 22)),
                );
            }
        }

        let cur_tier = &self.spellbook.tiers[self.current_tier_idx];

        // Left Page: Tier Title
        ui.print(
            bx + 3,
            by + 4,
            &cur_tier.title,
            Palette::TEXT_GOLD,
            Some(Color::rgb(14, 16, 22)),
        );

        // Left Page: Spell List
        for (s_idx, spell) in cur_tier.spells.iter().enumerate() {
            let sy = by + 6 + s_idx * 2;
            if sy >= by + book_h - 2 {
                break;
            }
            let is_sel = s_idx == self.selected_spell_idx;

            let (prefix, color) = if is_sel {
                ("▶ ", Palette::MAGE_CYAN)
            } else if spell.unlocked {
                ("  ", Palette::TEXT_BRIGHT)
            } else {
                ("  ", Palette::TEXT_MUTED)
            };

            let lock_str = if spell.unlocked { "" } else { " [SEALED]" };
            let line = format!("{}{}{}", prefix, spell.name, lock_str);
            let bg = if is_sel {
                Some(Color::rgb(24, 28, 42))
            } else {
                Some(Color::rgb(14, 16, 22))
            };

            if is_sel {
                ui.print_bold(bx + 3, sy, &line, color, bg);
            } else {
                ui.print(bx + 3, sy, &line, color, bg);
            }
        }

        // Right Page: Selected Spell Details
        let right_x = spine_col + 3;
        let right_max_w = (bx + book_w).saturating_sub(right_x + 3);

        if !cur_tier.spells.is_empty() && self.selected_spell_idx < cur_tier.spells.len() {
            let spell = &cur_tier.spells[self.selected_spell_idx];

            // Spell Name
            ui.print_bold(
                right_x,
                by + 2,
                &spell.name,
                Palette::MAGE_CYAN,
                Some(Color::rgb(14, 16, 22)),
            );

            // Cost & Element
            let stat_line = format!("Cost: {} FP  |  Element: {}", spell.cost, spell.element);
            ui.print(
                right_x,
                by + 4,
                &stat_line,
                Palette::TEXT_GOLD,
                Some(Color::rgb(14, 16, 22)),
            );

            // Status: Inscribed or Sealed
            if spell.unlocked {
                ui.print(
                    right_x,
                    by + 6,
                    "STATUS: INSCRIBED IN MEMORY",
                    Palette::TEXT_BRIGHT,
                    Some(Color::rgb(14, 16, 22)),
                );
            } else {
                ui.print(
                    right_x,
                    by + 6,
                    "STATUS: SEALED",
                    Palette::ASSASSIN_RED,
                    Some(Color::rgb(14, 16, 22)),
                );
                ui.print(
                    right_x,
                    by + 7,
                    &format!("Req: {}", cur_tier.requirement),
                    Palette::TEXT_MUTED,
                    Some(Color::rgb(14, 16, 22)),
                );
            }

            // Word-wrapped Description / Lore
            let desc_lines = word_wrap(&spell.desc, right_max_w);
            for (idx, line) in desc_lines.iter().enumerate() {
                let dy = by + 9 + idx;
                if dy >= by + book_h - 2 {
                    break;
                }
                ui.print(
                    right_x,
                    dy,
                    line,
                    Palette::TEXT_NORMAL,
                    Some(Color::rgb(14, 16, 22)),
                );
            }
        }

        // Footer instructions
        ui.print_centered(
            by + book_h + 1,
            "[A/D] Tiers  |  [W/S] Spells  |  [K / ESC] Close Book",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
