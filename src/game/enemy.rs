use crate::assets::enemy_sprites;
use crate::assets::palette::Palette;
use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::lighting::ShadowCaster;
use crate::game::combat::{CombatData, HazardPuddle, HeightBand, Hurtbox, Projectile, StatusEffectKind, FloatingText};
use crate::game::particles::ParticleSystem;
use crate::game::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Sludge,
    Skeleton,
    Fly,
    Worm,
}

impl EnemyKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            EnemyKind::Sludge => "Acid Sludge",
            EnemyKind::Skeleton => "Bogged Archer",
            EnemyKind::Fly => "Carrion Fly",
            EnemyKind::Worm => "Corpse Worm",
        }
    }

    pub fn default_band(&self) -> HeightBand {
        match self {
            EnemyKind::Sludge => HeightBand::Low,
            EnemyKind::Skeleton => HeightBand::Mid,
            EnemyKind::Fly => HeightBand::High,
            EnemyKind::Worm => HeightBand::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyState {
    Idle { timer: f32 },
    Chase,
    AttackWindup { timer: f32, max_timer: f32 },
    Attacking { timer: f32 },
    Recovery { timer: f32 },
    Flinch { timer: f32 },
    Dead,
}

pub struct Enemy {
    pub id: usize,
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub z: f32, // Height for aerial/flying enemies
    pub vx: f32,
    pub vy: f32,
    pub flip_x: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub souls: u32,
    pub state: EnemyState,
    pub band: HeightBand,
    pub hit_stop: u32,
    pub flinch_timer: f32,
    pub poison_timer: f32,
    pub poison_tick_timer: f32,
    pub slow_timer: f32,
    pub anim_timer: f32,
    pub anim_frame: usize,
    pub attack_cooldown: f32,
    pub is_corpse_consumed: bool,
    pub death_timer: f32,
    pub contact_damage: i32,
    pub move_speed: f32,
}

impl Enemy {
    pub fn new(id: usize, kind: EnemyKind, x: f32, y: f32, combat_data: &CombatData) -> Self {
        let (hp, souls, speed, contact_dmg) = match kind {
            EnemyKind::Sludge => (
                combat_data.enemies.sludge.max_hp,
                combat_data.enemies.sludge.souls,
                combat_data.enemies.sludge.move_speed,
                combat_data.enemies.sludge.contact_damage,
            ),
            EnemyKind::Skeleton => (
                combat_data.enemies.skeleton.max_hp,
                combat_data.enemies.skeleton.souls,
                combat_data.enemies.skeleton.move_speed,
                combat_data.enemies.skeleton.contact_damage,
            ),
            EnemyKind::Fly => (
                combat_data.enemies.fly.max_hp,
                combat_data.enemies.fly.souls,
                combat_data.enemies.fly.move_speed,
                combat_data.enemies.fly.contact_damage,
            ),
            EnemyKind::Worm => (
                combat_data.enemies.worm.max_hp,
                combat_data.enemies.worm.souls,
                combat_data.enemies.worm.move_speed,
                combat_data.enemies.worm.contact_damage,
            ),
        };

        let initial_z = if kind == EnemyKind::Fly {
            combat_data.enemies.fly.hover_height
        } else {
            0.0
        };

        Self {
            id,
            kind,
            x,
            y,
            z: initial_z,
            vx: 0.0,
            vy: 0.0,
            flip_x: false,
            hp,
            max_hp: hp,
            souls,
            state: EnemyState::Chase,
            band: kind.default_band(),
            hit_stop: 0,
            flinch_timer: 0.0,
            poison_timer: 0.0,
            poison_tick_timer: 0.0,
            slow_timer: 0.0,
            anim_timer: (id as f32) * 0.7, // Desync animation loops
            anim_frame: 0,
            attack_cooldown: 0.8 + ((id % 5) as f32) * 0.3,
            is_corpse_consumed: false,
            death_timer: 0.0,
            contact_damage: contact_dmg,
            move_speed: speed,
        }
    }

    pub fn get_hurtbox(&self) -> Hurtbox {
        let (radius, y_offset) = match self.kind {
            EnemyKind::Sludge => (14.0, 8.0),
            EnemyKind::Skeleton => (12.0, 14.0),
            EnemyKind::Fly => (10.0, 6.0),
            EnemyKind::Worm => (10.0, 4.0),
        };
        Hurtbox {
            center: (self.x, self.y - y_offset - self.z),
            radius,
            band: self.band,
            is_player: false,
        }
    }

    pub fn get_shadow_caster(&self) -> ShadowCaster {
        ShadowCaster {
            x: self.x,
            y: self.y,
            width: 12.0,
            height: 14.0,
        }
    }

    pub fn take_damage(
        &mut self,
        damage: i32,
        knockback: (f32, f32),
        hit_stop: u32,
        status: Option<StatusEffectKind>,
        floating_texts: &mut Vec<FloatingText>,
        particles: &mut ParticleSystem,
    ) {
        if self.state == EnemyState::Dead {
            return;
        }

        self.hp = (self.hp - damage).max(0);
        self.hit_stop = hit_stop;
        self.flinch_timer = 0.22;
        self.state = EnemyState::Flinch { timer: 0.22 };

        // Apply knockback
        self.vx = knockback.0;
        self.vy = knockback.1;

        // Floating damage number
        let text_col = if damage >= 5 {
            Color::rgb(255, 70, 70) // Critical / Heavy
        } else {
            Palette::TEXT_NORMAL
        };
        floating_texts.push(FloatingText::new(
            self.x,
            self.y - self.z - 16.0,
            format!("-{}", damage),
            text_col,
            0.8,
        ));

        // Spark particles
        for _ in 0..4 {
            particles.spawn_spark(self.x, self.y - self.z - 8.0, knockback.0 * 0.4, knockback.1 * 0.4);
        }

        // Apply status effect
        if let Some(effect) = status {
            match effect {
                StatusEffectKind::Poison => {
                    self.poison_timer = 6.0;
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - self.z - 22.0,
                        "POISON",
                        Color::rgb(100, 230, 80),
                        0.9,
                    ));
                }
                StatusEffectKind::Acid => {
                    self.hp = (self.hp - 1).max(0);
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - self.z - 22.0,
                        "ACID",
                        Color::rgb(220, 160, 40),
                        0.9,
                    ));
                }
                StatusEffectKind::Slow => {
                    self.slow_timer = 4.0;
                    floating_texts.push(FloatingText::new(
                        self.x,
                        self.y - self.z - 22.0,
                        "SLOW",
                        Color::rgb(80, 180, 255),
                        0.9,
                    ));
                }
            }
        }

        if self.hp <= 0 {
            self.state = EnemyState::Dead;
            self.z = 0.0;
            self.band = HeightBand::Low;
            floating_texts.push(FloatingText::new(
                self.x,
                self.y - 12.0,
                format!("+{} SOULS", self.souls),
                Palette::BONFIRE_ORANGE,
                1.4,
            ));
            for _ in 0..8 {
                particles.spawn_spark(self.x, self.y - 6.0, 0.0, -12.0);
            }
        }
    }

    pub fn apply_slow(&mut self, duration: f32) {
        self.slow_timer = duration;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: (f32, f32),
        world: &World,
        projectiles: &mut Vec<Projectile>,
        puddles: &mut Vec<HazardPuddle>,
        floating_texts: &mut Vec<FloatingText>,
        particles: &mut ParticleSystem,
        combat_data: &CombatData,
    ) {
        // Hit-stop freezes logic
        if self.hit_stop > 0 {
            self.hit_stop -= 1;
            return;
        }

        // Dead corpse logic
        if self.state == EnemyState::Dead {
            self.death_timer += dt;
            self.vx *= 0.85;
            self.vy *= 0.85;
            return;
        }

        // Status effects: Poison DoT
        if self.poison_timer > 0.0 {
            self.poison_timer -= dt;
            self.poison_tick_timer += dt;
            if self.poison_tick_timer >= 1.5 {
                self.poison_tick_timer = 0.0;
                self.hp = (self.hp - 1).max(0);
                floating_texts.push(FloatingText::new(
                    self.x,
                    self.y - self.z - 14.0,
                    "-1",
                    Color::rgb(80, 220, 80),
                    0.6,
                ));
                if self.hp <= 0 {
                    self.state = EnemyState::Dead;
                    self.z = 0.0;
                    self.band = HeightBand::Low;
                    return;
                }
            }
        }

        // Slow effect
        let speed_mult = if self.slow_timer > 0.0 {
            self.slow_timer -= dt;
            0.5
        } else {
            1.0
        };

        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        let dx = player_pos.0 - self.x;
        let dy = player_pos.1 - self.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.1);
        self.flip_x = dx < 0.0;

        match self.state {
            EnemyState::Flinch { mut timer } => {
                timer -= dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;
                self.vx *= 0.88;
                self.vy *= 0.88;
                if timer <= 0.0 {
                    self.state = EnemyState::Chase;
                } else {
                    self.state = EnemyState::Flinch { timer };
                }
            }
            EnemyState::Idle { mut timer } => {
                timer -= dt;
                if timer <= 0.0 || dist < 140.0 {
                    self.state = EnemyState::Chase;
                } else {
                    self.state = EnemyState::Idle { timer };
                }
            }
            EnemyState::Chase => {
                self.anim_timer += dt * 6.0;
                let cur_speed = self.move_speed * speed_mult;

                match self.kind {
                    EnemyKind::Sludge => {
                        // Advance towards player
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        self.vx = dir_x * cur_speed;
                        self.vy = dir_y * cur_speed;

                        // Vomit attack trigger
                        if dist <= combat_data.enemies.sludge.attack_reach && self.attack_cooldown <= 0.0 {
                            self.state = EnemyState::AttackWindup {
                                timer: 0.45,
                                max_timer: 0.45,
                            };
                            self.vx = 0.0;
                            self.vy = 0.0;
                        }
                    }
                    EnemyKind::Skeleton => {
                        let kite_dist = combat_data.enemies.skeleton.kite_distance;
                        if dist < kite_dist {
                            // Kite away from player
                            let dir_x = -dx / dist;
                            let dir_y = -dy / dist;
                            self.vx = dir_x * cur_speed * 1.1;
                            self.vy = dir_y * cur_speed * 1.1;
                        } else if dist > 110.0 {
                            // Approach player
                            let dir_x = dx / dist;
                            let dir_y = dy / dist;
                            self.vx = dir_x * cur_speed;
                            self.vy = dir_y * cur_speed;
                        } else {
                            // In firing range
                            self.vx = 0.0;
                            self.vy = 0.0;
                            if self.attack_cooldown <= 0.0 {
                                self.state = EnemyState::AttackWindup {
                                    timer: 0.55,
                                    max_timer: 0.55,
                                };
                            }
                        }
                    }
                    EnemyKind::Fly => {
                        // High band erratic flight
                        self.z = combat_data.enemies.fly.hover_height;
                        self.band = HeightBand::High;

                        let wobble_x = (self.anim_timer * 3.5).cos() * 22.0;
                        let wobble_y = (self.anim_timer * 2.8).sin() * 22.0;
                        let target_x = player_pos.0 + wobble_x;
                        let target_y = player_pos.1 + wobble_y;
                        let t_dx = target_x - self.x;
                        let t_dy = target_y - self.y;
                        let t_dist = (t_dx * t_dx + t_dy * t_dy).sqrt().max(1.0);

                        self.vx = (t_dx / t_dist) * cur_speed;
                        self.vy = (t_dy / t_dist) * cur_speed;

                        if dist < 65.0 && self.attack_cooldown <= 0.0 {
                            self.state = EnemyState::AttackWindup {
                                timer: 0.35,
                                max_timer: 0.35,
                            };
                        }
                    }
                    EnemyKind::Worm => {
                        // Worm swarming crawl
                        let dir_x = dx / dist;
                        let dir_y = dy / dist;
                        self.vx = dir_x * cur_speed;
                        self.vy = dir_y * cur_speed;

                        if dist <= combat_data.enemies.worm.lunge_reach && self.attack_cooldown <= 0.0 {
                            self.state = EnemyState::AttackWindup {
                                timer: 0.25,
                                max_timer: 0.25,
                            };
                        }
                    }
                }

                // Apply movement & collision with arena walls
                let move_x = self.vx * dt;
                let move_y = self.vy * dt;
                let col_test = crate::game::world::Collider::new(self.x + move_x - 5.0, self.y + move_y - 3.0, 10.0, 6.0);
                if !world.check_collision(&col_test) {
                    self.x += move_x;
                    self.y += move_y;
                }
            }
            EnemyState::AttackWindup { mut timer, max_timer } => {
                timer -= dt;
                self.anim_timer += dt * 4.0;
                if timer <= 0.0 {
                    // Execute attack
                    match self.kind {
                        EnemyKind::Sludge => {
                            // Vomit acid projectile + acid puddle
                            let dir_x = dx / dist;
                            let dir_y = dy / dist;
                            particles.spawn_spark(self.x, self.y - 4.0, dir_x * 20.0, dir_y * 20.0);
                            projectiles.push(Projectile {
                                x: self.x,
                                y: self.y - 4.0,
                                vx: dir_x * 90.0,
                                vy: dir_y * 90.0,
                                band: HeightBand::Low,
                                damage: combat_data.enemies.sludge.vomit_damage,
                                lifespan: 0.6,
                                max_lifespan: 0.6,
                                is_player: false,
                                status: Some(StatusEffectKind::Acid),
                                radius: 3.5,
                                color: Color::rgb(140, 220, 50),
                                emissive: false,
                                is_lobbed: false,
                                lob_timer: 0.0,
                                lob_duration: 0.0,
                                target_pos: (0.0, 0.0),
                            });
                            // Drop acid puddle at current position
                            puddles.push(HazardPuddle::new(self.x + dir_x * 12.0, self.y + dir_y * 12.0, 18.0, 2.0, 3.5, true));
                            self.attack_cooldown = combat_data.enemies.sludge.vomit_cooldown;
                            self.state = EnemyState::Recovery { timer: 0.4 };
                        }
                        EnemyKind::Skeleton => {
                            // Shoot poison arrow
                            let dir_x = dx / dist;
                            let dir_y = dy / dist;
                            projectiles.push(Projectile {
                                x: self.x,
                                y: self.y - 12.0,
                                vx: dir_x * combat_data.enemies.skeleton.arrow_speed,
                                vy: dir_y * combat_data.enemies.skeleton.arrow_speed,
                                band: HeightBand::Mid,
                                damage: combat_data.enemies.skeleton.arrow_damage,
                                lifespan: 0.9,
                                max_lifespan: 0.9,
                                is_player: false,
                                status: Some(StatusEffectKind::Poison),
                                radius: 2.5,
                                color: Color::rgb(141, 255, 102),
                                emissive: true,
                                is_lobbed: false,
                                lob_timer: 0.0,
                                lob_duration: 0.0,
                                target_pos: (0.0, 0.0),
                            });
                            self.attack_cooldown = combat_data.enemies.skeleton.attack_cooldown;
                            self.state = EnemyState::Recovery { timer: 0.35 };
                        }
                        EnemyKind::Fly => {
                            // Dive-bite: lower height to Mid band and dash towards player
                            self.z = 2.0;
                            self.band = HeightBand::Mid;
                            let dir_x = dx / dist;
                            let dir_y = dy / dist;
                            self.vx = dir_x * 120.0;
                            self.vy = dir_y * 120.0;
                            self.state = EnemyState::Attacking { timer: 0.3 };
                        }
                        EnemyKind::Worm => {
                            // Lunge forward
                            let dir_x = dx / dist;
                            let dir_y = dy / dist;
                            self.vx = dir_x * 95.0;
                            self.vy = dir_y * 95.0;
                            self.state = EnemyState::Attacking { timer: 0.25 };
                        }
                    }
                } else {
                    self.state = EnemyState::AttackWindup { timer, max_timer };
                }
            }
            EnemyState::Attacking { mut timer } => {
                timer -= dt;
                self.x += self.vx * dt;
                self.y += self.vy * dt;
                if timer <= 0.0 {
                    if self.kind == EnemyKind::Fly {
                        self.z = combat_data.enemies.fly.hover_height;
                        self.band = HeightBand::High;
                        self.attack_cooldown = combat_data.enemies.fly.attack_cooldown;
                    } else if self.kind == EnemyKind::Worm {
                        self.attack_cooldown = combat_data.enemies.worm.attack_cooldown;
                    }
                    self.state = EnemyState::Recovery { timer: 0.3 };
                } else {
                    self.state = EnemyState::Attacking { timer };
                }
            }
            EnemyState::Recovery { mut timer } => {
                timer -= dt;
                self.vx *= 0.85;
                self.vy *= 0.85;
                if timer <= 0.0 {
                    self.state = EnemyState::Chase;
                } else {
                    self.state = EnemyState::Recovery { timer };
                }
            }
            EnemyState::Dead => {}
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, bank: &SpriteBank, cam_x: i32, cam_y: i32) {
        if self.is_corpse_consumed {
            return;
        }

        let sx = (self.x - cam_x as f32).round() as i32;
        let ground_sy = (self.y - cam_y as f32).round() as i32;
        let spr_sy = ground_sy - (self.z.round() as i32);

        // Ground shadow
        match self.kind {
            EnemyKind::Fly => {
                // Dim ground shadow under aerial fly
                fb.draw_ellipse(sx, ground_sy, 5, 2, Color::rgba(5, 8, 14, 110));
            }
            EnemyKind::Sludge => {
                fb.draw_ellipse(sx, ground_sy, 11, 4, Color::rgba(10, 20, 10, 130));
            }
            EnemyKind::Skeleton => {
                fb.draw_ellipse(sx, ground_sy, 8, 3, Color::rgba(8, 10, 16, 130));
            }
            EnemyKind::Worm => {
                fb.draw_ellipse(sx, ground_sy, 6, 2, Color::rgba(12, 10, 14, 120));
            }
        }

        // Draw corpse
        if self.state == EnemyState::Dead {
            match self.kind {
                EnemyKind::Sludge => {
                    let death_frame = &bank.sludge_death[2];
                    death_frame.draw_alpha(fb, sx - 13, ground_sy - 12, self.flip_x, 180);
                }
                EnemyKind::Skeleton => {
                    bank.skeleton_death.draw_alpha(fb, sx - 8, ground_sy - 16, self.flip_x, 180);
                }
                EnemyKind::Fly => {
                    bank.fly_death.draw_alpha(fb, sx - 6, ground_sy - 8, self.flip_x, 160);
                }
                EnemyKind::Worm => {
                    bank.worm_death.draw_alpha(fb, sx - 6, ground_sy - 7, self.flip_x, 160);
                }
            }
            return;
        }

        // Flinch blink
        if matches!(self.state, EnemyState::Flinch { .. }) && ((self.flinch_timer * 30.0) as usize).is_multiple_of(2) {
            // Blink frame
            return;
        }

        // Draw living enemy
        match self.kind {
            EnemyKind::Sludge => {
                let frame = match self.state {
                    EnemyState::AttackWindup { .. } => &bank.sludge_vomit_windup,
                    EnemyState::Attacking { .. } => &bank.sludge_vomit_spit,
                    EnemyState::Flinch { .. } => &bank.sludge_hit,
                    EnemyState::Chase => &bank.sludge_walk[(self.anim_timer as usize) % 4],
                    _ => &bank.sludge_idle[(self.anim_timer as usize) % 4],
                };
                frame.draw(fb, sx - 13, spr_sy - 16, self.flip_x);
            }
            EnemyKind::Skeleton => {
                let spr = &enemy_sprites::SKELETON;
                let frame_idx = match self.state {
                    EnemyState::AttackWindup { .. } => 2, // Drawn bow with poison glow
                    EnemyState::Flinch { .. } => 1,
                    EnemyState::Chase => (self.anim_timer as usize) % 2,
                    _ => 0,
                };
                spr.draw_frame(frame_idx, sx, spr_sy, self.flip_x, |px, py, rgb, emissive| {
                    let col = Color::rgb(rgb[0], rgb[1], rgb[2]);
                    if emissive {
                        fb.set_pixel_add(px, py, col);
                    } else {
                        fb.set_pixel(px, py, col);
                    }
                });
            }
            EnemyKind::Fly => {
                let spr = &enemy_sprites::FLY;
                let frame_idx = (self.anim_timer * 12.0) as usize % 2; // Wings flutter rapidly
                spr.draw_frame(frame_idx, sx, spr_sy, self.flip_x, |px, py, rgb, _emissive| {
                    fb.set_pixel(px, py, Color::rgb(rgb[0], rgb[1], rgb[2]));
                });
            }
            EnemyKind::Worm => {
                let spr = &enemy_sprites::WORM;
                let frame_idx = match self.state {
                    EnemyState::AttackWindup { .. } | EnemyState::Attacking { .. } => 2,
                    _ => (self.anim_timer * 7.0) as usize % 3,
                };
                spr.draw_frame(frame_idx, sx, spr_sy, self.flip_x, |px, py, rgb, _emissive| {
                    fb.set_pixel(px, py, Color::rgb(rgb[0], rgb[1], rgb[2]));
                });
            }
        }

        // Draw floating HP bar if damaged
        if self.hp < self.max_hp && self.hp > 0 {
            let bar_w = 14;
            let bar_h = 2;
            let bx = sx - bar_w / 2;
            let by = spr_sy - 22;
            fb.draw_rect(bx - 1, by - 1, bar_w + 2, bar_h + 2, Color::rgb(20, 20, 25), true);
            let fill_w = ((self.hp as f32 / self.max_hp as f32) * bar_w as f32).ceil() as i32;
            fb.draw_rect(bx, by, fill_w.max(1), bar_h, Color::rgb(220, 50, 50), true);
        }
    }
}
