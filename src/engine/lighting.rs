use super::color::Color;
use super::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub intensity: f32,
    pub color: Color,
    pub flicker_amount: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ShadowCaster {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub struct LightingEngine {
    pub ambient: Color,
}

impl Default for LightingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LightingEngine {
    pub fn new() -> Self {
        Self {
            ambient: Color::rgb(18, 22, 32),
        }
    }

    /// Renders soft ground shadows cast away from the primary light source.
    pub fn render_shadows(
        &self,
        fb: &mut Framebuffer,
        light: &PointLight,
        casters: &[ShadowCaster],
        cam_x: i32,
        cam_y: i32,
    ) {
        let shadow_darkness = Color::rgba(5, 6, 10, 140);

        for caster in casters {
            let scr_cx = caster.x as i32 - cam_x;
            let scr_cy = caster.y as i32 - cam_y;
            let scr_lx = light.x as i32 - cam_x;
            let scr_ly = light.y as i32 - cam_y;

            let dx = (scr_cx - scr_lx) as f32;
            let dy = (scr_cy - scr_ly) as f32;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);

            let norm_dx = dx / dist;
            let norm_dy = dy / dist;

            // Shadow ellipse offset away from light
            let shadow_len = (dist * 0.15).clamp(3.0, 14.0);
            let shadow_x = scr_cx + (norm_dx * shadow_len) as i32;
            let shadow_y = scr_cy + (norm_dy * shadow_len * 0.5) as i32;

            let radius_x = (caster.width * 0.5).max(3.0) as i32;
            let radius_y = (caster.height * 0.25).max(2.0) as i32;

            // Draw soft ground shadow ellipse
            for y_off in -radius_y..=radius_y {
                let sy = shadow_y + y_off;
                if sy < 0 || sy >= fb.height as i32 {
                    continue;
                }
                let norm_y = y_off as f32 / radius_y as f32;
                let max_x = ((1.0 - norm_y * norm_y).max(0.0).sqrt() * radius_x as f32).round() as i32;

                let row_start = sy as usize * fb.width;
                for x_off in -max_x..=max_x {
                    let sx = shadow_x + x_off;
                    if sx >= 0 && sx < fb.width as i32 {
                        let idx = row_start + sx as usize;
                        fb.pixels[idx] = shadow_darkness.blend_over(fb.pixels[idx]);
                    }
                }
            }
        }
    }

    /// Applies dynamic radial lighting and ambient illumination pass across the framebuffer.
    pub fn apply_lighting(
        &self,
        fb: &mut Framebuffer,
        lights: &[PointLight],
        cam_x: i32,
        cam_y: i32,
        time_secs: f32,
    ) {
        let w = fb.width;
        let h = fb.height;

        for y in 0..h {
            let scr_y = y as i32;
            let row_offset = y * w;

            for x in 0..w {
                let scr_x = x as i32;
                let idx = row_offset + x;
                let pixel = fb.pixels[idx];

                // Calculate cumulative light at this screen pixel
                let mut lit_r = (pixel.r as f32 * self.ambient.r as f32) / 255.0;
                let mut lit_g = (pixel.g as f32 * self.ambient.g as f32) / 255.0;
                let mut lit_b = (pixel.b as f32 * self.ambient.b as f32) / 255.0;

                for light in lights {
                    let lx = light.x - cam_x as f32;
                    let ly = light.y - cam_y as f32;

                    let dx = scr_x as f32 - lx;
                    let dy = (scr_y as f32 - ly) * 1.4; // 2.5D vertical perspective flattening
                    let dist2 = dx * dx + dy * dy;

                    // Light flicker based on sine harmonics
                    let flicker = 1.0
                        + light.flicker_amount
                            * ((time_secs * 14.0 + light.x * 0.1).sin() * 0.5
                                + (time_secs * 23.0 + light.y * 0.1).cos() * 0.3
                                + (time_secs * 37.0).sin() * 0.2);

                    let eff_radius = light.radius * flicker;
                    let eff_radius2 = eff_radius * eff_radius;

                    if dist2 < eff_radius2 {
                        let dist = dist2.sqrt();
                        let falloff = (1.0 - (dist / eff_radius)).clamp(0.0, 1.0).powf(1.6);
                        let factor = falloff * light.intensity;

                        lit_r += pixel.r as f32 * (light.color.r as f32 / 255.0) * factor * 1.5;
                        lit_g += pixel.g as f32 * (light.color.g as f32 / 255.0) * factor * 1.5;
                        lit_b += pixel.b as f32 * (light.color.b as f32 / 255.0) * factor * 1.5;
                    }
                }

                fb.pixels[idx] = Color {
                    r: lit_r.min(255.0) as u8,
                    g: lit_g.min(255.0) as u8,
                    b: lit_b.min(255.0) as u8,
                    a: pixel.a,
                };
            }
        }
    }
}
