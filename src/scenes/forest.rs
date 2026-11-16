use super::backpack::BackpackScene;
use super::map::MapScene;
use super::spellbook::SpellbookScene;
use super::title::TitleScene;
use super::{Scene, SceneTransition};
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::LightingEngine;
use crate::engine::postprocess::PostProcessor;
use crate::game::bonfire::Bonfire;
use crate::game::camera::Camera;
use crate::game::classes::{ClassKind, SaveData};
use crate::game::combat::{
    Bloodstain, CombatData, FloatingText, HazardPuddle, HeightBand, Hitbox,
    SpatialHash, StatusEffectKind,
};
use crate::game::enemy::{Enemy, EnemyKind, EnemyState};
use crate::game::inventory::Inventory;
use crate::game::map::MapGrid;
use crate::game::particles::ParticleSystem;
use crate::game::player::Player;
use crate::game::world::{
    ForestAreaConfig, ForestPartConfig, ForestPartId, PropType, World,
};
use crate::ui::canvas::PanelStyle;
use crate::ui::layout::Rect;
use crate::ui::UiCanvas;
use rand::RngExt;

pub struct ForestScene {
    area_config: ForestAreaConfig,
    current_part_id: ForestPartId,
    world: World,
    player: Player,
    camera: Camera,
    bonfire: Option<Bonfire>,
    map_grid: MapGrid,
    inventory: Inventory,
    particles: ParticleSystem,
    lighting: LightingEngine,
    time: f32,

    // Area & UI Banners
    area_banner_timer: f32,
    controls_hint_timer: f32,
    show_controls: bool,
    show_debug: bool,
    is_paused: bool,
    pause_selected: usize,
    last_mouse_world: (f32, f32),

    // Combat & Resources
    enemies: Vec<Enemy>,
    projectiles: Vec<crate::game::combat::Projectile>,
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

    // Fog Wall Traversal
    traversal_timer: f32,
    traversal_target: Option<ForestPartId>,
    traversal_spawn: (f32, f32),
    traversal_prompt_timer: f32,
    traversal_prompt_text: String,
}

impl ForestScene {
    pub fn new(chosen_class: Option<ClassKind>, start_part: ForestPartId, carried_souls: u32) -> Self {
        let area_config = ForestAreaConfig::load();
        let part_cfg = area_config
            .get_part(start_part)
            .cloned()
            .unwrap_or_else(|| area_config.parts[0].clone());

        let world = World::build_forest_part(&part_cfg);
        let mut player = Player::new(part_cfg.player_spawn.0, part_cfg.player_spawn.1);
        player.set_class(chosen_class);
        player.souls = carried_souls;

        let mut camera = Camera::new(world.width as f32, world.height as f32, 120.0, 80.0);
        camera.follow(player.x, player.y);
        camera.x = camera.target_x;
        camera.y = camera.target_y;

        let map_grid = MapGrid::new(world.width, world.height);
        let inventory = Inventory::new();
        let combat_data = CombatData::load();

        let bonfire = part_cfg
            .bonfire
            .as_ref()
            .map(|b| Bonfire::new(b.x, b.y));

        let mut particles = ParticleSystem::new(128);
        Self::seed_ambient_particles(&mut particles, part_cfg.id, world.width, world.height);

        // Spawn enemies for starting part
        let mut enemies = Vec::with_capacity(part_cfg.enemy_spawns.len() + 8);
        for (i, sp) in part_cfg.enemy_spawns.iter().enumerate() {
            enemies.push(Enemy::new(i + 1, sp.kind, sp.x, sp.y, &combat_data));
        }

        Self {
            area_config,
            current_part_id: start_part,
            world,
            player,
            camera,
            bonfire,
            map_grid,
            inventory,
            particles,
            lighting: LightingEngine::new(),
            time: 0.0,
            area_banner_timer: 5.0,
            controls_hint_timer: 8.0,
            show_controls: true,
            show_debug: false,
            is_paused: false,
            pause_selected: 0,
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
            next_enemy_id: 20,
            bloodstain_dropped: false,
            retrieved_souls_banner_timer: 0.0,
            retrieved_souls_amount: 0,

            traversal_timer: 0.0,
            traversal_target: None,
            traversal_spawn: (0.0, 0.0),
            traversal_prompt_timer: 0.0,
            traversal_prompt_text: String::new(),
        }
    }

