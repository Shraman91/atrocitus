use super::layout::{text_width, Rect};
use crate::assets::palette::Palette;
use crate::engine::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
    pub dim: bool,
}

impl UiCell {
    pub const fn new(ch: char, fg: Color, bg: Option<Color>) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold: false,
            dim: false,
        }
    }

    pub const fn styled(ch: char, fg: Color, bg: Option<Color>, bold: bool, dim: bool) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold,
            dim,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderKind {
    Single,
    Double,
    Rounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelStyle {
    pub border: BorderKind,
    pub border_color: Color,
    pub title_color: Color,
    pub bg_color: Option<Color>,
}

impl PanelStyle {
    pub fn single() -> Self {
        Self {
            border: BorderKind::Single,
            border_color: Palette::UI_BORDER,
            title_color: Palette::TEXT_GOLD,
            bg_color: Some(Color::rgb(14, 16, 22)),
        }
    }

    pub fn double() -> Self {
        Self {
            border: BorderKind::Double,
            border_color: Palette::GOLD_TRIM,
            title_color: Palette::TEXT_GOLD,
            bg_color: Some(Color::rgb(14, 16, 22)),
        }
    }

    pub fn rounded() -> Self {
        Self {
            border: BorderKind::Rounded,
            border_color: Palette::UI_BORDER,
            title_color: Palette::TEXT_GOLD,
            bg_color: Some(Color::rgb(14, 16, 22)),
        }
    }
}

pub struct UiCanvas {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Option<UiCell>>,
    pub debug_rects: Vec<(Rect, Color)>,
    pub debug_enabled: bool,
}

impl UiCanvas {
    pub fn new(cols: usize, rows: usize) -> Self {
        let count = cols * rows;
        Self {
            cols,
            rows,
            cells: vec![None; count],
            debug_rects: Vec::with_capacity(32),
            debug_enabled: false,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if self.cols == cols && self.rows == rows {
            return;
        }
        self.cols = cols;
        self.rows = rows;
        self.cells = vec![None; cols * rows];
        self.debug_rects.clear();
    }

    pub fn clear(&mut self) {
        self.cells.fill(None);
        self.debug_rects.clear();
    }

    #[inline]
    pub fn set_cell(&mut self, col: i32, row: i32, cell: UiCell) {
        if col >= 0 && col < self.cols as i32 && row >= 0 && row < self.rows as i32 {
            let idx = row as usize * self.cols + col as usize;
            self.cells[idx] = Some(cell);
        }
    }

    #[inline]
    pub fn get_cell(&self, col: i32, row: i32) -> Option<UiCell> {
        if col >= 0 && col < self.cols as i32 && row >= 0 && row < self.rows as i32 {
            let idx = row as usize * self.cols + col as usize;
            self.cells[idx]
        } else {
            None
        }
    }

    pub fn print(&mut self, x: usize, y: usize, text: &str, fg: Color, bg: Option<Color>) {
        self.print_styled(x as i32, y as i32, text, fg, bg, false, false);
    }

    pub fn print_bold(&mut self, x: usize, y: usize, text: &str, fg: Color, bg: Option<Color>) {
        self.print_styled(x as i32, y as i32, text, fg, bg, true, false);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn print_styled(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        fg: Color,
        bg: Option<Color>,
        bold: bool,
        dim: bool,
    ) {
        for (cur_x, ch) in (x..).zip(text.chars()) {
            if ch == '\n' {
                break;
            }
            self.set_cell(cur_x, y, UiCell::styled(ch, fg, bg, bold, dim));
        }
    }

    pub fn print_centered(
        &mut self,
        y: usize,
        text: &str,
        fg: Color,
        bg: Option<Color>,
    ) {
        let w = text_width(text);
        let x = (self.cols.saturating_sub(w)) / 2;
        self.print(x, y, text, fg, bg);
    }

    pub fn print_in_rect_centered(
        &mut self,
        y: usize,
        text: &str,
        rect: &Rect,
        fg: Color,
        bg: Option<Color>,
    ) {
        let w = text_width(text) as i32;
        let x = rect.x + (rect.w - w) / 2;
        if x >= 0 {
            self.print(x as usize, y, text, fg, bg);
        }
    }

    /// Fills a rectangle with space characters of the given background color.
    pub fn fill_rect(&mut self, rect: &Rect, bg: Color) {
        for r in rect.y..rect.bottom() {
            for c in rect.x..rect.right() {
                self.set_cell(c, r, UiCell::new(' ', Color::BLACK, Some(bg)));
            }
        }
    }

    /// Draws a styled panel using box-drawing characters and an optional title.
    pub fn draw_panel(&mut self, rect: impl std::borrow::Borrow<Rect>, title: &str, style: PanelStyle) {
        let rect = rect.borrow();
        if rect.w < 2 || rect.h < 2 {
            return;
        }

        // Fill background if specified
        if let Some(bg) = style.bg_color {
            self.fill_rect(rect, bg);
        }

        let (tl, tr, bl, br, horiz, vert) = match style.border {
            BorderKind::Single => ('┌', '┐', '└', '┘', '─', '│'),
            BorderKind::Double => ('╔', '╗', '╚', '╝', '═', '║'),
            BorderKind::Rounded => ('╭', '╮', '╰', '╯', '─', '│'),
        };

        let right = rect.right() - 1;
        let bottom = rect.bottom() - 1;

        // Corners
        self.set_cell(rect.x, rect.y, UiCell::new(tl, style.border_color, style.bg_color));
        self.set_cell(right, rect.y, UiCell::new(tr, style.border_color, style.bg_color));
        self.set_cell(rect.x, bottom, UiCell::new(bl, style.border_color, style.bg_color));
        self.set_cell(right, bottom, UiCell::new(br, style.border_color, style.bg_color));

        // Horizontal bars
        for c in (rect.x + 1)..right {
            self.set_cell(c, rect.y, UiCell::new(horiz, style.border_color, style.bg_color));
            self.set_cell(c, bottom, UiCell::new(horiz, style.border_color, style.bg_color));
        }

        // Vertical bars
        for r in (rect.y + 1)..bottom {
            self.set_cell(rect.x, r, UiCell::new(vert, style.border_color, style.bg_color));
            self.set_cell(right, r, UiCell::new(vert, style.border_color, style.bg_color));
        }

        // Title bar centered along top border
        if !title.is_empty() {
            let t_width = text_width(title) as i32;
            let available = rect.w.saturating_sub(4);
            if t_width <= available {
                let start_x = rect.x + (rect.w - t_width - 2) / 2;
                self.set_cell(start_x, rect.y, UiCell::new('┤', style.border_color, style.bg_color));
                self.print_styled(start_x + 1, rect.y, title, style.title_color, style.bg_color, true, false);
                self.set_cell(start_x + 1 + t_width, rect.y, UiCell::new('├', style.border_color, style.bg_color));
            }
        }
    }

    /// Adds a debug rect to be visualized.
    pub fn add_debug_rect(&mut self, rect: Rect, color: Color) {
        self.debug_rects.push((rect, color));
    }

    /// Draws colored bracket corners for all registered debug rects.
    pub fn draw_debug_rects(&mut self) {
        let rects = std::mem::take(&mut self.debug_rects);
        for (rect, col) in rects {
            if rect.w < 2 || rect.h < 2 {
                continue;
            }
            let r = rect.right() - 1;
            let b = rect.bottom() - 1;
            self.set_cell(rect.x, rect.y, UiCell::new('╔', col, None));
            self.set_cell(r, rect.y, UiCell::new('╗', col, None));
            self.set_cell(rect.x, b, UiCell::new('╚', col, None));
            self.set_cell(r, b, UiCell::new('╝', col, None));
        }
    }
}
