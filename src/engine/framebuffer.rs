use super::color::Color;

/// 2D grid of logical RGB pixels.
/// The logical pixel resolution is (term_width x term_height * 2).
#[derive(Debug, Clone)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![Color::BLACK; width * height],
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.pixels.resize(width * height, Color::BLACK);
    }

    #[inline]
    pub fn clear(&mut self, color: Color) {
        self.pixels.fill(color);
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height
    }

    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if self.in_bounds(x, y) {
            Some(self.pixels[y as usize * self.width + x as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            let idx = y as usize * self.width + x as usize;
            self.pixels[idx] = color;
        }
    }

    #[inline]
    pub fn set_pixel_blend(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            let idx = y as usize * self.width + x as usize;
            self.pixels[idx] = color.blend_over(self.pixels[idx]);
        }
    }

    #[inline]
    pub fn set_pixel_add(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            let idx = y as usize * self.width + x as usize;
            self.pixels[idx] = color.blend_add(self.pixels[idx]);
        }
    }

    #[inline]
    pub fn set_pixel_multiply(&mut self, x: i32, y: i32, color: Color) {
        if self.in_bounds(x, y) {
            let idx = y as usize * self.width + x as usize;
            self.pixels[idx] = self.pixels[idx].blend_multiply(color);
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Color, filled: bool) {
        if w <= 0 || h <= 0 {
            return;
        }
        if filled {
            let x_start = x.max(0);
            let x_end = (x + w).min(self.width as i32);
            let y_start = y.max(0);
            let y_end = (y + h).min(self.height as i32);

            for cy in y_start..y_end {
                let row_start = cy as usize * self.width;
                for cx in x_start..x_end {
                    let idx = row_start + cx as usize;
                    self.pixels[idx] = if color.a == 255 {
                        color
                    } else {
                        color.blend_over(self.pixels[idx])
                    };
                }
            }
        } else {
            self.draw_line(x, y, x + w - 1, y, color);
            self.draw_line(x, y + h - 1, x + w - 1, y + h - 1, color);
            self.draw_line(x, y, x, y + h - 1, color);
            self.draw_line(x + w - 1, y, x + w - 1, y + h - 1, color);
        }
    }

    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set_pixel_blend(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, r: i32, color: Color, filled: bool) {
        if r <= 0 {
            self.set_pixel_blend(cx, cy, color);
            return;
        }

        if filled {
            let y_min = (cy - r).max(0);
            let y_max = (cy + r).min(self.height as i32 - 1);
            let r2 = r * r;

            for y in y_min..=y_max {
                let dy = y - cy;
                let dx_max = ((r2 - dy * dy) as f32).sqrt().round() as i32;
                let x_min = (cx - dx_max).max(0);
                let x_max = (cx + dx_max).min(self.width as i32 - 1);

                let row_start = y as usize * self.width;
                for x in x_min..=x_max {
                    let idx = row_start + x as usize;
                    self.pixels[idx] = color.blend_over(self.pixels[idx]);
                }
            }
        } else {
            let mut x = r;
            let mut y = 0;
            let mut err = 0;

            while x >= y {
                self.set_pixel_blend(cx + x, cy + y, color);
                self.set_pixel_blend(cx + y, cy + x, color);
                self.set_pixel_blend(cx - y, cy + x, color);
                self.set_pixel_blend(cx - x, cy + y, color);
                self.set_pixel_blend(cx - x, cy - y, color);
                self.set_pixel_blend(cx - y, cy - x, color);
                self.set_pixel_blend(cx + y, cy - x, color);
                self.set_pixel_blend(cx + x, cy - y, color);

                if err <= 0 {
                    y += 1;
                    err += 2 * y + 1;
                }
                if err > 0 {
                    x -= 1;
                    err -= 2 * x + 1;
                }
            }
        }
    }

    pub fn draw_gradient_v(&mut self, x: i32, y: i32, w: i32, h: i32, top: Color, bottom: Color) {
        if w <= 0 || h <= 0 {
            return;
        }
        let x_start = x.max(0);
        let x_end = (x + w).min(self.width as i32);
        let y_start = y.max(0);
        let y_end = (y + h).min(self.height as i32);

        for cy in y_start..y_end {
            let t = (cy - y) as f32 / h.max(1) as f32;
            let c = top.lerp(bottom, t);
            let row_start = cy as usize * self.width;
            for cx in x_start..x_end {
                let idx = row_start + cx as usize;
                self.pixels[idx] = c.blend_over(self.pixels[idx]);
            }
        }
    }
}
