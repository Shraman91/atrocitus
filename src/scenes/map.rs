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
    map_grid: MapGrid,
    player_pos: (f32, f32),
    player_angle: f32,
    bonfire_pos: (f32, f32),
    gate_pos: (f32, f32),
    npc_positions: Vec<(&'static str, (f32, f32), Color)>,
    world_size: (f32, f32),
}

impl MapScene {
    pub fn new(
        map_grid: MapGrid,
        player_pos: (f32, f32),
        player_angle: f32,
        bonfire_pos: (f32, f32),
        gate_pos: (f32, f32),
        npc_positions: Vec<(&'static str, (f32, f32), Color)>,
        world_size: (f32, f32),
    ) -> Self {
        Self {
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
        let pw = (ui.cols.saturating_sub(10)).min(92);
        let ph = (ui.rows.saturating_sub(6)).min(26);
        let px = (ui.cols.saturating_sub(pw)) / 2;
        let py = (ui.rows.saturating_sub(ph)) / 2;

        let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
        ui.draw_panel(panel_rect, "THE ASHEN COLOSSEUM - MAP", PanelStyle::double());

        let bg = Some(Color::rgb(14, 16, 22));

        // Map drawing area inside panel:
        // Left section: Arena Grid (width ~56 cols, height ~20 rows)
        // Right section: Legend & Intel (~26 cols)
        let map_w = 54;
        let map_h = ph.saturating_sub(4);
        let map_ox = px + 3;
        let map_oy = py + 2;

        // Draw Map Grid cells
        for r in 0..map_h {
            for c in 0..map_w {
                // Map screen cell (c, r) to grid coordinate
                let grid_c = (c * self.map_grid.width) / map_w;
                let grid_r = (r * self.map_grid.height) / map_h;

                let explored = self.map_grid.is_explored(grid_c, grid_r);

                // Colosseum oval arena boundary check
                let norm_x = (c as f32 - (map_w as f32 * 0.5)) / (map_w as f32 * 0.44);
                let norm_y = (r as f32 - (map_h as f32 * 0.5)) / (map_h as f32 * 0.44);
                let in_arena = norm_x * norm_x + norm_y * norm_y <= 1.0;
                let on_wall = (norm_x * norm_x + norm_y * norm_y - 1.0).abs() < 0.12;

                let (ch, fg) = if on_wall {
                    ('█', Palette::STONE_DARK)
                } else if !in_arena {
                    (' ', Color::BLACK)
                } else if explored {
                    ('·', Color::rgb(45, 52, 65))
                } else {
                    ('░', Color::rgb(20, 22, 28))
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

        // Gate marker
        let (gx, gy) = to_map_coord(self.gate_pos.0, self.gate_pos.1);
        ui.print_bold(gx, gy, "⛩", Palette::GOLD_TRIM, bg);

        // Bonfire marker
        let (bx, by) = to_map_coord(self.bonfire_pos.0, self.bonfire_pos.1);
        ui.print_bold(bx, by, "🔥", Palette::BONFIRE_YELLOW, bg);

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
        let legend_x = map_ox + map_w + 3;
        for r in (py + 2)..(py + ph - 2) {
            ui.print(legend_x - 2, r, "│", Color::rgb(40, 44, 56), bg);
        }

        // Right side: Map Legend
        ui.print_bold(legend_x, py + 2, "MAP LEGEND", Palette::TEXT_GOLD, bg);
        ui.print(legend_x, py + 4, "▶ / ▲ Player Pos", Color::rgb(255, 235, 120), bg);
        ui.print(legend_x, py + 6, "🔥    Ashen Bonfire", Palette::BONFIRE_YELLOW, bg);
        ui.print(legend_x, py + 8, "⛩    Gate of Trials", Palette::GOLD_TRIM, bg);
        ui.print(legend_x, py + 10, "F    Firekeeper", Palette::BONFIRE_ORANGE, bg);
        ui.print(legend_x, py + 12, "B    Blacksmith", Palette::BONFIRE_RED, bg);
        ui.print(legend_x, py + 14, "A    Adventurer", Palette::MAGE_CYAN, bg);
        ui.print(legend_x, py + 16, "W    Shady Witch", Color::rgb(190, 80, 245), bg);

        ui.print(
            legend_x,
            py + 19,
            &format!("Coord: ({:.0}, {:.0})", self.player_pos.0, self.player_pos.1),
            Palette::TEXT_MUTED,
            bg,
        );

        ui.print_centered(
            py + ph + 1,
            "[M / ESC] Close Map  |  Walk to uncover fog-of-war",
            Palette::TEXT_MUTED,
            None,
        );
    }
}
