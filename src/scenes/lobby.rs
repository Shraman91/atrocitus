use super::adventurer_dialogue::AdventurerDialogueScene;
use super::backpack::BackpackScene;
use super::blacksmith::BlacksmithScene;
use super::dialogue::DialogueScene;
use super::gallery::GalleryScene;
use super::map::MapScene;
use super::spellbook::SpellbookScene;
use super::title::TitleScene;
use super::witch_shop::WitchShopScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::LightingEngine;
use crate::engine::postprocess::PostProcessor;
use crate::engine::text::{FontSize, TextRenderer};
use crate::game::bonfire::Bonfire;
use crate::game::camera::Camera;
use crate::game::classes::{ClassKind, SaveData};
use crate::game::inventory::Inventory;
use crate::game::map::MapGrid;
use crate::game::npc::{Npc, NpcKind};
use crate::game::particles::ParticleSystem;
use crate::game::player::Player;
use crate::game::world::World;
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

enum RenderableItem<'a> {
    Prop(&'a crate::game::world::PropInstance),
    Player,
    Bonfire,
    Npc(&'a Npc),
}

pub struct LobbyScene {
    world: World,
    player: Player,
    camera: Camera,
    bonfire: Bonfire,
    firekeeper: Npc,
    blacksmith: Npc,
    adventurer: Npc,
    witch: Npc,
    map_grid: MapGrid,
    inventory: Inventory,
    particles: ParticleSystem,
    lighting: LightingEngine,
    time: f32,
    banner_timer: f32,
    area_banner_timer: f32,
    controls_hint_timer: f32,
    show_controls: bool,
    show_debug: bool,
    is_paused: bool,
    pause_selected: usize,
    gate_dialogue_timer: f32,
    last_mouse_world: (f32, f32),
}

impl LobbyScene {
    pub fn new(world: World) -> Self {
        let cx = world.width as f32 * 0.5;
        let cy = world.height as f32 * 0.5;

        let mut player = Player::new(cx, cy + 18.0);
        let save = SaveData::load();
        player.set_class(save.chosen_class);

        let camera = Camera::new(world.width as f32, world.height as f32, 120.0, 80.0);
        let bonfire = Bonfire::new(cx, cy);
        let firekeeper = Npc::new(NpcKind::Firekeeper, cx + 24.0, cy - 6.0);
        let blacksmith = Npc::new(NpcKind::Blacksmith, cx - 34.0, cy + 8.0);
        let adventurer = Npc::new(NpcKind::UnknownAdventurer, cx - 22.0, cy - 26.0);
        let witch = Npc::new(NpcKind::ShadyWitch, cx + 36.0, cy + 18.0);
        let map_grid = MapGrid::new(world.width, world.height);
        let inventory = Inventory::new();

        let mut particles = ParticleSystem::new(128);
        for _ in 0..25 {
            particles.spawn_ash(
                cx + rand::random::<f32>() * 120.0 - 60.0,
                cy + rand::random::<f32>() * 80.0 - 40.0,
            );
        }

        Self {
            world,
            player,
            camera,
            bonfire,
            firekeeper,
            blacksmith,
            adventurer,
            witch,
            map_grid,
            inventory,
            particles,
            lighting: LightingEngine::new(),
            time: 0.0,
            banner_timer: 0.0,
            area_banner_timer: 5.0,
            controls_hint_timer: 9.0,
            show_controls: true,
            show_debug: false,
            is_paused: false,
            pause_selected: 0,
            gate_dialogue_timer: 0.0,
            last_mouse_world: (0.0, 0.0),
        }
    }
}

impl Scene for LobbyScene {
    fn on_resize(&mut self, w: usize, h: usize) {
        self.camera.resize(w as f32, h as f32);
    }

    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        _bank: &SpriteBank,
    ) -> SceneTransition {
        self.time += dt;
        self.banner_timer += dt * 2.5;
        self.last_mouse_world = input.mouse_world_pos;

        // Keep player equipped class in sync with save.ron (e.g. after returning from ClassSelectScene)
        let save = SaveData::load();
        if self.player.equipped_class != save.chosen_class {
            self.player.set_class(save.chosen_class);
        }

        // Toggle Debug overlay (F3)
        if input.just_pressed(VirtualKey::F3) {
            self.show_debug = !self.show_debug;
        }

        // Open Sprite Gallery viewer (F4)
        if input.just_pressed(VirtualKey::F4) {
            return SceneTransition::Push(Box::new(GalleryScene::new()));
        }

        // Populate backpack with debug items (F5)
        if input.just_pressed(VirtualKey::F5) {
            self.inventory.add_debug_items();
        }

        // Reset class debug hotkey (F9)
        if input.just_pressed(VirtualKey::F9) {
            self.player.set_class(None);
            let mut save = SaveData::load();
            save.reset();
        }

        // Open Spell Book hotkey (K)
        if input.just_pressed(VirtualKey::K) {
            return SceneTransition::Push(Box::new(SpellbookScene::new()));
        }

        // Open Map hotkey (M)
        let gate_pos = (
            self.world.width as f32 * 0.5,
            self.world.height as f32 * 0.5 - self.world.height as f32 * 0.38 + 24.0,
        );
        if input.just_pressed(VirtualKey::M) {
            return SceneTransition::Push(Box::new(MapScene::new(
                self.map_grid.clone(),
                (self.player.x, self.player.y),
                self.player.facing_angle,
                (self.bonfire.x, self.bonfire.y),
                gate_pos,
                vec![
                    ("F", (self.firekeeper.x, self.firekeeper.y), Palette::BONFIRE_ORANGE),
                    ("B", (self.blacksmith.x, self.blacksmith.y), Palette::BONFIRE_RED),
                    ("A", (self.adventurer.x, self.adventurer.y), Palette::MAGE_CYAN),
                    ("W", (self.witch.x, self.witch.y), Color::rgb(190, 80, 245)),
                ],
                (self.world.width as f32, self.world.height as f32),
            )));
        }

        // Open Backpack hotkey (B)
        if input.just_pressed(VirtualKey::B) {
            return SceneTransition::Push(Box::new(BackpackScene::new(self.inventory.clone())));
        }

        // Toggle Controls hint (F1 / H)
        if input.just_pressed(VirtualKey::F1) || input.just_pressed(VirtualKey::Char('h')) {
            self.show_controls = !self.show_controls;
            if self.show_controls {
                self.controls_hint_timer = 6.0;
            }
        }

        // Toggle Pause (Escape)
        if input.just_pressed(VirtualKey::Escape) {
            self.is_paused = !self.is_paused;
            self.pause_selected = 0;
        }

        if self.is_paused {
            if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
                if self.pause_selected > 0 {
                    self.pause_selected -= 1;
                } else {
                    self.pause_selected = 2;
                }
            }
            if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
                if self.pause_selected < 2 {
                    self.pause_selected += 1;
                } else {
                    self.pause_selected = 0;
                }
            }
            if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
                match self.pause_selected {
                    0 => self.is_paused = false,
                    1 => return SceneTransition::Switch(Box::new(TitleScene::new())),
                    2 => return SceneTransition::Quit,
                    _ => {}
                }
            }
            return SceneTransition::None;
        }

        // Normal Gameplay Updates
        self.player.update(dt, input, &self.world, &mut self.particles);
        self.map_grid.reveal(self.player.x, self.player.y, 45.0);
        self.bonfire.update(dt, self.player.x, self.player.y, input, &mut self.particles);
        self.firekeeper.update(dt);
        self.blacksmith.update(dt);
        self.adventurer.update(dt);
        self.witch.update(dt);

        let mx = input.mouse_world_pos.0;
        let my = input.mouse_world_pos.1;
        let pressed_talk = input.just_pressed(VirtualKey::E);
        let clicked = input.mouse_left_just_pressed;

        // Firekeeper interaction check (Click or E when in range)
        if self.firekeeper.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.firekeeper.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(DialogueScene::new()));
        }

        // Blacksmith interaction check (Click or E when in range)
        if self.blacksmith.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.blacksmith.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(BlacksmithScene::new()));
        }

        // Adventurer interaction check (Click or E when in range)
        if self.adventurer.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.adventurer.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(AdventurerDialogueScene::new()));
        }

        // Witch interaction check (Click or E when in range)
        if self.witch.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.witch.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(WitchShopScene::new()));
        }

        // Gate of Trials interaction check (North edge)
        let dist_to_gate = ((self.player.x - gate_pos.0).powi(2) + (self.player.y - gate_pos.1).powi(2)).sqrt();
        let clicked_gate = input.mouse_left_just_pressed
            && (input.mouse_world_pos.0 - gate_pos.0).abs() < 18.0
            && (input.mouse_world_pos.1 - gate_pos.1).abs() < 18.0;

        if (dist_to_gate < 26.0 && input.just_pressed(VirtualKey::E)) || (dist_to_gate < 34.0 && clicked_gate) {
            self.gate_dialogue_timer = 3.5;
        }
        if self.gate_dialogue_timer > 0.0 {
            self.gate_dialogue_timer -= dt;
        }

        // Spawn ambient ash particles
        if rand::random::<f32>() < 0.25 {
            let cx = self.camera.x + rand::random::<f32>() * self.camera.screen_w;
            let cy = self.camera.y + rand::random::<f32>() * self.camera.screen_h;
            self.particles.spawn_ash(cx, cy);
        }

        self.particles.update(dt);
        self.camera.follow(self.player.x, self.player.y);
        self.camera.update(dt);

        if self.area_banner_timer > 0.0 {
            self.area_banner_timer -= dt;
        }
        if self.controls_hint_timer > 0.0 {
            self.controls_hint_timer -= dt;
        }

        SceneTransition::None
    }

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank) {
        let cam_x = self.camera.x.round() as i32;
        let cam_y = self.camera.y.round() as i32;
        let banner_frame = (self.banner_timer as usize) % 2;

        // 1. Parallax Distant Colosseum Sky & Blood Moon
        self.world.draw_parallax_sky(fb, self.camera.x, self.camera.y);

        // 2. Arena Ground Plane (Sand & Flagstones)
        self.world.draw_ground(fb, cam_x, cam_y);

        // 3. Y-Sorting Render Queue
        let mut render_queue = Vec::with_capacity(self.world.props.len() + 6);

        for prop in &self.world.props {
            render_queue.push((prop.base_y, RenderableItem::Prop(prop)));
        }
        render_queue.push((self.player.y.round() as i32, RenderableItem::Player));
        render_queue.push((self.bonfire.y.round() as i32, RenderableItem::Bonfire));
        render_queue.push((self.firekeeper.y.round() as i32, RenderableItem::Npc(&self.firekeeper)));
        render_queue.push((self.blacksmith.y.round() as i32, RenderableItem::Npc(&self.blacksmith)));
        render_queue.push((self.adventurer.y.round() as i32, RenderableItem::Npc(&self.adventurer)));
        render_queue.push((self.witch.y.round() as i32, RenderableItem::Npc(&self.witch)));

        // Sort by base_y ascending
        render_queue.sort_by_key(|&(y, _)| y);

        // 4. Render Depth-Sorted Entities
        for (_, item) in render_queue {
            match item {
                RenderableItem::Prop(p) => {
                    let sprite = self.world.get_prop_sprite(bank, p.sprite_index, banner_frame);
                    let sx = p.x - cam_x;
                    let sy = p.y - cam_y;
                    sprite.draw(fb, sx, sy, false);
                }
                RenderableItem::Player => {
                    self.player.draw(fb, bank, cam_x, cam_y);
                }
                RenderableItem::Bonfire => {
                    self.bonfire.draw(fb, bank, cam_x, cam_y);
                }
                RenderableItem::Npc(npc) => {
                    npc.draw(fb, bank, cam_x, cam_y);
                }
            }
        }

        // 5. Render Particle System (Embers, Ash, Dust)
        self.particles.draw(fb, cam_x, cam_y);

        // 6. Dynamic Lighting & Shadows Pass
        let bonfire_light = self.bonfire.get_light();
        let firekeeper_light = self.firekeeper.get_light(self.time);
        let blacksmith_light = self.blacksmith.get_light(self.time);
        let adventurer_light = self.adventurer.get_light(self.time);
        let witch_light = self.witch.get_light(self.time);
        let player_aura = crate::engine::lighting::PointLight {
            x: self.player.x,
            y: self.player.y - 8.0,
            radius: 38.0,
            intensity: 0.55,
            color: Color::rgb(180, 200, 235),
            flicker_amount: 0.03,
        };

        let mut shadow_casters = self.world.get_shadow_casters();
        shadow_casters.push(self.player.get_shadow_caster());
        shadow_casters.push(self.firekeeper.get_shadow_caster());
        shadow_casters.push(self.blacksmith.get_shadow_caster());
        shadow_casters.push(self.adventurer.get_shadow_caster());
        shadow_casters.push(self.witch.get_shadow_caster());

        self.lighting.render_shadows(fb, &bonfire_light, &shadow_casters, cam_x, cam_y);
        self.lighting.apply_lighting(
            fb,
            &[bonfire_light, firekeeper_light, blacksmith_light, adventurer_light, witch_light, player_aura],
            cam_x,
            cam_y,
            self.time,
        );

        // 7. Atmospheric Post-Processing
        PostProcessor::apply_drifting_fog(fb, self.time);
        PostProcessor::apply_film_grain(fb, (self.time * 60.0) as u64);
        PostProcessor::apply_vignette(fb, 0.42);

        // 8. World UI Elements (Speech bubbles, [E / Click] Prompts)
        self.bonfire.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y);
        self.firekeeper.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.blacksmith.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.adventurer.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.witch.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);

        // Gate of Trials prompt
        let gate_pos = (
            self.world.width as f32 * 0.5,
            self.world.height as f32 * 0.5 - self.world.height as f32 * 0.38 + 24.0,
        );
        let dist_to_gate = ((self.player.x - gate_pos.0).powi(2) + (self.player.y - gate_pos.1).powi(2)).sqrt();
        let gate_scr_x = (gate_pos.0 - cam_x as f32).round() as i32;
        let gate_scr_y = (gate_pos.1 - cam_y as f32).round() as i32;

        if self.gate_dialogue_timer > 0.0 {
            TextRenderer::draw_speech_bubble(
                fb,
                gate_scr_x,
                gate_scr_y - 20,
                "The Gate of Trials is sealed until combat begins...",
                Palette::TEXT_BRIGHT,
                Palette::UI_PANEL_BG,
                Palette::UI_BORDER,
            );
        } else if dist_to_gate < 28.0 {
            let prompt = "[Click / E] EXAMINE GATE";
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = gate_scr_x - prompt_w / 2;
            let by = gate_scr_y - 26;
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_BORDER, false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_MUTED);
        }

        // 9. UiCanvas Overlaid HUD Elements (Crisp Real Characters)
        // Area Banner "THE ASHEN COLOSSEUM"
        if self.area_banner_timer > 0.0 {
            ui.print_centered(2, "THE ASHEN COLOSSEUM", Palette::TEXT_BRIGHT, None);
            ui.print_centered(3, "PROVING GROUNDS OF THE FALLEN", Palette::TEXT_MUTED, None);
        }

        // Active Class Emblem in Top-Left HUD (2-cell margin)
        let class_name = if let Some(class) = self.player.equipped_class {
            class.display_name()
        } else {
            "UNBOUND WANDERER"
        };
        let class_col = if let Some(class) = self.player.equipped_class {
            class.accent_color()
        } else {
            Palette::TEXT_MUTED
        };
        let mantle_badge = format!("╺[MANTLE: {}]╸", class_name);
        ui.print(2, 1, &mantle_badge, class_col, Some(Color::rgb(14, 16, 22)));

        if self.player.equipped_class == Some(ClassKind::Mage) {
            ui.print(2, 2, " [K] SPELLBOOK", Palette::MAGE_CYAN, Some(Color::rgb(14, 16, 22)));
        }

        // Controls Hint Box (Bottom-Left 2-cell margin)
        if self.show_controls || self.controls_hint_timer > 0.0 {
            let base_r = ui.rows.saturating_sub(4);
            let hint_bg = Some(Color::rgb(14, 16, 22));
            ui.print(2, base_r, "[WASD] Move  [Mouse] Aim  [Space/RMB] Roll  [E] Interact", Palette::TEXT_GOLD, hint_bg);
            ui.print(2, base_r + 1, "[M] Map  [B] Backpack  [K] Spells  [F2] Layouts  [F4] Gallery", Palette::TEXT_NORMAL, hint_bg);
            ui.print(2, base_r + 2, "[F1] Toggle Hints  [F3] Debug  [Esc] Pause", Palette::TEXT_MUTED, hint_bg);
        }

        // Debug Overlay (F3)
        if self.show_debug {
            let px = self.player.x;
            let py = self.player.y;
            let dbg_bg = Some(Color::rgb(14, 16, 22));
            ui.print(2, 4, &format!("POS: ({:.1}, {:.1}) | DIR: {:?}", px, py, self.player.dir4), Palette::TEXT_GOLD, dbg_bg);
            ui.print(2, 5, &format!("COLLIDERS: {} | PROPS: {}", self.world.colliders.len(), self.world.props.len()), Palette::TEXT_NORMAL, dbg_bg);
        }

        // Pause Menu Overlay
        if self.is_paused {
            let pw = 36;
            let ph = 9;
            let px = (ui.cols.saturating_sub(pw)) / 2;
            let py = (ui.rows.saturating_sub(ph)) / 2;

            let pause_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
            ui.draw_panel(pause_rect, "PAUSED", PanelStyle::double());

            let pause_items = ["RESUME", "MAIN MENU", "QUIT TO DESKTOP"];
            let panel_bg = Some(Color::rgb(14, 16, 22));

            for (i, &item) in pause_items.iter().enumerate() {
                let row = py + 2 + i * 2;
                let is_sel = i == self.pause_selected;
                let (prefix, suffix, color, bold) = if is_sel {
                    ("▶ ", " ◀", Palette::TEXT_GOLD, true)
                } else {
                    ("  ", "  ", Palette::TEXT_NORMAL, false)
                };
                let full = format!("{}{}{}", prefix, item, suffix);
                let col_x = px + (pw.saturating_sub(full.len())) / 2;
                if bold {
                    ui.print_bold(col_x, row, &full, color, panel_bg);
                } else {
                    ui.print(col_x, row, &full, color, panel_bg);
                }
            }
        }
    }
}
