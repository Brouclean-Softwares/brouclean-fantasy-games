use crate::AppState;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink, role_playing_games};
use crate::data::role_playing_games::campaigns::sessions::GameSessionWithCampaign;
use crate::data::role_playing_games::characters::{Character, CharacterRow};
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;

pub fn breadcrumb() -> BreadCrumb {
    role_playing_games::breadcrumb().plus_link(UrlLink::from(
        "Personnages",
        "/role_playing_games/characters",
    ))
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/characters_page.html")]
pub struct CharactersPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    characters: Vec<CharacterRow>,
    add_new_character_button: AddNewCharacterButton,
}

impl CharactersPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        characters: Vec<CharacterRow>,
        games: Vec<Game>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: role_playing_games::breadcrumb(),
            characters,
            add_new_character_button: AddNewCharacterButton { profile, games },
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/add_new_character_button.html")]
pub struct AddNewCharacterButton {
    pub profile: Option<User>,
    pub games: Vec<Game>,
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/owned_characters_block.html")]
pub struct OwnedCharactersBlock {
    pub owned_characters: Vec<CharacterRow>,
    pub add_new_character_button: AddNewCharacterButton,
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/character_page.html")]
pub struct CharacterPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    character: Character,
    tab_name: String,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
    is_owner: bool,
    games: Vec<Game>,
    campaigns_with_arcs_and_sessions: Vec<(i32, String, Vec<(i32, String, Vec<(i32, String)>)>)>,
}

impl CharacterPage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        character: Character,
        tab_name: Option<String>,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
        games: Vec<Game>,
        sessions_with_campaign: Vec<GameSessionWithCampaign>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            character,
            tab_name: tab_name.unwrap_or("info".to_owned()),
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
            is_owner: editable,
            games,
            campaigns_with_arcs_and_sessions: Self::extract_campaigns_with_arcs_and_sessions(
                sessions_with_campaign,
            ),
        }
    }

    fn extract_campaigns_with_arcs_and_sessions(
        sessions_with_campaign: Vec<GameSessionWithCampaign>,
    ) -> Vec<(i32, String, Vec<(i32, String, Vec<(i32, String)>)>)> {
        let mut campaigns_with_arcs_and_sessions = Vec::new();

        let mut campaign_with_arcs = (0, String::new(), Vec::new());
        let mut arc_with_sessions = (0, String::new(), Vec::new());

        for (index, session_with_campaign) in sessions_with_campaign.iter().enumerate() {
            if campaign_with_arcs.0 != 0
                && campaign_with_arcs.0 != session_with_campaign.campaign_id
            {
                campaigns_with_arcs_and_sessions.push(campaign_with_arcs.clone());

                campaign_with_arcs.2 = Vec::new();
            }

            campaign_with_arcs.0 = session_with_campaign.campaign_id.clone();
            campaign_with_arcs.1 = session_with_campaign.campaign_name.clone();

            if arc_with_sessions.0 != 0 && arc_with_sessions.0 != session_with_campaign.arc_id {
                campaign_with_arcs.2.push(arc_with_sessions.clone());

                arc_with_sessions.2 = Vec::new();
            }

            arc_with_sessions.0 = session_with_campaign.arc_id.clone();
            arc_with_sessions.1 = session_with_campaign.arc_indexed_name();

            arc_with_sessions.2.push((
                session_with_campaign.id,
                session_with_campaign.session_indexed_name(),
            ));

            if index == sessions_with_campaign.len() - 1 {
                campaign_with_arcs.2.push(arc_with_sessions.clone());
                campaigns_with_arcs_and_sessions.push(campaign_with_arcs.clone());
            }
        }

        tracing::error!("Sessions: {:?}", campaigns_with_arcs_and_sessions);

        campaigns_with_arcs_and_sessions
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/character_selector.html")]
pub struct CharacterSelector {
    character_filtered_list: CharacterFilteredList,
    input_id_to_change: String,
    game_id: i32,
}

impl CharacterSelector {
    pub fn get(input_id_to_change: String, game_id: i32) -> Self {
        Self {
            character_filtered_list: CharacterFilteredList::get(vec![], input_id_to_change.clone()),
            input_id_to_change,
            game_id,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/characters/character_filtered_list.html")]
pub struct CharacterFilteredList {
    characters: Vec<CharacterRow>,
    input_id_to_change: String,
}

impl CharacterFilteredList {
    pub fn get(characters: Vec<CharacterRow>, input_id_to_change: String) -> Self {
        Self {
            characters,
            input_id_to_change,
        }
    }
}
