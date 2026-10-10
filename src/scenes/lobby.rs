use super::title::TitleScene;
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
use crate::game::npc::{Npc, NpcKind};
use crate::game::particles::ParticleSystem;
use crate::game::player::Player;
use crate::game::world::World;

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
    npcs: Vec<Npc>,
    particles: ParticleSystem,
    lighting: LightingEngine,
    time: f32,
    area_banner_timer: f32,
    controls_hint_timer: f32,
    show_controls: bool,
    show_debug: bool,
    is_paused: bool,
    pause_selected: usize,
    fog_gate_dialogue_timer: f32,
}

impl LobbyScene {
    pub fn new(world: World) -> Self {
        let cx = world.width as f32 * 0.5;
        let cy = world.height as f32 * 0.5;

        let player = Player::new(cx, cy + 18.0);
        let camera = Camera::new(world.width as f32, world.height as f32, 120.0, 80.0);
        let bonfire = Bonfire::new(cx, cy);

        let npcs = vec![
            Npc::new(NpcKind::Blacksmith, cx - 44.0, cy - 8.0),
            Npc::new(NpcKind::Firekeeper, cx + 42.0, cy - 8.0),
        ];

        let mut particles = ParticleSystem::new(128);
        for _ in 0..25 {
            particles.spawn_ash(cx + rand::random::<f32>() * 120.0 - 60.0, cy + rand::random::<f32>() * 80.0 - 40.0);
        }

        Self {
            world,
            player,
            camera,
            bonfire,
            npcs,
            particles,
            lighting: LightingEngine::new(),
            time: 0.0,
            area_banner_timer: 5.0,
            controls_hint_timer: 8.5,
            show_controls: true,
            show_debug: false,
            is_paused: false,
            pause_selected: 0,
            fog_gate_dialogue_timer: 0.0,
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

        // Toggle Debug overlay
        if input.just_pressed(VirtualKey::F3) {
            self.show_debug = !self.show_debug;
        }

        // Toggle Controls hint
        if input.just_pressed(VirtualKey::F1) || input.just_pressed(VirtualKey::Char('h')) {
            self.show_controls = !self.show_controls;
            if self.show_controls {
                self.controls_hint_timer = 6.0;
            }
        }

        // Toggle Pause
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
        self.bonfire.update(dt, self.player.x, self.player.y, input, &mut self.particles);

        for npc in &mut self.npcs {
            npc.update(dt, self.player.x, self.player.y, input);
        }

        // Fog gate interaction check
        let fog_gate_pos = (self.world.width as f32 * 0.5 + 20.0, self.world.height as f32 * 0.5 - 34.0);
        let dist_to_fog = ((self.player.x - fog_gate_pos.0).powi(2) + (self.player.y - fog_gate_pos.1).powi(2)).sqrt();
        if dist_to_fog < 24.0 && input.just_pressed(VirtualKey::E) {
            self.fog_gate_dialogue_timer = 3.0;
        }
        if self.fog_gate_dialogue_timer > 0.0 {
            self.fog_gate_dialogue_timer -= dt;
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

    fn render(&self, fb: &mut Framebuffer, bank: &SpriteBank) {
        let cam_x = self.camera.x.round() as i32;
        let cam_y = self.camera.y.round() as i32;

        // 1. Draw Parallax Sky & Mountain Silhouettes
        self.world.draw_parallax_sky(fb, self.camera.x, self.camera.y);

        // 2. Draw Plateau Ground
        self.world.draw_ground(fb, cam_x, cam_y);

        // 3. Collect and Depth-Sort All Renderable Entities & Props by Ground Y
        let mut render_queue = Vec::with_capacity(self.world.props.len() + self.npcs.len() + 2);

        for prop in &self.world.props {
            render_queue.push((prop.base_y, RenderableItem::Prop(prop)));
        }
        render_queue.push((self.player.y.round() as i32, RenderableItem::Player));
        render_queue.push((self.bonfire.y.round() as i32, RenderableItem::Bonfire));

        for npc in &self.npcs {
            render_queue.push((npc.y.round() as i32, RenderableItem::Npc(npc)));
        }

        // Sort by base_y ascending
        render_queue.sort_by_key(|&(y, _)| y);

        // 4. Render Depth-Sorted Entities
        for (_, item) in render_queue {
            match item {
                RenderableItem::Prop(p) => {
                    let sprite = self.world.get_prop_sprite(bank, p.sprite_index);
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
        let mut shadow_casters = self.world.get_shadow_casters();
        shadow_casters.push(self.player.get_shadow_caster());
        for npc in &self.npcs {
            shadow_casters.push(npc.get_shadow_caster());
        }

        self.lighting.render_shadows(fb, &bonfire_light, &shadow_casters, cam_x, cam_y);
        self.lighting.apply_lighting(fb, &[bonfire_light], cam_x, cam_y, self.time);

        // 7. Atmospheric Post-Processing
        PostProcessor::apply_drifting_fog(fb, self.time);
        PostProcessor::apply_film_grain(fb, (self.time * 60.0) as u64);
        PostProcessor::apply_vignette(fb, 0.42);

        // 8. World UI Elements (Speech bubbles, [E] Prompts)
        self.bonfire.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y);
        for npc in &self.npcs {
            npc.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y);
        }

        // Fog Gate prompt
        let fg_pos = (self.world.width as f32 * 0.5 + 20.0, self.world.height as f32 * 0.5 - 34.0);
        let dist_to_fg = ((self.player.x - fg_pos.0).powi(2) + (self.player.y - fg_pos.1).powi(2)).sqrt();
        let fg_scr_x = (fg_pos.0 - cam_x as f32).round() as i32;
        let fg_scr_y = (fg_pos.1 - cam_y as f32).round() as i32;

        if self.fog_gate_dialogue_timer > 0.0 {
            TextRenderer::draw_speech_bubble(
                fb,
                fg_scr_x,
                fg_scr_y - 20,
                "The seal remains unbroken...",
                Palette::TEXT_BRIGHT,
                Palette::UI_PANEL_BG,
                Palette::UI_BORDER,
            );
        } else if dist_to_fg < 24.0 {
            let prompt = "[E] ENTER FOG";
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = fg_scr_x - prompt_w / 2;
            let by = fg_scr_y - 26;
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_BORDER, false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_MUTED);
        }

        // 9. HUD Elements
        // Area Banner "THE ASHEN SANCTUM"
        if self.area_banner_timer > 0.0 {
            let banner_alpha = if self.area_banner_timer > 4.0 {
                (5.0 - self.area_banner_timer) / 1.0
            } else if self.area_banner_timer < 1.0 {
                self.area_banner_timer / 1.0
            } else {
                1.0
            };

            let title = "THE ASHEN SANCTUM";
            let title_w = TextRenderer::measure_width(title, FontSize::Standard5x7);
            let title_x = (fb.width as i32 - title_w) / 2;
            let title_y = 8;

            let col = Palette::TEXT_BRIGHT.scale(banner_alpha);
            TextRenderer::draw_text(fb, title_x, title_y, title, FontSize::Standard5x7, col);

            // Subtitle
            let sub = "SHRINE OF REST";
            let sub_w = TextRenderer::measure_width(sub, FontSize::Tiny3x5);
            let sub_x = (fb.width as i32 - sub_w) / 2;
            let sub_col = Palette::TEXT_MUTED.scale(banner_alpha);
            TextRenderer::draw_text(fb, sub_x, title_y + 8, sub, FontSize::Tiny3x5, sub_col);
        }

        // Controls Hint (bottom-left)
        if self.show_controls && self.controls_hint_timer > 0.0 {
            let hint_alpha = (self.controls_hint_timer / 1.0).clamp(0.0, 1.0);
            let hint_col = Palette::TEXT_MUTED.scale(hint_alpha);
            let hotkey_col = Palette::TEXT_GOLD.scale(hint_alpha);

            let by = fb.height as i32 - 14;
            TextRenderer::draw_text(fb, 4, by, "MOVE:", FontSize::Tiny3x5, hint_col);
            TextRenderer::draw_text(fb, 26, by, "WASD / ARROWS", FontSize::Tiny3x5, hotkey_col);
            TextRenderer::draw_text(fb, 4, by + 6, "INTERACT:", FontSize::Tiny3x5, hint_col);
            TextRenderer::draw_text(fb, 42, by + 6, "[E]", FontSize::Tiny3x5, hotkey_col);
            TextRenderer::draw_text(fb, 56, by + 6, "RUN:", FontSize::Tiny3x5, hint_col);
            TextRenderer::draw_text(fb, 72, by + 6, "[SHIFT]", FontSize::Tiny3x5, hotkey_col);
            TextRenderer::draw_text(fb, 102, by + 6, "MENU:", FontSize::Tiny3x5, hint_col);
            TextRenderer::draw_text(fb, 124, by + 6, "[ESC]", FontSize::Tiny3x5, hotkey_col);
        }

        // F3 Debug Overlay
        if self.show_debug {
            let dy = 4;
            let debug_panel_w = 90;
            let debug_panel_h = 24;
            fb.draw_rect(4, dy, debug_panel_w, debug_panel_h, Palette::UI_PANEL_BG, true);
            fb.draw_rect(4, dy, debug_panel_w, debug_panel_h, Palette::UI_BORDER, false);

            let pos_str = format!("POS: {:.1}, {:.1}", self.player.x, self.player.y);
            let cam_str = format!("CAM: {:.1}, {:.1}", self.camera.x, self.camera.y);
            let dir_str = format!("DIR: {:?} | RUN: {}", self.player.dir4, self.player.is_running);

            TextRenderer::draw_text(fb, 6, dy + 2, "DEBUG OVERLAY (F3)", FontSize::Tiny3x5, Palette::TEXT_GOLD);
            TextRenderer::draw_text(fb, 6, dy + 7, &pos_str, FontSize::Tiny3x5, Palette::TEXT_NORMAL);
            TextRenderer::draw_text(fb, 6, dy + 12, &cam_str, FontSize::Tiny3x5, Palette::TEXT_NORMAL);
            TextRenderer::draw_text(fb, 6, dy + 17, &dir_str, FontSize::Tiny3x5, Palette::TEXT_MUTED);
        }

        // Pause Menu Modal
        if self.is_paused {
            PostProcessor::apply_fade(fb, Color::BLACK, 0.6);

            let modal_w = 70;
            let modal_h = 38;
            let mx = (fb.width as i32 - modal_w) / 2;
            let my = (fb.height as i32 - modal_h) / 2;

            fb.draw_rect(mx, my, modal_w, modal_h, Palette::UI_PANEL_BG, true);
            fb.draw_rect(mx, my, modal_w, modal_h, Palette::UI_BORDER, false);

            TextRenderer::draw_text_centered(fb, my + 4, "PAUSED", FontSize::Standard5x7, Palette::TEXT_GOLD);

            let items = ["RESUME", "RETURN TO TITLE", "QUIT TO DESKTOP"];
            for (i, label) in items.iter().enumerate() {
                let py = my + 14 + i as i32 * 7;
                let (prefix, color) = if i == self.pause_selected {
                    ("> ", Palette::TEXT_GOLD)
                } else {
                    ("  ", Palette::TEXT_NORMAL)
                };
                let full = format!("{}{}", prefix, label);
                TextRenderer::draw_text_centered(fb, py, &full, FontSize::Tiny3x5, color);
            }
        }
    }
}