    fn seed_ambient_particles(
        particles: &mut ParticleSystem,
        part_id: ForestPartId,
        w: usize,
        h: usize,
    ) {
        let mut rng = rand::rng();
        match part_id {
            ForestPartId::Chapel => {
                for _ in 0..20 {
                    let rx = rng.random_range(20.0..(w as f32 - 20.0));
                    let ry = rng.random_range(20.0..(h as f32 - 20.0));
                    particles.spawn_ash(rx, ry);
                }
            }
            ForestPartId::AutumnWoods => {
                for _ in 0..25 {
                    let rx = rng.random_range(15.0..(w as f32 - 15.0));
                    let ry = rng.random_range(15.0..(h as f32 - 15.0));
                    particles.spawn_leaf(rx, ry);
                }
            }
            ForestPartId::DeepForest => {
                for _ in 0..25 {
                    let rx = rng.random_range(20.0..(w as f32 - 20.0));
                    let ry = rng.random_range(20.0..(h as f32 - 20.0));
                    particles.spawn_spore(rx, ry);
                }
            }
            ForestPartId::CastleKeep => {
                for _ in 0..20 {
                    let rx = rng.random_range(30.0..(w as f32 - 30.0));
                    let ry = rng.random_range(30.0..(h as f32 - 30.0));
                    particles.spawn_ember(rx, ry, 0.0, -8.0, 1.5);
                }
            }
        }
    }

    /// Transitions player into another part of the forest/castle via fog wall traversal.
    pub fn switch_to_part(&mut self, next_part_id: ForestPartId, spawn_pos: (f32, f32)) {
        self.current_part_id = next_part_id;
        let part_cfg = self
            .area_config
            .get_part(next_part_id)
            .cloned()
            .unwrap_or_else(|| self.area_config.parts[0].clone());

        self.world = World::build_forest_part(&part_cfg);
        self.camera = Camera::new(self.world.width as f32, self.world.height as f32, 120.0, 80.0);

        self.player.x = spawn_pos.0;
        self.player.y = spawn_pos.1;
        self.player.vx = 0.0;
        self.player.vy = 0.0;
        self.camera.follow(self.player.x, self.player.y);
        self.camera.x = self.camera.target_x;
        self.camera.y = self.camera.target_y;

        self.map_grid = MapGrid::new(self.world.width, self.world.height);

        self.bonfire = part_cfg
            .bonfire
            .as_ref()
            .map(|b| Bonfire::new(b.x, b.y));

        // Clear transient combat elements and spawn enemies for new part
        self.projectiles.clear();
        self.puddles.clear();
        self.hitboxes.clear();
        self.particles = ParticleSystem::new(128);
        Self::seed_ambient_particles(&mut self.particles, next_part_id, self.world.width, self.world.height);

        self.enemies.clear();
        for (i, sp) in part_cfg.enemy_spawns.iter().enumerate() {
            self.enemies.push(Enemy::new(
                self.next_enemy_id + i,
                sp.kind,
                sp.x,
                sp.y,
                &self.combat_data,
            ));
        }
        self.next_enemy_id += part_cfg.enemy_spawns.len() + 10;

        self.area_banner_timer = 5.0;
    }

    fn respawn_area_enemies(&mut self) {
        if let Some(part_cfg) = self.area_config.get_part(self.current_part_id) {
            self.enemies.clear();
            for (i, sp) in part_cfg.enemy_spawns.iter().enumerate() {
                self.enemies.push(Enemy::new(
                    self.next_enemy_id + i,
                    sp.kind,
                    sp.x,
                    sp.y,
                    &self.combat_data,
                ));
            }
            self.next_enemy_id += part_cfg.enemy_spawns.len() + 10;
        }
    }
}

impl Scene for ForestScene {
    fn on_resize(&mut self, w: usize, h: usize) {
        self.camera.resize(w as f32, h as f32);
    }

