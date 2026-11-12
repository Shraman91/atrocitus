#[derive(Debug, Clone)]
pub struct MapGrid {
    pub width: usize,
    pub height: usize,
    pub explored: Vec<bool>,
}

impl MapGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            explored: vec![false; width * height],
        }
    }

    /// Uncovers a circular radius around the player in world coordinates.
    pub fn reveal(&mut self, world_x: f32, world_y: f32, radius: f32) {
        self.reveal_radius(world_x, world_y, self.width as f32, self.height as f32, radius);
    }

    /// Uncovers a circular radius around the player in world coordinates.
    pub fn reveal_radius(&mut self, world_x: f32, world_y: f32, world_w: f32, world_h: f32, radius_cells: f32) {
        let gx = ((world_x / world_w) * self.width as f32) as i32;
        let gy = ((world_y / world_h) * self.height as f32) as i32;

        let r = radius_cells.ceil() as i32;
        let r2 = radius_cells * radius_cells;

        let y_min = (gy - r).max(0);
        let y_max = (gy + r).min(self.height as i32 - 1);
        let x_min = (gx - r).max(0);
        let x_max = (gx + r).min(self.width as i32 - 1);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let dx = (x - gx) as f32;
                let dy = (y - gy) as f32;
                if dx * dx + dy * dy <= r2 {
                    let idx = y as usize * self.width + x as usize;
                    self.explored[idx] = true;
                }
            }
        }
    }

    #[inline]
    pub fn is_explored(&self, col: usize, row: usize) -> bool {
        if col < self.width && row < self.height {
            self.explored[row * self.width + col]
        } else {
            false
        }
    }
}
