use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::game::map::MapGrid;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

pub struct MapScene {
    pub area_title: String,
    map_grid: MapGrid,
    player_pos: (f32, f32),
    player_angle: f32,
    bonfire_pos: (f32, f32),
    gate_pos: (f32, f32),
    npc_positions: Vec<(&'static str, (f32, f32), Color)>,
    world_size: (f32, f32),
}

impl MapScene {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        area_title: String,
        map_grid: MapGrid,
        player_pos: (f32, f32),
        player_angle: f32,
        bonfire_pos: (f32, f32),
        gate_pos: (f32, f32),
        npc_positions: Vec<(&'static str, (f32, f32), Color)>,
        world_size: (f32, f32),
    ) -> Self {
        Self {
            area_title,
            map_grid,
            player_pos,
            player_angle,
            bonfire_pos,
            gate_pos,
            npc_positions,
            world_size,
        }
    }
}

impl Scene for MapScene {
    fn update(
        &mut self,
        _dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        if input.just_pressed(VirtualKey::Escape) || input.just_pressed(VirtualKey::M) {
            return SceneTransition::Pop;
        }
        SceneTransition::None
    }

    fn render(&self, _fb: &mut Framebuffer, ui: &mut UiCanvas, _bank: &SpriteBank) {
        // Grand expanded layout taking almost full screen width and height
        let pw = ui.cols.saturating_sub(6).min(180);
        let ph = ui.rows.saturating_sub(4).min(56);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, &self.area_title, PanelStyle::double());

        let bg = Some(Color::rgb(14, 16, 22));

        // Map layout: Left = Large Exploration Grid, Right = Legend & Intel Panel
        let legend_w = 28;
        let map_w = pw.saturating_sub(legend_w + 4).max(30);
        let map_h = ph.saturating_sub(4).max(16);
        let map_ox = px + 2;
        let map_oy = py + 2;

        let is_colosseum = self.area_title.contains("COLOSSEUM");

        // Draw Map Grid
        for r in 0..map_h {
            for c in 0..map_w {
                let grid_c = (c * self.map_grid.width) / map_w;
                let grid_r = (r * self.map_grid.height) / map_h;

                let explored = self.map_grid.is_explored(grid_c, grid_r);

                let (ch, fg) = if is_colosseum {
                    let norm_x = (c as f32 - (map_w as f32 * 0.5)) / (map_w as f32 * 0.44);
                    let norm_y = (r as f32 - (map_h as f32 * 0.5)) / (map_h as f32 * 0.44);
                    let dist_sq = norm_x * norm_x + norm_y * norm_y;
                    let on_wall = (dist_sq - 1.0).abs() < 0.12;
                    let in_arena = dist_sq <= 1.0;

                    if on_wall {
                        ('█', Palette::STONE_DARK)
                    } else if !in_arena {
                        (' ', Color::BLACK)
                    } else if explored {
                        ('·', Color::rgb(55, 65, 80))
                    } else {
                        ('░', Color::rgb(22, 25, 34))
                    }
                } else {
                    // Rectangular gothic world boundaries with portal gaps
                    let on_border_x = c == 0 || c == map_w - 1;
                    let on_border_y = r == 0 || r == map_h - 1;
                    let center_c = map_w / 2;
                    let is_north_gate = r == 0 && (c as i32 - center_c as i32).abs() <= 2;
                    let is_south_gate = r == map_h - 1 && (c as i32 - center_c as i32).abs() <= 2;

                    if (on_border_x || on_border_y) && !is_north_gate && !is_south_gate {
                        ('█', Palette::STONE_COOL_DARK)
                    } else if is_north_gate || is_south_gate {
                        ('░', Palette::GOLD_TRIM)
                    } else if explored {
                        ('·', Color::rgb(60, 72, 88))
                    } else {
                        (' ', Color::rgb(14, 16, 22))
                    }
                };

                ui.print(map_ox + c, map_oy + r, &ch.to_string(), fg, bg);
            }
        }

        // Draw Entity Markers on Map
        let to_map_coord = |wx: f32, wy: f32| -> (usize, usize) {
            let cx = ((wx / self.world_size.0) * map_w as f32) as usize;
            let cy = ((wy / self.world_size.1) * map_h as f32) as usize;
            (
                map_ox + cx.min(map_w - 1),
                map_oy + cy.min(map_h - 1),
            )
        };

        // Gate / Fog Wall marker
        if self.gate_pos.0 > 0.0 || self.gate_pos.1 > 0.0 {
            let (gx, gy) = to_map_coord(self.gate_pos.0, self.gate_pos.1);
            let gate_symbol = if is_colosseum { "⛩" } else { "🌫" };
            ui.print_bold(gx, gy, gate_symbol, Palette::GOLD_TRIM, bg);
        }

