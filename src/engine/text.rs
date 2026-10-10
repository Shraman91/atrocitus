use crate::assets::font_data::{get_glyph_3x5, get_glyph_5x7};
use super::color::Color;
use super::framebuffer::Framebuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Tiny3x5,
    Standard5x7,
}

/// Helper functions for drawing rasterized pixel text to the framebuffer.
pub struct TextRenderer;

impl TextRenderer {
    /// Draws a single line of text in the specified pixel font.
    pub fn draw_text(
        fb: &mut Framebuffer,
        x: i32,
        y: i32,
        text: &str,
        size: FontSize,
        color: Color,
    ) {
        let mut cursor_x = x;
        for c in text.chars() {
            if c == '\n' {
                continue;
            }
            match size {
                FontSize::Tiny3x5 => {
                    if let Some(glyph) = get_glyph_3x5(c) {
                        for (row, bits) in glyph.iter().enumerate() {
                            for col in 0..3 {
                                if (bits & (1 << (2 - col))) != 0 {
                                    fb.set_pixel_blend(cursor_x + col, y + row as i32, color);
                                }
                            }
                        }
                    }
                    cursor_x += 4; // 3 width + 1 spacing
                }
                FontSize::Standard5x7 => {
                    if let Some(glyph) = get_glyph_5x7(c) {
                        for (row, bits) in glyph.iter().enumerate() {
                            for col in 0..5 {
                                if (bits & (1 << (4 - col))) != 0 {
                                    fb.set_pixel_blend(cursor_x + col, y + row as i32, color);
                                }
                            }
                        }
                    }
                    cursor_x += 6; // 5 width + 1 spacing
                }
            }
        }
    }

    /// Draws centered text horizontally at the specified y coordinate.
    pub fn draw_text_centered(
        fb: &mut Framebuffer,
        y: i32,
        text: &str,
        size: FontSize,
        color: Color,
    ) {
        let text_width = Self::measure_width(text, size);
        let x = (fb.width as i32 - text_width) / 2;
        Self::draw_text(fb, x, y, text, size, color);
    }

    /// Measures the pixel width of a string.
    pub fn measure_width(text: &str, size: FontSize) -> i32 {
        let count = text.chars().count() as i32;
        if count == 0 {
            return 0;
        }
        match size {
            FontSize::Tiny3x5 => count * 4 - 1,
            FontSize::Standard5x7 => count * 6 - 1,
        }
    }

    /// Draws a speech bubble with an outline and filled background at (x, y).
    pub fn draw_speech_bubble(
        fb: &mut Framebuffer,
        x: i32,
        y: i32,
        text: &str,
        text_color: Color,
        bg_color: Color,
        border_color: Color,
    ) {
        let text_w = Self::measure_width(text, FontSize::Tiny3x5);
        let pad_x = 3;
        let pad_y = 2;
        let bubble_w = text_w + pad_x * 2;
        let bubble_h = 5 + pad_y * 2;
        let bx = x - bubble_w / 2;
        let by = y - bubble_h - 4;

        // Background
        fb.draw_rect(bx, by, bubble_w, bubble_h, bg_color, true);
        // Border
        fb.draw_rect(bx, by, bubble_w, bubble_h, border_color, false);
        // Little pointer arrow downwards
        fb.draw_line(x - 2, by + bubble_h - 1, x, by + bubble_h + 2, border_color);
        fb.draw_line(x + 2, by + bubble_h - 1, x, by + bubble_h + 2, border_color);
        fb.set_pixel(x, by + bubble_h, bg_color);
        fb.set_pixel(x, by + bubble_h + 1, bg_color);

        // Text
        Self::draw_text(fb, bx + pad_x, by + pad_y, text, FontSize::Tiny3x5, text_color);
    }
}
