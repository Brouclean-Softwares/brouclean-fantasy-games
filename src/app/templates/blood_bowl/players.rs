use crate::app::templates::blood_bowl::teams;
use crate::app::templates::{AlertMessage, BreadCrumb, NavigationBar, UrlLink};
use crate::data::blood_bowl::players::PlayerAdvancement;
use crate::data::blood_bowl::statistics::players::PlayerStatistics;
use crate::data::blood_bowl::teams::TeamLogo;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use blood_bowl_rs::advancements::{Advancement, AdvancementChoice};
use blood_bowl_rs::players::Player;
use blood_bowl_rs::teams::Team;
use blood_bowl_rs::translation::TranslatedName;
use blood_bowl_rs::versions::Version;
use std::vec;

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/players/player_page.html")]
pub struct PlayerPage {
    navigation_bar: NavigationBar,
    alert_message: Option<AlertMessage>,
    breadcrumb: BreadCrumb,
    link_url: String,
    number: i32,
    player: Player,
    team: Team,
    editable: bool,
    edit_mode: bool,
    can_buy: bool,
    can_buyout: bool,
    statistics: PlayerStatistics,
    player_advancement_blocs: Vec<PlayerAdvancementBloc>,
}

impl PlayerPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        alert_message: Option<AlertMessage>,
        link_url: String,
        number: i32,
        player: Player,
        player_advancements: Vec<PlayerAdvancement>,
        team: Team,
        editable: bool,
        edit_mode: bool,
        can_buy: bool,
        can_buyout: bool,
        statistics: PlayerStatistics,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            alert_message,
            breadcrumb: teams::breadcrumb().plus_link(UrlLink::from(
                "Équipe",
                &format!("/blood_bowl/teams/team?id={}", team.id),
            )),
            link_url,
            number,
            player: player.clone(),
            team,
            editable,
            edit_mode,
            can_buy,
            can_buyout,
            statistics,
            player_advancement_blocs: vec![
                PlayerAdvancementBloc::get(&player, player_advancements.get(0), 1, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
                PlayerAdvancementBloc::get(&player, player_advancements.get(1), 2, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
                PlayerAdvancementBloc::get(&player, player_advancements.get(2), 3, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
                PlayerAdvancementBloc::get(&player, player_advancements.get(3), 4, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
                PlayerAdvancementBloc::get(&player, player_advancements.get(4), 5, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
                PlayerAdvancementBloc::get(&player, player_advancements.get(5), 6, editable)
                    .unwrap_or_else(|error| PlayerAdvancementBloc::get_error(error.to_string())),
            ],
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "blood_bowl/players/player_advancement_bloc.html")]
struct PlayerAdvancementBloc {
    error_message: Option<String>,
    advancement: Option<Advancement>,
    advancement_number: usize,
    choice: Option<AdvancementChoice>,
    cost: Option<i32>,
    advancements_to_choose: Option<Vec<Advancement>>,
    choices_available: Vec<(AdvancementChoice, bool)>,
    editable: bool,
}

impl PlayerAdvancementBloc {
    fn get(
        player: &Player,
        player_advancement: Option<&PlayerAdvancement>,
        advancement_number: usize,
        editable: bool,
    ) -> Result<Self, AppError> {
        if let Some(player_advancement) = player_advancement {
            let advancement = player_advancement.advancement()?;
            let choice = player_advancement.choice()?;
            let cost = player_advancement.star_player_points_cost();
            let advancements_to_choose = player_advancement.options_to_choose()?;

            Ok(Self {
                error_message: None,
                advancement,
                advancement_number,
                choice,
                cost,
                advancements_to_choose,
                choices_available: vec![],
                editable,
            })
        } else {
            let mut choices_available = vec![];

            if advancement_number == player.advancements.len() + 1 {
                for choice in AdvancementChoice::list_could_be_available_for_player(player) {
                    choices_available.push((choice.clone(), choice.is_buyable_for_player(player)));
                }
            }

            Ok(Self {
                error_message: None,
                advancement: None,
                advancement_number,
                choice: None,
                cost: None,
                advancements_to_choose: None,
                choices_available,
                editable,
            })
        }
    }

    fn get_error(error_message: String) -> Self {
        Self {
            error_message: Some(error_message),
            advancement: None,
            advancement_number: 0,
            choice: None,
            cost: None,
            advancements_to_choose: None,
            choices_available: vec![],
            editable: false,
        }
    }
}

fn characteristic_value_into_html(
    value: Option<u8>,
    value_from_position: Option<u8>,
    str_after_value: &str,
) -> String {
    match (value, value_from_position) {
        (Some(value), Some(initial_value)) => {
            if value == initial_value {
                format!("{}{}", value, str_after_value)
            } else {
                format!(
                    "<span class=\"uk-text-bold\">{}{}</span>",
                    value, str_after_value
                )
            }
        }
        (_, _) => "-".to_string(),
    }
}

pub fn movement_allowance_html(player: &Player) -> String {
    characteristic_value_into_html(
        player.movement_allowance(),
        player.movement_allowance_from_position(),
        "",
    )
}

pub fn strength_html(player: &Player) -> String {
    characteristic_value_into_html(player.strength(), player.strength_from_position(), "")
}

pub fn agility_html(player: &Player) -> String {
    characteristic_value_into_html(player.agility(), player.agility_from_position(), "+")
}

pub fn passing_ability_html(player: &Player) -> String {
    characteristic_value_into_html(
        player.passing_ability(),
        player.passing_ability_from_position(),
        "+",
    )
}

pub fn armour_value_html(player: &Player) -> String {
    characteristic_value_into_html(
        player.armour_value(),
        player.armour_value_from_position(),
        "+",
    )
}

pub fn skills_names_html(player: &Player, lang_id: &str) -> String {
    let initial_values: Vec<String> = player
        .skills_from_position()
        .iter()
        .map(|skill| skill.name(lang_id))
        .collect();

    let added_values: Vec<String> = player
        .added_skills()
        .iter()
        .map(|skill| {
            format!(
                "<span class=\"uk-text-bold\">{}</span>",
                skill.name(lang_id)
            )
        })
        .collect();

    vec![initial_values, added_values].concat().join(", ")
}
