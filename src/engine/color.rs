use std::fmt::Write;

/// Represents an 8-bit RGBA color with alpha blending and formatting utilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    pub const fn from_hex_alpha(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    /// Alpha composite `self` (source) over `background` (destination).
    #[inline]
    pub fn blend_over(self, bg: Color) -> Color {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return bg;
        }

        let sa = self.a as u32;
        let da = bg.a as u32;
        let inv_sa = 255 - sa;

        // Output alpha
        let out_a = sa + (da * inv_sa + 127) / 255;
        if out_a == 0 {
            return Color::TRANSPARENT;
        }

        let out_r = (self.r as u32 * sa + bg.r as u32 * da * inv_sa / 255) / out_a;
        let out_g = (self.g as u32 * sa + bg.g as u32 * da * inv_sa / 255) / out_a;
        let out_b = (self.b as u32 * sa + bg.b as u32 * da * inv_sa / 255) / out_a;

        Color {
            r: out_r.min(255) as u8,
            g: out_g.min(255) as u8,
            b: out_b.min(255) as u8,
            a: out_a.min(255) as u8,
        }
    }

    /// Additive blend: adds components saturated at 255, weighted by source alpha.
    #[inline]
    pub fn blend_add(self, other: Color) -> Color {
        let factor = self.a as f32 / 255.0;
        Color {
            r: (other.r as u16 + (self.r as f32 * factor) as u16).min(255) as u8,
            g: (other.g as u16 + (self.g as f32 * factor) as u16).min(255) as u8,
            b: (other.b as u16 + (self.b as f32 * factor) as u16).min(255) as u8,
            a: other.a.max(self.a),
        }
    }

    /// Multiply blend: scales color channels, useful for ambient light maps and shadows.
    #[inline]
    pub fn blend_multiply(self, other: Color) -> Color {
        Color {
            r: ((self.r as u32 * other.r as u32) / 255) as u8,
            g: ((self.g as u32 * other.g as u32) / 255) as u8,
            b: ((self.b as u32 * other.b as u32) / 255) as u8,
            a: ((self.a as u32 * other.a as u32) / 255) as u8,
        }
    }

    /// Linear interpolation between two colors.
    #[inline]
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;
        Color {
            r: (self.r as f32 * inv_t + other.r as f32 * t).round() as u8,
            g: (self.g as f32 * inv_t + other.g as f32 * t).round() as u8,
            b: (self.b as f32 * inv_t + other.b as f32 * t).round() as u8,
            a: (self.a as f32 * inv_t + other.a as f32 * t).round() as u8,
        }
    }

    /// Scale RGB brightness by a factor in [0.0, inf).
    #[inline]
    pub fn scale(self, factor: f32) -> Color {
        let factor = factor.max(0.0);
        Color {
            r: (self.r as f32 * factor).min(255.0).round() as u8,
            g: (self.g as f32 * factor).min(255.0).round() as u8,
            b: (self.b as f32 * factor).min(255.0).round() as u8,
            a: self.a,
        }
    }

    /// Write ANSI escape sequence for foreground color: `\x1b[38;2;R;G;Bm`
    #[inline]
    pub fn write_ansi_fg<W: Write>(&self, out: &mut W) {
        let _ = write!(out, "\x1b[38;2;{};{};{}m", self.r, self.g, self.b);
    }

    /// Write ANSI escape sequence for background color: `\x1b[48;2;R;G;Bm`
    #[inline]
    pub fn write_ansi_bg<W: Write>(&self, out: &mut W) {
        let _ = write!(out, "\x1b[48;2;{};{};{}m", self.r, self.g, self.b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let col = Color::from_hex(0xFF8800);
        assert_eq!(col.r, 255);
        assert_eq!(col.g, 136);
        assert_eq!(col.b, 0);
        assert_eq!(col.a, 255);
    }

    #[test]
    fn test_alpha_composite() {
        let bg = Color::rgb(0, 0, 0);
        let semi_white = Color::rgba(255, 255, 255, 128);
        let out = semi_white.blend_over(bg);
        assert!(out.r > 120 && out.r < 136);
    }

    #[test]
    fn test_color_lerp() {
        let c1 = Color::rgb(0, 0, 0);
        let c2 = Color::rgb(100, 200, 50);
        let mid = c1.lerp(c2, 0.5);
        assert_eq!(mid.r, 50);
        assert_eq!(mid.g, 100);
        assert_eq!(mid.b, 25);
    }
}

