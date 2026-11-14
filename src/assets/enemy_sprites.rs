//! src/assets/enemy_sprites.rs
//!
//! Forest enemy sprites for Atrocitus (skeleton, fly, worm), with animation frames.
//!
//! Each sprite is a grid of characters: `.` is transparent, anything else is a palette colour.
//! All frames of one sprite have the same size and are bottom-anchored (feet on the last row).
//!
//!   skeleton     3 frames  0 = idle, 1 = walk, 2 = bow drawn with a poisoned arrow (attack telegraph)
//!   fly          2 frames  wings up / down (play fast)
//!   worm         3 frames  wiggle cycle (spawn several, offset each one's start time)
//!
//! Usage:
//!   let spr = &enemy_sprites::SKELETON;
//!   spr.draw(time, feet_x, feet_y, flip, |x, y, rgb, emissive| fb.set_pixel(x, y, rgb, emissive));
//!
//! `emissive` pixels (eyes, flames, poison) should skip your lighting pass so they glow in the dark.

use crate::assets::sprites::Sprite;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;

pub type Rgb = [u8; 3];

pub fn palette(ch: char) -> Option<Rgb> {
    match ch {
        'K' => Some([0x15, 0x11, 0x1c]),
        'M' => Some([0x41, 0x63, 0x2b]),
        'N' => Some([0xa8, 0xa2, 0x8a]),
        'Q' => Some([0xb9, 0x91, 0x7f]),
        'X' => Some([0x8f, 0xb2, 0xcc]),
        'Z' => Some([0x6e, 0x2a, 0x3c]),
        'e' => Some([0xff, 0x5a, 0x4a]),
        'j' => Some([0x6e, 0x5e, 0x3a]),
        'k' => Some([0x3d, 0x36, 0x29]),
        'm' => Some([0x6f, 0x9f, 0x40]),
        'n' => Some([0xe8, 0xe2, 0xca]),
        'q' => Some([0xef, 0xd3, 0xc0]),
        'r' => Some([0xc0, 0x44, 0x3f]),
        'v' => Some([0x8d, 0xff, 0x66]),
        'w' => Some([0x8a, 0x5c, 0x34]),
        'x' => Some([0xc8, 0xdf, 0xf0]),
        _ => None,
    }
}

/// Characters that glow by themselves.
pub fn is_emissive(ch: char) -> bool {
    matches!(ch, 'e' | 'v')
}

#[derive(Clone, Copy)]
pub struct CharSprite {
    pub name: &'static str,
    pub frames: &'static [&'static [&'static str]],
    /// Seconds each frame is shown when looping.
    pub frame_secs: f32,
}

impl CharSprite {
    /// (width, height) in pixels.
    pub fn size(&self) -> (usize, usize) {
        let f = self.frames[0];
        (f.first().map_or(0, |r| r.chars().count()), f.len())
    }

    /// Frame to show at time `t` (seconds), looping.
    pub fn frame_at(&self, t: f32) -> &'static [&'static str] {
        let i = ((t / self.frame_secs).max(0.0) as usize) % self.frames.len();
        self.frames[i]
    }

