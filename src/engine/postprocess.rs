use super::color::Color;
use super::framebuffer::Framebuffer;

pub struct PostProcessor;

impl PostProcessor {
    /// Applies subtle circular vignette to darken edges and focus center view.
    pub fn apply_vignette(fb: &mut Framebuffer, strength: f32) {
        let w = fb.width as f32;
        let h = fb.height as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for y in 0..fb.height {
            let row_offset = y * fb.width;
            let dy = y as f32 - cy;

            for x in 0..fb.width {
                let dx = x as f32 - cx;
                let dist = (dx * dx + dy * dy).sqrt();
                let norm = (dist / max_dist).clamp(0.0, 1.0);
                let factor = 1.0 - (norm.powf(2.2) * strength).clamp(0.0, 0.85);

                let idx = row_offset + x;
                let p = fb.pixels[idx];
                fb.pixels[idx] = Color {
                    r: (p.r as f32 * factor) as u8,
                    g: (p.g as f32 * factor) as u8,
                    b: (p.b as f32 * factor) as u8,
                    a: p.a,
                };
            }
        }
    }

    /// Renders drifting ground fog bands with sine displacement.
    pub fn apply_drifting_fog(fb: &mut Framebuffer, time_secs: f32) {
        let fog_color = Color::rgb(35, 48, 68);

        for y in (fb.height / 3)..fb.height {
            let row_offset = y * fb.width;
            let y_factor = (y as f32 - fb.height as f32 / 3.0) / (fb.height as f32 * 0.67);

            for x in 0..fb.width {
                let wave1 = ((x as f32 * 0.08) + time_secs * 0.8).sin();
                let wave2 = ((x as f32 * 0.03) - time_secs * 0.4 + y as f32 * 0.1).cos();
                let density = ((wave1 + wave2) * 0.5 + 0.5).powf(2.0);

                let fog_alpha = (density * y_factor * 45.0).clamp(0.0, 50.0) as u8;
                if fog_alpha > 0 {
                    let idx = row_offset + x;
                    let fog = Color::rgba(fog_color.r, fog_color.g, fog_color.b, fog_alpha);
                    fb.pixels[idx] = fog.blend_over(fb.pixels[idx]);
                }
            }
        }
    }

    /// Adds subtle dark-fantasy film grain to prevent color banding.
    pub fn apply_film_grain(fb: &mut Framebuffer, frame_seed: u64) {
        for (i, p) in fb.pixels.iter_mut().enumerate() {
            let hash = (i as u64)
                .wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add(frame_seed)
                .rotate_left(13);
            let grain = ((hash & 0x0F) as i32) - 7; // -7 to +8 jitter

            p.r = (p.r as i32 + grain).clamp(0, 255) as u8;
            p.g = (p.g as i32 + grain).clamp(0, 255) as u8;
            p.b = (p.b as i32 + grain).clamp(0, 255) as u8;
        }
    }

    /// Applies a full-screen fade overlay with a given color and alpha (0.0 to 1.0).
    pub fn apply_fade(fb: &mut Framebuffer, color: Color, alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
        let fade_col = Color::rgba(color.r, color.g, color.b, a);

        for p in fb.pixels.iter_mut() {
            *p = fade_col.blend_over(*p);
        }
    }
}
