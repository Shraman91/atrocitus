use crate::engine::color::Color;

/// Curated dark-fantasy Souls-like palette:
/// Deep blue-black shadows, cold grey stone, and warm orange bonfire firelight.
pub struct Palette;

impl Palette {
    // Atmosphere & Void
    pub const VOID_BLACK: Color = Color::rgb(5, 7, 10);
    pub const ABYSS: Color = Color::rgb(11, 13, 19);
    pub const DEEP_NIGHT: Color = Color::rgb(16, 20, 28);
    pub const AMBIENT_BLUE: Color = Color::rgb(18, 24, 36);

    // Fog & Moonlight
    pub const FOG_DARK: Color = Color::rgba(20, 28, 40, 90);
    pub const FOG_MID: Color = Color::rgba(35, 48, 68, 120);
    pub const FOG_LIGHT: Color = Color::rgba(65, 85, 115, 140);
    pub const MOON_PALE: Color = Color::rgb(142, 163, 191);
    pub const MOON_HAZE: Color = Color::rgb(80, 95, 120);

    // Stone, Ash & Ground
    pub const STONE_DARK: Color = Color::rgb(28, 31, 38);
    pub const STONE_MID: Color = Color::rgb(48, 53, 64);
    pub const STONE_LIGHT: Color = Color::rgb(75, 82, 98);
    pub const STONE_CRACK: Color = Color::rgb(18, 20, 26);
    pub const ASH_WHITE: Color = Color::rgb(215, 220, 230);
    pub const WEED_DARK: Color = Color::rgb(25, 33, 28);

    // Iron, Armor & Knight
    pub const IRON_DARK: Color = Color::rgb(28, 30, 36);
    pub const ARMOR_SHADOW: Color = Color::rgb(20, 22, 28);
    pub const ARMOR_MID: Color = Color::rgb(50, 56, 68);
    pub const ARMOR_HIGHLIGHT: Color = Color::rgb(105, 115, 135);
    pub const STEEL_GLINT: Color = Color::rgb(195, 205, 225);
    pub const CAPE_CRIMSON: Color = Color::rgb(70, 22, 22);
    pub const CAPE_DARK: Color = Color::rgb(40, 12, 12);
    pub const GOLD_TRIM: Color = Color::rgb(140, 110, 45);

    // Bonfire & Embers
    pub const BONFIRE_CORE: Color = Color::rgb(255, 245, 180);
    pub const BONFIRE_YELLOW: Color = Color::rgb(255, 180, 30);
    pub const BONFIRE_ORANGE: Color = Color::rgb(240, 100, 10);
    pub const BONFIRE_RED: Color = Color::rgb(170, 35, 10);
    pub const BONFIRE_EMBER: Color = Color::rgb(255, 120, 20);
    pub const BONFIRE_SMOKE: Color = Color::rgba(30, 25, 25, 160);
    pub const COILED_SWORD: Color = Color::rgb(90, 85, 95);

    // UI & Text
    pub const TEXT_BRIGHT: Color = Color::rgb(235, 238, 245);
    pub const TEXT_NORMAL: Color = Color::rgb(180, 186, 200);
    pub const TEXT_MUTED: Color = Color::rgb(110, 116, 130);
    pub const TEXT_DARK: Color = Color::rgb(60, 65, 75);
    pub const TEXT_GOLD: Color = Color::rgb(230, 180, 60);
    pub const TEXT_CRIMSON: Color = Color::rgb(200, 60, 60);
    pub const UI_PANEL_BG: Color = Color::rgba(10, 12, 18, 220);
    pub const UI_BORDER: Color = Color::rgb(65, 72, 88);
    pub const UI_ACCENT_GLOW: Color = Color::rgb(255, 135, 30);
}
