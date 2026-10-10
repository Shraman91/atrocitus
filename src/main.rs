#![allow(dead_code)]

mod assets;
mod engine;
mod game;
mod scenes;

use assets::SpriteBank;
use crossterm::event::{self, Event};
use engine::{
    verify_truecolor_support, DiffRenderer, FontSize, Framebuffer, InputState, TerminalGuard,
    TextRenderer, Time, FIXED_DT_SECS, MIN_TERMINAL_COLS, MIN_TERMINAL_ROWS,
};
use scenes::{SceneManager, TitleScene};
use std::io;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Verify TrueColor terminal support
    if let Err(err_msg) = verify_truecolor_support() {
        eprintln!("{}", err_msg);
        std::process::exit(1);
    }

    // 2. Initialize RAII Terminal Guard (Raw mode, Alternate Screen, Panic Hook)
    let _guard = TerminalGuard::init()?;

    // 3. Precompute procedural sprites & assets
    let sprite_bank = SpriteBank::build();

    // 4. Initialize Terminal Resolution, Framebuffer & Renderer
    let (mut term_cols, mut term_rows) = TerminalGuard::get_size()?;
    let mut renderer = DiffRenderer::new(term_cols as usize, term_rows as usize);
    let mut framebuffer = Framebuffer::new(term_cols as usize, (term_rows * 2) as usize);

    // 5. Initialize Scene State Machine & Clock
    let mut scene_mgr = SceneManager::new(Box::new(TitleScene::new()));
    let mut input = InputState::new();
    let mut time = Time::new();
    let mut is_too_small = term_cols < MIN_TERMINAL_COLS || term_rows < MIN_TERMINAL_ROWS;

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
                        scene_mgr.on_resize(term_cols as usize, (term_rows * 2) as usize);
                    }
                }
                Event::Key(key_event) => {
                    input.process_key_event(key_event, frame_start);
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

        // 3. Run fixed-timestep updates
        for _ in 0..steps {
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
        scene_mgr.render(&mut framebuffer, &sprite_bank);

        // Optional FPS indicator in top-right corner
        let fps_text = format!("{:.0} FPS", time.current_fps);
        let fps_w = TextRenderer::measure_width(&fps_text, FontSize::Tiny3x5);
        let fps_x = framebuffer.width as i32 - fps_w - 3;
        TextRenderer::draw_text(
            &mut framebuffer,
            fps_x,
            2,
            &fps_text,
            FontSize::Tiny3x5,
            engine::Color::rgb(80, 90, 110),
        );

        // Diff-based terminal flush
        if let Err(e) = renderer.render(&framebuffer) {
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
