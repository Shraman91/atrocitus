use crate::assets::palette::Palette;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::text::{FontSize, TextRenderer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Default,
    Interact(&'static str),
}

pub struct CursorRenderer;

impl CursorRenderer {
    pub fn draw(
        fb: &mut Framebuffer,
        mx: f32,
        my: f32,
        style: CursorStyle,
        time: f32,
        mouse_detected: bool,
    ) {
        let ix = mx.round() as i32;
        let iy = my.round() as i32;

        if !mouse_detected {
            // One-time fallback reminder if no mouse events have been received
            if time < 10.0 {
                let msg = "MOUSE NOT DETECTED: USE WASD TO MOVE";
                let w = TextRenderer::measure_width(msg, FontSize::Tiny3x5);
                let x = (fb.width as i32 - w) / 2;
                let y = fb.height as i32 - 10;
                fb.draw_rect(x - 2, y - 1, w + 4, 7, Palette::UI_PANEL_BG, true);
                TextRenderer::draw_text(fb, x, y, msg, FontSize::Tiny3x5, Palette::TEXT_MUTED);
            }
            return;
        }

        if !fb.in_bounds(ix, iy) {
            return;
        }

        let glow = ((time * 8.0).sin() * 0.2 + 0.8).clamp(0.0, 1.0);
        let ember_core = Palette::BONFIRE_CORE;
        let ember_rim = Palette::BONFIRE_ORANGE.scale(glow);

        match style {
            CursorStyle::Default => {
                // Diamond pixel reticle
                fb.set_pixel_add(ix, iy - 2, ember_rim);
                fb.set_pixel_add(ix - 1, iy - 1, ember_rim);
                fb.set_pixel_add(ix + 1, iy - 1, ember_rim);
                fb.set_pixel_add(ix - 2, iy, ember_rim);
                fb.set_pixel(ix, iy, ember_core);
                fb.set_pixel_add(ix + 2, iy, ember_rim);
                fb.set_pixel_add(ix - 1, iy + 1, ember_rim);
                fb.set_pixel_add(ix + 1, iy + 1, ember_rim);
                fb.set_pixel_add(ix, iy + 2, ember_rim);
            }
            CursorStyle::Interact(prompt) => {
                // Interactive diamond icon with pulsing core
                let interact_gold = Palette::TEXT_GOLD;
                fb.set_pixel(ix, iy - 3, interact_gold);
                fb.set_pixel(ix - 1, iy - 2, interact_gold);
                fb.set_pixel(ix + 1, iy - 2, interact_gold);
                fb.set_pixel(ix - 2, iy - 1, interact_gold);
                fb.set_pixel(ix + 2, iy - 1, interact_gold);
                fb.set_pixel(ix - 3, iy, interact_gold);
                fb.set_pixel(ix, iy, ember_core);
                fb.set_pixel(ix + 3, iy, interact_gold);
                fb.set_pixel(ix - 2, iy + 1, interact_gold);
                fb.set_pixel(ix + 2, iy + 1, interact_gold);
                fb.set_pixel(ix - 1, iy + 2, interact_gold);
                fb.set_pixel(ix + 1, iy + 2, interact_gold);
                fb.set_pixel(ix, iy + 3, interact_gold);

                // Tooltip prompt next to cursor
                let tip_w = TextRenderer::measure_width(prompt, FontSize::Tiny3x5);
                let tx = (ix + 6).min(fb.width as i32 - tip_w - 4);
                let ty = (iy - 8).max(4);

                fb.draw_rect(tx - 2, ty - 1, tip_w + 4, 7, Palette::UI_PANEL_BG, true);
                fb.draw_rect(tx - 2, ty - 1, tip_w + 4, 7, Palette::UI_BORDER, false);
                TextRenderer::draw_text(fb, tx, ty, prompt, FontSize::Tiny3x5, Palette::TEXT_GOLD);
            }
        }
    }

    /// Renders a pulsing click destination ground ring that fades over time.
    pub fn draw_destination_marker(fb: &mut Framebuffer, wx: f32, wy: f32, cam_x: f32, cam_y: f32, life: f32) {
        if life <= 0.0 {
            return;
        }
        let sx = (wx - cam_x).round() as i32;
        let sy = (wy - cam_y).round() as i32;

        if !fb.in_bounds(sx, sy) {
            return;
        }

        let radius = ((1.0 - life) * 6.0 + 2.0) as i32;
        let alpha = (life * 200.0).clamp(0.0, 255.0) as u8;
        let ring_col = Color::rgba(Palette::BONFIRE_YELLOW.r, Palette::BONFIRE_YELLOW.g, Palette::BONFIRE_YELLOW.b, alpha);

        fb.draw_circle(sx, sy, radius, ring_col, false);
        fb.set_pixel_add(sx, sy, Palette::BONFIRE_CORE.scale(life));
    }
}
