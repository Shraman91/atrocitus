use std::time::{Duration, Instant};

pub const TARGET_FPS: u32 = 60;
pub const FIXED_DT_SECS: f32 = 1.0 / TARGET_FPS as f32;
pub const FIXED_DT: Duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);

pub struct Time {
    start_time: Instant,
    last_frame_time: Instant,
    accumulator: Duration,
    fps_timer: Instant,
    fps_frames: u32,
    pub current_fps: f32,
    pub total_frames: u64,
    pub elapsed_secs: f32,
    pub dt: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            accumulator: Duration::ZERO,
            fps_timer: now,
            fps_frames: 0,
            current_fps: 60.0,
            total_frames: 0,
            elapsed_secs: 0.0,
            dt: FIXED_DT_SECS,
        }
    }

    /// Advances the timer, updating accumulator and calculating FPS.
    /// Clamps large frame spikes (e.g. debugging pauses) to prevent spiral of death.
    pub fn update(&mut self) -> (usize, Instant) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).min(Duration::from_millis(100));
        self.last_frame_time = now;
        self.accumulator += frame_time;
        self.elapsed_secs = now.duration_since(self.start_time).as_secs_f32();

        // Calculate updates to run this tick
        let mut steps = 0;
        while self.accumulator >= FIXED_DT {
            self.accumulator -= FIXED_DT;
            steps += 1;
            if steps >= 5 {
                // Drop excess backlog if running too slow
                self.accumulator = Duration::ZERO;
                break;
            }
        }

        // Calculate smooth FPS once per ~0.5 seconds
        self.fps_frames += 1;
        self.total_frames += 1;
        let fps_elapsed = now.duration_since(self.fps_timer).as_secs_f32();
        if fps_elapsed >= 0.5 {
            self.current_fps = self.fps_frames as f32 / fps_elapsed;
            self.fps_frames = 0;
            self.fps_timer = now;
        }

        (steps, now)
    }
}
