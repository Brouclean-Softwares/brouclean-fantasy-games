use crate::app::templates::NavigationBar;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::players::Player;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use std::vec;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/players/player_page.html")]
pub struct PlayerPage {
    navigation_bar: NavigationBar,
    number: i32,
    player: Player,
    team: Team,
    editable: bool,
    edit_mode: bool,
    player_advancements: Vec<PlayerAdvancement>,
}

impl PlayerPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        number: i32,
        player: Player,
        team: Team,
        editable: bool,
        edit_mode: bool,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            number,
            player: player.clone(),
            team,
            editable,
            edit_mode,
            player_advancements: vec![
                PlayerAdvancement::get(&player, 1, editable),
                PlayerAdvancement::get(&player, 2, editable),
                PlayerAdvancement::get(&player, 3, editable),
                PlayerAdvancement::get(&player, 4, editable),
                PlayerAdvancement::get(&player, 5, editable),
                PlayerAdvancement::get(&player, 6, editable),
            ],
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/players/player_advancement.html")]
struct PlayerAdvancement {
    advancement_number: usize,
    player: Player,
    editable: bool,
}

impl PlayerAdvancement {
    fn get(player: &Player, advancement_number: usize, editable: bool) -> Self {
        Self {
            advancement_number,
            player: player.clone(),
            editable,
        }
    }
}

pub fn movement_allowance_html(player: &Player) -> Result<String, AppError> {
    let value = player.movement_allowance()?;
    let initial_value = player.movement_allowance_from_position()?;

    if value == initial_value {
        Ok(value.to_string())
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}</span>", value))
    }
}

pub fn strength_html(player: &Player) -> Result<String, AppError> {
    let value = player.strength()?;
    let initial_value = player.strength_from_position()?;

    if value == initial_value {
        Ok(value.to_string())
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}</span>", value))
    }
}

pub fn agility_html(player: &Player) -> Result<String, AppError> {
    let value = player.agility()?;
    let initial_value = player.agility_from_position()?;

    if value == initial_value {
        Ok(format!("{}+", value))
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}+</span>", value))
    }
}

pub fn passing_ability_html(player: &Player) -> Result<String, AppError> {
    let value = player.passing_ability()?;
    let initial_value = player.passing_ability_from_position()?;

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

pub fn armour_value_html(player: &Player) -> Result<String, AppError> {
    let value = player.armour_value()?;
    let initial_value = player.armour_value_from_position()?;

    if value == initial_value {
        Ok(format!("{}+", value))
    } else {
        Ok(format!("<span class=\"uk-text-bold\">{}+</span>", value))
    }
}

pub fn skills_names_html(player: &Player, lang_id: &str) -> Result<String, AppError> {
    let initial_values: Vec<String> = player
        .skills_from_position()?
        .iter()
        .map(|skill| skill.name(lang_id))
        .collect();

    let added_values: Vec<String> = player
        .added_skills()?
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