        // Bonfire marker
        if self.bonfire_pos.0 > 0.0 || self.bonfire_pos.1 > 0.0 {
            let (bx, by) = to_map_coord(self.bonfire_pos.0, self.bonfire_pos.1);
            ui.print_bold(bx, by, "🔥", Palette::BONFIRE_YELLOW, bg);
        }

        // NPCs
        for &(abbr, pos, col) in &self.npc_positions {
            let (nx, ny) = to_map_coord(pos.0, pos.1);
            ui.print_bold(nx, ny, abbr, col, bg);
        }

        // Player marker with directional arrow
        let (px_coord, py_coord) = to_map_coord(self.player_pos.0, self.player_pos.1);
        let arrow = match (self.player_angle.to_degrees() + 360.0) % 360.0 {
            d if (337.5..=360.0).contains(&d) || (0.0..22.5).contains(&d) => "▶",
            d if (22.5..67.5).contains(&d) => "◢",
            d if (67.5..112.5).contains(&d) => "▼",
            d if (112.5..157.5).contains(&d) => "◣",
            d if (157.5..202.5).contains(&d) => "◀",
            d if (202.5..247.5).contains(&d) => "◤",
            d if (247.5..292.5).contains(&d) => "▲",
            _ => "◥",
        };
        ui.print_bold(px_coord, py_coord, arrow, Color::rgb(255, 235, 120), bg);

        // Divider
        let legend_x = map_ox + map_w + 2;
        for r in (py + 1)..(py + ph - 1) {
            ui.print(legend_x - 1, r, "│", Color::rgb(45, 50, 65), bg);
        }

        // Exploration Percentage
        let explored_cells = self.map_grid.explored.iter().filter(|&&e| e).count();
        let total_cells = self.map_grid.explored.len().max(1);
        let explore_pct = (explored_cells * 100) / total_cells;

        // Right side: Map Legend & Telemetry
        ui.print_bold(legend_x + 1, py + 2, "MAP & TELEMETRY", Palette::TEXT_GOLD, bg);

        // Progress bar
        let bar_width = 14;
        let filled = (explore_pct * bar_width) / 100;
        let bar_str = format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(bar_width - filled),
            explore_pct
        );
        ui.print(legend_x + 1, py + 4, "Explored:", Palette::TEXT_MUTED, bg);
        ui.print_bold(legend_x + 1, py + 5, &bar_str, Palette::MAGE_CYAN, bg);

        ui.print(legend_x + 1, py + 7, "MARKERS", Palette::TEXT_BRIGHT, bg);
        ui.print(legend_x + 1, py + 9, "▶ / ▲ Player Pos", Color::rgb(255, 235, 120), bg);
        ui.print(legend_x + 1, py + 11, "🔥    Ashen Bonfire", Palette::BONFIRE_YELLOW, bg);
        if is_colosseum {
            ui.print(legend_x + 1, py + 13, "⛩    Gate of Trials", Palette::GOLD_TRIM, bg);
            ui.print(legend_x + 1, py + 15, "F    Firekeeper", Palette::BONFIRE_ORANGE, bg);
            ui.print(legend_x + 1, py + 17, "B    Blacksmith", Palette::BONFIRE_RED, bg);
            ui.print(legend_x + 1, py + 19, "A    Adventurer", Palette::MAGE_CYAN, bg);
            ui.print(legend_x + 1, py + 21, "W    Shady Witch", Color::rgb(190, 80, 245), bg);
        } else {
            ui.print(legend_x + 1, py + 13, "🌫    Shimmering Fog", Palette::GOLD_TRIM, bg);
            ui.print(legend_x + 1, py + 15, "·    Explored Path", Color::rgb(60, 72, 88), bg);
            ui.print(legend_x + 1, py + 17, "░    Passage Gate", Palette::GOLD_TRIM, bg);
            ui.print(legend_x + 1, py + 19, "█    Perimeter Wall", Palette::STONE_COOL_DARK, bg);
        }

        ui.print(
            legend_x + 1,
            py + ph.saturating_sub(6),
            &format!("World: {:.0}x{:.0}", self.world_size.0, self.world_size.1),
            Palette::TEXT_MUTED,
            bg,
        );
        ui.print(
            legend_x + 1,
            py + ph.saturating_sub(4),
            &format!("Coord: ({:.0}, {:.0})", self.player_pos.0, self.player_pos.1),
            Palette::TEXT_GOLD,
            bg,
        );

        ui.print_centered(
            py + ph - 1,
            "[M / ESC] Close Map  |  Walk to unveil world fog-of-war",
            Palette::TEXT_MUTED,
            bg,
        );
    }
}
