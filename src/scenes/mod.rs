use crate::assets::sprites::SpriteBank;
use crate::engine::color::Color;
use crate::engine::framebuffer::Framebuffer;
use crate::engine::input::InputState;
use crate::engine::postprocess::PostProcessor;

pub mod loading;
pub mod lobby;
pub mod title;

pub use title::TitleScene;

pub enum SceneTransition {
    None,
    Switch(Box<dyn Scene>),
    Push(Box<dyn Scene>),
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

    fn render(&self, fb: &mut Framebuffer, bank: &SpriteBank);

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

    pub fn render(&self, fb: &mut Framebuffer, bank: &SpriteBank) {
        if let Some(active_scene) = self.scenes.last() {
            active_scene.render(fb, bank);
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
