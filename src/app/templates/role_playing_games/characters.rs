use crate::app::templates::{role_playing_games, BreadCrumb, NavigationBar, UrlLink};
use crate::data::role_playing_games::characters::{Character, CharacterRow};
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use crate::AppState;
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
        }
    }
}
