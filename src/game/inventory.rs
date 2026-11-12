use crate::assets::palette::Palette;
use crate::engine::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
}

impl ItemRarity {
    pub fn color(&self) -> Color {
        match self {
            ItemRarity::Common => Palette::TEXT_NORMAL,
            ItemRarity::Uncommon => Palette::MAGE_CYAN,
            ItemRarity::Rare => Palette::TEXT_GOLD,
            ItemRarity::Mythic => Color::rgb(220, 80, 255),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub name: String,
    pub count: u32,
    pub rarity: ItemRarity,
    pub description: String,
    pub category: &'static str,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub slots: [Option<Item>; 20], // 5x4 grid
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        let mut inv = Self {
            slots: Default::default(),
        };

        // Starting items for the Adventurer
        inv.add_item(Item {
            name: "Estus Flask".to_string(),
            count: 3,
            rarity: ItemRarity::Rare,
            description: "Restores weary flesh and kindles the inner ember.".to_string(),
            category: "Consumable",
        });

        inv.add_item(Item {
            name: "Traveler's Map".to_string(),
            count: 1,
            rarity: ItemRarity::Uncommon,
            description: "Parchment tracking the ruined colosseum boundaries.".to_string(),
            category: "Key Item",
        });

        inv.add_item(Item {
            name: "Titanite Shard".to_string(),
            count: 2,
            rarity: ItemRarity::Common,
            description: "Hardened mineral used to reinforce basic armaments.".to_string(),
            category: "Material",
        });

        inv
    }

    pub fn add_item(&mut self, item: Item) -> bool {
        // First try stacking with existing matching item
        for slot in &mut self.slots {
            if let Some(existing) = slot.as_mut().filter(|e| e.name == item.name) {
                existing.count += item.count;
                return true;
            }
        }

        // Otherwise find first empty slot
        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(item);
                return true;
            }
        }

        false
    }

    /// Adds debug items on F5 hotkey
    pub fn add_debug_items(&mut self) {
        self.add_item(Item {
            name: "Firebomb".to_string(),
            count: 5,
            rarity: ItemRarity::Common,
            description: "Earthen pot filled with volatile pitch. Explodes in flames.".to_string(),
            category: "Consumable",
        });

        self.add_item(Item {
            name: "Homeward Bone".to_string(),
            count: 1,
            rarity: ItemRarity::Uncommon,
            description: "White fragment that returns the carrier to the last bonfire rested at.".to_string(),
            category: "Consumable",
        });

        self.add_item(Item {
            name: "Ashen Cinderstone".to_string(),
            count: 3,
            rarity: ItemRarity::Mythic,
            description: "Condensed primordial cinder from ancient champion pyres.".to_string(),
            category: "Material",
        });
    }
}
