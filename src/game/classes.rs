use crate::assets::palette::Palette;
use crate::engine::color::Color;
use serde::{Deserialize, Serialize};
use std::fs;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ClassKind {
    Mage,
    Knight,
    Assassin,
    Hunter,
    Alchemist,
    #[default]
    Wanderer,
}

impl ClassKind {
    pub fn id(&self) -> &'static str {
        match self {
            ClassKind::Mage => "mage",
            ClassKind::Knight => "knight",
            ClassKind::Assassin => "assassin",
            ClassKind::Hunter => "hunter",
            ClassKind::Alchemist => "alchemist",
            ClassKind::Wanderer => "wanderer",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ClassKind::Mage => "MAGE",
            ClassKind::Knight => "KNIGHT",
            ClassKind::Assassin => "ASSASSIN",
            ClassKind::Hunter => "HUNTER",
            ClassKind::Alchemist => "ALCHEMIST",
            ClassKind::Wanderer => "ASHEN WANDERER",
        }
    }

    pub fn accent_color(&self) -> Color {
        match self {
            ClassKind::Mage => Palette::MAGE_CYAN,
            ClassKind::Knight => Palette::KNIGHT_GOLD,
            ClassKind::Assassin => Palette::ASSASSIN_RED,
            ClassKind::Hunter => Palette::HUNTER_BRASS,
            ClassKind::Alchemist => Palette::ALCHEMIST_TEAL,
            ClassKind::Wanderer => Palette::TEXT_MUTED,
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "mage" => ClassKind::Mage,
            "knight" => ClassKind::Knight,
            "assassin" => ClassKind::Assassin,
            "hunter" => ClassKind::Hunter,
            "alchemist" => ClassKind::Alchemist,
            _ => ClassKind::Wanderer,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ClassKind::Wanderer => ClassKind::Mage,
            ClassKind::Mage => ClassKind::Knight,
            ClassKind::Knight => ClassKind::Assassin,
            ClassKind::Assassin => ClassKind::Hunter,
            ClassKind::Hunter => ClassKind::Alchemist,
            ClassKind::Alchemist => ClassKind::Wanderer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDef {
    pub id: String,
    pub name: String,
    pub title: String,
    pub lore: String,
    pub accent_hex: u32,
    pub gear_summary: String,
    pub weapon_name: String,
    pub offhand_name: String,
    pub has_spellbook: bool,
}

impl ClassDef {
    pub fn kind(&self) -> ClassKind {
        ClassKind::from_id(&self.id)
    }

    pub fn accent_color(&self) -> Color {
        Color::from_hex(self.accent_hex)
    }

    /// Loads class definitions from `data/classes.ron` with resilient built-in fallback.
    pub fn load_all() -> Vec<ClassDef> {
        fs::read_to_string("data/classes.ron")
            .ok()
            .and_then(|c| ron::from_str::<Vec<ClassDef>>(&c).ok())
            .unwrap_or_else(Self::builtin_classes)
    }

    pub fn builtin_classes() -> Vec<ClassDef> {
        vec![
            ClassDef {
                id: "mage".to_string(),
                name: "MAGE".to_string(),
                title: "Weaver of the Starless Deep".to_string(),
                lore: "Scholar of cold sorceries and forgotten cosmic fires.".to_string(),
                accent_hex: 0x58A6FF,
                gear_summary: "Pointed Witch Hat, Twilight Robe, Gnarled Staff".to_string(),
                weapon_name: "Gnarled Astrologer Staff".to_string(),
                offhand_name: "Focus Catalyst".to_string(),
                has_spellbook: true,
            },
            ClassDef {
                id: "knight".to_string(),
                name: "KNIGHT".to_string(),
                title: "Bulwark of the Shattered Vow".to_string(),
                lore: "An armored wanderer clad in scarred plate and an iron oath.".to_string(),
                accent_hex: 0xE6B800,
                gear_summary: "Great Helm, Plate Cuirass & Tabard, Longsword & Heater Shield".to_string(),
                weapon_name: "Tempered Longsword".to_string(),
                offhand_name: "Iron Heater Shield".to_string(),
                has_spellbook: false,
            },
            ClassDef {
                id: "assassin".to_string(),
                name: "ASSASSIN".to_string(),
                title: "Shadow of the Forgotten Crypt".to_string(),
                lore: "Silent stalker draped in shadowy cowls who strikes from veiled dark.".to_string(),
                accent_hex: 0xB32D2E,
                gear_summary: "Shadow Hood, Nightcloak Robes, Dual Obsidian Daggers".to_string(),
                weapon_name: "Obsidian Stiletto".to_string(),
                offhand_name: "Parrying Dagger".to_string(),
                has_spellbook: false,
            },
            ClassDef {
                id: "hunter".to_string(),
                name: "HUNTER".to_string(),
                title: "Privateer of the Ashen Tide".to_string(),
                lore: "Grit-tested mariner wielding blades forged with integrated flintlocks.".to_string(),
                accent_hex: 0xC48B52,
                gear_summary: "Hunter Tricorn, Dust-Worn Duster Coat, Dual Pistol Swords".to_string(),
                weapon_name: "Flintlock Pistol Sword (Main)".to_string(),
                offhand_name: "Flintlock Pistol Sword (Off)".to_string(),
                has_spellbook: false,
            },
            ClassDef {
                id: "alchemist".to_string(),
                name: "ALCHEMIST".to_string(),
                title: "Acolyte of the Arcane Eye".to_string(),
                lore: "Seeker of forbidden transmutations bearing a floating sigil of dark arts.".to_string(),
                accent_hex: 0x2EC4B6,
                gear_summary: "Alchemist Cowl, Vials & Leather Coat, Arcane Eye Emblem".to_string(),
                weapon_name: "Alchemical Pestle & Athame".to_string(),
                offhand_name: "Satchel of Volatile Vials".to_string(),
                has_spellbook: false,
            },
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellDef {
    pub name: String,
    pub cost: u32,
    pub element: String,
    pub desc: String,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellTier {
    pub tier: u32,
    pub title: String,
    pub requirement: String,
    pub spells: Vec<SpellDef>,
}

#[derive(Debug, Clone)]
pub struct SpellBook {
    pub tiers: Vec<SpellTier>,
}

impl Default for SpellBook {
    fn default() -> Self {
        Self::load()
    }
}

impl SpellBook {
    pub fn load() -> Self {
        fs::read_to_string("data/spells.ron")
            .ok()
            .and_then(|c| ron::from_str::<Vec<SpellTier>>(&c).ok())
            .map(|tiers| Self { tiers })
            .unwrap_or_else(Self::builtin_spells)
    }

    pub fn builtin_spells() -> Self {
        Self {
            tiers: vec![
                SpellTier {
                    tier: 1,
                    title: "TIER I: NOVICE INCANTATIONS".to_string(),
                    requirement: "Available at start".to_string(),
                    spells: vec![
                        SpellDef {
                            name: "Soul Dart".to_string(),
                            cost: 8,
                            element: "Arcane".to_string(),
                            desc: "Fires a sharp piercing dart of concentrated star-light.".to_string(),
                            unlocked: true,
                        },
                        SpellDef {
                            name: "Ashen Ward".to_string(),
                            cost: 14,
                            element: "Protection".to_string(),
                            desc: "Conjures a revolving crystalline barrier deflecting harm.".to_string(),
                            unlocked: true,
                        },
                        SpellDef {
                            name: "Kindle Spark".to_string(),
                            cost: 10,
                            element: "Flame".to_string(),
                            desc: "Ignites a crackling burst of embers that clings to foes.".to_string(),
                            unlocked: true,
                        },
                    ],
                },
                SpellTier {
                    tier: 2,
                    title: "TIER II: COLOSSEUM SORCERIES".to_string(),
                    requirement: "Requires Defeating: The Blind Gladiator".to_string(),
                    spells: vec![
                        SpellDef {
                            name: "Great Soul Spear".to_string(),
                            cost: 26,
                            element: "Arcane".to_string(),
                            desc: "Hurls a heavy lance of crystallized cosmos penetrating armor.".to_string(),
                            unlocked: false,
                        },
                    ],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub chosen_class: Option<ClassKind>,
}

impl SaveData {
    pub fn load() -> Self {
        fs::read_to_string("save.ron")
            .ok()
            .and_then(|c| ron::from_str::<SaveData>(&c).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(serialized) = ron::to_string(self) {
            let _ = fs::write("save.ron", serialized);
        }
    }

    pub fn reset(&mut self) {
        self.chosen_class = None;
        self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_definitions_and_kinds() {
        let classes = ClassDef::builtin_classes();
        assert_eq!(classes.len(), 5);
        assert_eq!(classes[0].kind(), ClassKind::Mage);
        assert_eq!(classes[1].kind(), ClassKind::Knight);
        assert_eq!(classes[2].kind(), ClassKind::Assassin);
        assert_eq!(classes[3].kind(), ClassKind::Hunter);
        assert_eq!(classes[4].kind(), ClassKind::Alchemist);
    }

    #[test]
    fn test_ron_serialization() {
        let save = SaveData {
            chosen_class: Some(ClassKind::Hunter),
        };
        let serialized = ron::to_string(&save).expect("Serialization failed");
        let deserialized: SaveData = ron::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.chosen_class, Some(ClassKind::Hunter));
    }

    #[test]
    fn test_spellbook_loading() {
        let book = SpellBook::builtin_spells();
        assert_eq!(book.tiers.len(), 2);
        assert_eq!(book.tiers[0].spells.len(), 3);
        assert!(book.tiers[0].spells[0].unlocked);
        assert!(!book.tiers[1].spells[0].unlocked);
    }
}