    fn update(&mut self, dt: f32, input: &InputState, _bank: &SpriteBank) -> SceneTransition {
        self.time += dt;

        // Pause Menu Toggle (ESC)
        if input.just_pressed(VirtualKey::Escape) {
            self.is_paused = !self.is_paused;
        }

        if self.is_paused {
            if input.just_pressed(VirtualKey::W) || input.just_pressed(VirtualKey::Up) {
                self.pause_selected = self.pause_selected.saturating_sub(1);
            }
            if input.just_pressed(VirtualKey::S) || input.just_pressed(VirtualKey::Down) {
                self.pause_selected = (self.pause_selected + 1).min(3);
            }
            if input.just_pressed(VirtualKey::Enter) || input.just_pressed(VirtualKey::Space) {
                match self.pause_selected {
                    0 => self.is_paused = false,
                    1 => self.show_controls = !self.show_controls,
                    2 => {
                        // Return to Ashen Colosseum hub
                        let _save = SaveData::load();
                        return SceneTransition::Switch(Box::new(super::lobby::LobbyScene::new(
                            World::build(240, 160),
                        )));
                    }
                    3 => return SceneTransition::Switch(Box::new(TitleScene::new())),
                    _ => {}
                }
            }
            return SceneTransition::None;
        }

        // Hotkeys for Inventory (B), Spells (K), Map (M)
        if input.just_pressed(VirtualKey::B) {
            return SceneTransition::Push(Box::new(BackpackScene::new(self.inventory.clone())));
        }

        if input.just_pressed(VirtualKey::K) {
            return SceneTransition::Push(Box::new(SpellbookScene::new()));
        }

        // Open Map (M)
        if input.just_pressed(VirtualKey::M) {
            let part_cfg = self
                .area_config
                .get_part(self.current_part_id)
                .cloned()
                .unwrap_or_else(|| self.area_config.parts[0].clone());

            let b_pos = self.bonfire.as_ref().map(|b| (b.x, b.y)).unwrap_or((0.0, 0.0));
            let fog_pos = part_cfg.fog_wall.as_ref().map(|f| (f.x, f.y)).unwrap_or((0.0, 0.0));

            return SceneTransition::Push(Box::new(MapScene::new(
                self.map_grid.clone(),
                (self.player.x, self.player.y),
                self.player.facing_angle,
                b_pos,
                fog_pos,
                vec![],
                (self.world.width as f32, self.world.height as f32),
            )));
        }

        // Debug hotkeys
        if input.just_pressed(VirtualKey::F1) {
            self.show_controls = !self.show_controls;
        }
        if input.just_pressed(VirtualKey::F3) {
            self.show_debug = !self.show_debug;
        }
        if input.just_pressed(VirtualKey::F6) {
            self.debug_hitboxes = !self.debug_hitboxes;
        }
        if input.just_pressed(VirtualKey::F7) {
            let (mx, my) = input.mouse_world_at(self.camera.x, self.camera.y);
            let kinds = [EnemyKind::Sludge, EnemyKind::Skeleton, EnemyKind::Fly, EnemyKind::Worm];
            let kind = kinds[self.spawn_cycle_idx % kinds.len()];
            self.spawn_cycle_idx += 1;
            self.enemies.push(Enemy::new(self.next_enemy_id, kind, mx, my, &self.combat_data));
            self.next_enemy_id += 1;
            self.floating_texts.push(FloatingText::new(
                mx,
                my - 16.0,
                format!("SPAWNED {}", kind.display_name().to_uppercase()),
                Palette::TEXT_GOLD,
                1.5,
            ));
        }

        // Active Fog Traversal Sequence
        if self.traversal_timer > 0.0 {
            self.traversal_timer -= dt;
            if self.traversal_timer <= 0.3 && self.traversal_target.is_some() {
                let target = self.traversal_target.take().unwrap();
                let spawn = self.traversal_spawn;
                self.switch_to_part(target, spawn);
            }
            return SceneTransition::None;
        }

        let mouse_world = input.mouse_world_at(self.camera.x, self.camera.y);
        self.last_mouse_world = mouse_world;

        // 1. Update Player
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

        // 2. Mage Corpse Siphon (synergy with rotting corpses in Chapel + mob corpses)
        if self.player.equipped_class == Some(ClassKind::Mage) && !self.player.is_dead {
            let mut nearest_corpse: Option<(f32, f32, bool)> = None;
            let mut best_dist_sq = 28.0 * 28.0;

            for enemy in &mut self.enemies {
                if enemy.state == EnemyState::Dead && !enemy.is_corpse_consumed {
                    let dx = self.player.x - enemy.x;
                    let dy = self.player.y - enemy.y;
                    let d2 = dx * dx + dy * dy;
                    if d2 < best_dist_sq {
                        best_dist_sq = d2;
                        nearest_corpse = Some((enemy.x, enemy.y, true));
                    }
                }
            }

            for prop in &self.world.props {
                if prop.sprite_index == PropType::RottingCorpse {
                    let dx = self.player.x - prop.x as f32;
                    let dy = self.player.y - prop.y as f32;
                    let d2 = dx * dx + dy * dy;
                    if d2 < best_dist_sq {
                        best_dist_sq = d2;
                        nearest_corpse = Some((prop.x as f32, prop.y as f32, false));
                    }
                }
            }

            if let Some((cx, cy, is_enemy)) = nearest_corpse {
                if input.is_held(VirtualKey::E) {
                    self.player.corpse_channel_timer += dt;
                    for _ in 0..2 {
                        self.particles.spawn_spark(cx, cy, 0.0, -10.0);
                    }
                    if self.player.corpse_channel_timer >= 0.8 {
                        self.player.mana = (self.player.mana + 40.0).min(self.player.max_mana);
                        self.player.corpse_channel_timer = 0.0;
                        if let Some(target) = self.enemies.iter_mut().find(|e| is_enemy && (e.x - cx).abs() < 1.0 && (e.y - cy).abs() < 1.0) {
                            target.is_corpse_consumed = true;
                        }
                        self.floating_texts.push(FloatingText::new(
                            self.player.x,
                            self.player.y - 20.0,
                            "+40 MANA (SIPHONED)",
                            Palette::MAGE_PURPLE,
                            1.4,
                        ));
                    }
                } else {
                    self.player.corpse_channel_timer = 0.0;
                }
            } else {
                self.player.corpse_channel_timer = 0.0;
            }
        }

        // 3. Update Enemies & Spatial Hash Broadphase
        self.spatial_hash.clear();
        let p_pos = (self.player.x, self.player.y);
        for i in 0..self.enemies.len() {
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
                let contact_dist_sq = (p_hb.radius + 6.0).powi(2);
                if dx * dx + dy * dy <= contact_dist_sq {
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

            if self.enemies[i].state != EnemyState::Dead {
                self.spatial_hash.insert(i, self.enemies[i].x, self.enemies[i].y, 16.0);
            }
        }

        // 4. Melee Attack Hitbox Collision
        let mut candidates = Vec::with_capacity(16);
        for hitbox in &self.hitboxes {
            candidates.clear();
            self.spatial_hash.query_radius(hitbox.origin.0, hitbox.origin.1, 70.0, &mut candidates);
            for i in 0..self.enemies.len() {
                if !candidates.contains(&i) {
                    candidates.push(i);
                }
            }
            for &target_id in &candidates {
                if target_id < self.enemies.len() {
                    let enemy = &mut self.enemies[target_id];
                    if enemy.state == EnemyState::Dead {
                        continue;
                    }
                    if self.player.hit_enemies_this_attack.contains(&target_id) {
                        continue;
                    }
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
                            self.floating_texts.push(FloatingText::new(
                                enemy.x,
                                enemy.y - 18.0,
                                format!("+{} SOULS", enemy.souls),
                                Palette::TEXT_GOLD,
                                1.5,
                            ));
                        }

                        self.player.screen_shake.add(0.18);
                        for _ in 0..8 {
                            self.particles.spawn_spark(hb.center.0, hb.center.1, 0.0, -12.0);
                        }
                    }
                }
            }
        }
        self.hitboxes.clear();

