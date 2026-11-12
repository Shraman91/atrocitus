use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::InputState;
use crate::engine::postprocess::PostProcessor;
use crate::ui::UiCanvas;

pub mod adventurer_dialogue;
pub mod backpack;
pub mod blacksmith;
pub mod class_select;
pub mod dialogue;
pub mod gallery;
pub mod loading;
pub mod lobby;
pub mod map;
pub mod spellbook;
pub mod title;
pub mod witch_shop;

pub use title::TitleScene;

pub enum SceneTransition {
    None,
    Switch(Box<dyn Scene>),
    Push(Box<dyn Scene>),
    Swap(Box<dyn Scene>),
    Pop,
    Quit,
}

pub trait Scene {
    fn update(
        &mut self,
        dt: f32,
        input: &InputState,
        bank: &SpriteBank,
    ) -> SceneTransition;

    fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank);

    fn on_resize(&mut self, _w: usize, _h: usize) {}
}

enum FadeState {
    None,
    FadingOut {
        target: Box<dyn Scene>,
        timer: f32,
        duration: f32,
    },
    FadingIn {
        timer: f32,
        duration: f32,
    },
}

pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    fade_state: FadeState,
    pub is_quitting: bool,
}

impl SceneManager {
    pub fn new(initial_scene: Box<dyn Scene>) -> Self {
        Self {
            scenes: vec![initial_scene],
            fade_state: FadeState::FadingIn {
                timer: 0.8,
                duration: 0.8,
            },
            is_quitting: false,
        }
    }

    pub fn update(&mut self, dt: f32, input: &InputState, bank: &SpriteBank) {
        // Handle active fade animations
        match &mut self.fade_state {
            FadeState::FadingOut {
                target,
                timer,
                duration,
            } => {
                *timer -= dt;
                if *timer <= 0.0 {
                    // Reached full black, perform switch
                    let next_scene = std::mem::replace(target, Box::new(title::TitleScene::new()));
                    if let Some(top) = self.scenes.last_mut() {
                        *top = next_scene;
                    }
                    self.fade_state = FadeState::FadingIn {
                        timer: *duration,
                        duration: *duration,
                    };
                }
                return;
            }
            FadeState::FadingIn { timer, duration: _ } => {
                *timer -= dt;
                if *timer <= 0.0 {
                    self.fade_state = FadeState::None;
                }
            }
            FadeState::None => {}
        }

        if let Some(active_scene) = self.scenes.last_mut() {
            let transition = active_scene.update(dt, input, bank);
            match transition {
                SceneTransition::None => {}
                SceneTransition::Switch(new_scene) => {
                    self.fade_state = FadeState::FadingOut {
                        target: new_scene,
                        timer: 0.4,
                        duration: 0.4,
                    };
                }
                SceneTransition::Push(new_scene) => {
                    self.scenes.push(new_scene);
                }
                SceneTransition::Swap(new_scene) => {
                    if let Some(top) = self.scenes.last_mut() {
                        *top = new_scene;
                    }
                }
                SceneTransition::Pop => {
                    self.scenes.pop();
                    if self.scenes.is_empty() {
                        self.is_quitting = true;
                    }
                }
                SceneTransition::Quit => {
                    self.is_quitting = true;
                }
            }
        } else {
            self.is_quitting = true;
        }
    }

    pub fn render(&self, fb: &mut Framebuffer, ui: &mut UiCanvas, bank: &SpriteBank) {
        // Render stacked scenes if top scene is an overlay
        if self.scenes.len() > 1 {
            // Render underlying scene first
            let base_idx = self.scenes.len() - 2;
            self.scenes[base_idx].render(fb, ui, bank);
        }

        if let Some(active_scene) = self.scenes.last() {
            active_scene.render(fb, ui, bank);
        }

        // Apply fade overlay
        match &self.fade_state {
            FadeState::FadingOut {
                timer,
                duration,
                ..
            } => {
                let alpha = (1.0 - (timer / duration)).clamp(0.0, 1.0);
                PostProcessor::apply_fade(fb, Color::BLACK, alpha);
            }
            FadeState::FadingIn { timer, duration } => {
                let alpha = (timer / duration).clamp(0.0, 1.0);
                PostProcessor::apply_fade(fb, Color::BLACK, alpha);
            }
            FadeState::None => {}
        }
    }

