use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CharacterProfile {
    #[serde(rename = "RichText")]
    RichText { rich_text: String },
}

/*
    #[serde(rename = "WarhammerV1CharacterSheet")]
    WarhammerV1CharacterSheet {
        // Description
        race: String,
        gender: String,
        vocation: String,
        alignment: String,
        age: i32,
        height: String,
        weight: String,
        hairs: String,
        eyes: String,
        description: String,

        // Characteristics
        initial_movement: i32,
        career_movement: i32,
        actual_movement: i32,
    },


-------------------------------------- Variante encore plus flexible (si tu veux évoluer)
Si tu veux éviter de casser des saves :


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum CharacterProfile {
    Warhammer {
        strength: i32,
        toughness: i32,
        #[serde(default)]
        agility: i32,
    },

    StarWars {
        force_sensitive: bool,
        midichlorians: i32,
    },

    #[serde(other)]
    Unknown,
}
 */

impl CharacterProfile {
    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string_pretty(self).ok()
    }

    pub fn from_json(json: &String) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
