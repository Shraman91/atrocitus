use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

const FIRST_PRESS_TIMEOUT: Duration = Duration::from_millis(120);
const REPEAT_TIMEOUT: Duration = Duration::from_millis(100);

#[cfg(target_os = "windows")]
fn is_key_physically_down(key: VirtualKey) -> Option<bool> {
    unsafe extern "system" {
        fn GetAsyncKeyState(vKey: i32) -> i16;
    }
    let vk = match key {
        VirtualKey::W => b'W' as i32,
        VirtualKey::A => b'A' as i32,
        VirtualKey::S => b'S' as i32,
        VirtualKey::D => b'D' as i32,
        VirtualKey::E => b'E' as i32,
        VirtualKey::B => b'B' as i32,
        VirtualKey::M => b'M' as i32,
        VirtualKey::K => b'K' as i32,
        VirtualKey::Space => 0x20, // VK_SPACE
        VirtualKey::Shift => 0x10, // VK_SHIFT
        VirtualKey::Up => 0x26,    // VK_UP
        VirtualKey::Down => 0x28,  // VK_DOWN
        VirtualKey::Left => 0x25,  // VK_LEFT
        VirtualKey::Right => 0x27, // VK_RIGHT
        VirtualKey::Tab => 0x09,   // VK_TAB
        VirtualKey::Escape => 0x1B,// VK_ESCAPE
        VirtualKey::Enter => 0x0D, // VK_RETURN
        _ => return None,
    };
    unsafe {
        let state = GetAsyncKeyState(vk);
        Some((state as u16 & 0x8000) != 0)
    }
}

#[cfg(not(target_os = "windows"))]
fn is_key_physically_down(_key: VirtualKey) -> Option<bool> {
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    Up,
    Down,
    Left,
    Right,
    W,
    A,
    S,
    D,
    E,
    B,
    M,
    K,
    Space,
    Shift,
    Tab,
    Enter,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F9,
    Char(char),
    Any,
}

pub struct InputState {
    // (last_seen, is_repeat)
    held_keys: HashMap<VirtualKey, (Instant, bool)>,
    just_pressed: HashSet<VirtualKey>,
    just_released: HashSet<VirtualKey>,
    pub any_key_just_pressed: bool,

    // Mouse Tracking
    pub mouse_pos: (f32, f32),          // Screen logical pixels (0..width, 0..height)
    pub mouse_cell: (usize, usize),     // Terminal grid cell (col, row)
    pub mouse_world_pos: (f32, f32),    // Ground plane coordinates (screen + camera)
    pub mouse_left_held: bool,
    pub mouse_left_just_pressed: bool,
    pub mouse_left_just_released: bool,
    pub mouse_right_held: bool,
    pub mouse_right_just_pressed: bool,
    pub mouse_right_just_released: bool,
    pub mouse_scroll_delta: i32,
    pub mouse_ever_detected: bool,
    pub mouse_moved: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            held_keys: HashMap::with_capacity(32),
            just_pressed: HashSet::with_capacity(16),
            just_released: HashSet::with_capacity(16),
            any_key_just_pressed: false,
            mouse_pos: (60.0, 40.0),
            mouse_cell: (60, 20),
            mouse_world_pos: (120.0, 80.0),
            mouse_left_held: false,
            mouse_left_just_pressed: false,
            mouse_left_just_released: false,
            mouse_right_held: false,
            mouse_right_just_pressed: false,
            mouse_right_just_released: false,
            mouse_scroll_delta: 0,
            mouse_ever_detected: false,
            mouse_moved: false,
        }
    }

    pub fn press_key_for_test(&mut self, key: VirtualKey) {
        self.just_pressed.insert(key);
        self.held_keys.insert(key, (Instant::now(), false));
        self.any_key_just_pressed = true;
    }

    /// Prepares state for a new frame, decaying transient press/release events
    /// and checking held key timeouts for terminals without release event reporting.
    pub fn begin_frame(&mut self, now: Instant) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.any_key_just_pressed = false;

        self.mouse_left_just_pressed = false;
        self.mouse_left_just_released = false;
        self.mouse_right_just_pressed = false;
        self.mouse_right_just_released = false;
        self.mouse_scroll_delta = 0;
        self.mouse_moved = false;

        // Check fallback timeout or physical key state for keys that might not receive release events.
        // On Windows, GetAsyncKeyState checks if the physical key has been released.
        let mut expired = Vec::new();
        for (&key, val) in self.held_keys.iter_mut() {
            if let Some(is_down) = is_key_physically_down(key) {
                if is_down {
                    val.0 = now; // Actively held physically
                } else {
                    expired.push(key); // Physical key released
                }
            } else {
                let timeout = if val.1 { REPEAT_TIMEOUT } else { FIRST_PRESS_TIMEOUT };
                if now.duration_since(val.0) > timeout {
                    expired.push(key);
                }
            }
        }

        for key in expired {
            self.held_keys.remove(&key);
            self.just_released.insert(key);
        }
    }

    pub fn process_key_event(&mut self, key_event: KeyEvent, now: Instant) {
        let vkeys = map_crossterm_key(&key_event);

        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => {
                self.any_key_just_pressed = true;
                for &vk in &vkeys {
                    let is_repeat = self.held_keys.contains_key(&vk) || key_event.kind == KeyEventKind::Repeat;
                    self.just_pressed.insert(vk);
                    self.held_keys.insert(vk, (now, is_repeat));
                }
            }
            KeyEventKind::Release => {
                for &vk in &vkeys {
                    self.held_keys.remove(&vk);
                    self.just_released.insert(vk);
                }
            }
        }
    }

    pub fn process_mouse_event(&mut self, mouse: MouseEvent, cam_x: f32, cam_y: f32) {
        self.mouse_ever_detected = true;
        self.mouse_moved = true;

        // Convert terminal column/row into logical pixels (each cell is 1 col x 2 rows in half-block mode)
        let sx = mouse.column as f32;
        let sy = (mouse.row as f32) * 2.0 + 0.5;

        self.mouse_pos = (sx, sy);
        self.mouse_cell = (mouse.column as usize, mouse.row as usize);
        self.mouse_world_pos = (sx + cam_x, sy + cam_y);

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.mouse_left_held = true;
                self.mouse_left_just_pressed = true;
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.mouse_left_held = false;
                self.mouse_left_just_released = true;
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.mouse_right_held = true;
                self.mouse_right_just_pressed = true;
            }
            MouseEventKind::Up(MouseButton::Right) => {
                self.mouse_right_held = false;
                self.mouse_right_just_released = true;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.mouse_left_held = true;
            }
            MouseEventKind::Drag(MouseButton::Right) => {
                self.mouse_right_held = true;
            }
            MouseEventKind::ScrollUp => {
                self.mouse_scroll_delta += 1;
            }
            MouseEventKind::ScrollDown => {
                self.mouse_scroll_delta -= 1;
            }
            MouseEventKind::Moved => {}
            _ => {}
        }
    }

    pub fn update_camera_projection(&mut self, cam_x: f32, cam_y: f32) {
        self.mouse_world_pos = (self.mouse_pos.0 + cam_x, self.mouse_pos.1 + cam_y);
    }

    #[inline]
    pub fn mouse_world_at(&self, cam_x: f32, cam_y: f32) -> (f32, f32) {
        (self.mouse_pos.0 + cam_x, self.mouse_pos.1 + cam_y)
    }

    #[inline]
    pub fn is_held(&self, key: VirtualKey) -> bool {
        self.held_keys.contains_key(&key)
    }

    #[inline]
    pub fn just_pressed(&self, key: VirtualKey) -> bool {
        self.just_pressed.contains(&key)
    }

    #[inline]
    pub fn just_released(&self, key: VirtualKey) -> bool {
        self.just_released.contains(&key)
    }

    /// Calculates a normalized (dx, dy) movement vector from WASD and arrow keys.
    pub fn movement_vector(&self) -> (f32, f32) {
        let mut dx = 0.0;
        let mut dy = 0.0;

        if self.is_held(VirtualKey::W) || self.is_held(VirtualKey::Up) {
            dy -= 1.0;
        }
        if self.is_held(VirtualKey::S) || self.is_held(VirtualKey::Down) {
            dy += 1.0;
        }
        if self.is_held(VirtualKey::A) || self.is_held(VirtualKey::Left) {
            dx -= 1.0;
        }
        if self.is_held(VirtualKey::D) || self.is_held(VirtualKey::Right) {
            dx += 1.0;
        }

        if dx != 0.0 && dy != 0.0 {
            let inv_len = 1.0 / (2.0f32).sqrt();
            (dx * inv_len, dy * inv_len)
        } else {
            (dx, dy)
        }
    }
}

