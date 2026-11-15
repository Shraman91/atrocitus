use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::text::{FontSize, TextRenderer};

/// 2.5D Height Bands for vertical attack and hurtbox filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeightBand {
    Low,  // Ground crawlers, acid puddles, worms
    Mid,  // Standard humanoid standing height (player, skeletons)
    High, // Flying enemies (carrion flies), rising slashes, high gunshots
    All,  // Explosions / novas affecting all vertical levels
}

impl HeightBand {
    /// Determines whether an attack originating in this band can hit an entity hurtbox in `target`.
    pub fn hits(&self, target: HeightBand) -> bool {
        match self {
            HeightBand::All => true,
            HeightBand::High => target == HeightBand::High || target == HeightBand::Mid,
            HeightBand::Mid => target == HeightBand::Mid || target == HeightBand::Low,
            HeightBand::Low => target == HeightBand::Low,
        }
    }

    pub fn display_tag(&self) -> &'static str {
        match self {
            HeightBand::Low => "[↓Low]",
            HeightBand::Mid => "[Mid]",
            HeightBand::High => "[↑High]",
            HeightBand::All => "[*All]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffectKind {
    Poison,
    Acid,
    Slow,
}

#[derive(Debug, Clone)]
pub enum HitboxShape {
    Circle { radius: f32 },
    Arc { radius: f32, angle: f32, spread: f32 }, // spread is half-angle in radians
    Aabb { half_w: f32, half_h: f32 },
}

impl HitboxShape {
    pub fn contains_point(&self, origin: (f32, f32), point: (f32, f32), target_radius: f32) -> bool {
        let dx = point.0 - origin.0;
        let dy = point.1 - origin.1;
        let dist_sq = dx * dx + dy * dy;

        match self {
            HitboxShape::Circle { radius } => {
                let r = radius + target_radius;
                dist_sq <= r * r
            }
            HitboxShape::Arc { radius, angle, spread } => {
                let r = radius + target_radius;
                if dist_sq > r * r {
                    return false;
                }
                // If target is in point-blank melee contact (within 12px), hit reliably
                if dist_sq <= 144.0 {
                    return true;
                }
                let pt_angle = dy.atan2(dx);
                let mut diff = pt_angle - angle;
                while diff < -PI {
                    diff += 2.0 * PI;
                }
                while diff > PI {
                    diff -= 2.0 * PI;
                }
                diff.abs() <= *spread
            }
            HitboxShape::Aabb { half_w, half_h } => {
                dx.abs() <= half_w + target_radius && dy.abs() <= half_h + target_radius
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hitbox {
    pub origin: (f32, f32),
    pub shape: HitboxShape,
    pub band: HeightBand,
    pub damage: i32,
    pub knockback: f32,
    pub knockback_dir: (f32, f32),
    pub is_player: bool,
    pub hit_stop: u32,
    pub status: Option<StatusEffectKind>,
}

#[derive(Debug, Clone, Copy)]
pub struct Hurtbox {
    pub center: (f32, f32),
    pub radius: f32,
    pub band: HeightBand,
    pub is_player: bool,
}

/// Projectile in 2.5D space
#[derive(Debug, Clone)]
pub struct Projectile {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub band: HeightBand,
    pub damage: i32,
    pub lifespan: f32,
    pub max_lifespan: f32,
    pub is_player: bool,
    pub status: Option<StatusEffectKind>,
    pub radius: f32,
    pub color: Color,
    pub emissive: bool,
    pub is_lobbed: bool,
    pub lob_timer: f32,
    pub lob_duration: f32,
    pub target_pos: (f32, f32),
}

impl Projectile {
    pub fn update(&mut self, dt: f32) -> bool {
        self.lifespan -= dt;
        if self.is_lobbed {
            self.lob_timer += dt;
            let t = (self.lob_timer / self.lob_duration).clamp(0.0, 1.0);
            self.x += self.vx * dt;
            self.y += self.vy * dt;
            if t >= 1.0 || self.lifespan <= 0.0 {
                return false; // Impact ground
            }
        } else {
            self.x += self.vx * dt;
            self.y += self.vy * dt;
        }
        self.lifespan > 0.0
    }

    pub fn draw(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32) {
        let sx = (self.x - cam_x as f32).round() as i32;
        let sy = (self.y - cam_y as f32).round() as i32;

        if self.is_lobbed {
            // Arc parabolic height offset
            let t = (self.lob_timer / self.lob_duration).clamp(0.0, 1.0);
            let arc_height = (4.0 * t * (1.0 - t) * 16.0).round() as i32;
            let ground_y = sy;
            let lob_y = sy - arc_height;

            // Shadow on ground
            fb.draw_ellipse(sx, ground_y, 3, 2, Color::rgba(0, 0, 0, 100));
            // Flying vial
            fb.draw_circle(sx, lob_y, self.radius.round() as i32, self.color, true);
            fb.set_pixel(sx, lob_y, Color::rgb(220, 255, 120));
        } else {
            // Fast streak
            let trail_x = sx - (self.vx * 0.02).round() as i32;
            let trail_y = sy - (self.vy * 0.02).round() as i32;
            fb.draw_line(trail_x, trail_y, sx, sy, self.color);
            fb.draw_circle(sx, sy, self.radius.round() as i32, self.color, true);
            if self.emissive {
                fb.set_pixel(sx, sy, Color::WHITE);
            }
        }
    }
}

/// Acid / Hazard puddle on the ground plane (HeightBand::Low)
#[derive(Debug, Clone)]
pub struct HazardPuddle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub dps: f32,
    pub duration: f32,
    pub max_duration: f32,
    pub is_acid: bool,
    pub tick_timer: f32,
    pub band: HeightBand,
}

impl HazardPuddle {
    pub fn new(x: f32, y: f32, radius: f32, dps: f32, duration: f32, is_acid: bool) -> Self {
        Self {
            x,
            y,
            radius,
            dps,
            duration,
            max_duration: duration,
            is_acid,
            tick_timer: 0.0,
            band: HeightBand::Low,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.duration -= dt;
        self.tick_timer += dt;
        self.duration > 0.0
    }

    pub fn draw(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32, time: f32) {
        let sx = (self.x - cam_x as f32).round() as i32;
        let sy = (self.y - cam_y as f32).round() as i32;
        let r = self.radius.round() as i32;
        let fade = (self.duration / self.max_duration).clamp(0.0, 1.0);
        let alpha = (fade * 140.0) as u8;

        let col = if self.is_acid {
            Color::rgba(90, 180, 40, alpha)
        } else {
            Color::rgba(180, 50, 40, alpha)
        };

        fb.draw_ellipse(sx, sy, r, (r as f32 * 0.6).round() as i32, col);

        // Bubbles
        let bubble_seed = (time * 6.0 + self.x) as i32;
        for i in 0..4 {
            let bx = sx + ((i * 17 + bubble_seed) % (r * 2) - r);
            let by = sy + ((i * 13 + bubble_seed / 2) % (r) - r / 2);
            fb.set_pixel(bx, by, Color::rgb(180, 255, 90));
        }
    }
}

/// Bloodstain dropped on player death
#[derive(Debug, Clone)]
pub struct Bloodstain {
    pub x: f32,
    pub y: f32,
    pub souls: u32,
    pub active: bool,
}

impl Bloodstain {
    pub fn new(x: f32, y: f32, souls: u32) -> Self {
        Self {
            x,
            y,
            souls,
            active: true,
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32, time: f32) {
        if !self.active {
            return;
        }
        let sx = (self.x - cam_x as f32).round() as i32;
        let sy = (self.y - cam_y as f32).round() as i32;

        // Pulsing golden/blood rune
        let pulse = ((time * 3.0).sin() * 0.3 + 0.7).clamp(0.0, 1.0);
        let alpha = (pulse * 200.0) as u8;
        fb.draw_ellipse(sx, sy, 8, 4, Color::rgba(215, 160, 40, alpha));
        fb.draw_ellipse(sx, sy, 4, 2, Color::rgba(255, 230, 120, alpha));

        // Floating rune glow
        let float_y = sy - 6 - ((time * 4.0).sin() * 2.0) as i32;
        fb.set_pixel(sx, float_y, Color::rgb(255, 220, 80));
        fb.set_pixel(sx - 1, float_y + 1, Color::rgb(255, 180, 40));
        fb.set_pixel(sx + 1, float_y + 1, Color::rgb(255, 180, 40));
        fb.set_pixel(sx, float_y + 2, Color::rgb(255, 240, 140));
    }
}

/// Floating combat feedback text (damage numbers, status alerts, soul pickups)
#[derive(Debug, Clone)]
pub struct FloatingText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: Color,
    pub timer: f32,
    pub max_timer: f32,
}

impl FloatingText {
    pub fn new(x: f32, y: f32, text: impl Into<String>, color: Color, duration: f32) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            color,
            timer: duration,
            max_timer: duration,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.timer -= dt;
        self.y -= 14.0 * dt; // Float upward
        self.timer > 0.0
    }

    pub fn draw(&self, fb: &mut Framebuffer, cam_x: i32, cam_y: i32) {
        let sx = (self.x - cam_x as f32).round() as i32;
        let sy = (self.y - cam_y as f32).round() as i32;
        let w = TextRenderer::measure_width(&self.text, FontSize::Tiny3x5);
        let rx = sx - w / 2;
        TextRenderer::draw_text(fb, rx, sy, &self.text, FontSize::Tiny3x5, self.color);
    }
}

/// Screen shake manager using trauma accumulation
#[derive(Debug, Clone, Copy)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn new() -> Self {
        Self { trauma: 0.0 }
    }

    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f32) {
        self.trauma = (self.trauma - dt * 2.2).max(0.0);
    }

