#![allow(dead_code)]

mod assets;
mod engine;
mod game;
mod scenes;
pub mod ui;

use assets::SpriteBank;
use crossterm::event::{self, Event};
use engine::{
    verify_truecolor_support, CursorRenderer, CursorStyle, DiffRenderer, Framebuffer,
    InputState, TerminalGuard, Time, FIXED_DT_SECS, MIN_TERMINAL_COLS,
    MIN_TERMINAL_ROWS,
};
use engine::input::VirtualKey;
use scenes::{SceneManager, TitleScene};
use std::io;
use std::time::{Duration, Instant};
use ui::UiCanvas;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Verify TrueColor terminal support
    if let Err(err_msg) = verify_truecolor_support() {
        eprintln!("{}", err_msg);
        std::process::exit(1);
    }

    // 2. Initialize RAII Terminal Guard (Raw mode, Alternate Screen, Mouse capture, Panic Hook)
    let _guard = TerminalGuard::init()?;

    // 3. Precompute procedural sprites & assets
    let sprite_bank = SpriteBank::build();

    // 4. Initialize Terminal Resolution, Framebuffer, UiCanvas & Renderer
    let (mut term_cols, mut term_rows) = TerminalGuard::get_size()?;
    let mut renderer = DiffRenderer::new(term_cols as usize, term_rows as usize);
    let mut framebuffer = Framebuffer::new(term_cols as usize, (term_rows * 2) as usize);
    let mut ui = UiCanvas::new(term_cols as usize, term_rows as usize);

    // 5. Initialize Scene State Machine & Clock
    let mut scene_mgr = SceneManager::new(Box::new(TitleScene::new()));
    let mut input = InputState::new();
    let mut time = Time::new();
    let mut total_time = 0.0f32;
    let mut is_too_small = term_cols < MIN_TERMINAL_COLS || term_rows < MIN_TERMINAL_ROWS;
    let mut debug_layout = false;

    // 6. Main Game Loop (60 Hz fixed timestep with decoupled rendering)
    while !scene_mgr.is_quitting {
        let frame_start = Instant::now();
        let (steps, now) = time.update();

        // 1. Prepare input state for the current frame
        input.begin_frame(now);

        // 2. Poll and dispatch all incoming terminal events
        while event::poll(Duration::ZERO)? {
            let ev = event::read()?;
            match ev {
                Event::Resize(new_cols, new_rows) => {
                    term_cols = new_cols;
                    term_rows = new_rows;
                    is_too_small = term_cols < MIN_TERMINAL_COLS || term_rows < MIN_TERMINAL_ROWS;

                    if !is_too_small {
                        renderer.resize(term_cols as usize, term_rows as usize);
                        framebuffer.resize(term_cols as usize, (term_rows * 2) as usize);
                        ui.resize(term_cols as usize, term_rows as usize);
                        scene_mgr.on_resize(term_cols as usize, (term_rows * 2) as usize);
                    }
                }
                Event::Key(key_event) => {
                    input.process_key_event(key_event, frame_start);
                }
                Event::Mouse(mouse_event) => {
                    input.process_mouse_event(mouse_event, 0.0, 0.0);
                }
                _ => {}
            }
        }

        // Live resize barrier if terminal is smaller than 120x40
        if is_too_small {
            renderer.render_resize_warning(
                term_cols,
                term_rows,
                MIN_TERMINAL_COLS,
                MIN_TERMINAL_ROWS,
            )?;
            std::thread::sleep(Duration::from_millis(50));
            continue;
        }

        // Toggle layout debug rects on F2
        if input.just_pressed(VirtualKey::F2) {
            debug_layout = !debug_layout;
        }

        // 3. Run fixed-timestep updates
        for _ in 0..steps {
            total_time += FIXED_DT_SECS;
            scene_mgr.update(FIXED_DT_SECS, &input, &sprite_bank);
            if scene_mgr.is_quitting {
                break;
            }
        }

        if scene_mgr.is_quitting {
            break;
        }

        // Decoupled Frame Render
        framebuffer.clear(engine::Color::BLACK);
        ui.clear();
        scene_mgr.render(&mut framebuffer, &mut ui, &sprite_bank);

        // Optional F2 layout debugger overlays
        if debug_layout {
            ui.draw_debug_rects();
        }

        // In-game custom reticle / ember cursor in framebuffer
        CursorRenderer::draw(
            &mut framebuffer,
            input.mouse_pos.0,
            input.mouse_pos.1,
            CursorStyle::Default,
            total_time,
            input.mouse_ever_detected,
        );

        // Crisp FPS indicator in top-right corner of UiCanvas
        let fps_text = format!("{:.0} FPS", time.current_fps);
        let fps_col = ui.cols.saturating_sub(fps_text.len() + 2);
        ui.print(
            fps_col,
            0,
            &fps_text,
            engine::Color::rgb(80, 90, 110),
            None,
        );

        // Diff-based terminal flush combining Framebuffer (half-blocks) + UiCanvas (real text)
        if let Err(e) = renderer.render(&framebuffer, &ui) {
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(Box::new(e));
        }

        // Frame rate limiter (~60 FPS)
        let elapsed = frame_start.elapsed();
        if elapsed < engine::timing::FIXED_DT {
            std::thread::sleep(engine::timing::FIXED_DT - elapsed);
        }
    }

    Ok(())
}
