use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::{Direction4, Sprite, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::classes::ClassKind;
use crate::game::equipment::LayeredGear;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

pub struct GalleryScene {
    character_idx: usize,
    dir_idx: usize,
    anim_timer: f32,
    term_cols: usize,
    term_rows: usize,
}

impl Default for GalleryScene {
    fn default() -> Self {
        Self::new()
    }
}

impl GalleryScene {
    pub fn new() -> Self {
        Self {
            character_idx: 0,
            dir_idx: 0,
            anim_timer: 0.0,
            term_cols: 120,
            term_rows: 40,
        }
    }
}

impl Scene for GalleryScene {
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
        self.anim_timer += dt * 8.0;

        if input.just_pressed(VirtualKey::Escape) || input.just_pressed(VirtualKey::F4) {
            return SceneTransition::Pop;
        }

        let num_characters = 10;
        // Cycle character with Left/Right or A/D
        if input.just_pressed(VirtualKey::A) || input.just_pressed(VirtualKey::Left) {
            if self.character_idx > 0 {
                self.character_idx -= 1;
            } else {
                self.character_idx = num_characters - 1;
            }
        }
        if input.just_pressed(VirtualKey::D) || input.just_pressed(VirtualKey::Right) {
            if self.character_idx + 1 < num_characters {
                self.character_idx += 1;
            } else {
                self.character_idx = 0;
            }
        }

        // Cycle direction with Up/Down or W/S
        if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
            if self.dir_idx > 0 {
                self.dir_idx -= 1;
            } else {
                self.dir_idx = 3;
            }
        }
        if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
            if self.dir_idx < 3 {
                self.dir_idx += 1;
            } else {
                self.dir_idx = 0;
            }
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank) {
        // Darkened background on framebuffer
        fb.clear(Palette::VOID_BLACK);

        let pw = (ui.cols.saturating_sub(10)).min(94);
        let ph = (ui.rows.saturating_sub(6)).min(26);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, "F4 SPRITE GALLERY & SILHOUETTE VIEWER", PanelStyle::double());

        let characters = [
            ("Adventurer (Base)", "Starting unarmored mantle with bedroll & traveler boots."),
            ("Mage (Pyromancer/Sorcerer)", "Oversized witch brim, twilight mantle, and arcane staff."),
            ("Knight (Ashen Sentinel)", "Winged greathelm, heraldic tabard, broadsword & heater shield."),
            ("Assassin (Shadow Weaver)", "Veiled cowl with face shadow, red wrap, and dual daggers."),
            ("Hunter (Beast Stalker)", "Feathered tricorn, buff duster coat, and pistol-swords."),
            ("Alchemist (Plague Scholar)", "Curved plague mask beak, leather apron, and vitriol vial."),
            ("The Veiled Firekeeper", "Blindfolded maiden tending the sacred coiled flame."),
            ("Blacksmith Valen", "Master of the anvil with heavy leather apron & hammer."),
            ("Unknown Adventurer", "Cynical wanderer sharpening a weathered gladius."),
            ("Shady Witch Morgana", "Purveyor of forbidden elixirs and abyssal sorceries."),
        ];

        let cur_char = characters[self.character_idx];
        let bg = Some(Color::rgb(14, 16, 22));

        // Header info
        ui.print_bold(
            px + 4,
            py + 2,
            &format!("Viewing [{}/10]: {}", self.character_idx + 1, cur_char.0),
            Palette::TEXT_GOLD,
            bg,
        );
        ui.print(px + 4, py + 3, cur_char.1, Palette::TEXT_MUTED, bg);

        let dir = match self.dir_idx {
            0 => Direction4::South,
            1 => Direction4::North,
            2 => Direction4::East,
            _ => Direction4::SouthEast,
        };
        let dir_str = match self.dir_idx {
            0 => "South (Front)",
            1 => "North (Back)",
            2 => "East (Profile)",
            _ => "SouthEast (3/4 Isometric)",
        };
        ui.print(px + 4, py + 5, &format!("Direction: {}", dir_str), Palette::MAGE_CYAN, bg);

        let walk_frame = (self.anim_timer as usize) % 4;
        let dummy_gear = LayeredGear::empty();

        // Pixel rendering on Framebuffer: 1x Scale (Left) and 3x Scaled Preview (Right)
        let render_1x_x = px as i32 + 8;
        let render_1x_y = ((py + 8) * 2) as i32;

        let render_3x_x = px as i32 + 36;
        let render_3x_y = ((py + 7) * 2) as i32;

        ui.print(px + 6, py + 7, "1x Scale:", Palette::TEXT_BRIGHT, bg);
        ui.print(px + 36, py + 6, "3x Scale (Clarity & Outlines):", Palette::TEXT_BRIGHT, bg);

        // Helper to draw scaled sprite
        let draw_scaled = |fb: &mut Framebuffer, sp: &Sprite, ox: i32, oy: i32, scale: usize| {
            for sy in 0..sp.height {
                for sx in 0..sp.width {
                    let col = sp.get_pixel(sx, sy);
                    if col.a > 0 {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                fb.set_pixel(ox + (sx * scale + dx) as i32, oy + (sy * scale + dy) as i32, col);
                            }
                        }
                    }
                }
            }
        };

        match self.character_idx {
            0..=5 => {
                // Playable Class composite sprites
                let body_sprite = &bank.body_walk[self.dir_idx][walk_frame];

                let class_kind = match self.character_idx {
                    1 => Some(ClassKind::Mage),
                    2 => Some(ClassKind::Knight),
                    3 => Some(ClassKind::Assassin),
                    4 => Some(ClassKind::Hunter),
                    5 => Some(ClassKind::Alchemist),
                    _ => None,
                };

                // Contact shadow
                fb.draw_ellipse(render_1x_x + 8, render_1x_y + 20, 6, 2, Color::rgba(4, 5, 10, 140));

                if let Some(gear_set) = class_kind.and_then(|c| bank.get_class_gear(c)) {
                    let gear_sprites: [Option<&Sprite>; 6] = [
                        Some(&gear_set.head[self.dir_idx]),
                        Some(&gear_set.torso[self.dir_idx]),
                        gear_set.cape[self.dir_idx].as_ref(),
                        gear_set.main_hand[self.dir_idx].as_ref(),
                        gear_set.off_hand[self.dir_idx].as_ref(),
                        gear_set.accessory[self.dir_idx].as_ref(),
                    ];
                    dummy_gear.draw_composite(fb, body_sprite, &gear_sprites, render_1x_x, render_1x_y, dir, false);
                } else {
                    let empty_gear = [None, None, None, None, None, None];
                    dummy_gear.draw_composite(fb, body_sprite, &empty_gear, render_1x_x, render_1x_y, dir, false);
                }

                // 3x magnified rendering of body
                draw_scaled(fb, body_sprite, render_3x_x, render_3x_y, 3);
            }
            6 => {
                // Firekeeper
                let sp = &bank.firekeeper_anim[(walk_frame / 2) % 2];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_3x_x, render_3x_y, 3);
            }
            7 => {
                // Blacksmith
                let sp = &bank.blacksmith_anim[(walk_frame / 2) % 2];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_3x_x, render_3x_y, 3);
            }
            8 => {
                // Unknown Adventurer
                let sp = &bank.adventurer_anim[(walk_frame / 2) % 2];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_3x_x, render_3x_y, 3);
            }
            _ => {
                // Shady Witch
                let sp = &bank.witch_anim[(walk_frame / 2) % 2];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_3x_x, render_3x_y, 3);
            }
        }

        // Footer instructions
        ui.print_centered(
            py + ph + 1,
            "[A/D] Character  |  [W/S] Direction  |  [F4 / ESC] Exit Gallery",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
