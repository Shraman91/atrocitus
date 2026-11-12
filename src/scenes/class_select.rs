use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::{Direction4, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::classes::{ClassDef, SaveData};
use crate::game::equipment::LayeredGear;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

pub struct ClassSelectScene {
    classes: Vec<ClassDef>,
    selected_idx: usize,
    preview_timer: f32,
    confirm_modal: bool,
    time: f32,
    term_cols: usize,
    term_rows: usize,
}

impl Default for ClassSelectScene {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassSelectScene {
    pub fn new() -> Self {
        let classes = ClassDef::load_all();
        let save = SaveData::load();
        let initial_idx = if let Some(current) = save.chosen_class {
            classes.iter().position(|c| c.kind() == current).unwrap_or(0)
        } else {
            0
        };

        Self {
            classes,
            selected_idx: initial_idx,
            preview_timer: 0.0,
            confirm_modal: false,
            time: 0.0,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for ClassSelectScene {
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
        self.preview_timer += dt;

        if self.confirm_modal {
            if input.just_pressed(VirtualKey::Enter)
                || input.just_pressed(VirtualKey::Space)
                || input.just_pressed(VirtualKey::E)
                || input.mouse_left_just_pressed
            {
                let chosen = self.classes[self.selected_idx].kind();
                let mut save = SaveData::load();
                save.chosen_class = Some(chosen);
                save.save();
                return SceneTransition::Pop;
            }

            if input.just_pressed(VirtualKey::Escape) {
                self.confirm_modal = false;
            }
            return SceneTransition::None;
        }

        // Keyboard Navigation
        if input.just_pressed(VirtualKey::A) || input.just_pressed(VirtualKey::Left) {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = self.classes.len().saturating_sub(1);
            }
        }
        if input.just_pressed(VirtualKey::D) || input.just_pressed(VirtualKey::Right) {
            if self.selected_idx + 1 < self.classes.len() {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }

        // Mouse hover and click detection on cards
        let num_cards = self.classes.len();
        let card_w = ((self.term_cols.saturating_sub(14)) / num_cards).clamp(18, 24);
        let card_gap = 2;
        let total_w = num_cards * card_w + (num_cards - 1) * card_gap;
        let start_col = (self.term_cols.saturating_sub(total_w)) / 2;
        let start_row = 6;
        let card_h = 16;

        let mx = input.mouse_cell.0;
        let my = input.mouse_cell.1;

        for i in 0..num_cards {
            let cx = start_col + i * (card_w + card_gap);
            if mx >= cx && mx < cx + card_w && my >= start_row && my < start_row + card_h {
                self.selected_idx = i;
                if input.mouse_left_just_pressed {
                    self.confirm_modal = true;
                }
            }
        }

        // Confirm / Cancel keys
        if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
            self.confirm_modal = true;
        }

        if input.just_pressed(VirtualKey::Escape) {
            return SceneTransition::Pop;
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank) {
        // Dark background on framebuffer
        fb.clear(Palette::VOID_BLACK);

        // Header Title in UiCanvas
        ui.print_centered(2, "CHOOSE YOUR MANTLE", Palette::TEXT_GOLD, None);
        ui.print_centered(
            3,
            "The Firekeeper binds your soul to an ancient discipline.",
            Palette::TEXT_MUTED,
            None,
        );

        // Calculate card layout
        let num_cards = self.classes.len();
        let card_w = ((ui.cols.saturating_sub(14)) / num_cards).clamp(18, 24);
        let card_gap = 2;
        let total_w = num_cards * card_w + (num_cards - 1) * card_gap;
        let start_col = (ui.cols.saturating_sub(total_w)) / 2;
        let start_row = 5;
        let card_h = 16;

        let rot_dirs = [
            Direction4::South,
            Direction4::SouthEast,
            Direction4::East,
            Direction4::North,
        ];
        let rot_idx = ((self.preview_timer / 0.8) as usize) % 4;
        let preview_dir = rot_dirs[rot_idx];
        let preview_dir_idx = match preview_dir {
            Direction4::South => 0,
            Direction4::North => 1,
            Direction4::East => 2,
            Direction4::SouthEast => 3,
        };

        let dummy_gear = LayeredGear::empty();

        for (i, class_def) in self.classes.iter().enumerate() {
            let cx = start_col + i * (card_w + card_gap);
            let is_sel = i == self.selected_idx;
            let accent = class_def.accent_color();

            let card_rect = Rect::new(cx as i32, start_row as i32, card_w as i32, card_h as i32);
            let panel_style = if is_sel {
                crate::ui::canvas::PanelStyle {
                    border: crate::ui::canvas::BorderKind::Double,
                    border_color: accent,
                    title_color: accent,
                    bg_color: Some(Color::rgb(14, 16, 22)),
                }
            } else {
                PanelStyle::single()
            };
            ui.draw_panel(card_rect, &class_def.name, panel_style);

            // Rotating Character Preview in framebuffer
            let body_sprite = &bank.body_idle[preview_dir_idx];
            let char_pixel_x = (cx as i32) + (card_w as i32 - 16) / 2;
            let char_pixel_y = ((start_row + 2) * 2) as i32;

            if let Some(gear_set) = bank.get_class_gear(class_def.kind()) {
                let gear_sprites: [Option<&crate::assets::sprites::Sprite>; 6] = [
                    Some(&gear_set.head[preview_dir_idx]),
                    Some(&gear_set.torso[preview_dir_idx]),
                    gear_set.cape[preview_dir_idx].as_ref(),
                    gear_set.main_hand[preview_dir_idx].as_ref(),
                    gear_set.off_hand[preview_dir_idx].as_ref(),
                    gear_set.accessory[preview_dir_idx].as_ref(),
                ];
                dummy_gear.draw_composite(
                    fb,
                    body_sprite,
                    &gear_sprites,
                    char_pixel_x,
                    char_pixel_y,
                    preview_dir,
                    false,
                );
            }

            // Weapon / Style tag at bottom of card
            let wep_abbr = match class_def.kind() {
                crate::game::classes::ClassKind::Mage => "STAFF",
                crate::game::classes::ClassKind::Knight => "SWORD",
                crate::game::classes::ClassKind::Assassin => "DAGGERS",
                crate::game::classes::ClassKind::Hunter => "GUNBLADE",
                crate::game::classes::ClassKind::Alchemist => "VIAL",
                _ => "BARE",
            };
            let tag_col = cx + (card_w.saturating_sub(wep_abbr.len())) / 2;
            ui.print(tag_col, start_row + card_h - 2, wep_abbr, Palette::TEXT_MUTED, Some(Color::rgb(14, 16, 22)));
        }

        // Details Panel for currently selected class
        let cur = &self.classes[self.selected_idx];
        let det_row = start_row + card_h + 1;
        let det_h = 5;
        let det_rect = Rect::new(start_col as i32, det_row as i32, total_w as i32, det_h);
        ui.draw_panel(det_rect, "CLASS ATTRIBUTES", PanelStyle::single());

        let title_line = format!("{}: {}", cur.name, cur.title);
        ui.print(start_col + 2, det_row + 1, &title_line, cur.accent_color(), Some(Color::rgb(14, 16, 22)));

        let gear_line = format!("Loadout: {} & {}  |  {}", cur.weapon_name, cur.offhand_name, cur.gear_summary);
        ui.print(start_col + 2, det_row + 2, &gear_line, Palette::TEXT_NORMAL, Some(Color::rgb(14, 16, 22)));

        // Footer Instructions
        ui.print_centered(
            ui.rows.saturating_sub(2),
            "[< / > / Hover] Select  |  [Enter / Click] Awaken  |  [Esc] Back",
            Palette::TEXT_MUTED,
            None,
        );

        // Confirmation Modal Popup
        if self.confirm_modal {
            let mw = 52;
            let mh = 8;
            let mx = (ui.cols.saturating_sub(mw)) / 2;
            let my = (ui.rows.saturating_sub(mh)) / 2;

            let modal_rect = Rect::new(mx as i32, my as i32, mw as i32, mh as i32);
            ui.draw_panel(modal_rect, "CONFIRM SELECTION", PanelStyle::double());

            let bg = Some(Color::rgb(14, 16, 22));
            ui.print_centered(my + 2, &format!("AWAKEN AS {}?", cur.name), Palette::TEXT_GOLD, bg);
            ui.print_centered(my + 4, "[ENTER / CLICK] CONFIRM    [ESC] CANCEL", Palette::TEXT_BRIGHT, bg);
        }
    }
}
