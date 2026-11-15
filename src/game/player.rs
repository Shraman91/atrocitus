use super::classes::ClassKind;
use super::combat::{CombatData, HazardPuddle, HeightBand, Hitbox, HitboxShape, Hurtbox, Projectile, ScreenShake, StatusEffectKind, FloatingText};
use super::equipment::LayeredGear;
use super::particles::ParticleSystem;
use super::world::{Collider, World};
use crate::assets::palette::Palette;
use crate::assets::sprites::{Direction4, SpriteBank};
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::{InputState, VirtualKey};
use crate::engine::lighting::ShadowCaster;
use std::f32::consts::PI;

pub const PLAYER_WALK_SPEED: f32 = 56.0;
pub const PLAYER_RUN_SPEED: f32 = 86.0;
pub const ROLL_DURATION: f32 = 0.38;
pub const ROLL_COOLDOWN: f32 = 0.48;
pub const ROLL_DISTANCE: f32 = 68.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAttackKind {
    Primary(usize), // Combo step: 0, 1, 2
    Secondary,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub dir4: Direction4,
    pub flip_x: bool,
    pub facing_angle: f32, // Continuous angle for 8-direction hysteresis
    pub is_moving: bool,
    pub is_running: bool,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub dust_timer: f32,
    pub equipped_class: Option<ClassKind>,
    pub gear: LayeredGear,

    // Dodge Roll state machine
    pub is_rolling: bool,
    pub roll_timer: f32,
    pub roll_cooldown: f32,
    pub roll_dir: (f32, f32),
    pub roll_buffered: bool,
    pub roll_ghosts: Vec<(f32, f32, f32, Direction4, bool)>, // (x, y, life, dir, flip)

    // Combat Stats & Resources
    pub hp: i32,
    pub max_hp: i32,
    pub souls: u32,
    pub mana: f32,
    pub max_mana: f32,
    pub mana_regen_rate: f32,
    pub poison_timer: f32,
    pub poison_tick_timer: f32,
    pub acid_timer: f32,
    pub acid_tick_timer: f32,
    pub i_frames_timer: f32,
    pub hit_stop: u32,
    pub flinch_timer: f32,
    pub is_dead: bool,
    pub death_timer: f32,
    pub screen_shake: ScreenShake,

    // Attack State Machine
    pub is_attacking: bool,
    pub attack_kind: PlayerAttackKind,
    pub attack_timer: f32,
    pub attack_duration: f32,
    pub attack_hitbox_spawned: bool,
    pub attack_buffered: bool,
    pub hit_enemies_this_attack: Vec<usize>,
    pub combo_step: usize,
    pub combo_window_timer: f32,
    pub secondary_cooldown: f32,
    pub is_blocking: bool,
    pub backstab_buff_timer: f32,
    pub corpse_channel_timer: f32,
    pub corpse_channel_target: Option<usize>,
    pub is_charging_nova: bool,
    pub nova_charge_timer: f32,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            dir4: Direction4::South,
            flip_x: false,
            facing_angle: std::f32::consts::FRAC_PI_2, // facing South initially
            is_moving: false,
            is_running: false,
            anim_timer: 0.0,
            anim_frame: 0,
            dust_timer: 0.0,
            equipped_class: None,
            gear: LayeredGear::empty(),
            is_rolling: false,
            roll_timer: 0.0,
            roll_cooldown: 0.0,
            roll_dir: (0.0, 1.0),
            roll_buffered: false,
            roll_ghosts: Vec::with_capacity(8),

            hp: 10,
            max_hp: 10,
            souls: 100,
            mana: 100.0,
            max_mana: 100.0,
            mana_regen_rate: 4.0,
            poison_timer: 0.0,
            poison_tick_timer: 0.0,
            acid_timer: 0.0,
            acid_tick_timer: 0.0,
            i_frames_timer: 0.0,
            hit_stop: 0,
            flinch_timer: 0.0,
            is_dead: false,
            death_timer: 0.0,
            screen_shake: ScreenShake::new(),

            is_attacking: false,
            attack_kind: PlayerAttackKind::Primary(0),
            attack_timer: 0.0,
            attack_duration: 0.0,
            attack_hitbox_spawned: false,
            attack_buffered: false,
            hit_enemies_this_attack: Vec::with_capacity(8),
            combo_step: 0,
            combo_window_timer: 0.0,
            secondary_cooldown: 0.0,
            is_blocking: false,
            backstab_buff_timer: 0.0,
            corpse_channel_timer: 0.0,
            corpse_channel_target: None,
            is_charging_nova: false,
            nova_charge_timer: 0.0,
        }
    }

    pub fn set_class(&mut self, class: Option<ClassKind>) {
        self.equipped_class = class;
    }

    pub fn get_collider(&self) -> Collider {
        Collider::new(self.x - 6.0, self.y - 4.0, 12.0, 8.0)
    }

    pub fn get_hurtbox(&self) -> Hurtbox {
        Hurtbox {
            center: (self.x, self.y - 12.0),
            radius: 7.5,
            band: HeightBand::Mid,
            is_player: true,
        }
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        ShadowCaster {
            x: self.x,
            y: self.y,
            width: 14.0,
            height: 28.0,
        }
    }

    pub fn is_invulnerable(&self) -> bool {
        if self.is_dead || self.i_frames_timer > 0.0 {
            return true;
        }
        if self.is_rolling {
            // Frames 2..14 of 23-frame roll (~0.033s to ~0.231s elapsed)
            let elapsed = ROLL_DURATION - self.roll_timer;
            let start_iframe = (2.0 / 23.0) * ROLL_DURATION;
            let end_iframe = (14.0 / 23.0) * ROLL_DURATION;
            if elapsed >= start_iframe && elapsed <= end_iframe {
                return true;
            }
        }
        false
    }

    pub fn take_damage(
        &mut self,
        damage: i32,
        attack_origin: (f32, f32),
        band: HeightBand,
        status: Option<StatusEffectKind>,
        floating_texts: &mut Vec<FloatingText>,
        particles: &mut ParticleSystem,
    ) -> bool {
        if self.is_invulnerable() {
            return false;
        }

        let hurtbox = self.get_hurtbox();
        if !band.hits(hurtbox.band) {
            return false;
        }

        let mut final_damage = damage;
        let dx = attack_origin.0 - self.x;
        let dy = attack_origin.1 - self.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.01);
        let attack_dir = (dx / dist, dy / dist);

        // Check Knight Shield Block
        if self.is_blocking && self.equipped_class == Some(ClassKind::Knight) {
            let facing_x = self.facing_angle.cos();
            let facing_y = self.facing_angle.sin();
            // Dot product between facing and incoming attack direction
            let dot = facing_x * attack_dir.0 + facing_y * attack_dir.1;
            if dot > 0.2 {
                // Frontal block! 80% damage reduction
                final_damage = ((damage as f32) * 0.20).round() as i32;
                self.hit_stop = 4;
                self.screen_shake.add(0.25);
                floating_texts.push(FloatingText::new(
                    self.x,
                    self.y - 28.0,
                    "BLOCKED!",
                    Palette::TEXT_GOLD,
                    0.8,
                ));
                for _ in 0..6 {
                    particles.spawn_spark(self.x + facing_x * 8.0, self.y - 12.0, -facing_x * 12.0, -facing_y * 12.0);
                }
                if final_damage == 0 {
                    return false;
                }
            }
        }

        self.hp = (self.hp - final_damage).max(0);
        self.i_frames_timer = 0.6;
        self.flinch_timer = 0.20;
        self.hit_stop = 3;
        self.screen_shake.add(0.45);

        // Directional flinch knockback
        self.vx = -attack_dir.0 * 22.0;
        self.vy = -attack_dir.1 * 22.0;

        floating_texts.push(FloatingText::new(
            self.x,
            self.y - 24.0,
            format!("-{}", final_damage),
            Color::rgb(255, 60, 60),
            0.85,
        ));

        for _ in 0..5 {
            particles.spawn_spark(self.x, self.y - 12.0, -attack_dir.0 * 15.0, -attack_dir.1 * 15.0);
        }

        if let Some(effect) = status {
            match effect {
                StatusEffectKind::Poison => {
                    self.poison_timer = 6.0;
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - 30.0,
                        "POISONED",
                        Color::rgb(100, 240, 80),
                        0.9,
                    ));
                }
                StatusEffectKind::Acid => {
                    self.acid_timer = 3.5;
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - 30.0,
                        "ACID BURN",
                        Color::rgb(240, 160, 40),
                        0.9,
                    ));
                }
                StatusEffectKind::Slow => {}
            }
        }

        if self.hp <= 0 {
            self.is_dead = true;
            self.death_timer = 0.0;
            self.screen_shake.add(0.8);
            true
        } else {
            false
        }
    }

    pub fn respawn_at(&mut self, bonfire_x: f32, bonfire_y: f32) {
        self.x = bonfire_x;
        self.y = bonfire_y + 16.0;
        self.vx = 0.0;
        self.vy = 0.0;
        self.hp = self.max_hp;
        self.mana = self.max_mana;
        self.poison_timer = 0.0;
        self.acid_timer = 0.0;
        self.i_frames_timer = 1.0;
        self.is_dead = false;
        self.death_timer = 0.0;
        self.is_attacking = false;
        self.is_blocking = false;
        self.flinch_timer = 0.0;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        mouse_world: (f32, f32),
        world: &World,
        projectiles: &mut Vec<Projectile>,
        puddles: &mut Vec<HazardPuddle>,
        hitboxes: &mut Vec<Hitbox>,
        floating_texts: &mut Vec<FloatingText>,
        particles: &mut ParticleSystem,
        combat_data: &CombatData,
    ) {
        // Hit-stop frames freeze player action
        if self.hit_stop > 0 {
            self.hit_stop -= 1;
            return;
        }

        // Screen shake decay
        self.screen_shake.update(dt);

        // Dead state
        if self.is_dead {
            self.death_timer += dt;
            self.vx *= 0.82;
            self.vy *= 0.82;
            return;
        }

        // Decay i-frames & flinch
        if self.i_frames_timer > 0.0 {
            self.i_frames_timer -= dt;
        }
        if self.flinch_timer > 0.0 {
            self.flinch_timer -= dt;
        }
        if self.backstab_buff_timer > 0.0 {
            self.backstab_buff_timer -= dt;
        }
        if self.secondary_cooldown > 0.0 {
            self.secondary_cooldown -= dt;
        }

        // Status effects: Poison DoT (ticks every 1.5s for 6s)
        if self.poison_timer > 0.0 {
            self.poison_timer -= dt;
            self.poison_tick_timer += dt;
            if self.poison_tick_timer >= 1.5 {
                self.poison_tick_timer = 0.0;
                self.hp = (self.hp - 1).max(0);
                floating_texts.push(FloatingText::new(
                    self.x,
                    self.y - 20.0,
                    "-1",
                    Color::rgb(80, 220, 80),
                    0.6,
                ));
                if self.hp <= 0 {
                    self.is_dead = true;
                    self.death_timer = 0.0;
                    self.screen_shake.add(0.8);
                    return;
                }
            }
        }

        // Status effects: Acid burn
        if self.acid_timer > 0.0 {
            self.acid_timer -= dt;
            self.acid_tick_timer += dt;
            if self.acid_tick_timer >= 1.0 {
                self.acid_tick_timer = 0.0;
                self.hp = (self.hp - 1).max(0);
                floating_texts.push(FloatingText::new(
                    self.x,
                    self.y - 20.0,
                    "-1",
                    Color::rgb(220, 150, 40),
                    0.6,
                ));
                if self.hp <= 0 {
                    self.is_dead = true;
                    self.death_timer = 0.0;
                    self.screen_shake.add(0.8);
                    return;
                }
            }
        }

        // Mana regeneration for Mage
        if self.equipped_class == Some(ClassKind::Mage) {
            self.mana = (self.mana + combat_data.player.mana_regen_rate * dt).min(self.max_mana);
        }

        // Decay ghost trails
        self.roll_ghosts.retain_mut(|(_, _, life, _, _)| {
            *life -= dt * 3.5;
            *life > 0.0
        });

        // Roll Cooldown & Input Buffering
        if self.roll_cooldown > 0.0 {
            self.roll_cooldown -= dt;
            if self.roll_cooldown <= 0.15 && input.just_pressed(VirtualKey::Space) {
                self.roll_buffered = true;
            }
        }

        // Combo window timer
        if self.combo_window_timer > 0.0 {
            self.combo_window_timer -= dt;
            if self.combo_window_timer <= 0.0 {
                self.combo_step = 0;
            }
        }

        // Mouse facing hysteresis
        if !self.is_rolling && input.mouse_ever_detected {
            let aim_dx = mouse_world.0 - self.x;
            let aim_dy = mouse_world.1 - self.y;
            if aim_dx * aim_dx + aim_dy * aim_dy >= 144.0 {
                let (new_dir, new_flip) = update_direction_hysteresis(
                    aim_dx,
                    aim_dy,
                    &mut self.facing_angle,
                );
                self.dir4 = new_dir;
                self.flip_x = new_flip;
            }
        }

        // -------------------------------------------------------------
        // COMBAT ACTIONS: ATTACK & BLOCK
        // -------------------------------------------------------------
        let aim_dx = mouse_world.0 - self.x;
        let aim_dy = mouse_world.1 - self.y;
        let aim_dist_sq = aim_dx * aim_dx + aim_dy * aim_dy;
        let (aim_norm, aim_angle) = if input.mouse_ever_detected && aim_dist_sq >= 36.0 {
            let d = aim_dist_sq.sqrt();
            ((aim_dx / d, aim_dy / d), aim_dy.atan2(aim_dx))
        } else {
            ((self.facing_angle.cos(), self.facing_angle.sin()), self.facing_angle)
        };
        let aim_dist = aim_dist_sq.sqrt().max(0.01);

        // Knight Shield Block (Hold RMB)
        if self.equipped_class == Some(ClassKind::Knight) {
            self.is_blocking = input.mouse_right_held && !self.is_rolling && !self.is_attacking;
        } else {
            self.is_blocking = false;
        }

        // Mage Nova Charging
        if self.is_charging_nova {
            self.nova_charge_timer -= dt;
            // Gather flame particles
            particles.spawn_spark(
                self.x + ((self.nova_charge_timer * 15.0).cos() * 18.0),
                self.y - 12.0 + ((self.nova_charge_timer * 15.0).sin() * 18.0),
                0.0,
                0.0,
            );
            if self.nova_charge_timer <= 0.0 {
                self.is_charging_nova = false;
                self.secondary_cooldown = combat_data.mage.nova_cooldown;
                self.mana -= combat_data.mage.nova_mana_cost;
                self.screen_shake.add(0.65);
                // Cinder Nova radial AoE in HeightBand::All!
                hitboxes.push(Hitbox {
                    origin: (self.x, self.y - 12.0),
                    shape: HitboxShape::Circle { radius: combat_data.mage.nova_radius },
                    band: HeightBand::All,
                    damage: combat_data.mage.nova_damage,
                    knockback: combat_data.mage.nova_knockback,
                    knockback_dir: (0.0, 0.0),
                    is_player: true,
                    hit_stop: 4,
                    status: None,
                });
                floating_texts.push(FloatingText::new(
                    self.x,
                    self.y - 32.0,
                    "CINDER NOVA!",
                    Palette::MAGE_CYAN,
                    1.0,
                ));
                for _ in 0..16 {
                    let angle = rand::random::<f32>() * 2.0 * PI;
                    let spd = 30.0 + rand::random::<f32>() * 40.0;
                    particles.spawn_spark(self.x, self.y - 10.0, angle.cos() * spd, angle.sin() * spd);
                }
            }
        }

        // Process Secondary Skill (RMB)
        if input.mouse_right_just_pressed && !self.is_rolling && !self.is_attacking && self.secondary_cooldown <= 0.0 {
            match self.equipped_class {
                Some(ClassKind::Knight) => {
                    // Handled above via is_blocking hold
                }
                Some(ClassKind::Mage) => {
                    if self.mana >= combat_data.mage.nova_mana_cost && !self.is_charging_nova {
                        self.is_charging_nova = true;
                        self.nova_charge_timer = combat_data.mage.nova_charge_time;
                    }
                }
                Some(ClassKind::Assassin) => {
                    // Shadowstep blink
                    self.secondary_cooldown = combat_data.assassin.shadowstep_cooldown;
                    for _ in 0..6 {
                        particles.spawn_dust(self.x, self.y - 8.0);
                    }
                    let step_dist = combat_data.assassin.shadowstep_distance;
                    let target_x = self.x + aim_norm.0 * step_dist;
                    let target_y = self.y + aim_norm.1 * step_dist;
                    let test_col = Collider::new(target_x - 5.0, target_y - 3.0, 10.0, 6.0);
                    if !world.check_collision(&test_col) {
                        self.x = target_x;
                        self.y = target_y;
                    }
                    self.backstab_buff_timer = combat_data.assassin.backstab_window;
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - 28.0,
                        "SHADOWSTEP",
                        Palette::ASSASSIN_RED,
                        0.8,
                    ));
                    for _ in 0..8 {
                        particles.spawn_dust(self.x, self.y - 8.0);
                    }
                }
                Some(ClassKind::Hunter) => {
                    // Pistol-Sword Flintlock Shot (High band)
                    self.secondary_cooldown = combat_data.hunter.flintlock_cooldown;
                    let shot_speed = combat_data.hunter.flintlock_speed;
                    projectiles.push(Projectile {
                        x: self.x + aim_norm.0 * 12.0,
                        y: self.y - 14.0 + aim_norm.1 * 4.0,
                        vx: aim_norm.0 * shot_speed,
                        vy: aim_norm.1 * shot_speed,
                        band: HeightBand::High,
                        damage: combat_data.hunter.flintlock_damage,
                        lifespan: combat_data.hunter.flintlock_range / shot_speed,
                        max_lifespan: combat_data.hunter.flintlock_range / shot_speed,
                        is_player: true,
                        status: None,
                        radius: 3.0,
                        color: Color::rgb(240, 240, 255),
                        emissive: true,
                        is_lobbed: false,
                        lob_timer: 0.0,
                        lob_duration: 0.0,
                        target_pos: (0.0, 0.0),
                    });
                    // Recoil
                    self.vx -= aim_norm.0 * combat_data.hunter.flintlock_recoil * 3.5;
                    self.vy -= aim_norm.1 * combat_data.hunter.flintlock_recoil * 3.5;
                    self.screen_shake.add(0.35);
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - 28.0,
                        "FLINTLOCK [↑High]",
                        Palette::HUNTER_BRASS,
                        0.8,
                    ));
                }
                Some(ClassKind::Alchemist) => {
                    // Eye Rite slowing field
                    self.secondary_cooldown = combat_data.alchemist.eye_cooldown;
                    let eye_x = self.x + aim_norm.0 * combat_data.alchemist.eye_distance;
                    let eye_y = self.y + aim_norm.1 * combat_data.alchemist.eye_distance;
                    puddles.push(HazardPuddle::new(
                        eye_x,
                        eye_y,
                        combat_data.alchemist.eye_radius,
                        1.0,
                        combat_data.alchemist.eye_slow_duration,
                        false,
                    ));
                    floating_texts.push(FloatingText::new(
                        eye_x,
                        eye_y - 16.0,
                        "EYE RITE (SLOW)",
                        Palette::ALCHEMIST_TEAL,
                        1.0,
                    ));
                }
                None | Some(ClassKind::Wanderer) => {}
            }
        }

        // Buffer or Trigger Primary Attack (LMB)
        if input.mouse_left_just_pressed {
            if self.is_attacking {
                let progress = 1.0 - (self.attack_timer / self.attack_duration).clamp(0.0, 1.0);
                if progress >= 0.45 {
                    self.attack_buffered = true;
                }
            } else if !self.is_rolling && !self.is_blocking {
                self.attack_buffered = true;
            }
        }

        if self.attack_buffered && !self.is_attacking && !self.is_rolling && !self.is_blocking {
            self.attack_buffered = false;
            self.hit_enemies_this_attack.clear();

            match self.equipped_class {
                Some(ClassKind::Mage) => {
                    if self.mana >= combat_data.mage.bolt_mana_cost {
                        self.mana -= combat_data.mage.bolt_mana_cost;
                        self.is_attacking = true;
                        self.attack_kind = PlayerAttackKind::Primary(0);
                        self.attack_timer = 0.25;
                        self.attack_duration = 0.25;
                        self.attack_hitbox_spawned = true;

                        let bolt_speed = combat_data.mage.bolt_speed;
                        projectiles.push(Projectile {
                            x: self.x + aim_norm.0 * 12.0,
                            y: self.y - 12.0 + aim_norm.1 * 4.0,
                            vx: aim_norm.0 * bolt_speed,
                            vy: aim_norm.1 * bolt_speed,
                            band: HeightBand::All,
                            damage: combat_data.mage.bolt_damage,
                            lifespan: combat_data.mage.bolt_range / bolt_speed,
                            max_lifespan: combat_data.mage.bolt_range / bolt_speed,
                            is_player: true,
                            status: None,
                            radius: 3.5,
                            color: Color::rgb(100, 210, 255),
                            emissive: true,
                            is_lobbed: false,
                            lob_timer: 0.0,
                            lob_duration: 0.0,
                            target_pos: (0.0, 0.0),
                        });
                    }
                }
                Some(ClassKind::Alchemist) => {
                    // Lob acid vial
                    self.is_attacking = true;
                    self.attack_kind = PlayerAttackKind::Primary(0);
                    self.attack_timer = 0.30;
                    self.attack_duration = 0.30;
                    self.attack_hitbox_spawned = true;

                    let lob_dist = aim_dist.min(100.0);
                    let lob_dur = (lob_dist / combat_data.alchemist.vial_arc_speed).max(0.25);
                    projectiles.push(Projectile {
                        x: self.x,
                        y: self.y - 12.0,
                        vx: aim_norm.0 * (lob_dist / lob_dur),
                        vy: aim_norm.1 * (lob_dist / lob_dur),
                        band: HeightBand::Mid,
                        damage: 1,
                        lifespan: lob_dur,
                        max_lifespan: lob_dur,
                        is_player: true,
                        status: Some(StatusEffectKind::Acid),
                        radius: 3.0,
                        color: Color::rgb(160, 230, 40),
                        emissive: true,
                        is_lobbed: true,
                        lob_timer: 0.0,
                        lob_duration: lob_dur,
                        target_pos: (self.x + aim_norm.0 * lob_dist, self.y + aim_norm.1 * lob_dist),
                    });
                }
                _ => {
                    // Combo melee attacks (Knight, Assassin, Hunter, Wanderer)
                    let step = self.combo_step;
                    self.is_attacking = true;
                    self.attack_kind = PlayerAttackKind::Primary(step);
                    self.attack_timer = 0.28;
                    self.attack_duration = 0.28;
                    self.attack_hitbox_spawned = false;
                    self.combo_step = (step + 1) % 3;
                    self.combo_window_timer = 0.70;
                }
            }
        }

        // Active Attack Update & Hitbox Generation
        if self.is_attacking {
            self.attack_timer -= dt;
            let progress = 1.0 - (self.attack_timer / self.attack_duration).clamp(0.0, 1.0);

            // Active frame window: emit hitbox continuously between 15% and 75% of swing
            if (0.15..=0.75).contains(&progress) {
                match self.equipped_class {
                    Some(ClassKind::Knight) => {
                        let step = match self.attack_kind {
                            PlayerAttackKind::Primary(s) => s,
                            _ => 0,
                        };
                        let (dmg, reach, band, k_back) = match step {
                            0 => (combat_data.knight.combo1_damage, combat_data.knight.combo1_reach, HeightBand::Mid, 18.0),
                            1 => (combat_data.knight.combo2_damage, combat_data.knight.combo2_reach, HeightBand::Mid, 22.0),
                            _ => (combat_data.knight.combo3_damage, combat_data.knight.combo3_reach, HeightBand::All, combat_data.knight.combo3_knockback),
                        };
                        hitboxes.push(Hitbox {
                            origin: (self.x, self.y - 12.0),
                            shape: HitboxShape::Arc { radius: reach, angle: aim_angle, spread: 1.45 },
                            band,
                            damage: dmg,
                            knockback: k_back,
                            knockback_dir: aim_norm,
                            is_player: true,
                            hit_stop: 3,
                            status: None,
                        });
                        self.screen_shake.add(0.12);
                    }
                    Some(ClassKind::Assassin) => {
                        let step = match self.attack_kind {
                            PlayerAttackKind::Primary(s) => s,
                            _ => 0,
                        };
                        let crit_mult = if self.backstab_buff_timer > 0.0 {
                            combat_data.assassin.backstab_multiplier
                        } else {
                            1.0
                        };
                        let base_dmg = match step {
                            0 => combat_data.assassin.combo1_damage,
                            1 => combat_data.assassin.combo2_damage,
                            _ => combat_data.assassin.combo3_damage,
                        };
                        let final_dmg = ((base_dmg as f32) * crit_mult).round() as i32;
                        hitboxes.push(Hitbox {
                            origin: (self.x, self.y - 12.0),
                            shape: HitboxShape::Arc { radius: combat_data.assassin.combo_reach, angle: aim_angle, spread: 1.35 },
                            band: HeightBand::Mid,
                            damage: final_dmg,
                            knockback: 16.0,
                            knockback_dir: aim_norm,
                            is_player: true,
                            hit_stop: 2,
                            status: None,
                        });
                        if crit_mult > 1.0 {
                            floating_texts.push(FloatingText::new(
                                self.x,
                                self.y - 28.0,
                                "BACKSTAB CRIT!",
                                Palette::ASSASSIN_RED,
                                0.9,
                            ));
                            self.screen_shake.add(0.35);
                        }
                    }
                    Some(ClassKind::Hunter) => {
                        hitboxes.push(Hitbox {
                            origin: (self.x, self.y - 12.0),
                            shape: HitboxShape::Arc { radius: combat_data.hunter.blade_reach, angle: aim_angle, spread: 1.50 },
                            band: HeightBand::Mid,
                            damage: combat_data.hunter.blade_damage,
                            knockback: 18.0,
                            knockback_dir: aim_norm,
                            is_player: true,
                            hit_stop: 2,
                            status: None,
                        });
                    }
                    _ => {
                        // Wanderer basic cane
                        hitboxes.push(Hitbox {
                            origin: (self.x, self.y - 12.0),
                            shape: HitboxShape::Arc { radius: 32.0, angle: aim_angle, spread: 1.35 },
                            band: HeightBand::Mid,
                            damage: 2,
                            knockback: 16.0,
                            knockback_dir: aim_norm,
                            is_player: true,
                            hit_stop: 2,
                            status: None,
                        });
                    }
                }
            }

            if self.attack_timer <= 0.0 {
                self.is_attacking = false;
                self.hit_enemies_this_attack.clear();
            }
        }

        // Trigger Dodge Roll (Space only)
        let wants_roll = (input.just_pressed(VirtualKey::Space) || self.roll_buffered)
            && self.roll_cooldown <= 0.0
            && !self.is_attacking;

        if wants_roll {
            self.roll_buffered = false;
            self.is_rolling = true;
            self.roll_timer = ROLL_DURATION;
            self.roll_cooldown = ROLL_COOLDOWN;

            let (key_dx, key_dy) = input.movement_vector();
            let aim_len_sq = aim_dx * aim_dx + aim_dy * aim_dy;

            if input.mouse_ever_detected && aim_len_sq >= 144.0 {
                let aim_len = aim_len_sq.sqrt();
                self.roll_dir = (aim_dx / aim_len, aim_dy / aim_len);
            } else if key_dx != 0.0 || key_dy != 0.0 {
                self.roll_dir = (key_dx, key_dy);
            } else {
                self.roll_dir = (self.facing_angle.cos(), self.facing_angle.sin());
            }

            let (r_dir, r_flip) = update_direction_hysteresis(
                self.roll_dir.0,
                self.roll_dir.1,
                &mut self.facing_angle,
            );
            self.dir4 = r_dir;
            self.flip_x = r_flip;

            self.roll_ghosts.push((self.x, self.y, 1.0, self.dir4, self.flip_x));
            particles.spawn_dust(self.x, self.y);
        }

        // Active Dodge Roll Execution
        if self.is_rolling {
            self.roll_timer -= dt;
            if self.roll_timer <= 0.0 {
                self.is_rolling = false;
                self.roll_timer = 0.0;
            } else {
                let v0 = (2.0 * ROLL_DISTANCE) / ROLL_DURATION;
                let current_roll_speed = v0 * (self.roll_timer / ROLL_DURATION);

                let move_x = self.roll_dir.0 * current_roll_speed * dt;
                let move_y = self.roll_dir.1 * current_roll_speed * dt;

                let test_x_col = Collider::new(self.x + move_x - 5.0, self.y - 3.0, 10.0, 6.0);
                if !world.check_collision(&test_x_col) {
                    self.x += move_x;
                }
                let test_y_col = Collider::new(self.x - 5.0, self.y + move_y - 3.0, 10.0, 6.0);
                if !world.check_collision(&test_y_col) {
                    self.y += move_y;
                }

                let roll_progress = (1.0 - (self.roll_timer / ROLL_DURATION)).clamp(0.0, 0.99);
                self.anim_frame = (roll_progress * 7.0) as usize;

                self.dust_timer += dt;
                if self.dust_timer > 0.05 {
                    self.dust_timer = 0.0;
                    self.roll_ghosts.push((self.x, self.y, 0.75, self.dir4, self.flip_x));
                    particles.spawn_dust(self.x, self.y);
                }
                return;
            }
        }

        // Movement with smooth acceleration & deceleration
        let (key_dx, key_dy) = input.movement_vector();
        self.is_running = input.is_held(VirtualKey::Shift);

        let mut max_speed = if self.is_running {
            PLAYER_RUN_SPEED
        } else {
            PLAYER_WALK_SPEED
        };

        if self.is_blocking {
            max_speed *= combat_data.knight.shield_move_speed_mult;
        }

        let target_vx = key_dx * max_speed;
        let target_vy = key_dy * max_speed;

        let accel_rate = max_speed / 0.07;
        let decel_rate = max_speed / 0.04;

        let rate_x = if target_vx.abs() > self.vx.abs() { accel_rate } else { decel_rate };
        let rate_y = if target_vy.abs() > self.vy.abs() { accel_rate } else { decel_rate };

        self.vx = approach(self.vx, target_vx, rate_x * dt);
        self.vy = approach(self.vy, target_vy, rate_y * dt);

        let current_speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        self.is_moving = current_speed > 1.0;

        if self.is_moving {
            let move_x = self.vx * dt;
            let move_y = self.vy * dt;

            let test_x_col = Collider::new(self.x + move_x - 6.0, self.y - 4.0, 12.0, 8.0);
            if !world.check_collision(&test_x_col) {
                self.x += move_x;
            } else {
                self.vx = 0.0;
            }

            let test_y_col = Collider::new(self.x - 6.0, self.y + move_y - 4.0, 12.0, 8.0);
            if !world.check_collision(&test_y_col) {
                self.y += move_y;
            } else {
                self.vy = 0.0;
            }

            let anim_speed_factor = (current_speed / PLAYER_WALK_SPEED).clamp(0.5, 2.0);
            self.anim_timer += dt * 9.0 * anim_speed_factor;
            self.anim_frame = (self.anim_timer as usize) % 6;

            self.dust_timer += dt;
            let dust_interval = if self.is_running { 0.15 } else { 0.25 };
            if self.dust_timer > dust_interval {
                self.dust_timer = 0.0;
                particles.spawn_dust(self.x + if self.flip_x { 2.0 } else { -2.0 }, self.y);
            }
        } else {
            self.anim_timer += dt * 2.5;
            self.anim_frame = (self.anim_timer as usize) % 4;
            self.dust_timer = 0.0;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        let draw_x = (self.x - 9.0).round() as i32 - cam_x;
        let draw_y = (self.y - 28.0).round() as i32 - cam_y;

        // Contact shadow
        let shadow_cx = draw_x + 9;
        let shadow_cy = draw_y + 28;
        fb.draw_ellipse(shadow_cx, shadow_cy, 7, 3, Color::rgba(5, 6, 12, 140));

        // Draw death collapse
        if self.is_dead {
            if let Some(gear_set) = self.equipped_class.and_then(|c| bank.get_class_gear(c)) {
                gear_set.death_frame.draw(fb, draw_x, draw_y, self.flip_x);
            } else {
                bank.body_idle[0][0].draw(fb, draw_x, draw_y, self.flip_x);
            }
            return;
        }

        // Post-damage blink: skip drawing on alternate frames
        if self.i_frames_timer > 0.0 && ((self.i_frames_timer * 30.0) as usize).is_multiple_of(2) {
            return;
        }

        // Motion blur ghost trails
        for &(gx, gy, glife, gdir, gflip) in &self.roll_ghosts {
            let g_sx = (gx - 9.0).round() as i32 - cam_x;
            let g_sy = (gy - 28.0).round() as i32 - cam_y;
            let g_dir_idx = match gdir {
                Direction4::South => 0,
                Direction4::North => 1,
                Direction4::East => 2,
                Direction4::SouthEast => 3,
            };
            let body = &bank.body_roll[g_dir_idx][2];
            let alpha = (glife * 110.0).clamp(0.0, 255.0) as u8;
            body.draw_alpha(fb, g_sx, g_sy, gflip, alpha);
        }

        let dir_idx = match self.dir4 {
            Direction4::South => 0,
            Direction4::North => 1,
            Direction4::East => 2,
            Direction4::SouthEast => 3,
        };

        // Base Body sprite
        let body_sprite = if self.is_rolling {
            &bank.body_roll[dir_idx][self.anim_frame % 7]
        } else if self.flinch_timer > 0.0 {
            &bank.body_hit[dir_idx]
        } else if self.is_attacking {
            let f = ((1.0 - (self.attack_timer / self.attack_duration).clamp(0.0, 1.0)) * 4.0) as usize % 4;
            &bank.body_lmb[dir_idx][f]
        } else if self.is_moving {
            &bank.body_walk[dir_idx][self.anim_frame % 6]
        } else {
            &bank.body_idle[dir_idx][self.anim_frame % 4]
        };

        // Cosmetic i-frame glow during roll
        if self.is_rolling {
            let glow_alpha = ((self.roll_timer / ROLL_DURATION) * 70.0) as u8;
            let glow_col = Color::rgba(255, 230, 180, glow_alpha);
            fb.draw_circle(shadow_cx, shadow_cy - 12, 12, glow_col, false);
            body_sprite.draw(fb, draw_x, draw_y, self.flip_x);
            return;
        }

        // Layered gear composite rendering
        if let Some(gear_set) = self.equipped_class.and_then(|class| bank.get_class_gear(class)) {
            if self.flinch_timer > 0.0 {
                gear_set.hit_frame[dir_idx].draw(fb, draw_x, draw_y, self.flip_x);
                return;
            }
            if self.is_attacking {
                let f = ((1.0 - (self.attack_timer / self.attack_duration).clamp(0.0, 1.0)) * 4.0) as usize % 4;
                gear_set.lmb_frames[dir_idx][f].draw(fb, draw_x, draw_y, self.flip_x);
                return;
            }

            let gear_sprites: [Option<&crate::assets::sprites::Sprite>; 6] = [
                Some(&gear_set.head[dir_idx]),
                Some(&gear_set.torso[dir_idx]),
                gear_set.cape[dir_idx].as_ref(),
                gear_set.main_hand[dir_idx].as_ref(),
                gear_set.off_hand[dir_idx].as_ref(),
                gear_set.accessory[dir_idx].as_ref(),
            ];
            self.gear.draw_composite(
                fb,
                body_sprite,
                &gear_sprites,
                draw_x,
                draw_y,
                self.dir4,
                self.flip_x,
            );
            return;
        }

        // Unarmored Wanderer default
        let empty_gear: [Option<&crate::assets::sprites::Sprite>; 6] = [None, None, None, None, None, None];
        self.gear.draw_composite(
            fb,
            body_sprite,
            &empty_gear,
            draw_x,
            draw_y,
            self.dir4,
            self.flip_x,
        );
    }
}

