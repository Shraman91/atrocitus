#[derive(Debug, Clone)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub smoothness: f32,
    pub world_w: f32,
    pub world_h: f32,
    pub screen_w: f32,
    pub screen_h: f32,
}

impl Camera {
    pub fn new(world_w: f32, world_h: f32, screen_w: f32, screen_h: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            smoothness: 8.0,
            world_w,
            world_h,
            screen_w,
            screen_h,
        }
    }

    pub fn resize(&mut self, screen_w: f32, screen_h: f32) {
        self.screen_w = screen_w;
        self.screen_h = screen_h;
    }

    pub fn follow(&mut self, px: f32, py: f32) {
        self.target_x = px - self.screen_w * 0.5;
        self.target_y = py - self.screen_h * 0.5;
    }

    pub fn update(&mut self, dt: f32) {
        // Smooth exponential moving average
        let factor = (self.smoothness * dt).clamp(0.0, 1.0);
        self.x += (self.target_x - self.x) * factor;
        self.y += (self.target_y - self.y) * factor;

        // Clamp to world bounds
        let max_x = (self.world_w - self.screen_w).max(0.0);
        let max_y = (self.world_h - self.screen_h).max(0.0);
        self.x = self.x.clamp(0.0, max_x);
        self.y = self.y.clamp(0.0, max_y);
    }

    #[inline]
    pub fn to_screen(&self, wx: f32, wy: f32) -> (i32, i32) {
        ((wx - self.x).round() as i32, (wy - self.y).round() as i32)
    }
}