    pub fn get_offset(&self, time: f32) -> (f32, f32) {
        if self.trauma <= 0.001 {
            return (0.0, 0.0);
        }
        let shake = self.trauma * self.trauma;
        let max_offset = 6.0;
        let ox = (time * 45.0).sin() * max_offset * shake;
        let oy = (time * 55.0).cos() * max_offset * shake;
        (ox, oy)
    }
}

/// Zero-allocation Spatial Hash Grid for entity collision broadphase
pub const SPATIAL_CELL_SIZE: f32 = 32.0;
pub const SPATIAL_COLS: usize = 16;
pub const SPATIAL_ROWS: usize = 16;

pub struct SpatialHash {
    buckets: Vec<Vec<usize>>,
}

impl SpatialHash {
    pub fn new() -> Self {
        let mut buckets = Vec::with_capacity(SPATIAL_COLS * SPATIAL_ROWS);
        for _ in 0..(SPATIAL_COLS * SPATIAL_ROWS) {
            buckets.push(Vec::with_capacity(8));
        }
        Self { buckets }
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    #[inline]
    fn cell_index(cx: i32, cy: i32) -> Option<usize> {
        if cx >= 0 && cx < SPATIAL_COLS as i32 && cy >= 0 && cy < SPATIAL_ROWS as i32 {
            Some((cy as usize) * SPATIAL_COLS + (cx as usize))
        } else {
            None
        }
    }

    pub fn insert(&mut self, id: usize, x: f32, y: f32, radius: f32) {
        let min_cx = ((x - radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let max_cx = ((x + radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let min_cy = ((y - radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let max_cy = ((y + radius) / SPATIAL_CELL_SIZE).floor() as i32;

        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(idx) = Self::cell_index(cx, cy) {
                    self.buckets[idx].push(id);
                }
            }
        }
    }

    pub fn query_radius(&self, x: f32, y: f32, radius: f32, results: &mut Vec<usize>) {
        results.clear();
        let min_cx = ((x - radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let max_cx = ((x + radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let min_cy = ((y - radius) / SPATIAL_CELL_SIZE).floor() as i32;
        let max_cy = ((y + radius) / SPATIAL_CELL_SIZE).floor() as i32;

        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(idx) = Self::cell_index(cx, cy) {
                    for &id in &self.buckets[idx] {
                        if !results.contains(&id) {
                            results.push(id);
                        }
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------
// Data-Driven Combat Configurations (data/combat.ron)
// -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatData {
    pub player: PlayerCombatStats,
    pub knight: KnightConfig,
    pub mage: MageConfig,
    pub assassin: AssassinConfig,
    pub hunter: HunterConfig,
    pub alchemist: AlchemistConfig,
    pub enemies: EnemyConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCombatStats {
    pub max_health: i32,
    pub initial_souls: u32,
    pub max_mana: f32,
    pub mana_regen_rate: f32,
    pub post_damage_iframes: f32,
    pub corpse_channel_time: f32,
    pub corpse_mana_restore: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnightConfig {
    pub combo1_damage: i32,
    pub combo1_reach: f32,
    pub combo2_damage: i32,
    pub combo2_reach: f32,
    pub combo3_damage: i32,
    pub combo3_reach: f32,
    pub combo3_knockback: f32,
    pub shield_damage_reduction: f32,
    pub shield_move_speed_mult: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MageConfig {
    pub bolt_damage: i32,
    pub bolt_speed: f32,
    pub bolt_range: f32,
    pub bolt_mana_cost: f32,
    pub nova_damage: i32,
    pub nova_radius: f32,
    pub nova_knockback: f32,
    pub nova_mana_cost: f32,
    pub nova_cooldown: f32,
    pub nova_charge_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssassinConfig {
    pub combo1_damage: i32,
    pub combo2_damage: i32,
    pub combo3_damage: i32,
    pub combo_reach: f32,
    pub shadowstep_distance: f32,
    pub shadowstep_cooldown: f32,
    pub backstab_multiplier: f32,
    pub backstab_window: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HunterConfig {
    pub blade_damage: i32,
    pub blade_reach: f32,
    pub flintlock_damage: i32,
    pub flintlock_speed: f32,
    pub flintlock_range: f32,
    pub flintlock_recoil: f32,
    pub flintlock_cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistConfig {
    pub vial_arc_speed: f32,
    pub vial_puddle_radius: f32,
    pub vial_puddle_duration: f32,
    pub vial_puddle_dps: f32,
    pub vial_cooldown: f32,
    pub eye_distance: f32,
    pub eye_radius: f32,
    pub eye_slow_duration: f32,
    pub eye_slow_multiplier: f32,
    pub eye_cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyConfigs {
    pub sludge: SludgeConfig,
    pub skeleton: SkeletonConfig,
    pub fly: FlyConfig,
    pub worm: WormConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SludgeConfig {
    pub max_hp: i32,
    pub souls: u32,
    pub move_speed: f32,
    pub contact_damage: i32,
    pub attack_reach: f32,
    pub vomit_damage: i32,
    pub vomit_cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonConfig {
    pub max_hp: i32,
    pub souls: u32,
    pub move_speed: f32,
    pub contact_damage: i32,
    pub arrow_damage: i32,
    pub arrow_speed: f32,
    pub arrow_range: f32,
    pub kite_distance: f32,
    pub attack_cooldown: f32,
    pub poison_duration: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlyConfig {
    pub max_hp: i32,
    pub souls: u32,
    pub move_speed: f32,
    pub contact_damage: i32,
    pub dive_damage: i32,
    pub hover_height: f32,
    pub attack_cooldown: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WormConfig {
    pub max_hp: i32,
    pub souls: u32,
    pub move_speed: f32,
    pub contact_damage: i32,
    pub lunge_damage: i32,
    pub lunge_reach: f32,
    pub attack_cooldown: f32,
}

impl Default for CombatData {
    fn default() -> Self {
        Self {
            player: PlayerCombatStats {
                max_health: 10,
                initial_souls: 100,
                max_mana: 100.0,
                mana_regen_rate: 4.0,
                post_damage_iframes: 0.6,
                corpse_channel_time: 0.8,
                corpse_mana_restore: 40.0,
            },
            knight: KnightConfig {
                combo1_damage: 2,
                combo1_reach: 34.0,
                combo2_damage: 3,
                combo2_reach: 36.0,
                combo3_damage: 5,
                combo3_reach: 42.0,
                combo3_knockback: 45.0,
                shield_damage_reduction: 0.80,
                shield_move_speed_mult: 0.50,
            },
            mage: MageConfig {
                bolt_damage: 4,
                bolt_speed: 180.0,
                bolt_range: 160.0,
                bolt_mana_cost: 5.0,
                nova_damage: 8,
                nova_radius: 46.0,
                nova_knockback: 60.0,
                nova_mana_cost: 25.0,
                nova_cooldown: 2.0,
                nova_charge_time: 0.4,
            },
            assassin: AssassinConfig {
                combo1_damage: 2,
                combo2_damage: 2,
                combo3_damage: 4,
                combo_reach: 28.0,
                shadowstep_distance: 45.0,
                shadowstep_cooldown: 3.0,
                backstab_multiplier: 3.0,
                backstab_window: 0.5,
            },
            hunter: HunterConfig {
                blade_damage: 3,
                blade_reach: 38.0,
                flintlock_damage: 6,
                flintlock_speed: 260.0,
                flintlock_range: 200.0,
                flintlock_recoil: 25.0,
                flintlock_cooldown: 1.5,
            },
            alchemist: AlchemistConfig {
                vial_arc_speed: 130.0,
                vial_puddle_radius: 24.0,
                vial_puddle_duration: 3.5,
                vial_puddle_dps: 2.0,
                vial_cooldown: 1.0,
                eye_distance: 40.0,
                eye_radius: 36.0,
                eye_slow_duration: 4.0,
                eye_slow_multiplier: 0.5,
                eye_cooldown: 4.0,
            },
            enemies: EnemyConfigs {
                sludge: SludgeConfig {
                    max_hp: 8,
                    souls: 20,
                    move_speed: 24.0,
                    contact_damage: 2,
                    attack_reach: 32.0,
                    vomit_damage: 3,
                    vomit_cooldown: 3.5,
                },
                skeleton: SkeletonConfig {
                    max_hp: 10,
                    souls: 30,
                    move_speed: 32.0,
                    contact_damage: 1,
                    arrow_damage: 2,
                    arrow_speed: 140.0,
                    arrow_range: 130.0,
                    kite_distance: 45.0,
                    attack_cooldown: 2.5,
                    poison_duration: 6.0,
                },
                fly: FlyConfig {
                    max_hp: 4,
                    souls: 15,
                    move_speed: 48.0,
                    contact_damage: 1,
                    dive_damage: 2,
                    hover_height: 14.0,
                    attack_cooldown: 2.2,
                },
                worm: WormConfig {
                    max_hp: 6,
                    souls: 10,
                    move_speed: 38.0,
                    contact_damage: 1,
                    lunge_damage: 2,
                    lunge_reach: 22.0,
                    attack_cooldown: 1.8,
                },
            },
        }
    }
}

impl CombatData {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("data/combat.ron")
            && let Ok(data) = ron::from_str::<CombatData>(&content)
        {
            return data;
        }
        Self::default()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_height_band_collision_rules() {
        // High attacks hit High (flying) and Mid (humanoid standing), but miss Low (worms/puddles)
        assert!(HeightBand::High.hits(HeightBand::High));
        assert!(HeightBand::High.hits(HeightBand::Mid));
        assert!(!HeightBand::High.hits(HeightBand::Low));

        // Mid attacks hit Mid and Low (crawlers on ground), but CANNOT hit High flying enemies
        assert!(HeightBand::Mid.hits(HeightBand::Mid));
        assert!(HeightBand::Mid.hits(HeightBand::Low));
        assert!(!HeightBand::Mid.hits(HeightBand::High));

        // Low attacks (puddles, worms) hit Low only; CANNOT hit Mid or High
        assert!(HeightBand::Low.hits(HeightBand::Low));
        assert!(!HeightBand::Low.hits(HeightBand::Mid));
        assert!(!HeightBand::Low.hits(HeightBand::High));

        // All attacks (novas, explosions) hit Low, Mid, and High
        assert!(HeightBand::All.hits(HeightBand::Low));
        assert!(HeightBand::All.hits(HeightBand::Mid));
        assert!(HeightBand::All.hits(HeightBand::High));
    }

    #[test]
    fn test_spatial_hash_queries() {
        let mut grid = SpatialHash::new();
        grid.insert(1, 50.0, 50.0, 10.0);
        grid.insert(2, 250.0, 250.0, 10.0);

        let mut results = Vec::new();
        grid.query_radius(50.0, 50.0, 15.0, &mut results);
        assert!(results.contains(&1));
        assert!(!results.contains(&2));

        grid.query_radius(250.0, 250.0, 15.0, &mut results);
        assert!(results.contains(&2));
        assert!(!results.contains(&1));

        grid.clear();
        grid.query_radius(50.0, 50.0, 50.0, &mut results);
        assert!(results.is_empty());
    }

    #[test]
    fn test_combat_data_ron_loading() {
        let data = CombatData::load();
        assert_eq!(data.player.max_health, 10);
        assert_eq!(data.player.initial_souls, 100);
        assert_eq!(data.player.max_mana, 100.0);
        assert_eq!(data.knight.combo1_damage, 2);
        assert_eq!(data.knight.combo2_damage, 3);
        assert_eq!(data.knight.combo3_damage, 5);
        assert_eq!(data.mage.nova_damage, 8);
        assert_eq!(data.assassin.backstab_multiplier, 3.0);
        assert_eq!(data.hunter.flintlock_damage, 6);
        assert_eq!(data.alchemist.vial_puddle_duration, 3.5);

        assert_eq!(data.enemies.sludge.max_hp, 8);
        assert_eq!(data.enemies.skeleton.max_hp, 10);
        assert_eq!(data.enemies.fly.max_hp, 4);
        assert_eq!(data.enemies.worm.max_hp, 6);
    }

    #[test]
    fn test_hitbox_shape_arc_and_circle() {
        let arc = HitboxShape::Arc {
            radius: 25.0,
            angle: 0.0, // Facing East (+X)
            spread: std::f32::consts::FRAC_PI_4, // 45 degrees spread
        };
        // Point directly in front
        assert!(arc.contains_point((100.0, 100.0), (120.0, 100.0), 4.0));
        // Point behind
        assert!(!arc.contains_point((100.0, 100.0), (80.0, 100.0), 4.0));
        // Point too far
        assert!(!arc.contains_point((100.0, 100.0), (150.0, 100.0), 4.0));

        let circle = HitboxShape::Circle { radius: 30.0 };
        assert!(circle.contains_point((100.0, 100.0), (110.0, 110.0), 2.0));
        assert!(!circle.contains_point((100.0, 100.0), (150.0, 150.0), 2.0));
    }
}
