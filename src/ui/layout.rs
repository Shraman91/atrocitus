use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_dims(x: i32, y: i32, w: usize, h: usize) -> Self {
        Self {
            x,
            y,
            w: w as i32,
            h: h as i32,
        }
    }

    pub fn from_i32(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x,
            y,
            w: w.max(0),
            h: h.max(0),
        }
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.x + self.w
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }

    #[inline]
    pub fn contains(&self, col: i32, row: i32) -> bool {
        col >= self.x && col < self.right() && row >= self.y && row < self.bottom()
    }

    pub fn inset(&self, margin: Margin) -> Self {
        let left = margin.left as i32;
        let right = margin.right as i32;
        let top = margin.top as i32;
        let bottom = margin.bottom as i32;

        let new_x = self.x + left;
        let new_y = self.y + top;
        let new_w = (self.w - left - right).max(0);
        let new_h = (self.h - top - bottom).max(0);

        Self {
            x: new_x,
            y: new_y,
            w: new_w,
            h: new_h,
        }
    }

    pub fn center_in(&self, parent: &Rect) -> Self {
        let new_x = parent.x + (parent.w - self.w) / 2;
        let new_y = parent.y + (parent.h - self.h) / 2;
        Self {
            x: new_x,
            y: new_y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn center_horizontally(&self, parent_w: i32) -> Self {
        let new_x = (parent_w - self.w) / 2;
        Self {
            x: new_x,
            y: self.y,
            w: self.w,
            h: self.h,
        }
    }

    pub fn center_vertically(&self, parent_h: i32) -> Self {
        let new_y = (parent_h - self.h) / 2;
        Self {
            x: self.x,
            y: new_y,
            w: self.w,
            h: self.h,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Margin {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}

impl Margin {
    pub const fn all(val: u16) -> Self {
        Self {
            top: val,
            bottom: val,
            left: val,
            right: val,
        }
    }

    pub const fn symmetric(h: u16, v: u16) -> Self {
        Self {
            top: v,
            bottom: v,
            left: h,
            right: h,
        }
    }
}

pub type Padding = Margin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

#[inline]
pub fn text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Truncates text to fit within `max_width`, appending `…` if truncated.
pub fn truncate_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if text_width(text) <= max_width {
        return text.to_string();
    }

    let target_width = max_width.saturating_sub(1);
    let mut current_width = 0;
    let mut result = String::new();

    for c in text.chars() {
        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + cw > target_width {
            break;
        }
        result.push(c);
        current_width += cw;
    }
    result.push('…');
    result
}

/// Word-wraps text to fit within `max_width` per line.
pub fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for word in raw_line.split_whitespace() {
            let word_w = text_width(word);

            if current_line.is_empty() {
                if word_w > max_width {
                    // Split word that exceeds max_width
                    let mut chunk = String::new();
                    let mut chunk_w = 0;
                    for c in word.chars() {
                        let cw = UnicodeWidthChar::width(c).unwrap_or(0);
                        if chunk_w + cw > max_width {
                            lines.push(chunk);
                            chunk = String::new();
                            chunk_w = 0;
                        }
                        chunk.push(c);
                        chunk_w += cw;
                    }
                    if !chunk.is_empty() {
                        current_line = chunk;
                        current_width = chunk_w;
                    }
                } else {
                    current_line.push_str(word);
                    current_width = word_w;
                }
            } else if current_width + 1 + word_w <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_w;
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                current_width = word_w;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_inset_and_contains() {
        let r = Rect::new(10, 10, 30, 20);
        assert!(r.contains(10, 10));
        assert!(r.contains(39, 29));
        assert!(!r.contains(40, 30));

        let inner = r.inset(Margin::all(2));
        assert_eq!(inner, Rect::new(12, 12, 26, 16));
    }

    #[test]
    fn test_truncate_ellipsis() {
        assert_eq!(truncate_ellipsis("Hello World", 5), "Hell…");
        assert_eq!(truncate_ellipsis("Hi", 5), "Hi");
    }

    #[test]
    fn test_word_wrap() {
        let wrapped = word_wrap("The quick brown fox jumps over the lazy dog", 15);
        for line in &wrapped {
            assert!(text_width(line) <= 15);
        }
        assert!(wrapped.len() > 1);
    }
}
