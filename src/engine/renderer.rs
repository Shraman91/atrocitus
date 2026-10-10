use super::color::Color;
use super::framebuffer::Framebuffer;
use std::io::{self, stdout, BufWriter, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Cell {
    fg: Color,
    bg: Color,
}

/// Differential terminal renderer using half-block characters ('▀').
/// Logical resolution: `width = term_cols`, `height = term_rows * 2`.
pub struct DiffRenderer {
    pub cols: usize,
    pub rows: usize,
    prev_cells: Vec<Cell>,
    curr_cells: Vec<Cell>,
    writer: BufWriter<io::Stdout>,
    force_full_redraw: bool,
}

impl DiffRenderer {
    pub fn new(cols: usize, rows: usize) -> Self {
        let count = cols * rows;
        Self {
            cols,
            rows,
            prev_cells: vec![Cell::default(); count],
            curr_cells: vec![Cell::default(); count],
            writer: BufWriter::with_capacity(128 * 1024, stdout()),
            force_full_redraw: true,
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if self.cols == cols && self.rows == rows {
            return;
        }
        self.cols = cols;
        self.rows = rows;
        let count = cols * rows;
        self.prev_cells = vec![Cell::default(); count];
        self.curr_cells = vec![Cell::default(); count];
        self.force_full_redraw = true;
    }

    pub fn force_redraw(&mut self) {
        self.force_full_redraw = true;
    }

    /// Renders the logical pixel framebuffer to the terminal screen.
    pub fn render(&mut self, fb: &Framebuffer) -> io::Result<()> {
        let cols = self.cols.min(fb.width);
        let rows = self.rows.min(fb.height / 2);

        // Convert 2-pixel vertical slices from Framebuffer to terminal cells
        for row in 0..rows {
            let top_y = row * 2;
            let bot_y = top_y + 1;
            let cell_row_offset = row * self.cols;
            let top_fb_offset = top_y * fb.width;
            let bot_fb_offset = bot_y * fb.width;

            for col in 0..cols {
                let fg = fb.pixels[top_fb_offset + col];
                let bg = fb.pixels[bot_fb_offset + col];
                self.curr_cells[cell_row_offset + col] = Cell { fg, bg };
            }
        }

        let mut last_fg = Color::TRANSPARENT;
        let mut last_bg = Color::TRANSPARENT;
        let mut cursor_row = usize::MAX;
        let mut cursor_col = usize::MAX;

        for row in 0..rows {
            let row_offset = row * self.cols;
            for col in 0..cols {
                let idx = row_offset + col;
                let cell = self.curr_cells[idx];
                let changed = self.force_full_redraw || cell != self.prev_cells[idx];

                if changed {
                    // Update cursor if not adjacent to last printed cell
                    if cursor_row != row || cursor_col != col {
                        write!(self.writer, "\x1b[{};{}H", row + 1, col + 1)?;
                        cursor_row = row;
                        cursor_col = col;
                    }

                    if cell.fg != last_fg {
                        write!(
                            self.writer,
                            "\x1b[38;2;{};{};{}m",
                            cell.fg.r, cell.fg.g, cell.fg.b
                        )?;
                        last_fg = cell.fg;
                    }

                    if cell.bg != last_bg {
                        write!(
                            self.writer,
                            "\x1b[48;2;{};{};{}m",
                            cell.bg.r, cell.bg.g, cell.bg.b
                        )?;
                        last_bg = cell.bg;
                    }

                    write!(self.writer, "▀")?;
                    cursor_col += 1;
                }
            }
        }

        // Reset text styling at end of draw batch
        write!(self.writer, "\x1b[0m")?;
        self.writer.flush()?;

        // Swap buffers for next frame diff
        std::mem::swap(&mut self.prev_cells, &mut self.curr_cells);
        self.force_full_redraw = false;

        Ok(())
    }

    /// Helper to render the live "Please enlarge your terminal" message when window is too small.
    pub fn render_resize_warning(&mut self, cur_w: u16, cur_h: u16, min_w: u16, min_h: u16) -> io::Result<()> {
        write!(self.writer, "\x1b[2J\x1b[H\x1b[0m")?;
        let center_y = (cur_h / 2).max(1);
        let msg1 = "=== ATROCITUS ===";
        let msg2 = "PLEASE ENLARGE YOUR TERMINAL WINDOW";
        let msg3 = format!("Current: {}x{}  |  Required: {}x{}", cur_w, cur_h, min_w, min_h);
        let msg4 = "Resize the window to continue...";

        let write_centered = |w: &mut BufWriter<io::Stdout>, row: u16, text: &str| -> io::Result<()> {
            let col = if (cur_w as usize) > text.len() {
                (cur_w as usize - text.len()) / 2 + 1
            } else {
                1
            };
            write!(w, "\x1b[{};{}H{}", row, col, text)
        };

        write!(self.writer, "\x1b[38;2;220;80;30m")?;
        write_centered(&mut self.writer, center_y.saturating_sub(2), msg1)?;
        write!(self.writer, "\x1b[38;2;255;220;180m")?;
        write_centered(&mut self.writer, center_y, msg2)?;
        write!(self.writer, "\x1b[38;2;160;160;170m")?;
        write_centered(&mut self.writer, center_y + 2, &msg3)?;
        write!(self.writer, "\x1b[38;2;100;100;110m")?;
        write_centered(&mut self.writer, center_y + 4, msg4)?;

        self.writer.flush()?;
        self.force_full_redraw = true;
        Ok(())
    }
}
