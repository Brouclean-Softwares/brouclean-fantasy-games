use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

----------------------- Sérialisation (Rust → JSON)

fn main() {
    let profile = CharacterProfile::Warhammer {
        strength: 40,
        toughness: 30,
        agility: 25,
    };

    let json = serde_json::to_string_pretty(&profile).unwrap();

    println!("{}", json);
}

-------------------------- Désérialisation (JSON → Rust)

fn main() {
    let data = r#"
    {
        "type": "starwars",
        "force_sensitive": true,
        "midichlorians": 12000
    }
    "#;

    let profile: CharacterProfile =
        serde_json::from_str(data).unwrap();

    println!("{:?}", profile);
}
 */
