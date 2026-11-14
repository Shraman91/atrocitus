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
    state_idx: usize, // 0: Idle, 1: Walk, 2: Roll, 3: LMB, 4: RMB, 5: Hit, 6: Death
    paused: bool,
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
            state_idx: 0,
            paused: false,
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
        if !self.paused {
            let speed = match self.state_idx {
                0 => 3.0, // gentle idle
                1 => 8.0, // walk
                2 => 10.0, // roll
                3 | 4 => 7.0, // attacks
                _ => 4.0,
            };
            self.anim_timer += dt * speed;
        }

        if input.just_pressed(VirtualKey::Escape) || input.just_pressed(VirtualKey::F4) {
            return SceneTransition::Pop;
        }

        // Space toggles pause
        if input.just_pressed(VirtualKey::Space) {
            self.paused = !self.paused;
        }

        let num_characters = 14;
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

        // Direct animation state selection (1..=7)
        if input.just_pressed(VirtualKey::Char('1')) { self.state_idx = 0; }
        if input.just_pressed(VirtualKey::Char('2')) { self.state_idx = 1; }
        if input.just_pressed(VirtualKey::Char('3')) { self.state_idx = 2; }
        if input.just_pressed(VirtualKey::Char('4')) { self.state_idx = 3; }
        if input.just_pressed(VirtualKey::Char('5')) { self.state_idx = 4; }
        if input.just_pressed(VirtualKey::Char('6')) { self.state_idx = 5; }
        if input.just_pressed(VirtualKey::Char('7')) { self.state_idx = 6; }

        // Tab cycles state forward
        if input.just_pressed(VirtualKey::Tab) {
            self.state_idx = (self.state_idx + 1) % 7;
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank) {
        fb.clear(Palette::VOID_BLACK);

        let pw = (ui.cols.saturating_sub(6)).min(112);
        let ph = (ui.rows.saturating_sub(4)).min(36);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, "F4 SPRITE GALLERY & SILHOUETTE VIEWER", PanelStyle::double());

        let characters: [(&str, &str, Color); 14] = [
            ("Adventurer (Base Wanderer)", "Starting mantle: traveler scarf, linen jerkin, bedroll backpack & sturdy boots.", Palette::ASH_WHITE),
            ("Mage (Pyromancer / Sorcerer)", "Pointed witch brim, twilight robe with celestial gold trim, and glowing staff orb.", Palette::MAGE_CYAN),
            ("Knight (Ashen Sentinel)", "Winged steel greathelm, plate cuirass, heraldic tabard, heater shield & broadsword.", Palette::KNIGHT_GOLD),
            ("Assassin (Shadow Weaver)", "Veiled shadow cowl with crimson eyes, red silk sash, smoke cape & dual daggers.", Palette::ASSASSIN_RED),
            ("Hunter (Beast Stalker)", "Feathered tricorn, buff duster coat, cross-bandolier & dual flintlock pistol-swords.", Palette::HUNTER_BRASS),
            ("Alchemist (Plague Scholar)", "Plague beak cowl with teal goggles, leather apron, and orbiting arcane eye.", Palette::ALCHEMIST_TEAL),
            ("The Veiled Firekeeper", "Blindfolded maiden tending the sacred flame with an eternal ember iron lantern.", Palette::BONFIRE_ORANGE),
            ("Blacksmith Valen", "Master of the forge; thick leather apron, striking red-hot steel on anvil with sparks.", Palette::BONFIRE_RED),
            ("Unknown Adventurer", "Cynical wanderer sharpening a weathered gladius with a whetstone.", Palette::MAGE_CYAN),
            ("Shady Witch Morgana", "Purveyor of forbidden elixirs; crooked hat with candle flame & bubbling boiling flask.", Color::rgb(215, 80, 255)),
            ("Acid Sludge (Corrosive Ooze)", "Lurking subterranean slime; squash-and-stretch body, dripping acidic maw & vomit spit.", Palette::SLUDGE_ACID_YELLOW),
            ("Bogged Skeleton (Archer)", "Moss-covered bones overgrown with red mushrooms; draws a poison-tipped recurve bow.", Palette::POISON_ARROW_GREEN),
            ("Carrion Fly (Swarmer)", "Swarming arena vermin; buzzing translucent wings, toxic red eyes & ground shadow.", Palette::FLY_EYE_TOXIC),
            ("Corpse Worm (Burrower)", "Pale segmented grub; flares concentric rows of needle-sharp teeth to devour flesh.", Palette::WORM_PALE_FLESH),
        ];

        let state_names = [
            "1: Idle (Breathing)",
            "2: Walk (Movement)",
            "3: Roll / Dodge / Scurry",
            "4: LMB (Primary Attack)",
            "5: RMB (Secondary Skill)",
            "6: Hit (Damage Flinch)",
            "7: Death (Defeat / Collapse)",
        ];

        let cur_char = characters[self.character_idx];
        let bg = Some(Color::rgb(14, 16, 22));

        // Header info - compact and bold
        let char_header = format!("◄ [A]  Entity [{}/14]: {}  [D] ►", self.character_idx + 1, cur_char.0);
        let char_hx = px + (pw.saturating_sub(char_header.len())) / 2;
        ui.print_bold(char_hx, py + 1, &char_header, cur_char.2, bg);

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
            _ => "SouthEast (3/4)",
        };

        let pause_tag = if self.paused { " [PAUSED]" } else { "" };
        let state_header = format!("[1-7/Tab] {}   |   [W/S] Dir: {}{}", state_names[self.state_idx], dir_str, pause_tag);
        let state_hx = px + (pw.saturating_sub(state_header.len())) / 2;
        ui.print(state_hx, py + 2, &state_header, Palette::TEXT_GOLD, bg);

        // Subtitle lore
        ui.print_centered(py + 3, cur_char.1, Palette::TEXT_MUTED, bg);

        // Frame calculation
        let t = self.anim_timer;

        // Magnification scale: use 3x if terminal is large (>= 50 rows), else 2x
        let mag_scale = if ui.rows >= 50 { 3 } else { 2 };

        // Define clear showcase stage rectangle and punch out all UI cells
        // so the half-block TrueColor Framebuffer displays without any text box overlay
        let stage_top = py + 5;
        let stage_bottom = py + ph - 3;
        let stage_rect = Rect::new(px as i32 + 2, stage_top as i32, pw as i32 - 4, (stage_bottom - stage_top) as i32);
        ui.clear_rect(stage_rect);

        // Draw dark stone stage floor directly in Framebuffer
        fb.draw_rect(
            stage_rect.x,
            stage_rect.y * 2,
            stage_rect.w,
            stage_rect.h * 2,
            Color::rgb(11, 13, 19),
            true,
        );

        let left_cx = stage_rect.x + (stage_rect.w / 4);
        let right_cx = stage_rect.x + (stage_rect.w * 3 / 5);

        // Category labels right above the stage
        ui.print((left_cx - 7).max(0) as usize, py + 4, "─ 1x SCALE ─", Palette::TEXT_BRIGHT, bg);
        ui.print(
            (right_cx - 12).max(0) as usize,
            py + 4,
            &format!("─ {}x MAGNIFIED VIEW ─", mag_scale),
            Palette::TEXT_GOLD,
            bg,
        );

        // Pedestal ground line anchored near bottom of the stage
        let pedestal_pixel_y = (stage_bottom as i32 * 2) - 8;

        // Ground stone dais under 1x preview
        fb.draw_rect(left_cx - 16, pedestal_pixel_y, 32, 2, Palette::STONE_MID, true);
        fb.draw_ellipse(left_cx, pedestal_pixel_y - 1, 10, 2, Color::rgba(4, 5, 10, 150));

        // Ground stone dais under magnified preview
        let plinth_w = (20 * mag_scale) as i32;
        fb.draw_rect(right_cx - (plinth_w / 2), pedestal_pixel_y, plinth_w, 3, Palette::STONE_WARM_MID, true);
        fb.draw_ellipse(right_cx, pedestal_pixel_y - 1, (9 * mag_scale) as i32, 3, Color::rgba(4, 5, 10, 160));

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

        // Standardized anchor calculations for humanoid (18x30)
        let render_1x_x = left_cx - 9;
        let render_1x_y = pedestal_pixel_y - 30;
        let render_mag_x = right_cx - ((9 * mag_scale) as i32);
        let render_mag_y = pedestal_pixel_y - ((30 * mag_scale) as i32);

        let dummy_gear = LayeredGear::empty();

        match self.character_idx {
            0 => {
                // Base Adventurer
                let sp = match self.state_idx {
                    0 => &bank.body_idle[self.dir_idx][(t as usize) % 4],
                    1 => &bank.body_walk[self.dir_idx][(t as usize) % 6],
                    2 => &bank.body_roll[self.dir_idx][(t as usize) % 7],
                    3 => &bank.body_lmb[self.dir_idx][(t as usize) % 4],
                    4 => &bank.body_rmb[self.dir_idx][(t as usize) % 4],
                    5 => &bank.body_hit[self.dir_idx],
                    _ => &bank.body_death,
                };
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
            }
            1..=5 => {
                // Classes: Mage, Knight, Assassin, Hunter, Alchemist
                let class_kind = match self.character_idx {
                    1 => ClassKind::Mage,
                    2 => ClassKind::Knight,
                    3 => ClassKind::Assassin,
                    4 => ClassKind::Hunter,
                    _ => ClassKind::Alchemist,
                };

                if let Some(gear_set) = bank.get_class_gear(class_kind) {
                    match self.state_idx {
                        0 => {
                            let body = &bank.body_idle[self.dir_idx][(t as usize) % 4];
                            let gear_sprites = [
                                Some(&gear_set.head[self.dir_idx]),
                                Some(&gear_set.torso[self.dir_idx]),
                                gear_set.cape[self.dir_idx].as_ref(),
                                gear_set.main_hand[self.dir_idx].as_ref(),
                                gear_set.off_hand[self.dir_idx].as_ref(),
                                gear_set.accessory[self.dir_idx].as_ref(),
                            ];
                            dummy_gear.draw_composite(fb, body, &gear_sprites, render_1x_x, render_1x_y, dir, false);
                            draw_scaled(fb, body, render_mag_x, render_mag_y, mag_scale);
                            for sp in gear_sprites.iter().flatten() {
                                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                            }
                        }
                        1 => {
                            let body = &bank.body_walk[self.dir_idx][(t as usize) % 6];
                            let gear_sprites = [
                                Some(&gear_set.head[self.dir_idx]),
                                Some(&gear_set.torso[self.dir_idx]),
                                gear_set.cape[self.dir_idx].as_ref(),
                                gear_set.main_hand[self.dir_idx].as_ref(),
                                gear_set.off_hand[self.dir_idx].as_ref(),
                                gear_set.accessory[self.dir_idx].as_ref(),
                            ];
                            dummy_gear.draw_composite(fb, body, &gear_sprites, render_1x_x, render_1x_y, dir, false);
                            draw_scaled(fb, body, render_mag_x, render_mag_y, mag_scale);
                            for sp in gear_sprites.iter().flatten() {
                                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                            }
                        }
                        2 => {
                            let sp = &bank.body_roll[self.dir_idx][(t as usize) % 7];
                            sp.draw(fb, render_1x_x, render_1x_y, false);
                            draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                        }
                        3 => {
                            let sp = &gear_set.lmb_frames[self.dir_idx][(t as usize) % 4];
                            sp.draw(fb, render_1x_x, render_1x_y, false);
                            draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                        }
                        4 => {
                            let sp = &gear_set.rmb_frames[self.dir_idx][(t as usize) % 4];
                            sp.draw(fb, render_1x_x, render_1x_y, false);
                            draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                        }
                        5 => {
                            let sp = &gear_set.hit_frame[self.dir_idx];
                            sp.draw(fb, render_1x_x, render_1x_y, false);
                            draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                        }
                        _ => {
                            let sp = &gear_set.death_frame;
                            sp.draw(fb, render_1x_x, render_1x_y, false);
                            draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
                        }
                    }
                }
            }
            6 => {
                // Firekeeper (18x30)
                let sp = &bank.firekeeper_anim[(t as usize) % 4];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
            }
            7 => {
                // Blacksmith (22x34)
                let b_x = left_cx - 11;
                let b_y = pedestal_pixel_y - 34;
                let b_mag_x = right_cx - ((11 * mag_scale) as i32);
                let b_mag_y = pedestal_pixel_y - ((34 * mag_scale) as i32);
                let sp = &bank.blacksmith_anim[(t as usize) % 4];
                sp.draw(fb, b_x, b_y, false);
                draw_scaled(fb, sp, b_mag_x, b_mag_y, mag_scale);
            }
            8 => {
                // Unknown Adventurer (18x30)
                let sp = &bank.adventurer_anim[(t as usize) % 4];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
            }
            9 => {
                // Shady Witch (18x30)
                let sp = &bank.witch_anim[(t as usize) % 4];
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
            }
            10 => {
                // Acid Sludge (26x20)
                let s_x = left_cx - 13;
                let s_y = pedestal_pixel_y - 20;
                let s_mag_x = right_cx - ((13 * mag_scale) as i32);
                let s_mag_y = pedestal_pixel_y - ((20 * mag_scale) as i32);
                let sp = match self.state_idx {
                    0 => &bank.sludge_idle[(t as usize) % 4],
                    1 => &bank.sludge_walk[(t as usize) % 4],
                    2 => &bank.sludge_walk[1],
                    3 => &bank.sludge_vomit_spit,
                    4 => &bank.sludge_vomit_windup,
                    5 => &bank.sludge_hit,
                    _ => &bank.sludge_death[(t as usize) % 3],
                };
                sp.draw(fb, s_x, s_y, false);
                draw_scaled(fb, sp, s_mag_x, s_mag_y, mag_scale);
            }
            11 => {
                // Bogged Skeleton (18x30)
                let sp = match self.state_idx {
                    0 => &bank.skeleton_idle[self.dir_idx][(t as usize) % 4],
                    1 => &bank.skeleton_walk[self.dir_idx][(t as usize) % 4],
                    2 => &bank.skeleton_walk[self.dir_idx][2],
                    3 => &bank.skeleton_draw[self.dir_idx][(t as usize) % 3],
                    4 => &bank.skeleton_release[self.dir_idx],
                    5 => &bank.skeleton_hit[self.dir_idx],
                    _ => &bank.skeleton_death,
                };
                sp.draw(fb, render_1x_x, render_1x_y, false);
                draw_scaled(fb, sp, render_mag_x, render_mag_y, mag_scale);
            }
            12 => {
                // Carrion Fly (12x12 hovering)
                let f_x = left_cx - 6;
                let f_y = pedestal_pixel_y - 26;
                let f_mag_x = right_cx - ((6 * mag_scale) as i32);
                let f_mag_y = pedestal_pixel_y - ((26 * mag_scale) as i32);
                let sp = match self.state_idx {
                    3..=4 => &bank.fly_attack[(t as usize) % 2],
                    5 => &bank.fly_hit,
                    6 => &bank.fly_death,
                    _ => &bank.fly_anim[(t as usize) % 2],
                };
                bank.fly_shadow.draw(fb, f_x, pedestal_pixel_y - 4, false);
                sp.draw(fb, f_x, f_y, false);
                draw_scaled(fb, sp, f_mag_x, f_mag_y, mag_scale);
            }
            _ => {
                // Corpse Worm (14x10 crawling)
                let w_x = left_cx - 7;
                let w_y = pedestal_pixel_y - 10;
                let w_mag_x = right_cx - ((7 * mag_scale) as i32);
                let w_mag_y = pedestal_pixel_y - ((10 * mag_scale) as i32);
                let sp = match self.state_idx {
                    2..=4 => &bank.worm_lunge[(t as usize) % 2],
                    5 => &bank.worm_hit,
                    6 => &bank.worm_death,
                    _ => &bank.worm_anim[(t as usize) % 4],
                };
                sp.draw(fb, w_x, w_y, false);
                draw_scaled(fb, sp, w_mag_x, w_mag_y, mag_scale);
            }
        }

        // Footer instructions - anchored at bottom of panel
        ui.print_centered(
            py + ph - 2,
            "[A/D] Character (14)  [W/S] Direction  [1-7/Tab] Animation  [Space] Pause  [F4/ESC] Close",
            Palette::TEXT_GOLD,
            bg,
        );
        ui.print_centered(
            py + ph - 1,
            "Pixel Standards: 1px Dark Outline | Cool Rim Illumination | 3-4 Tone Ramps",
            Palette::TEXT_MUTED,
            bg,
        );
    }
}