    /// Convert a given frame to an engine `Sprite`.
    pub fn to_sprite(self, frame: usize) -> Sprite {
        let (w, h) = self.size();
        let mut sp = Sprite::new(w, h);
        let rows = self.frames[frame % self.frames.len()];
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(rgb) = palette(ch) {
                    sp.set_pixel(x, y, Color::rgb(rgb[0], rgb[1], rgb[2]));
                }
            }
        }
        sp
    }

    /// Draw frame `frame` with its bottom-centre at (feet_x, feet_y).
    /// `put(x, y, colour, emissive)` is called for every visible pixel; `flip_x` mirrors the sprite.
    pub fn draw_frame<F: FnMut(i32, i32, Rgb, bool)>(
        &self,
        frame: usize,
        feet_x: i32,
        feet_y: i32,
        flip_x: bool,
        mut put: F,
    ) {
        let rows = self.frames[frame % self.frames.len()];
        let (w, h) = self.size();
        let x0 = feet_x - (w as i32) / 2;
        let y0 = feet_y - h as i32;
        for (j, row) in rows.iter().enumerate() {
            for (i, ch) in row.chars().enumerate() {
                if let Some(rgb) = palette(ch) {
                    let dx = if flip_x { w - 1 - i } else { i };
                    put(x0 + dx as i32, y0 + j as i32, rgb, is_emissive(ch));
                }
            }
        }
    }

    /// Draw the looping animation at time `t`.
    pub fn draw<F: FnMut(i32, i32, Rgb, bool)>(&self, t: f32, feet_x: i32, feet_y: i32, flip_x: bool, put: F) {
        let i = ((t / self.frame_secs).max(0.0) as usize) % self.frames.len();
        self.draw_frame(i, feet_x, feet_y, flip_x, put);
    }

    /// Helper to draw directly onto a Framebuffer
    pub fn draw_to_framebuffer(&self, t: f32, feet_x: i32, feet_y: i32, flip_x: bool, fb: &mut Framebuffer) {
        self.draw(t, feet_x, feet_y, flip_x, |x, y, rgb, _emissive| {
            fb.set_pixel(x, y, Color::rgb(rgb[0], rgb[1], rgb[2]));
        });
    }
}

// ------------------------------------------------------------------ FRAME DATA
pub const SKELETON_0: &[&str] = &[
    "....KKK.........",
    "...KrnrK........",
    "....KwKKKKK.....",
    "...KmMnnnnMmK...",
    "...KnnnnnnnnK...",
    "...KnKKnnKKnK...",
    "...KneKnnKenK...",
    "...KnnnKKnnnK...",
    "...KnNmNnNnNK...",
    "...KKKNNNNKKK...",
    "......KnnK....ww",
    "...KNNnnnnNNKw.n",
    "..KmMKNNNNKnNw.n",
    "..KnNKKNNKKNnw.n",
    "..KNnKNNNNKnNw.n",
    "..KnNKKNNKKNnw.n",
    "..KnnKNNNNKMnw.n",
    "..KKKKNNNNKKKKww",
    "..KnnKKNNKKnnK..",
    "...KnK....KnK...",
    "...KmK....KnK...",
    "...KnK....KnK...",
    "...KnK....KnK...",
    "...KnnK..KnnK...",
    "...KKKK..KKKK...",
];

pub const SKELETON_1: &[&str] = &[
    "....KKK.........",
    "...KrnrK........",
    "....KwKKKKK.....",
    "...KmMnnnnMmK...",
    "...KnnnnnnnnK...",
    "...KnKKnnKKnK...",
    "...KneKnnKenK...",
    "...KnnnKKnnnK...",
    "...KnNmNnNnNK...",
    "...KKKNNNNKKK...",
    "......KnnK....ww",
    "...KNNnnnnNNKw.n",
    "..KmMKNNNNKnNw.n",
    "..KnNKKNNKKNnw.n",
    "..KNnKNNNNKnNw.n",
    "..KnNKKNNKKNnw.n",
    "..KnnKNNNNKMnw.n",
    "..KKKKNNNNKKKKww",
    "..KnnKKNNKKnnK..",
    "....KnK..KnK....",
    "....KnK..KnK....",
    "....KnK..KnK....",
    "....KnK..KnK....",
    "....KnnKKnnK....",
    "....KKKKKKKK....",
];

