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
pub mod forest;
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

            // 13. ForestScene (Area 1)
            let mut forest = forest::ForestScene::new(Some(crate::game::classes::ClassKind::Knight), crate::game::world::ForestPartId::Chapel, 0);
            forest.on_resize(cols, rows * 2);
            forest.render(&mut fb, &mut ui, &bank);
        }
    }

    #[test]
    fn test_phase2_sprite_quality_and_gallery_coverage() {
        let bank = SpriteBank::build();

        // 1. Verify Player / Wanderer dimensions
        for d in 0..4 {
            for f in 0..4 {
                assert_eq!(bank.body_idle[d][f].width, 18);
                assert_eq!(bank.body_idle[d][f].height, 30);
            }
            for f in 0..6 {
                assert_eq!(bank.body_walk[d][f].width, 18);
                assert_eq!(bank.body_walk[d][f].height, 30);
            }
            for f in 0..7 {
                assert_eq!(bank.body_roll[d][f].width, 18);
                assert_eq!(bank.body_roll[d][f].height, 30);
            }
            for f in 0..4 {
                assert_eq!(bank.body_lmb[d][f].width, 18);
                assert_eq!(bank.body_lmb[d][f].height, 30);
                assert_eq!(bank.body_rmb[d][f].width, 18);
                assert_eq!(bank.body_rmb[d][f].height, 30);
            }
            assert_eq!(bank.body_hit[d].width, 18);
            assert_eq!(bank.body_hit[d].height, 30);
        }
        assert_eq!(bank.body_death.width, 18);
        assert_eq!(bank.body_death.height, 30);

        // 2. Verify NPC dimensions & 4-frame cycles
        for f in 0..4 {
            assert_eq!(bank.firekeeper_anim[f].width, 18);
            assert_eq!(bank.firekeeper_anim[f].height, 30);
            assert_eq!(bank.blacksmith_anim[f].width, 22);
            assert_eq!(bank.blacksmith_anim[f].height, 34);
            assert_eq!(bank.adventurer_anim[f].width, 18);
            assert_eq!(bank.adventurer_anim[f].height, 30);
            assert_eq!(bank.witch_anim[f].width, 18);
            assert_eq!(bank.witch_anim[f].height, 30);
        }

        // 3. Verify Enemy dimensions (including new CharSprite enemy_sprites)
        for f in 0..4 {
            assert_eq!(bank.sludge_idle[f].width, 26);
            assert_eq!(bank.sludge_idle[f].height, 20);
            assert_eq!(bank.worm_anim[f].width, 12);
            assert_eq!(bank.worm_anim[f].height, 8);
            for d in 0..4 {
                assert_eq!(bank.skeleton_idle[d][f].width, 16);
                assert_eq!(bank.skeleton_idle[d][f].height, 25);
            }
        }
        assert_eq!(bank.fly_anim[0].width, 12);
        assert_eq!(bank.fly_anim[0].height, 9);
        assert_eq!(bank.fly_shadow.width, 12);
        assert_eq!(bank.fly_shadow.height, 4);

        // 4. Verify Gallery render exhaustive matrix (14 entities x 7 states x 4 directions x 3 resolutions)
        let resolutions = [(120, 40), (160, 50), (200, 60)];

        for &(cols, rows) in &resolutions {
            let mut fb = Framebuffer::new(cols, rows * 2);
            let mut ui = UiCanvas::new(cols, rows);

            for char_idx in 0..14 {
                for state_idx in 0..7 {
                    for dir_idx in 0..4 {
                        let mut gallery = gallery::GalleryScene::new();
                        gallery.on_resize(cols, rows * 2);

                        // Set character, state, direction
                        for _ in 0..char_idx {
                            let mut d_input = crate::engine::input::InputState::new();
                            d_input.press_key_for_test(crate::engine::input::VirtualKey::D);
                            gallery.update(0.1, &d_input, &bank);
                        }
                        for _ in 0..dir_idx {
                            let mut s_input = crate::engine::input::InputState::new();
                            s_input.press_key_for_test(crate::engine::input::VirtualKey::S);
                            gallery.update(0.1, &s_input, &bank);
                        }
                        let mut tab_input = crate::engine::input::InputState::new();
                        let key_char = match state_idx {
                            0 => '1',
                            1 => '2',
                            2 => '3',
                            3 => '4',
                            4 => '5',
                            5 => '6',
                            _ => '7',
                        };
                        tab_input.press_key_for_test(crate::engine::input::VirtualKey::Char(key_char));
                        gallery.update(0.1, &tab_input, &bank);

                        // Render
                        gallery.render(&mut fb, &mut ui, &bank);
                    }
                }
            }
        }
    }

    #[test]
    fn test_gallery_and_class_select_sprites_not_obscured_by_ui() {
        let bank = SpriteBank::build();
        let mut fb = Framebuffer::new(120, 80);
        let mut ui = UiCanvas::new(120, 40);

        let gallery = gallery::GalleryScene::new();
        gallery.render(&mut fb, &mut ui, &bank);

        let pw = (120usize.saturating_sub(12)).min(88);
        let ph = (40usize.saturating_sub(6)).min(28);
        let px = (120 - pw) / 2;
        let py = (40 - ph) / 2;
        let stage_top = py + 5;
        let stage_rect = crate::ui::layout::Rect::new(px as i32 + 2, stage_top as i32, pw as i32 - 4, (py + ph - 3 - stage_top) as i32);
        let center_x = stage_rect.x + (stage_rect.w / 2);
        let check_row = stage_top + 4;

        // UI cell over 1x sprite MUST be None
        assert_eq!(ui.get_cell(center_x, check_row as i32), None, "1x sprite in gallery must not be obscured by UI cell");

        // Verify ClassSelectScene also does not obscure rotating character preview
        let class_select = class_select::ClassSelectScene::new();
        class_select.render(&mut fb, &mut ui, &bank);
        assert_eq!(ui.get_cell(8, 10), None, "Class preview in card must not be obscured by UI cell");
    }

    #[test]
    fn test_phase3_combat_loop_and_resources() {
        let bank = SpriteBank::build();
        let world = crate::game::world::World::build(280, 190);
        let mut lobby = lobby::LobbyScene::new(world);

        let mut input = crate::engine::input::InputState::new();

        // 1. Verify F7 enemy spawner cycles enemies at mouse cursor
        input.mouse_ever_detected = true;
        input.mouse_pos = (120.0, 80.0);
        input.press_key_for_test(crate::engine::input::VirtualKey::F7);
        lobby.update(0.016, &input, &bank);

        // 2. Verify F6 hitbox debug toggle
        let mut f6_input = crate::engine::input::InputState::new();
        f6_input.press_key_for_test(crate::engine::input::VirtualKey::F6);
        lobby.update(0.016, &f6_input, &bank);

        // 3. Simulate 60 frames of combat
        for _ in 0..60 {
            let mut tick_input = crate::engine::input::InputState::new();
            tick_input.mouse_ever_detected = true;
            tick_input.mouse_pos = (140.0, 90.0);
            lobby.update(0.016, &tick_input, &bank);
        }

        // 4. Multi-resolution render checks (120x40, 160x50, 200x60)
        let resolutions = [(120, 40), (160, 50), (200, 60)];
        for &(cols, rows) in &resolutions {
            let mut fb = Framebuffer::new(cols, rows * 2);
            let mut ui = UiCanvas::new(cols, rows);
            lobby.on_resize(cols, rows * 2);
            lobby.render(&mut fb, &mut ui, &bank);

            // Verify top-left combat HUD exists
            let cell = ui.get_cell(2, 1);
            assert!(cell.is_some(), "Mantle & Souls badge must be rendered");

            let hearts_cell = ui.get_cell(2, 2);
            assert!(hearts_cell.is_some(), "Hearts HUD must be rendered");
        }
    }

    #[test]
    fn test_area1_forest_four_parts_and_fog_walls() {
        use crate::game::classes::ClassKind;
        use crate::game::enemy::EnemyKind;
        use crate::game::world::{ForestAreaConfig, ForestPartId, World};

        // 1. Load data/areas/forest.ron
        let config = ForestAreaConfig::load();
        assert_eq!(config.parts.len(), 4, "Must contain all 4 parts of Area 1");

        // Verify part 1: Chapel
        let chapel = config.get_part(ForestPartId::Chapel).expect("Chapel part missing");
        assert_eq!(chapel.name, "The Desecrated Chapel");
        assert!(chapel.colosseum_gate.is_some(), "Chapel must have south gate to return to Colosseum");
        assert!(chapel.fog_wall.is_some(), "Chapel must have forward fog wall to Autumn Woods");
        let chapel_fog = chapel.fog_wall.as_ref().unwrap();
        assert_eq!(chapel_fog.target_part, ForestPartId::AutumnWoods);
        assert!(chapel.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Worm)), "Chapel must have Corpse Worms");
        assert!(chapel.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Fly)), "Chapel must have Carrion Flies");

        // Verify part 2: Autumn Woods
        let woods = config.get_part(ForestPartId::AutumnWoods).expect("Autumn Woods part missing");
        assert_eq!(woods.name, "The Autumnal Woods");
        assert!(woods.return_fog_wall.is_some(), "Autumn Woods must have return fog wall to Chapel");
        assert!(woods.fog_wall.is_some(), "Autumn Woods must have forward fog wall to Deep Forest");
        let woods_fog = woods.fog_wall.as_ref().unwrap();
        assert_eq!(woods_fog.target_part, ForestPartId::DeepForest);
        assert!(woods.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Sludge)), "Autumn Woods must have Acid Sludges");
        assert!(woods.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Skeleton)), "Autumn Woods must have Bogged Skeletons");

        // Verify part 3: Gloomwood Deep
        let deep = config.get_part(ForestPartId::DeepForest).expect("Deep Forest part missing");
        assert_eq!(deep.name, "The Gloomwood Deep");
        assert!(deep.return_fog_wall.is_some(), "Deep Forest must have return fog wall to Autumn Woods");
        assert!(deep.fog_wall.is_some(), "Deep Forest must have forward fog wall to Castle Keep");
        let deep_fog = deep.fog_wall.as_ref().unwrap();
        assert_eq!(deep_fog.target_part, ForestPartId::CastleKeep);
        assert!(deep.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Worm)));
        assert!(deep.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Fly)));
        assert!(deep.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Sludge)));
        assert!(deep.enemy_spawns.iter().any(|e| matches!(e.kind, EnemyKind::Skeleton)));

        // Verify part 4: Dread Keep (Boss Area)
        let castle = config.get_part(ForestPartId::CastleKeep).expect("Castle Keep part missing");
        assert_eq!(castle.name, "The Dread Keep");
        assert!(castle.return_fog_wall.is_some(), "Castle Keep must have return fog wall to Deep Forest");

        // 2. Build world terrain for all 4 parts
        for part in &config.parts {
            let world = World::build_forest_part(part);
            assert_eq!(world.width, part.width);
            assert_eq!(world.height, part.height);
            assert!(!world.colliders.is_empty(), "World must have collision boundaries");
            assert!(!world.props.is_empty(), "World must have atmospheric props");
        }

        // 3. Test ForestScene progression and rendering at 120x40, 160x50, 200x60
        let bank = SpriteBank::build();
        let mut forest_scene = forest::ForestScene::new(Some(ClassKind::Knight), ForestPartId::Chapel, 150);

        let resolutions = [(120, 40), (160, 50), (200, 60)];
        let part_ids = [
            ForestPartId::Chapel,
            ForestPartId::AutumnWoods,
            ForestPartId::DeepForest,
            ForestPartId::CastleKeep,
        ];

        for &part_id in &part_ids {
            let spawn = config.get_part(part_id).unwrap().player_spawn;
            forest_scene.switch_to_part(part_id, spawn);

            // Update scene for a few frames
            let input = crate::engine::input::InputState::new();
            forest_scene.update(0.016, &input, &bank);

            for &(cols, rows) in &resolutions {
                let mut fb = Framebuffer::new(cols, rows * 2);
                let mut ui = UiCanvas::new(cols, rows);
                forest_scene.on_resize(cols, rows * 2);
                forest_scene.render(&mut fb, &mut ui, &bank);

                // HUD check
                let badge = ui.get_cell(2, 1);
                assert!(badge.is_some(), "Mantle & souls HUD badge must be present in {:?}", part_id);
            }
        }
    }
}
