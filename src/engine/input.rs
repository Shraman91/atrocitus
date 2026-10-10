use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

const HELD_FALLBACK_TIMEOUT: Duration = Duration::from_millis(130);

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
    Space,
    Shift,
    Enter,
    Escape,
    F1,
    F3,
    Char(char),
    Any,
}

pub struct InputState {
    held_timestamps: HashMap<VirtualKey, Instant>,
    just_pressed: HashSet<VirtualKey>,
    just_released: HashSet<VirtualKey>,
    pub any_key_just_pressed: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            held_timestamps: HashMap::with_capacity(32),
            just_pressed: HashSet::with_capacity(16),
            just_released: HashSet::with_capacity(16),
            any_key_just_pressed: false,
        }
    }

    /// Prepares state for a new frame, decaying transient press/release events
    /// and checking held key timeouts for terminals without release event reporting.
    pub fn begin_frame(&mut self, now: Instant) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.any_key_just_pressed = false;

        // Check fallback timeout for keys that might not receive release events
        let expired: Vec<VirtualKey> = self
            .held_timestamps
            .iter()
            .filter_map(|(&key, &time)| {
                if now.duration_since(time) > HELD_FALLBACK_TIMEOUT {
                    Some(key)
                } else {
                    None
                }
            })
            .collect();

        for key in expired {
            self.held_timestamps.remove(&key);
            self.just_released.insert(key);
        }
    }

    pub fn process_event(&mut self, event: Event, now: Instant) {
        if let Event::Key(key_event) = event {
            self.process_key_event(key_event, now);
        }
    }

    pub fn process_key_event(&mut self, key_event: KeyEvent, now: Instant) {
        let vkeys = map_crossterm_key(&key_event);

        match key_event.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => {
                self.any_key_just_pressed = true;
                for &vk in &vkeys {
                    self.just_pressed.insert(vk);
                    self.held_timestamps.insert(vk, now);
                }
            }
            KeyEventKind::Release => {
                for &vk in &vkeys {
                    self.held_timestamps.remove(&vk);
                    self.just_released.insert(vk);
                }
            }
        }
    }

    #[inline]
    pub fn is_held(&self, key: VirtualKey) -> bool {
        self.held_timestamps.contains_key(&key)
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
        KeyCode::Esc => keys.push(VirtualKey::Escape),
        KeyCode::F(1) => keys.push(VirtualKey::F1),
        KeyCode::F(3) => keys.push(VirtualKey::F3),
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