#[inline]
fn approach(current: f32, target: f32, max_step: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_step {
        target
    } else {
        current + diff.signum() * max_step
    }
}

fn update_direction_hysteresis(
    aim_dx: f32,
    aim_dy: f32,
    current_angle: &mut f32,
) -> (Direction4, bool) {
    if aim_dx.abs() < 0.001 && aim_dy.abs() < 0.001 {
        return (Direction4::South, false);
    }

    let target_angle = aim_dy.atan2(aim_dx);
    let mut diff = target_angle - *current_angle;
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    while diff > std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }

    if diff.abs() > 0.1745 {
        *current_angle = target_angle;
    }

    let mut deg = current_angle.to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }

    match deg {
        d if (22.5..67.5).contains(&d) => (Direction4::SouthEast, false),
        d if (67.5..112.5).contains(&d) => (Direction4::South, false),
        d if (112.5..157.5).contains(&d) => (Direction4::SouthEast, true),
        d if (157.5..247.5).contains(&d) => (Direction4::East, true),
        d if (247.5..292.5).contains(&d) => (Direction4::North, false),
        _ => (Direction4::East, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_invulnerability_and_iframes() {
        let mut player = Player::new(100.0, 100.0);
        assert!(!player.is_invulnerable());

        // 1. Post-damage i-frames
        player.i_frames_timer = 0.5;
        assert!(player.is_invulnerable());
        player.i_frames_timer = 0.0;
        assert!(!player.is_invulnerable());

        // 2. Roll i-frames (frames 2..14 out of 23 frames, roll duration 0.38s)
        player.is_rolling = true;
        // Frame 0: elapsed 0.0 -> roll_timer = 0.38
        player.roll_timer = ROLL_DURATION;
        assert!(!player.is_invulnerable());

        // Frame 5: elapsed (5/23)*0.38 = 0.0826 -> roll_timer = 0.38 - 0.0826 = 0.2974
        player.roll_timer = ROLL_DURATION - (5.0 / 23.0) * ROLL_DURATION;
        assert!(player.is_invulnerable());

        // Frame 18: elapsed (18/23)*0.38 -> outside frames 2..14
        player.roll_timer = ROLL_DURATION - (18.0 / 23.0) * ROLL_DURATION;
        assert!(!player.is_invulnerable());
    }

    #[test]
    fn test_knight_frontal_block_damage_reduction() {
        let mut player = Player::new(100.0, 100.0);
        player.set_class(Some(ClassKind::Knight));
        player.facing_angle = 0.0; // Facing East (+X)
        player.is_blocking = true;

        let mut floating_texts = Vec::new();
        let mut particles = ParticleSystem::new(16);

        // Attack from East (front): origin is at (130, 100)
        let died = player.take_damage(
            5,
            (130.0, 100.0),
            HeightBand::Mid,
            None,
            &mut floating_texts,
            &mut particles,
        );
        assert!(!died);
        // 5 damage with 80% reduction -> (5 * 0.20) = 1 damage taken
        assert_eq!(player.hp, 9);
    }

    #[test]
    fn test_player_death_and_respawn_loop() {
        let mut player = Player::new(100.0, 100.0);
        player.souls = 100;
        let mut floating_texts = Vec::new();
        let mut particles = ParticleSystem::new(16);

        let died = player.take_damage(
            15,
            (110.0, 100.0),
            HeightBand::Mid,
            None,
            &mut floating_texts,
            &mut particles,
        );
        assert!(died);
        assert!(player.is_dead);
        assert_eq!(player.hp, 0);

        // Respawn at bonfire
        player.respawn_at(140.0, 95.0);
        assert!(!player.is_dead);
        assert_eq!(player.hp, player.max_hp);
        assert_eq!(player.mana, player.max_mana);
        assert_eq!(player.x, 140.0);
        assert_eq!(player.y, 111.0);
    }
}
