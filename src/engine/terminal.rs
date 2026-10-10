use std::env;
use std::io::{self, stdout, Write};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

pub const MIN_TERMINAL_COLS: u16 = 120;
pub const MIN_TERMINAL_ROWS: u16 = 40;

static TERMINAL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Verifies whether the terminal environment supports 24-bit TrueColor.
pub fn verify_truecolor_support() -> Result<(), String> {
    let colorterm = env::var("COLORTERM").unwrap_or_default().to_lowercase();
    let term_program = env::var("TERM_PROGRAM").unwrap_or_default().to_lowercase();
    let wt_session = env::var("WT_SESSION").is_ok();
    let alacritty = env::var("ALACRITTY_LOG").is_ok() || env::var("ALACRITTY_WINDOW_ID").is_ok();
    let ghostty = env::var("GHOSTTY_RESOURCES_DIR").is_ok();

    if colorterm == "truecolor"
        || colorterm == "24bit"
        || wt_session
        || alacritty
        || ghostty
        || term_program.contains("vscode")
        || term_program.contains("wezterm")
        || term_program.contains("iterm")
    {
        Ok(())
    } else {
        Err("24-bit TrueColor support was not detected!\n\
            Atrocitus requires a TrueColor terminal for rendering.\n\n\
            How to resolve:\n\
            1. Set the environment variable: set COLORTERM=truecolor (Windows CMD) or $env:COLORTERM=\"truecolor\" (PowerShell)\n\
            2. Run in a modern terminal such as Windows Terminal, WezTerm, Alacritty, or Ghostty.".to_string())
    }
}

/// RAII Guard for managing terminal state, raw mode, alternate screen, and keyboard protocols.
pub struct TerminalGuard {
    _private: (),
}

impl TerminalGuard {
    pub fn init() -> io::Result<Self> {
        // Setup custom panic hook to restore terminal cleanly on unexpected panics
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            default_hook(panic_info);
        }));

        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Hide)?;
        let _ = execute!(out, EnableBracketedPaste);

        // Best-effort enabling Kitty progressive keyboard enhancement (ignored if unsupported)
        let _ = execute!(
            out,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            )
        );
        out.flush()?;

        TERMINAL_INITIALIZED.store(true, Ordering::SeqCst);
        Ok(TerminalGuard { _private: () })
    }

    pub fn get_size() -> io::Result<(u16, u16)> {
        size()
    }

    fn restore_terminal() -> io::Result<()> {
        if TERMINAL_INITIALIZED.swap(false, Ordering::SeqCst) {
            let mut out = stdout();
            let _ = execute!(out, PopKeyboardEnhancementFlags);
            let _ = execute!(out, DisableBracketedPaste);
            let _ = execute!(out, Show, LeaveAlternateScreen);
            let _ = out.flush();
            let _ = disable_raw_mode();
        }
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = Self::restore_terminal();
    }
}