pub const SKELETON_2: &[&str] = &[
    "....KKK.........",
    "...KrnrK........",
    "....KwKKKKK.....",
    "...KmMnnnnMmK...",
    "...KnnnnnnnnK...",
    "...KnKKnnKKnK...",
    "..vKneKnnKenKv..",
    "...KnnnKKnnnK...",
    "...KnNmNnNnNK...",
    "...KKKNNNNKKK...",
    "......KnnK......",
    "...KNNnnnnNNK.ww",
    "..KmMKNNNNKnNw.n",
    "..KnNKKNNKKNnwn.",
    "..KNnKNNNNKnNwvv",
    "..KnNKKNNKKNnwn.",
    "..KnnKNNNNKMnw.n",
    "..KKKKNNNNKKKKww",
    "..KnnKKNNKKnnK..",
    "...KnK....KnK...",
    "...KmK....KnK...",
    "...KnK....KnK...",
    "...KnK....KnK...",
    "...KnnK..KnnK...",
    "...KKKK..KKKK...",
];

pub const FLY_0: &[&str] = &[
    "xxx......xxx",
    "xxxxKKKKxxxx",
    "XxxKeKKeKxxX",
    ".xxKkkkkKxx.",
    "..XKkjjkKX..",
    "...KkkkkK...",
    "....KkkK....",
    "....KvvK....",
    ".....KK.....",
];

pub const FLY_1: &[&str] = &[
    "............",
    "....KKKK....",
    "...KeKKeK...",
    "...KkkkkK...",
    ".xxKkjjkKxx.",
    "xxxKkkkkKxxx",
    "XxxxKkkKxxxX",
    ".XXxKvvKxXX.",
    ".....KK.....",
];

pub const WORM_0: &[&str] = &[
    ".....KKK....",
    "....KQqqKK..",
    "...KqQqqQq..",
    "..KqqQqqQZqK",
    ".KqqqQqqQZZK",
    ".KqqqKKKQqZK",
    "..KqK...KKqK",
    "...K........",
];

pub const WORM_1: &[&str] = &[
    "...KK.......",
    "...qqKK....K",
    "..KqqQqKKKqK",
    ".KqqqQqqQqZK",
    ".KqqqQqqQZZK",
    "..KKKQqqQZq.",
    ".....KKqQq..",
    ".......KKK..",
];

pub const WORM_2: &[&str] = &[
    "........KK..",
    "..KK...KQqq.",
    ".KqqKKKqQZZK",
    ".KqqqQqqQZZK",
    "..KqqQqqQqqK",
    "...qqQqqKK.K",
    "...KqQqK....",
    "....KKK.....",
];

// ---------------------------------------------------------------- SPRITE TABLE
pub const SKELETON: CharSprite = CharSprite {
    name: "skeleton",
    frames: &[SKELETON_0, SKELETON_1, SKELETON_2],
    frame_secs: 0.3,
};

pub const FLY: CharSprite = CharSprite {
    name: "fly",
    frames: &[FLY_0, FLY_1],
    frame_secs: 0.06,
};

pub const WORM: CharSprite = CharSprite {
    name: "worm",
    frames: &[WORM_0, WORM_1, WORM_2],
    frame_secs: 0.12,
};

pub const ALL: &[&CharSprite] = &[&SKELETON, &FLY, &WORM];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_rectangular_same_size_and_use_known_colours() {
        for s in ALL {
            let (w, h) = s.size();
            assert!(w > 0 && h > 0 && s.frames.len() >= 2, "{}", s.name);
            for f in s.frames {
                assert_eq!(f.len(), h, "{} height", s.name);
                for r in f.iter() {
                    assert_eq!(r.chars().count(), w, "{} width", s.name);
                    assert!(r.chars().all(|c| c == '.' || palette(c).is_some()), "{} colour", s.name);
                }
            }
        }
    }

    #[test]
    fn draw_stays_inside_the_sprite_box() {
        let s = &SKELETON;
        let (w, h) = s.size();
        let mut n = 0;
        s.draw(0.0, 100, 200, false, |x, y, _, _| {
            n += 1;
            assert!(x >= 100 - w as i32 / 2 && x < 100 + w as i32 && y >= 200 - h as i32 && y < 200);
        });
        assert!(n > 50);
    }
}
