use crate::engine::color::Color;

/// Curated dark-fantasy Souls-like palette:
/// Deep blue-black shadows, cold grey stone, warm orange bonfire firelight,
/// Roman Colosseum sandstone/sand, and signature class colors.
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

    // Stone, Ash & Ground (Hue-shifted ramps)
    pub const STONE_DARK: Color = Color::rgb(28, 31, 38);
    pub const STONE_MID: Color = Color::rgb(48, 53, 64);
    pub const STONE_LIGHT: Color = Color::rgb(75, 82, 98);
    pub const STONE_CRACK: Color = Color::rgb(18, 20, 26);
    pub const STONE_WARM_LIGHT: Color = Color::rgb(112, 102, 90);
    pub const STONE_WARM_MID: Color = Color::rgb(78, 68, 58);
    pub const STONE_WARM_DARK: Color = Color::rgb(48, 40, 34);
    pub const STONE_COOL_LIGHT: Color = Color::rgb(82, 94, 118);
    pub const STONE_COOL_MID: Color = Color::rgb(42, 50, 68);
    pub const STONE_COOL_DARK: Color = Color::rgb(24, 28, 40);
    pub const STONE_COOL_DEEP: Color = Color::rgb(14, 16, 26);
    pub const ASH_WHITE: Color = Color::rgb(215, 220, 230);
    pub const WEED_DARK: Color = Color::rgb(25, 33, 28);

    // Moss, Roots & Lichen
    pub const MOSS_GREEN_DARK: Color = Color::rgb(22, 34, 22);
    pub const MOSS_GREEN_MID: Color = Color::rgb(42, 62, 38);
    pub const MOSS_GREEN_LIGHT: Color = Color::rgb(72, 98, 62);

    // Puddle & Reflective Water
    pub const PUDDLE_WATER: Color = Color::rgb(20, 26, 38);
    pub const PUDDLE_REFLECT: Color = Color::rgb(75, 60, 42); // Warm bonfire reflection

    // Colosseum Sand, Blood & Bone
    pub const ARENA_SAND_DARK: Color = Color::rgb(35, 29, 23);
    pub const ARENA_SAND_MID: Color = Color::rgb(55, 46, 36);
    pub const ARENA_SAND_LIGHT: Color = Color::rgb(82, 69, 54);
    pub const BLOOD_STAIN: Color = Color::rgb(44, 14, 14);
    pub const BONE_WHITE: Color = Color::rgb(195, 187, 170);
    pub const CAVEA_STONE_DARK: Color = Color::rgb(22, 24, 30);
    pub const CAVEA_STONE_MID: Color = Color::rgb(45, 49, 58);
    pub const CAVEA_STONE_LIGHT: Color = Color::rgb(75, 81, 95);
    pub const SPECTATOR_SILHOUETTE: Color = Color::rgb(12, 13, 17);
    pub const SPECTATOR_EYE_GLOW: Color = Color::rgb(220, 160, 40);

    // Bronze Statues & Iron Cages
    pub const BRONZE_STATUE_DARK: Color = Color::rgb(38, 30, 22);
    pub const BRONZE_STATUE_MID: Color = Color::rgb(72, 58, 42);
    pub const BRONZE_STATUE_LIGHT: Color = Color::rgb(125, 102, 75);
    pub const CAGE_IRON_DARK: Color = Color::rgb(20, 22, 26);
    pub const CAGE_IRON_BAR: Color = Color::rgb(60, 66, 76);

    // Torches & Wall Sconces
    pub const TORCH_CORE: Color = Color::rgb(255, 240, 190);
    pub const TORCH_FLAME: Color = Color::rgb(255, 145, 30);
    pub const TORCH_EMBER: Color = Color::rgb(220, 75, 15);
    pub const TORCH_WOOD: Color = Color::rgb(50, 36, 24);

    // Iron, Armor & Knight
    pub const IRON_DARK: Color = Color::rgb(28, 30, 36);
    pub const IRON_MID: Color = Color::rgb(55, 60, 72);
    pub const IRON_LIGHT: Color = Color::rgb(110, 120, 140);
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

    // Signature Class Accent Colors
    pub const MAGE_CYAN: Color = Color::rgb(88, 166, 255);
    pub const MAGE_PURPLE: Color = Color::rgb(155, 93, 229);
    pub const KNIGHT_GOLD: Color = Color::rgb(230, 184, 0);
    pub const ASSASSIN_RED: Color = Color::rgb(179, 45, 46);
    pub const HUNTER_BRASS: Color = Color::rgb(196, 139, 82);
    pub const ALCHEMIST_TEAL: Color = Color::rgb(46, 196, 182);

    // Character Shading Ramps & Outlines (Hue-shifted: cool shadows, warm highlights)
    pub const OUTLINE_DARK: Color = Color::rgb(8, 10, 14);
    pub const RIM_LIGHT_COOL: Color = Color::rgb(125, 150, 185);
    pub const SKIN_SHADOW: Color = Color::rgb(130, 95, 80);
    pub const SKIN_MID: Color = Color::rgb(190, 145, 120);
    pub const SKIN_LIGHT: Color = Color::rgb(225, 185, 155);
    pub const SKIN_WARM_HIGHLIGHT: Color = Color::rgb(250, 220, 190);
    pub const CLOTH_DARK: Color = Color::rgb(26, 30, 38);
    pub const CLOTH_MID: Color = Color::rgb(45, 52, 65);
    pub const CLOTH_LIGHT: Color = Color::rgb(70, 80, 98);
    pub const LEATHER_DARK: Color = Color::rgb(42, 28, 20);
    pub const LEATHER_MID: Color = Color::rgb(75, 50, 32);
    pub const LEATHER_LIGHT: Color = Color::rgb(115, 80, 52);

    // Enemy Palette 1: Acid Sludge (Acid Ooze)
    pub const SLUDGE_MOSS_DARK: Color = Color::rgb(24, 38, 20);
    pub const SLUDGE_ACID_MID: Color = Color::rgb(55, 115, 45);
    pub const SLUDGE_ACID_YELLOW: Color = Color::rgb(155, 215, 50);
    pub const SLUDGE_CORE: Color = Color::rgb(225, 255, 120);

    // Enemy Palette 2: Bogged Skeleton (Overgrown Archer)
    pub const BOG_BONE_MOSS: Color = Color::rgb(50, 68, 45);
    pub const MUSHROOM_RED: Color = Color::rgb(180, 35, 40);
    pub const MUSHROOM_PALE: Color = Color::rgb(235, 225, 200);
    pub const POISON_ARROW_GREEN: Color = Color::rgb(80, 220, 90);

    // Enemy Palette 3: Carrion Fly (Swarmer)
    pub const FLY_CHITIN_DARK: Color = Color::rgb(20, 22, 25);
    pub const FLY_CHITIN_MID: Color = Color::rgb(45, 55, 50);
    pub const FLY_EYE_TOXIC: Color = Color::rgb(255, 60, 40);
    pub const FLY_WING_TRANSLUCENT: Color = Color::rgba(180, 220, 240, 160);

    // Enemy Palette 4: Corpse Worm (Burrower)
    pub const WORM_PALE_FLESH: Color = Color::rgb(195, 165, 160);
    pub const WORM_FLESH_SHADOW: Color = Color::rgb(110, 80, 85);
    pub const WORM_RING_TEETH: Color = Color::rgb(240, 235, 220);

    // UI & Text
    pub const TEXT_BRIGHT: Color = Color::rgb(235, 238, 245);
    pub const TEXT_NORMAL: Color = Color::rgb(180, 186, 200);
    pub const TEXT_MUTED: Color = Color::rgb(110, 116, 130);
    pub const TEXT_DARK: Color = Color::rgb(60, 65, 75);
    pub const TEXT_GOLD: Color = Color::rgb(230, 180, 60);
    pub const TEXT_CRIMSON: Color = Color::rgb(200, 60, 60);
    pub const UI_PANEL_BG: Color = Color::rgba(10, 12, 18, 230);
    pub const UI_BORDER: Color = Color::rgb(65, 72, 88);
    pub const UI_BORDER_HIGHLIGHT: Color = Color::rgb(140, 150, 180);
    pub const UI_ACCENT_GLOW: Color = Color::rgb(255, 135, 30);
}