        // 5. Update Projectiles
        let mut surviving_projectiles = Vec::with_capacity(self.projectiles.len());
        for mut proj in self.projectiles.drain(..) {
            let active = proj.update(dt);
            if !active {
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

            if proj.is_player {
                let mut hit = false;
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
                            self.floating_texts.push(FloatingText::new(
                                enemy.x,
                                enemy.y - 18.0,
                                format!("+{} SOULS", enemy.souls),
                                Palette::TEXT_GOLD,
                                1.5,
                            ));
                        }
                        hit = true;
                        for _ in 0..8 {
                            self.particles.spawn_spark(proj.x, proj.y, 0.0, -12.0);
                        }
                        break;
                    }
                }
                if !hit {
                    surviving_projectiles.push(proj);
                }
            } else if !self.player.is_dead {
                let p_hb = self.player.get_hurtbox();
                let dx = proj.x - p_hb.center.0;
                let dy = proj.y - p_hb.center.1;
                if dx * dx + dy * dy <= (proj.radius + p_hb.radius).powi(2) {
                    self.player.take_damage(
                        proj.damage,
                        (proj.x - proj.vx * 0.1, proj.y - proj.vy * 0.1),
                        proj.band,
                        proj.status,
                        &mut self.floating_texts,
                        &mut self.particles,
                    );
                } else {
                    surviving_projectiles.push(proj);
                }
            }
        }
        self.projectiles = surviving_projectiles;

        // 6. Hazard Puddles
        let mut surviving_puddles = Vec::with_capacity(self.puddles.len());
        for mut puddle in self.puddles.drain(..) {
            let active = puddle.update(dt);
            if active {
                let p_rad_sq = puddle.radius * puddle.radius;
                if puddle.tick_timer <= 0.0 {
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

        // 7. Souls Death & Bloodstain Loop
        if self.player.is_dead {
            if !self.bloodstain_dropped {
                self.bloodstain_dropped = true;
                if self.player.souls > 0 {
                    self.bloodstain = Some(Bloodstain::new(self.player.x, self.player.y, self.player.souls));
                    self.player.souls = 0;
                }
            }
            if self.player.death_timer >= 2.5 {
                let respawn_pos = self.bonfire.as_ref().map(|b| (b.x, b.y)).unwrap_or((self.world.width as f32 * 0.5, self.world.height as f32 * 0.7));
                self.player.respawn_at(respawn_pos.0, respawn_pos.1);
                self.bloodstain_dropped = false;
                self.respawn_area_enemies();
                self.floating_texts.push(FloatingText::new(
                    self.player.x,
                    self.player.y - 28.0,
                    "RESTORED AT BONFIRE",
                    Palette::BONFIRE_ORANGE,
                    1.5,
                ));
            }
        } else if let Some(stain) = &self.bloodstain {
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
            }
        }

        if self.retrieved_souls_banner_timer > 0.0 {
            self.retrieved_souls_banner_timer -= dt;
        }

        // 8. Update Floating Texts & Map Reveal
        self.floating_texts.retain_mut(|ft| ft.update(dt));
        self.map_grid.reveal(self.player.x, self.player.y, 45.0);

        // 9. Bonfire Update & Resting
        if let Some(bonfire) = &mut self.bonfire {
            let was_resting = bonfire.is_resting;
            bonfire.update(dt, self.player.x, self.player.y, input, &mut self.particles);
            if !was_resting && bonfire.is_resting {
                self.player.hp = self.player.max_hp;
                self.player.mana = self.player.max_mana;
                self.player.poison_timer = 0.0;
                self.player.acid_timer = 0.0;
                self.respawn_area_enemies();
                self.floating_texts.push(FloatingText::new(
                    self.player.x,
                    self.player.y - 28.0,
                    "RESTED AT BONFIRE - FOES RESPAWNED",
                    Palette::BONFIRE_YELLOW,
                    2.0,
                ));
            }
        }

        // 10. Fog Wall & Gate Proximity Checks
        let part_cfg = self
            .area_config
            .get_part(self.current_part_id)
            .cloned()
            .unwrap_or_else(|| self.area_config.parts[0].clone());

        let mx = mouse_world.0;
        let my = mouse_world.1;
        let clicked = input.mouse_left_just_pressed;
        let pressed_e = input.just_pressed(VirtualKey::E);

        // Northern Fog Wall (advance to next part)
        if let Some(fog) = &part_cfg.fog_wall {
            let dx = (self.player.x - fog.x).abs();
            let dy = (self.player.y - fog.y).abs();
            let in_range = dx < (fog.width * 0.5 + 10.0) && dy < 22.0;
            let mouse_hover = (mx - fog.x).abs() < (fog.width * 0.5 + 8.0) && (my - fog.y).abs() < 18.0;

            if in_range {
                self.traversal_prompt_timer = 0.2;
                self.traversal_prompt_text = format!("[Click / E] {}", fog.prompt);

                if (pressed_e || (clicked && mouse_hover)) && fog.target_part != self.current_part_id {
                    self.traversal_timer = 0.6;
                    self.traversal_target = Some(fog.target_part);
                    self.traversal_spawn = fog.target_spawn;
                }
            }
        }

        // Southern Return Fog Wall (backtrack to previous part)
        if let Some(return_fog) = &part_cfg.return_fog_wall {
            let dx = (self.player.x - return_fog.x).abs();
            let dy = (self.player.y - return_fog.y).abs();
            let in_range = dx < (return_fog.width * 0.5 + 10.0) && dy < 22.0;
            let mouse_hover = (mx - return_fog.x).abs() < (return_fog.width * 0.5 + 8.0) && (my - return_fog.y).abs() < 18.0;

            if in_range {
                self.traversal_prompt_timer = 0.2;
                self.traversal_prompt_text = format!("[Click / E] {}", return_fog.prompt);

                if pressed_e || (clicked && mouse_hover) {
                    self.traversal_timer = 0.6;
                    self.traversal_target = Some(return_fog.target_part);
                    self.traversal_spawn = return_fog.target_spawn;
                }
            }
        }

        // Return to Ashen Colosseum Gate (in Chapel)
        if let Some(col_gate) = part_cfg.colosseum_gate {
            let dx = (self.player.x - col_gate.0).abs();
            let dy = (self.player.y - col_gate.1).abs();
            let in_range = dx < 24.0 && dy < 20.0;
            let mouse_hover = (mx - col_gate.0).abs() < 22.0 && (my - col_gate.1).abs() < 18.0;

            if in_range {
                self.traversal_prompt_timer = 0.2;
                self.traversal_prompt_text = "[Click / E] Return to The Ashen Colosseum".to_string();

                if pressed_e || (clicked && mouse_hover) {
                    let mut lobby = super::lobby::LobbyScene::new(World::build(240, 160));
                    lobby.player.souls = self.player.souls;
                    lobby.player.set_class(self.player.equipped_class);
                    return SceneTransition::Switch(Box::new(lobby));
                }
            }
        }

        if self.traversal_prompt_timer > 0.0 {
            self.traversal_prompt_timer -= dt;
        }

        // Ambient particles spawn
        let mut rng = rand::rng();
        match self.current_part_id {
            ForestPartId::Chapel => {
                if rng.random_range(0..10) < 3 {
                    let rx = rng.random_range(20.0..(self.world.width as f32 - 20.0));
                    let ry = rng.random_range(20.0..(self.world.height as f32 - 20.0));
                    self.particles.spawn_ash(rx, ry);
                }
            }
            ForestPartId::AutumnWoods => {
                if rng.random_range(0..10) < 4 {
                    let rx = rng.random_range(10.0..(self.world.width as f32 - 10.0));
                    let ry = rng.random_range(0.0..(self.world.height as f32 * 0.4));
                    self.particles.spawn_leaf(rx, ry);
                }
            }
            ForestPartId::DeepForest => {
                if rng.random_range(0..10) < 4 {
                    let rx = rng.random_range(20.0..(self.world.width as f32 - 20.0));
                    let ry = rng.random_range(20.0..(self.world.height as f32 - 20.0));
                    self.particles.spawn_spore(rx, ry);
                }
            }
            ForestPartId::CastleKeep => {
                if rng.random_range(0..10) < 3 {
                    let rx = rng.random_range(40.0..(self.world.width as f32 - 40.0));
                    let ry = rng.random_range(40.0..(self.world.height as f32 - 40.0));
                    self.particles.spawn_ember(rx, ry, 0.0, -8.0, 1.4);
                }
            }
        }

        self.particles.update(dt);
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

        let part_cfg = self
            .area_config
            .get_part(self.current_part_id)
            .cloned()
            .unwrap_or_else(|| self.area_config.parts[0].clone());

        // 1. Draw Sky / Background & Ground
        self.world.draw_parallax_sky(fb, self.camera.x, self.camera.y);
        self.world.draw_ground(fb, cam_x, cam_y);

        // 2. Draw Bloodstain
        if let Some(stain) = &self.bloodstain {
            stain.draw(fb, cam_x, cam_y, self.time);
        }

        // 3. Draw Bonfire
        if let Some(bonfire) = &self.bonfire {
            bonfire.draw(fb, bank, cam_x, cam_y);
            bonfire.draw_ui(fb, cam_x, cam_y, self.player.x, self.player.y);
        }

        // 4. Draw Hazard Puddles
        for puddle in &self.puddles {
            puddle.draw(fb, cam_x, cam_y, self.time);
        }

        // 5. Y-Sorted Render Entities (Corpses, Props, Enemies, Player)
        enum RenderItem<'a> {
            Prop(&'a crate::game::world::PropInstance),
            Enemy(&'a Enemy),
            Player,
        }

        let mut render_queue = Vec::with_capacity(self.world.props.len() + self.enemies.len() + 1);
        for p in &self.world.props {
            render_queue.push((p.base_y, RenderItem::Prop(p)));
        }
        for e in &self.enemies {
            render_queue.push((e.y.round() as i32, RenderItem::Enemy(e)));
        }
        render_queue.push((self.player.y.round() as i32, RenderItem::Player));
        render_queue.sort_by_key(|k| k.0);

        let banner_frame = (self.time * 2.0) as usize;
        for (_, item) in render_queue {
            match item {
                RenderItem::Prop(p) => {
                    let sprite = self.world.get_prop_sprite(bank, p.sprite_index, banner_frame);
                    let sx = p.x - cam_x;
                    let sy = p.y - cam_y;
                    sprite.draw(fb, sx, sy, false);
                }
                RenderItem::Enemy(e) => {
                    e.draw(fb, bank, cam_x, cam_y);
                }
                RenderItem::Player => {
                    self.player.draw(fb, bank, cam_x, cam_y);
                }
            }
        }

        // 6. Draw Projectiles
        for proj in &self.projectiles {
            proj.draw(fb, cam_x, cam_y);
        }

        // 7. Draw Shimmering Fog Walls
        if let Some(fog) = &part_cfg.fog_wall {
            Self::draw_shimmering_fog_wall(fb, fog.x, fog.y, fog.width, cam_x, cam_y, self.time);
        }
        if let Some(ret_fog) = &part_cfg.return_fog_wall {
            Self::draw_shimmering_fog_wall(fb, ret_fog.x, ret_fog.y, ret_fog.width, cam_x, cam_y, self.time);
        }

        // 8. Draw Ambient Particles
        self.particles.draw(fb, cam_x, cam_y);

        // 9. Dynamic Lighting Pass
        let mut all_lights = Vec::with_capacity(4 + self.world.torch_lights.len());
        if let Some(bonfire) = &self.bonfire {
            all_lights.push(bonfire.get_light());
        }
        all_lights.push(crate::engine::lighting::PointLight {
            x: self.player.x,
            y: self.player.y - 8.0,
            radius: 38.0,
            intensity: 0.55,
            color: Color::rgb(180, 200, 235),
            flicker_amount: 0.03,
        });
        all_lights.extend_from_slice(&self.world.torch_lights);

        let mut shadow_casters = self.world.get_shadow_casters();
        shadow_casters.push(self.player.get_shadow_caster());
        for enemy in &self.enemies {
            if enemy.state != EnemyState::Dead {
                shadow_casters.push(enemy.get_shadow_caster());
            }
        }

        if let Some(bonfire) = &self.bonfire {
            self.lighting.render_shadows(fb, &bonfire.get_light(), &shadow_casters, cam_x, cam_y);
        }
        self.lighting.apply_lighting(fb, &all_lights, cam_x, cam_y, self.time);

        // 10. Fog Wall Traversal Whiteout Overlay
        if self.traversal_timer > 0.0 {
            let progress = 1.0 - (self.traversal_timer / 0.6);
            let alpha = if progress < 0.5 {
                (progress * 2.0 * 220.0) as u8
            } else {
                ((1.0 - progress) * 2.0 * 220.0) as u8
            };
            fb.draw_rect(0, 0, fb.width as i32, fb.height as i32, Color::rgba(215, 230, 255, alpha), true);
        }

        // 11. Atmospheric Post-Processing
        PostProcessor::apply_film_grain(fb, (self.time * 60.0) as u64);
        PostProcessor::apply_vignette(fb, 0.40);

        // 12. Floating Texts
        for ft in &self.floating_texts {
            ft.draw(fb, cam_x, cam_y);
        }

        // 13. UI Canvas Layer (Real Monospace Characters)
        self.render_ui(ui, &part_cfg);
    }
}

impl ForestScene {
    fn draw_shimmering_fog_wall(
        fb: &mut Framebuffer,
        fx: f32,
        fy: f32,
        width: f32,
        cam_x: i32,
        cam_y: i32,
        time: f32,
    ) {
        let scr_x = (fx - cam_x as f32).round() as i32;
        let scr_y = (fy - cam_y as f32).round() as i32;
        let half_w = (width * 0.5).round() as i32;

        let left_x = scr_x - half_w;
        let right_x = scr_x + half_w;
        let top_y = scr_y - 28;
        let bot_y = scr_y + 4;

        for y in top_y..=bot_y {
            if y < 0 || y >= fb.height as i32 {
                continue;
            }
            let dy_norm = (y - top_y) as f32 / (bot_y - top_y) as f32;
            for x in left_x..=right_x {
                if x < 0 || x >= fb.width as i32 {
                    continue;
                }
                let dx_norm = ((x - scr_x) as f32 / half_w as f32).abs();
                let edge_alpha = (1.0 - dx_norm * dx_norm).max(0.0);

                // Fluid undulating wave pattern
                let wave1 = ((y as f32 * 0.35 + time * 4.0).sin() * 2.0) as i32;
                let wave2 = ((x as f32 * 0.25 - time * 3.0).cos() * 1.5) as i32;
                let is_mist = (x + wave1 + wave2) % 4 <= 2;

                if is_mist {
                    let pulse = ((time * 3.0 + y as f32 * 0.2).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
                    let a = ((180.0 * edge_alpha * pulse) * (0.6 + dy_norm * 0.4)) as u8;

                    let mist_col = if (x + y) % 3 == 0 {
                        Color::rgba(225, 240, 255, a)
                    } else if (x + y) % 2 == 0 {
                        Color::rgba(140, 185, 225, a)
                    } else {
                        Color::rgba(70, 105, 145, a)
                    };
                    fb.set_pixel_blend(x, y, mist_col);
                }
            }
        }
    }

    fn render_ui(&self, ui: &mut UiCanvas, part_cfg: &ForestPartConfig) {
        let cols = ui.cols;
        let rows = ui.rows;
        let hud_bg = Some(Color::rgb(14, 16, 22));

        // Area Title Banner (Centred at Top)
        if self.area_banner_timer > 0.0 {
            let title = format!("- {} -", part_cfg.name.to_uppercase());
            let sub = &part_cfg.subtitle;
            let title_w = title.len();
            let sub_w = sub.len();

            let tx = (cols.saturating_sub(title_w)) / 2;
            let sx = (cols.saturating_sub(sub_w)) / 2;

            let alpha_col = if self.area_banner_timer > 4.0 {
                Palette::TEXT_GOLD
            } else if self.area_banner_timer > 1.0 {
                Palette::TEXT_BRIGHT
            } else {
                Palette::TEXT_MUTED
            };

            ui.print(tx, 2, &title, alpha_col, hud_bg);
            ui.print(sx, 3, sub, Palette::TEXT_MUTED, hud_bg);
        }

        // Souls Counter & Class Mantle (Top-Left)
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
            Color::WHITE
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
        }

        // Retrieved Souls Banner
        if self.retrieved_souls_banner_timer > 0.0 {
            let msg = format!("✦ +{} SOULS RETRIEVED ✦", self.retrieved_souls_amount);
            let mx = (cols.saturating_sub(msg.len())) / 2;
            ui.print(mx, 5, &msg, Palette::TEXT_GOLD, Some(Color::rgb(20, 18, 12)));
        }

        // "YOU DIED" Banner
        if self.player.is_dead {
            let death_str = "✦ YOU DIED ✦";
            let dx = (cols.saturating_sub(death_str.len())) / 2;
            let dy = rows / 2;
            ui.print(dx, dy, death_str, Color::rgb(255, 45, 45), Some(Color::rgb(20, 8, 8)));
        }

        // Traversal / Interaction Prompt
        if self.traversal_prompt_timer > 0.0 {
            let px = (cols.saturating_sub(self.traversal_prompt_text.len())) / 2;
            let py = rows.saturating_sub(5);
            ui.print(
                px,
                py,
                &self.traversal_prompt_text,
                Palette::TEXT_BRIGHT,
                Some(Palette::UI_PANEL_BG),
            );
        }

        // Bottom Controls Hint Bar
        if self.show_controls {
            let hint = "[WASD] Move  [Mouse] Aim  [LMB] Attack  [RMB] Skill  [Space] Roll  [E] Interact  [M] Map  [B] Pack  [K] Spells  [Esc] Pause";
            let cy = rows.saturating_sub(2);
            ui.print(2, cy, hint, Palette::TEXT_MUTED, None);
        }

        // Pause Menu Panel
        if self.is_paused {
            let pw = 42;
            let ph = 12;
            let px = (cols.saturating_sub(pw)) / 2;
            let py = (rows.saturating_sub(ph)) / 2;

            let panel_rect = Rect::new(px as i32, py as i32, pw as i32, ph as i32);
            ui.draw_panel(panel_rect, "PAUSED - ATROCITUS", PanelStyle::double());

            let opts = [
                "Resume Game",
                "Toggle Controls HUD",
                "Return to Ashen Colosseum",
                "Exit to Title Menu",
            ];
            for (i, opt) in opts.iter().enumerate() {
                let is_sel = i == self.pause_selected;
                let text = if is_sel {
                    format!(" > {} < ", opt)
                } else {
                    format!("   {}   ", opt)
                };
                let col = if is_sel {
                    Palette::TEXT_GOLD
                } else {
                    Palette::TEXT_MUTED
                };
                let ox = px + (pw.saturating_sub(text.len())) / 2;
                ui.print(ox, py + 3 + i * 2, &text, col, None);
            }
        }
    }
}