    pub fn on_resize(&mut self, w: usize, h: usize) {
        for s in &mut self.scenes {
            s.on_resize(w, h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::palette::Palette;
    use crate::assets::sprites::SpriteBank;
    use crate::game::inventory::Inventory;
    use crate::game::map::MapGrid;
    use crate::game::world::World;

    #[test]
    fn test_all_scenes_render_at_required_resolutions() {
        let resolutions = [(120, 40), (160, 50), (200, 60)];
        let bank = SpriteBank::build();

        for &(cols, rows) in &resolutions {
            let mut fb = Framebuffer::new(cols, rows * 2);
            let mut ui = UiCanvas::new(cols, rows);

            // 1. TitleScene
            let mut title = title::TitleScene::new();
            title.on_resize(cols, rows * 2);
            title.render(&mut fb, &mut ui, &bank);

            // 2. LoadingScene
            let mut loading = loading::LoadingScene::new();
            loading.on_resize(cols, rows * 2);
            loading.render(&mut fb, &mut ui, &bank);

            // 3. LobbyScene
            let world = World::build(280, 190);
            let mut lobby = lobby::LobbyScene::new(world);
            lobby.on_resize(cols, rows * 2);
            lobby.render(&mut fb, &mut ui, &bank);

            // 4. ClassSelectScene
            let mut class_select = class_select::ClassSelectScene::new();
            class_select.on_resize(cols, rows * 2);
            class_select.render(&mut fb, &mut ui, &bank);

            // 5. DialogueScene
            let mut dialogue = dialogue::DialogueScene::new();
            dialogue.on_resize(cols, rows * 2);
            dialogue.render(&mut fb, &mut ui, &bank);

            // 6. BlacksmithScene
            let mut blacksmith = blacksmith::BlacksmithScene::new();
            blacksmith.on_resize(cols, rows * 2);
            blacksmith.render(&mut fb, &mut ui, &bank);

            // 7. AdventurerDialogueScene
            let mut adventurer = adventurer_dialogue::AdventurerDialogueScene::new();
            adventurer.on_resize(cols, rows * 2);
            adventurer.render(&mut fb, &mut ui, &bank);

            // 8. WitchShopScene
            let mut witch = witch_shop::WitchShopScene::new();
            witch.on_resize(cols, rows * 2);
            witch.render(&mut fb, &mut ui, &bank);

            // 9. MapScene
            let map_grid = MapGrid::new(280, 190);
            let mut map = map::MapScene::new(
                map_grid,
                (140.0, 95.0),
                0.0,
                (140.0, 95.0),
                (140.0, 30.0),
                vec![
                    ("F", (164.0, 89.0), Palette::BONFIRE_ORANGE),
                    ("B", (106.0, 103.0), Palette::BONFIRE_RED),
                ],
                (280.0, 190.0),
            );
            map.on_resize(cols, rows * 2);
            map.render(&mut fb, &mut ui, &bank);

            // 10. BackpackScene
            let mut backpack = backpack::BackpackScene::new(Inventory::new());
            backpack.on_resize(cols, rows * 2);
            backpack.render(&mut fb, &mut ui, &bank);

            // 11. GalleryScene
            let mut gallery = gallery::GalleryScene::new();
            gallery.on_resize(cols, rows * 2);
            gallery.render(&mut fb, &mut ui, &bank);

            // 12. SpellbookScene
            let mut spellbook = spellbook::SpellbookScene::new();
            spellbook.on_resize(cols, rows * 2);
            spellbook.render(&mut fb, &mut ui, &bank);
        }
    }
}