fn map_crossterm_key(event: &KeyEvent) -> Vec<VirtualKey> {
    let mut keys = Vec::with_capacity(3);

    match event.code {
        KeyCode::Up => keys.push(VirtualKey::Up),
        KeyCode::Down => keys.push(VirtualKey::Down),
        KeyCode::Left => keys.push(VirtualKey::Left),
        KeyCode::Right => keys.push(VirtualKey::Right),
        KeyCode::Enter => keys.push(VirtualKey::Enter),
        KeyCode::Tab => keys.push(VirtualKey::Tab),
        KeyCode::Esc => keys.push(VirtualKey::Escape),
        KeyCode::F(1) => keys.push(VirtualKey::F1),
        KeyCode::F(2) => keys.push(VirtualKey::F2),
        KeyCode::F(3) => keys.push(VirtualKey::F3),
        KeyCode::F(4) => keys.push(VirtualKey::F4),
        KeyCode::F(5) => keys.push(VirtualKey::F5),
        KeyCode::F(6) => keys.push(VirtualKey::F6),
        KeyCode::F(7) => keys.push(VirtualKey::F7),
        KeyCode::F(9) => keys.push(VirtualKey::F9),
        KeyCode::Char(' ') => keys.push(VirtualKey::Space),
        KeyCode::Char(c) => {
            let lower = c.to_ascii_lowercase();
            match lower {
                'w' => {
                    keys.push(VirtualKey::W);
                    keys.push(VirtualKey::Up);
                }
                's' => {
                    keys.push(VirtualKey::S);
                    keys.push(VirtualKey::Down);
                }
                'a' => {
                    keys.push(VirtualKey::A);
                    keys.push(VirtualKey::Left);
                }
                'd' => {
                    keys.push(VirtualKey::D);
                    keys.push(VirtualKey::Right);
                }
                'e' => keys.push(VirtualKey::E),
                'b' => keys.push(VirtualKey::B),
                'm' => keys.push(VirtualKey::M),
                'k' => keys.push(VirtualKey::K),
                _ => {}
            }
            keys.push(VirtualKey::Char(lower));
        }
        _ => {}
    }

    if event.modifiers.contains(KeyModifiers::SHIFT) {
        keys.push(VirtualKey::Shift);
    }

    keys.push(VirtualKey::Any);
    keys
}
