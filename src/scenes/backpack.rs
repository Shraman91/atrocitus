use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::classes::SaveData;
use crate::game::inventory::Inventory;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::{word_wrap, Rect};
use crate::ui::UiCanvas;

pub struct BackpackScene {
    pub inventory: Inventory,
    selected_slot: usize, // 0..20
    term_cols: usize,
    term_rows: usize,
}

impl BackpackScene {
    pub fn new(inventory: Inventory) -> Self {
        Self {
            inventory,
            selected_slot: 0,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for BackpackScene {
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
        if input.just_pressed(VirtualKey::Escape) || input.just_pressed(VirtualKey::B) {
            return SceneTransition::Pop;
        }

        // F5 debug hotkey: add items
        if input.just_pressed(VirtualKey::F5) {
            self.inventory.add_debug_items();
        }

        // Grid navigation: 5 columns x 4 rows
        let col = self.selected_slot % 5;
        let row = self.selected_slot / 5;

        if (input.just_pressed(VirtualKey::A) || input.just_pressed(VirtualKey::Left)) && col > 0 {
            self.selected_slot -= 1;
        }
        if (input.just_pressed(VirtualKey::D) || input.just_pressed(VirtualKey::Right)) && col < 4 {
            self.selected_slot += 1;
        }
        if (input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up)) && row > 0 {
            self.selected_slot -= 5;
        }
        if (input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down)) && row < 3 {
            self.selected_slot += 5;
        }

        // Mouse hover / click on grid slots
        let pw = (self.term_cols.saturating_sub(10)).min(94);
        let ph = (self.term_rows.saturating_sub(6)).min(26);
        let px = (self.term_cols.saturating_sub(pw)) / 2;
        let py = (self.term_rows.saturating_sub(ph)) / 2;

        let grid_ox = px + 4;
        let grid_oy = py + 4;
        let slot_w = 9;
        let slot_h = 3;

        for r in 0..4 {
            for c in 0..5 {
                let sx = grid_ox + c * (slot_w + 1);
                let sy = grid_oy + r * (slot_h + 1);
                if input.mouse_cell.0 >= sx && input.mouse_cell.0 < sx + slot_w
                    && input.mouse_cell.1 >= sy && input.mouse_cell.1 < sy + slot_h
                {
                    self.selected_slot = r * 5 + c;
                }
            }
        }

        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        let pw = (ui.cols.saturating_sub(10)).min(94);
        let ph = (ui.rows.saturating_sub(6)).min(26);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, "ADVENTURER'S PACK & EQUIPMENT", PanelStyle::double());

        let bg = Some(Color::rgb(14, 16, 22));

        // 1. Left Section: 5x4 Grid Inventory
        let grid_ox = px + 4;
        let grid_oy = py + 3;
        let slot_w = 9;
        let slot_h = 3;

        ui.print_bold(grid_ox, grid_oy - 1, "INVENTORY POUCH (5x4)", Palette::TEXT_GOLD, bg);

        for r in 0..4 {
            for c in 0..5 {
                let slot_idx = r * 5 + c;
                let sx = grid_ox + c * (slot_w + 1);
                let sy = grid_oy + r * (slot_h + 1);
                let is_sel = slot_idx == self.selected_slot;

                let slot_rect = Rect::new(sx as i32, sy as i32, slot_w as i32, slot_h as i32);
                let border_col = if is_sel {
                    Palette::TEXT_GOLD
                } else {
                    Color::rgb(45, 48, 62)
                };

                let style = PanelStyle {
                    border: if is_sel { crate::ui::canvas::BorderKind::Double } else { crate::ui::canvas::BorderKind::Single },
                    border_color: border_col,
                    title_color: border_col,
                    bg_color: bg,
                };
                ui.draw_panel(slot_rect, "", style);

                // Slot item content
                if let Some(item) = &self.inventory.slots[slot_idx] {
                    let abbr: String = item.name.chars().take(6).collect();
                    ui.print(sx + 1, sy + 1, &abbr, item.rarity.color(), bg);
                    let count_str = format!("x{}", item.count);
                    ui.print(sx + slot_w.saturating_sub(count_str.len() + 1), sy + slot_h - 2, &count_str, Palette::TEXT_MUTED, bg);
                }
            }
        }

        // Vertical divider
        let div_x = grid_ox + 5 * (slot_w + 1) + 2;
        for r in (py + 2)..(py + ph - 2) {
            ui.print(div_x, r, "│", Color::rgb(40, 44, 56), bg);
        }

        // 2. Right Section: Equipment Paper-Doll & Selected Item Tooltip
        let right_ox = div_x + 3;
        let right_w = (px + pw).saturating_sub(right_ox + 3);

        ui.print_bold(right_ox, py + 2, "PAPER-DOLL EQUIPMENT", Palette::TEXT_GOLD, bg);

        let save = SaveData::load();
        let class_name = if let Some(c) = save.chosen_class { c.display_name() } else { "Adventurer" };

        let equip_slots = [
            ("HEAD", format!("{} Hood", class_name)),
            ("TORSO", format!("{} Tunic", class_name)),
            ("CAPE", "Traveler Cloak".to_string()),
            ("MAIN", format!("{} Armament", class_name)),
            ("OFF", "Buckler / Focus".to_string()),
            ("ACC", "Bone Charm".to_string()),
        ];

        for (i, (slot_name, item_name)) in equip_slots.iter().enumerate() {
            let ey = py + 4 + i * 2;
            ui.print(right_ox, ey, &format!("[{:<4}]", slot_name), Palette::TEXT_MUTED, bg);
            ui.print_bold(right_ox + 7, ey, item_name, Palette::TEXT_BRIGHT, bg);
        }

        // Horizontal line before tooltip
        let tip_y = py + 17;
        for c in right_ox..(right_ox + right_w) {
            ui.print(c, tip_y - 1, "─", Color::rgb(40, 44, 56), bg);
        }

        // 3. Selected Item Tooltip
        if let Some(item) = &self.inventory.slots[self.selected_slot] {
            ui.print_bold(right_ox, tip_y, &format!("{} (x{})", item.name, item.count), item.rarity.color(), bg);
            ui.print(right_ox, tip_y + 1, &format!("Category: {}", item.category), Palette::TEXT_MUTED, bg);

            let desc_lines = word_wrap(&item.description, right_w);
            for (idx, line) in desc_lines.iter().enumerate() {
                if tip_y + 2 + idx < py + ph - 1 {
                    ui.print(right_ox, tip_y + 2 + idx, line, Palette::TEXT_NORMAL, bg);
                }
            }
        } else {
            ui.print(right_ox, tip_y, "Empty Inventory Slot", Palette::TEXT_DARK, bg);
        }

        // Footer instructions
        ui.print_centered(
            py + ph + 1,
            "[WASD/Hover] Select Slot  |  [F5] Add Test Items  |  [B / ESC] Close Pack",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
