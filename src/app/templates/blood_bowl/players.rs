use crate::errors::AppError;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::rosters::Roster;
use blood_bowl_rs::translation::TranslatedName;

pub fn movement_allowance_html(player: &Player, roster: &Roster) -> Result<String, AppError> {
    let value = player.movement_allowance(roster)?;
    let initial_value = player.movement_allowance_from_position(roster)?;

    if value == initial_value {
        Ok(value.to_string())
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}</span>", value))
    }
}

pub fn strength_html(player: &Player, roster: &Roster) -> Result<String, AppError> {
    let value = player.strength(roster)?;
    let initial_value = player.strength_from_position(roster)?;

    if value == initial_value {
        Ok(value.to_string())
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}</span>", value))
    }
}

pub fn agility_html(player: &Player, roster: &Roster) -> Result<String, AppError> {
    let value = player.agility(roster)?;
    let initial_value = player.agility_from_position(roster)?;

    if value == initial_value {
        Ok(format!("{}+", value))
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}+</span>", value))
    }
}

pub fn passing_ability_html(player: &Player, roster: &Roster) -> Result<String, AppError> {
    let value = player.passing_ability(roster)?;
    let initial_value = player.passing_ability_from_position(roster)?;

    match (value, initial_value) {
        (Some(value), Some(initial_value)) => {
            if value == initial_value {
                Ok(format!("{}+", value))
            } else {
                Ok(format!("<span class=\"uk-text-bold\">{}+</span>", value))
            }
        }
        (_, _) => Ok("-".to_string()),
    }
}

pub fn armour_value_html(player: &Player, roster: &Roster) -> Result<String, AppError> {
    let value = player.armour_value(roster)?;
    let initial_value = player.armour_value_from_position(roster)?;

    if value == initial_value {
        Ok(format!("{}+", value))
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}+</span>", value))
    }
}

pub fn skills_names_html(
    player: &Player,
    roster: &Roster,
    lang_id: &str,
) -> Result<String, AppError> {
    let initial_values: Vec<String> = player
        .skills_from_position(roster)?
        .iter()
        .map(|skill| skill.name(lang_id))
        .collect();

    let added_values: Vec<String> = player
        .added_skills(roster)?
        .iter()
        .map(|skill| {
            format!(
                "<span class=\"uk-text-bold\">{}</span>",
                skill.name(lang_id)
            )
        })
        .collect();

    Ok(vec![initial_values, added_values].concat().join(", "))
}
