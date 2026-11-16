use super::adventurer_dialogue::AdventurerDialogueScene;
use super::backpack::BackpackScene;
use super::blacksmith::BlacksmithScene;
use super::dialogue::DialogueScene;
use super::forest::ForestScene;
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
use crate::game::combat::{
    Bloodstain, CombatData, FloatingText, HazardPuddle, HeightBand, Hitbox, Projectile,
    SpatialHash, StatusEffectKind,
};
use crate::game::enemy::{Enemy, EnemyKind, EnemyState};
use crate::game::inventory::Inventory;
use crate::game::map::MapGrid;
use crate::game::npc::{Npc, NpcKind};
use crate::game::particles::ParticleSystem;
use crate::game::player::Player;
use crate::game::world::{ForestPartId, World};
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;

enum RenderableItem<'a> {
    Prop(&'a crate::game::world::PropInstance),
    Player,
    Bonfire,
    Npc(&'a Npc),
    Enemy(&'a Enemy),
}

pub struct LobbyScene {
    pub world: World,
    pub player: Player,
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

    // Phase 3 Combat & Resources
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    puddles: Vec<HazardPuddle>,
    hitboxes: Vec<Hitbox>,
    floating_texts: Vec<FloatingText>,
    bloodstain: Option<Bloodstain>,
    spatial_hash: SpatialHash,
    combat_data: CombatData,
    debug_hitboxes: bool,
    spawn_cycle_idx: usize,
    next_enemy_id: usize,
    bloodstain_dropped: bool,
    retrieved_souls_banner_timer: f32,
    retrieved_souls_amount: u32,
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
        let combat_data = CombatData::load();

        let mut particles = ParticleSystem::new(128);
        for _ in 0..25 {
            particles.spawn_ash(
                cx + rand::random::<f32>() * 120.0 - 60.0,
                cy + rand::random::<f32>() * 80.0 - 40.0,
            );
        }

        // Starting area is a peaceful Firelink-style hub - no initial enemies
        let enemies = Vec::new();

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

            enemies,
            projectiles: Vec::with_capacity(32),
            puddles: Vec::with_capacity(16),
            hitboxes: Vec::with_capacity(16),
            floating_texts: Vec::with_capacity(32),
            bloodstain: None,
            spatial_hash: SpatialHash::new(),
            combat_data,
            debug_hitboxes: false,
            spawn_cycle_idx: 0,
            next_enemy_id: 2,
            bloodstain_dropped: false,
            retrieved_souls_banner_timer: 0.0,
            retrieved_souls_amount: 0,
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
        let mouse_world = input.mouse_world_at(self.camera.x, self.camera.y);
        self.last_mouse_world = mouse_world;

        // Keep player equipped class in sync with save.ron
        let save = SaveData::load();
        if self.player.equipped_class != save.chosen_class {
            self.player.set_class(save.chosen_class);
        }

        // Toggle Debug overlay (F3)
        if input.just_pressed(VirtualKey::F3) {
            self.show_debug = !self.show_debug;
        }

        // Toggle Hitbox / Hurtbox / Height Band debug visualization (F6)
        if input.just_pressed(VirtualKey::F6) {
            self.debug_hitboxes = !self.debug_hitboxes;
            self.floating_texts.push(FloatingText::new(
                self.player.x,
                self.player.y - 32.0,
                if self.debug_hitboxes { "DEBUG: HITBOXES ON" } else { "DEBUG: HITBOXES OFF" },
                Palette::TEXT_GOLD,
                1.0,
            ));
        }

        // F7: Enemy Spawner at cursor! Cycles Sludge -> Skeleton -> Fly -> Worm
        if input.just_pressed(VirtualKey::F7) {
            let spawn_types = [
                EnemyKind::Sludge,
                EnemyKind::Skeleton,
                EnemyKind::Fly,
                EnemyKind::Worm,
            ];
            let kind = spawn_types[self.spawn_cycle_idx % spawn_types.len()];
            self.spawn_cycle_idx += 1;
            let id = self.next_enemy_id;
            self.next_enemy_id += 1;
            self.enemies.push(Enemy::new(id, kind, mouse_world.0, mouse_world.1, &self.combat_data));
            self.floating_texts.push(FloatingText::new(
                mouse_world.0,
                mouse_world.1 - 20.0,
                format!("SPAWNED: {}", kind.display_name()),
                Palette::BONFIRE_ORANGE,
                1.2,
            ));
            for _ in 0..8 {
                self.particles.spawn_spark(mouse_world.0, mouse_world.1, 0.0, -15.0);
            }
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
                "THE ASHEN COLOSSEUM - MAP".to_string(),
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

        // -------------------------------------------------------------
        // COMBAT & ENTITY SIMULATION
        // -------------------------------------------------------------

        // 1. Player Update
        self.player.update(
            dt,
            input,
            mouse_world,
            &self.world,
            &mut self.projectiles,
            &mut self.puddles,
            &mut self.hitboxes,
            &mut self.floating_texts,
            &mut self.particles,
            &self.combat_data,
        );

        // 2. Mage Corpse Consumption (Hold E near defeated corpse for 0.8s)
        if self.player.equipped_class == Some(ClassKind::Mage) && input.is_held(VirtualKey::E) && !self.player.is_dead {
            let mut nearest_corpse: Option<&mut Enemy> = None;
            let mut nearest_dist = 36.0;
            for e in &mut self.enemies {
                if e.state == EnemyState::Dead && !e.is_corpse_consumed {
                    let d = ((e.x - self.player.x).powi(2) + (e.y - self.player.y).powi(2)).sqrt();
                    if d < nearest_dist {
                        nearest_dist = d;
                        nearest_corpse = Some(e);
                    }
                }
            }

            if let Some(corpse) = nearest_corpse {
                self.player.corpse_channel_timer += dt;
                // Purple siphon particle stream
                self.particles.spawn_spark(
                    corpse.x + (rand::random::<f32>() * 6.0 - 3.0),
                    corpse.y - 4.0,
                    (self.player.x - corpse.x) * 2.0,
                    (self.player.y - 12.0 - corpse.y) * 2.0,
                );
                if self.player.corpse_channel_timer >= self.combat_data.player.corpse_channel_time {
                    corpse.is_corpse_consumed = true;
                    self.player.corpse_channel_timer = 0.0;
                    self.player.mana = (self.player.mana + self.combat_data.player.corpse_mana_restore).min(self.player.max_mana);
                    self.floating_texts.push(FloatingText::new(
                        self.player.x,
                        self.player.y - 32.0,
                        "+40 MANA (SIPHONED)",
                        Palette::MAGE_CYAN,
                        1.2,
                    ));
                    for _ in 0..12 {
                        self.particles.spawn_spark(self.player.x, self.player.y - 12.0, 0.0, -20.0);
                    }
                }
            } else {
                self.player.corpse_channel_timer = 0.0;
            }
        } else {
            self.player.corpse_channel_timer = 0.0;
        }

        // 3. Enemy Updates & Broadphase Spatial Hash Insertion
        self.spatial_hash.clear();
        for i in 0..self.enemies.len() {
            let p_pos = (self.player.x, self.player.y);
            self.enemies[i].update(
                dt,
                p_pos,
                &self.world,
                &mut self.projectiles,
                &mut self.puddles,
                &mut self.floating_texts,
                &mut self.particles,
                &self.combat_data,
            );

            // Contact damage to player
            if self.enemies[i].state != EnemyState::Dead && !self.player.is_dead {
                let e_pos = (self.enemies[i].x, self.enemies[i].y - self.enemies[i].z);
                let p_hb = self.player.get_hurtbox();
                let dx = e_pos.0 - p_hb.center.0;
                let dy = e_pos.1 - p_hb.center.1;
                let r_sum = self.enemies[i].get_hurtbox().radius + p_hb.radius;
                if dx * dx + dy * dy <= r_sum * r_sum {
                    self.player.take_damage(
                        self.enemies[i].contact_damage,
                        (self.enemies[i].x, self.enemies[i].y),
                        self.enemies[i].band,
                        None,
                        &mut self.floating_texts,
                        &mut self.particles,
                    );
                }
            }

            // Insert into spatial hash if alive
            if self.enemies[i].state != EnemyState::Dead {
                let hb = self.enemies[i].get_hurtbox();
                self.spatial_hash.insert(i, hb.center.0, hb.center.1, hb.radius);
            }
        }

        // 4. Hitbox vs Hurtbox Collisions (Player Attacks)
        let mut candidates = Vec::with_capacity(16);
        for hitbox in &self.hitboxes {
            if hitbox.is_player {
                self.spatial_hash.query_radius(hitbox.origin.0, hitbox.origin.1, 70.0, &mut candidates);
                // Also ensure all nearby alive enemies are checked so zero hits are missed
                for i in 0..self.enemies.len() {
                    if !candidates.contains(&i) {
                        candidates.push(i);
                    }
                }
                for &target_id in &candidates {
                    if self.player.hit_enemies_this_attack.contains(&target_id) {
                        continue;
                    }
                    let enemy = &mut self.enemies[target_id];
                    if enemy.state == EnemyState::Dead {
                        continue;
                    }
                    // Height Band Check!
                    if !hitbox.band.hits(enemy.band) {
                        continue;
                    }
                    let hb = enemy.get_hurtbox();
                    if hitbox.shape.contains_point(hitbox.origin, hb.center, hb.radius) {
                        self.player.hit_enemies_this_attack.push(target_id);
                        let k_dir = if hitbox.knockback_dir.0.abs() > 0.001 || hitbox.knockback_dir.1.abs() > 0.001 {
                            hitbox.knockback_dir
                        } else {
                            let dx = hb.center.0 - hitbox.origin.0;
                            let dy = hb.center.1 - hitbox.origin.1;
                            let len = (dx * dx + dy * dy).sqrt().max(0.01);
                            (dx / len, dy / len)
                        };

                        let prev_hp = enemy.hp;
                        enemy.take_damage(
                            hitbox.damage,
                            (k_dir.0 * hitbox.knockback, k_dir.1 * hitbox.knockback),
                            hitbox.hit_stop,
                            hitbox.status,
                            &mut self.floating_texts,
                            &mut self.particles,
                        );

                        if prev_hp > 0 && enemy.hp <= 0 {
                            self.player.souls += enemy.souls;
                        }
                    }
                }
            }
        }
        self.hitboxes.clear();

        // 5. Projectiles Update & Collision
        let mut surviving_projectiles = Vec::with_capacity(self.projectiles.len());
        for mut proj in self.projectiles.drain(..) {
            let active = proj.update(dt);
            if !active {
                // Ground impact for lobbed acid vials
                if proj.is_lobbed {
                    self.puddles.push(HazardPuddle::new(
                        proj.x,
                        proj.y,
                        self.combat_data.alchemist.vial_puddle_radius,
                        self.combat_data.alchemist.vial_puddle_dps,
                        self.combat_data.alchemist.vial_puddle_duration,
                        true,
                    ));
                    self.floating_texts.push(FloatingText::new(
                        proj.x,
                        proj.y - 12.0,
                        "ACID PUDDLE",
                        Color::rgb(140, 220, 50),
                        0.8,
                    ));
                    for _ in 0..8 {
                        self.particles.spawn_spark(proj.x, proj.y, 0.0, -10.0);
                    }
                }
                continue;
            }

            let mut hit = false;
            if proj.is_player {
                for enemy in &mut self.enemies {
                    if enemy.state == EnemyState::Dead {
                        continue;
                    }
                    if !proj.band.hits(enemy.band) {
                        continue;
                    }
                    let hb = enemy.get_hurtbox();
                    let dx = proj.x - hb.center.0;
                    let dy = proj.y - hb.center.1;
                    if dx * dx + dy * dy <= (proj.radius + hb.radius).powi(2) {
                        let prev_hp = enemy.hp;
                        enemy.take_damage(
                            proj.damage,
                            (proj.vx * 0.08, proj.vy * 0.08),
                            2,
                            proj.status,
                            &mut self.floating_texts,
                            &mut self.particles,
                        );
                        if prev_hp > 0 && enemy.hp <= 0 {
                            self.player.souls += enemy.souls;
                        }
                        hit = true;
                        break;
                    }
                }
            } else if !self.player.is_dead {
                let p_hb = self.player.get_hurtbox();
                if proj.band.hits(p_hb.band) {
                    let dx = proj.x - p_hb.center.0;
                    let dy = proj.y - p_hb.center.1;
                    if dx * dx + dy * dy <= (proj.radius + p_hb.radius).powi(2) {
                        self.player.take_damage(
                            proj.damage,
                            (proj.x, proj.y),
                            proj.band,
                            proj.status,
                            &mut self.floating_texts,
                            &mut self.particles,
                        );
                        hit = true;
                    }
                }
            }

            if !hit {
                surviving_projectiles.push(proj);
            }
        }
        self.projectiles = surviving_projectiles;

        // 6. Hazard Puddles Update & Tick Damage
        let mut surviving_puddles = Vec::with_capacity(self.puddles.len());
        for mut puddle in self.puddles.drain(..) {
            if puddle.update(dt) {
                if puddle.tick_timer >= 0.5 {
                    puddle.tick_timer = 0.0;
                    let p_rad_sq = puddle.radius * puddle.radius;

                    // Tick against Low-band ground enemies
                    for enemy in &mut self.enemies {
                        if enemy.state == EnemyState::Dead {
                            continue;
                        }
                        if puddle.band.hits(enemy.band) {
                            let dx = enemy.x - puddle.x;
                            let dy = enemy.y - puddle.y;
                            if dx * dx + dy * dy <= p_rad_sq {
                                let prev_hp = enemy.hp;
                                enemy.take_damage(
                                    puddle.dps.round() as i32,
                                    (0.0, 0.0),
                                    1,
                                    if puddle.is_acid { Some(StatusEffectKind::Acid) } else { Some(StatusEffectKind::Slow) },
                                    &mut self.floating_texts,
                                    &mut self.particles,
                                );
                                if prev_hp > 0 && enemy.hp <= 0 {
                                    self.player.souls += enemy.souls;
                                }
                            }
                        }
                    }

                    // Tick against player if standing in puddle
                    if !self.player.is_dead && puddle.is_acid {
                        let dx = self.player.x - puddle.x;
                        let dy = self.player.y - puddle.y;
                        if dx * dx + dy * dy <= p_rad_sq {
                            self.player.take_damage(
                                1,
                                (puddle.x, puddle.y),
                                HeightBand::Low,
                                Some(StatusEffectKind::Acid),
                                &mut self.floating_texts,
                                &mut self.particles,
                            );
                        }
                    }
                }
                surviving_puddles.push(puddle);
            }
        }
        self.puddles = surviving_puddles;

        // 7. Bloodstain & Souls Death Loop
        if self.player.is_dead {
            if !self.bloodstain_dropped {
                self.bloodstain_dropped = true;
                if self.player.souls > 0 {
                    self.bloodstain = Some(Bloodstain::new(self.player.x, self.player.y, self.player.souls));
                    self.player.souls = 0;
                }
            }
            if self.player.death_timer >= 2.5 {
                // Respawn at bonfire
                self.player.respawn_at(self.bonfire.x, self.bonfire.y);
                self.bloodstain_dropped = false;
                self.floating_texts.push(FloatingText::new(
                    self.player.x,
                    self.player.y - 28.0,
                    "RESTORED AT BONFIRE",
                    Palette::BONFIRE_ORANGE,
                    1.5,
                ));
            }
        } else if let Some(stain) = &self.bloodstain {
            // Retrieve bloodstain
            let dx = self.player.x - stain.x;
            let dy = self.player.y - stain.y;
            if dx * dx + dy * dy <= 24.0 * 24.0 {
                let retrieved = stain.souls;
                self.player.souls += retrieved;
                self.retrieved_souls_amount = retrieved;
                self.retrieved_souls_banner_timer = 2.5;
                self.bloodstain = None;
                self.bloodstain_dropped = false;
                self.floating_texts.push(FloatingText::new(
                    self.player.x,
                    self.player.y - 28.0,
                    format!("+{} SOULS RETRIEVED", retrieved),
                    Palette::TEXT_GOLD,
                    1.5,
                ));
                for _ in 0..16 {
                    self.particles.spawn_spark(self.player.x, self.player.y - 10.0, 0.0, -18.0);
                }
            }
        }

        if self.retrieved_souls_banner_timer > 0.0 {
            self.retrieved_souls_banner_timer -= dt;
        }

        // 8. Update Floating Texts
        self.floating_texts.retain_mut(|ft| ft.update(dt));

        // 9. Standard Lobby World Updates
        self.map_grid.reveal(self.player.x, self.player.y, 45.0);
        self.bonfire.update(dt, self.player.x, self.player.y, input, &mut self.particles);
        self.firekeeper.update(dt);
        self.blacksmith.update(dt);
        self.adventurer.update(dt);
        self.witch.update(dt);

        let mx = mouse_world.0;
        let my = mouse_world.1;
        let pressed_talk = input.just_pressed(VirtualKey::E);
        let clicked = input.mouse_left_just_pressed;

        if self.firekeeper.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.firekeeper.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(DialogueScene::new()));
        }

        if self.blacksmith.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.blacksmith.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(BlacksmithScene::new()));
        }

        if self.adventurer.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.adventurer.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(AdventurerDialogueScene::new()));
        }

        if self.witch.is_player_in_range(self.player.x, self.player.y)
            && (pressed_talk || (clicked && self.witch.is_mouse_hovered(mx, my)))
        {
            return SceneTransition::Push(Box::new(WitchShopScene::new()));
        }

        // Gate of Trials interaction (Transitions to Dilapidated Forest Area)
        let dist_to_gate = ((self.player.x - gate_pos.0).powi(2) + (self.player.y - gate_pos.1).powi(2)).sqrt();
        let clicked_gate = input.mouse_left_just_pressed
            && (mx - gate_pos.0).abs() < 24.0
            && (my - gate_pos.1).abs() < 24.0;

        if (dist_to_gate < 28.0 && input.just_pressed(VirtualKey::E)) || (dist_to_gate < 36.0 && clicked_gate) {
            return SceneTransition::Switch(Box::new(ForestScene::new(
                self.player.equipped_class,
                ForestPartId::Chapel,
                self.player.souls,
            )));
        }

        // Ambient particles
        if rand::random::<f32>() < 0.25 {
            let cx = self.camera.x + rand::random::<f32>() * self.camera.screen_w;
            let cy = self.camera.y + rand::random::<f32>() * self.camera.screen_h;
            self.particles.spawn_ash(cx, cy);
        }

        self.particles.update(dt);

        // Camera Follow with Trauma-based Screen Shake
        let (shake_ox, shake_oy) = self.player.screen_shake.get_offset(self.time);
        self.camera.follow(self.player.x + shake_ox, self.player.y + shake_oy);
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

        // 3. Ground Hazards & Bloodstain
        for puddle in &self.puddles {
            puddle.draw(fb, cam_x, cam_y, self.time);
        }
        if let Some(stain) = &self.bloodstain {
            stain.draw(fb, cam_x, cam_y, self.time);
        }

        // 4. Y-Sorting Render Queue
        let mut render_queue = Vec::with_capacity(self.world.props.len() + 6 + self.enemies.len());

        for prop in &self.world.props {
            render_queue.push((prop.base_y, RenderableItem::Prop(prop)));
        }
        render_queue.push((self.player.y.round() as i32, RenderableItem::Player));
        render_queue.push((self.bonfire.y.round() as i32, RenderableItem::Bonfire));
        render_queue.push((self.firekeeper.y.round() as i32, RenderableItem::Npc(&self.firekeeper)));
        render_queue.push((self.blacksmith.y.round() as i32, RenderableItem::Npc(&self.blacksmith)));
        render_queue.push((self.adventurer.y.round() as i32, RenderableItem::Npc(&self.adventurer)));
        render_queue.push((self.witch.y.round() as i32, RenderableItem::Npc(&self.witch)));

        for enemy in &self.enemies {
            render_queue.push((enemy.y.round() as i32, RenderableItem::Enemy(enemy)));
        }

        render_queue.sort_by_key(|&(y, _)| y);

        // Render Depth-Sorted Entities
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
                RenderableItem::Enemy(enemy) => {
                    enemy.draw(fb, bank, cam_x, cam_y);
                }
            }
        }

        // 5. In-flight Projectiles
        for proj in &self.projectiles {
            proj.draw(fb, cam_x, cam_y);
        }

        // 6. Particles (Embers, Ash, Sparks, Dust)
        self.particles.draw(fb, cam_x, cam_y);

        // 7. Dynamic Lighting Pass
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
        for enemy in &self.enemies {
            if enemy.state != EnemyState::Dead {
                shadow_casters.push(enemy.get_shadow_caster());
            }
        }

        let mut all_lights = Vec::with_capacity(6 + self.world.torch_lights.len());
        all_lights.push(bonfire_light);
        all_lights.push(firekeeper_light);
        all_lights.push(blacksmith_light);
        all_lights.push(adventurer_light);
        all_lights.push(witch_light);
        all_lights.push(player_aura);
        all_lights.extend_from_slice(&self.world.torch_lights);

        self.lighting.render_shadows(fb, &bonfire_light, &shadow_casters, cam_x, cam_y);
        self.lighting.apply_lighting(fb, &all_lights, cam_x, cam_y, self.time);

        // Volumetric Light Shafts
        self.world.draw_light_shafts(fb, cam_x, cam_y, self.time);

        // 8. Atmospheric Post-Processing
        PostProcessor::apply_drifting_fog(fb, self.time);
        PostProcessor::apply_film_grain(fb, (self.time * 60.0) as u64);
        PostProcessor::apply_vignette(fb, 0.42);

        // 9. Floating Combat Text
        for ft in &self.floating_texts {
            ft.draw(fb, cam_x, cam_y);
        }

        // 10. F6 Debug Overlay: Hitboxes, Hurtboxes, Height Bands
        if self.debug_hitboxes {
            // Player hurtbox (Green)
            let p_hb = self.player.get_hurtbox();
            let p_sx = (p_hb.center.0 - cam_x as f32).round() as i32;
            let p_sy = (p_hb.center.1 - cam_y as f32).round() as i32;
            fb.draw_circle(p_sx, p_sy, p_hb.radius.round() as i32, Color::rgb(40, 240, 60), false);
            TextRenderer::draw_text(fb, p_sx - 10, p_sy - 16, "[Player Mid]", FontSize::Tiny3x5, Color::rgb(40, 240, 60));

            // Player active attack swing (Red)
            if self.player.is_attacking {
                let reach = match self.player.equipped_class {
                    Some(ClassKind::Knight) => 38,
                    Some(ClassKind::Assassin) => 28,
                    Some(ClassKind::Hunter) => 38,
                    _ => 32,
                };
                fb.draw_circle(p_sx, p_sy, reach, Color::rgba(255, 60, 60, 180), false);
            }

            // Enemy hurtboxes & Height Bands
            for enemy in &self.enemies {
                if enemy.state != EnemyState::Dead {
                    let hb = enemy.get_hurtbox();
                    let sx = (hb.center.0 - cam_x as f32).round() as i32;
                    let sy = (hb.center.1 - cam_y as f32).round() as i32;
                    let band_col = match hb.band {
                        HeightBand::High => Color::rgb(80, 180, 255), // Blue High
                        HeightBand::Low => Color::rgb(255, 220, 50),   // Yellow Low
                        HeightBand::Mid => Color::rgb(60, 240, 100),  // Green Mid
                        HeightBand::All => Color::rgb(255, 80, 255),  // Magenta All
                    };
                    fb.draw_circle(sx, sy, hb.radius.round() as i32, band_col, false);
                    TextRenderer::draw_text(fb, sx - 12, sy - 14, hb.band.display_tag(), FontSize::Tiny3x5, band_col);
                }
            }

            // Puddles (Yellow Low)
            for puddle in &self.puddles {
                let sx = (puddle.x - cam_x as f32).round() as i32;
                let sy = (puddle.y - cam_y as f32).round() as i32;
                fb.draw_circle(sx, sy, puddle.radius.round() as i32, Color::rgb(255, 230, 40), false);
            }
        }

        // 11. NPC & World UI Prompts
        self.bonfire.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y);
        self.firekeeper.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.blacksmith.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.adventurer.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);
        self.witch.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y, self.last_mouse_world.0, self.last_mouse_world.1);

        // Mage Corpse Siphon Prompt
        if self.player.equipped_class == Some(ClassKind::Mage) && !self.player.is_dead {
            for e in &self.enemies {
                if e.state == EnemyState::Dead && !e.is_corpse_consumed {
                    let d = ((e.x - self.player.x).powi(2) + (e.y - self.player.y).powi(2)).sqrt();
                    if d < 35.0 {
                        let sx = (e.x - cam_x as f32).round() as i32;
                        let sy = (e.y - cam_y as f32).round() as i32;
                        let prompt = "[Hold E] SIPHON CORPSE";
                        let w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
                        TextRenderer::draw_text(fb, sx - w / 2, sy - 18, prompt, FontSize::Tiny3x5, Palette::MAGE_CYAN);
                        break;
                    }
                }
            }
        }

        // Gate of Trials Prompt
        let gate_pos = (
            self.world.width as f32 * 0.5,
            self.world.height as f32 * 0.5 - self.world.height as f32 * 0.38 + 24.0,
        );
        let dist_to_gate = ((self.player.x - gate_pos.0).powi(2) + (self.player.y - gate_pos.1).powi(2)).sqrt();
        let gate_scr_x = (gate_pos.0 - cam_x as f32).round() as i32;
        let gate_scr_y = (gate_pos.1 - cam_y as f32).round() as i32;

        if dist_to_gate < 30.0 {
            let prompt = "[Click / E] ENTER THE DILAPIDATED FOREST";
            let prompt_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
            let bx = gate_scr_x - prompt_w / 2;
            let by = gate_scr_y - 26;
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_PANEL_BG, true);
            fb.draw_rect(bx - 2, by - 1, prompt_w + 4, 7, Palette::UI_BORDER, false);
            TextRenderer::draw_text(fb, bx, by, prompt, FontSize::Tiny3x5, Palette::TEXT_GOLD);
        }

        // -------------------------------------------------------------
        // 12. UiCanvas Overlaid HUD Elements (Crisp Real Characters)
        // -------------------------------------------------------------

        // Area Banner "THE ASHEN COLOSSEUM"
        if self.area_banner_timer > 0.0 {
            ui.print_centered(2, "THE ASHEN COLOSSEUM", Palette::TEXT_BRIGHT, None);
            ui.print_centered(3, "PROVING GROUNDS OF THE FALLEN", Palette::TEXT_MUTED, None);
        }

        // Top-Left Combat HUD: Mantle Badge & Souls Counter
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
        let hud_bg = Some(Color::rgb(14, 16, 22));

        let top_line = format!("╺[MANTLE: {}]╸   ✦ {} SOULS", class_name, self.player.souls);
        ui.print(2, 1, &top_line, class_col, hud_bg);

        // Hearts Display (5 Hearts, 10 HP total, half-heart granularity)
        let mut hearts_str = String::with_capacity(32);
        for i in 0..5 {
            let heart_hp = (self.player.hp - i * 2).clamp(0, 2);
            if heart_hp == 2 {
                hearts_str.push_str("♥ ");
            } else if heart_hp == 1 {
                hearts_str.push_str("♡ ");
            } else {
                hearts_str.push_str("· ");
            }
        }
        let hearts_col = if self.player.i_frames_timer > 0.4 {
            Color::WHITE // Damage flash
        } else if self.player.hp <= 3 {
            Color::rgb(255, 60, 60)
        } else {
            Color::rgb(235, 75, 75)
        };
        let hp_line = format!("{} ({}/10 HP)", hearts_str, self.player.hp);
        ui.print(2, 2, &hp_line, hearts_col, hud_bg);

        // Mage-only Mana Bar
        let mut row_offset = 3;
        if self.player.equipped_class == Some(ClassKind::Mage) {
            let mana_int = self.player.mana.round() as i32;
            let bar_len = 10;
            let fill = ((self.player.mana / self.player.max_mana) * bar_len as f32).round() as usize;
            let mut bar = String::with_capacity(16);
            for b in 0..bar_len {
                if b < fill {
                    bar.push('█');
                } else {
                    bar.push('░');
                }
            }
            let mana_str = format!("MANA [{}] {}/100", bar, mana_int);
            ui.print(2, row_offset, &mana_str, Palette::MAGE_CYAN, hud_bg);
            row_offset += 1;
        }

        // Status Effects Badges
        let mut status_str = String::new();
        if self.player.poison_timer > 0.0 {
            status_str.push_str(&format!("[☣ POISON {:.1}s] ", self.player.poison_timer));
        }
        if self.player.acid_timer > 0.0 {
            status_str.push_str(&format!("[⚡ ACID {:.1}s] ", self.player.acid_timer));
        }
        if !status_str.is_empty() {
            ui.print(2, row_offset, &status_str, Color::rgb(140, 240, 80), hud_bg);
            row_offset += 1;
        }

        // Class Ability HUD Move Hints & High-Band Indicators
        let move_hint = match self.player.equipped_class {
            Some(ClassKind::Knight) => "[LMB] 3-Hit Combo [Low/Mid/↑High]   [RMB] Frontal Shield Block",
            Some(ClassKind::Mage) => "[LMB] Ember Bolt [Mid]   [RMB] Cinder Nova [*All]   [E] Siphon Corpse",
            Some(ClassKind::Assassin) => "[LMB] Dagger Flurry [Mid]   [RMB] Shadowstep (3x Backstab)",
            Some(ClassKind::Hunter) => "[LMB] Twin Blade Sweep [Mid]   [RMB] Pistol Flintlock [↑High]",
            Some(ClassKind::Alchemist) => "[LMB] Acid Vial Lob [↓Low]   [RMB] Eye Rite Slowing Field",
            None | Some(ClassKind::Wanderer) => "[LMB] Strike [Mid]   [RMB] Kick [Low]",
        };
        ui.print(2, row_offset, move_hint, Palette::TEXT_GOLD, hud_bg);

        // Retrieved Souls Banner
        if self.retrieved_souls_banner_timer > 0.0 {
            let msg = format!("✦ +{} SOULS RETRIEVED ✦", self.retrieved_souls_amount);
            ui.print_centered(ui.rows / 2 - 2, &msg, Palette::TEXT_GOLD, Some(Color::rgb(14, 16, 22)));
        }

        // "YOU DIED" Death Screen
        if self.player.is_dead {
            let banner_r = ui.rows / 2;
            ui.print_centered(banner_r - 1, "═════════════════════════════════════════", Color::rgb(180, 20, 20), Some(Color::BLACK));
            ui.print_centered(banner_r, "           Y O U   D I E D           ", Color::rgb(255, 40, 40), Some(Color::BLACK));
            ui.print_centered(banner_r + 1, "═════════════════════════════════════════", Color::rgb(180, 20, 20), Some(Color::BLACK));
            ui.print_centered(banner_r + 3, "Returning to bonfire...", Palette::TEXT_MUTED, None);
        }

        // Controls Hint Box (Bottom-Left 2-cell margin)
        if self.show_controls || self.controls_hint_timer > 0.0 {
            let base_r = ui.rows.saturating_sub(4);
            ui.print(2, base_r, "[WASD] Move  [Mouse] Aim  [LMB] Attack  [RMB] Ability  [Space] Roll", Palette::TEXT_GOLD, hud_bg);
            ui.print(2, base_r + 1, "[E] Interact/Siphon  [F6] Hitbox Debug  [F7] Spawn Enemy (Cycle)", Palette::TEXT_NORMAL, hud_bg);
            ui.print(2, base_r + 2, "[M] Map  [B] Backpack  [K] Spells  [F4] Gallery  [Esc] Pause", Palette::TEXT_MUTED, hud_bg);
        }

        // Debug Overlay (F3)
        if self.show_debug {
            let px = self.player.x;
            let py = self.player.y;
            ui.print(2, 6, &format!("POS: ({:.1}, {:.1}) | DIR: {:?} | HP: {}", px, py, self.player.dir4, self.player.hp), Palette::TEXT_GOLD, hud_bg);
            ui.print(2, 7, &format!("ENEMIES: {} | PROJECTILES: {} | PUDDLES: {}", self.enemies.len(), self.projectiles.len(), self.puddles.len()), Palette::TEXT_NORMAL, hud_bg);
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
