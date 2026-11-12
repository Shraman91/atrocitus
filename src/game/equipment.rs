use super::classes::ClassKind;
use crate::assets::sprites::{Direction4, Sprite};
use crate::engine::framebuffer::Framebuffer;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    Head,
    Torso,
    Cape,
    MainHand,
    OffHand,
    Accessory,
}

pub struct LayeredGear {
    pub class: ClassKind,
    pub head: Option<Sprite>,
    pub torso: Option<Sprite>,
    pub cape: Option<Sprite>,
    pub main_hand: Option<Sprite>,
    pub off_hand: Option<Sprite>,
    pub accessory: Option<Sprite>,
}

#[inline]
fn draw_layer(fb: &mut Framebuffer, sp_opt: Option<&Sprite>, x: i32, y: i32, flip_x: bool) {
    if let Some(sp) = sp_opt {
        sp.draw(fb, x, y, flip_x);
    }
}

impl LayeredGear {
    pub fn empty() -> Self {
        Self {
            class: ClassKind::Wanderer,
            head: None,
            torso: None,
            cape: None,
            main_hand: None,
            off_hand: None,
            accessory: None,
        }
    }

    /// Tries loading a custom PNG override for a gear layer, e.g. `assets/classes/mage/head.png`.
    pub fn try_load_png_override(class_id: &str, layer: LayerKind) -> Option<Sprite> {
        let layer_name = match layer {
            LayerKind::Head => "head",
            LayerKind::Torso => "torso",
            LayerKind::Cape => "cape",
            LayerKind::MainHand => "main_hand",
            LayerKind::OffHand => "off_hand",
            LayerKind::Accessory => "accessory",
        };
        let path_str = format!("assets/classes/{}/{}.png", class_id, layer_name);
        let path = Path::new(&path_str);
        if path.exists() {
            Sprite::load_png(path).ok()
        } else {
            None
        }
    }

    /// Composites all equipped layers in proper depth order for the given facing direction.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_composite(
        &self,
        fb: &mut Framebuffer,
        body_sprite: &Sprite,
        gear_sprites: &[Option<&Sprite>], // [head, torso, cape, main_hand, off_hand, accessory]
        draw_x: i32,
        draw_y: i32,
        dir: Direction4,
        flip_x: bool,
    ) {
        let head = gear_sprites[0];
        let torso = gear_sprites[1];
        let cape = gear_sprites[2];
        let main = gear_sprites[3];
        let off = gear_sprites[4];
        let acc = gear_sprites[5];

        match dir {
            Direction4::South | Direction4::SouthEast => {
                // Front / 3/4 front view: Cape behind, body, armor, weapons in front
                draw_layer(fb, cape, draw_x, draw_y, flip_x);
                body_sprite.draw(fb, draw_x, draw_y, flip_x);
                draw_layer(fb, torso, draw_x, draw_y, flip_x);
                draw_layer(fb, head, draw_x, draw_y, flip_x);
                draw_layer(fb, off, draw_x, draw_y, flip_x);
                draw_layer(fb, main, draw_x, draw_y, flip_x);
                draw_layer(fb, acc, draw_x, draw_y, flip_x);
            }
            Direction4::North => {
                // Back view: Weapons behind, body, armor, cape draped over back
                draw_layer(fb, main, draw_x, draw_y, flip_x);
                draw_layer(fb, off, draw_x, draw_y, flip_x);
                body_sprite.draw(fb, draw_x, draw_y, flip_x);
                draw_layer(fb, torso, draw_x, draw_y, flip_x);
                draw_layer(fb, head, draw_x, draw_y, flip_x);
                draw_layer(fb, cape, draw_x, draw_y, flip_x);
                draw_layer(fb, acc, draw_x, draw_y, flip_x);
            }
            Direction4::East => {
                // Side view: Cape behind, far weapon, body, torso, head, near weapon
                draw_layer(fb, cape, draw_x, draw_y, flip_x);
                draw_layer(fb, off, draw_x, draw_y, flip_x);
                body_sprite.draw(fb, draw_x, draw_y, flip_x);
                draw_layer(fb, torso, draw_x, draw_y, flip_x);
                draw_layer(fb, head, draw_x, draw_y, flip_x);
                draw_layer(fb, main, draw_x, draw_y, flip_x);
                draw_layer(fb, acc, draw_x, draw_y, flip_x);
            }
        }
    }
}
