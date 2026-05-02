use crate::app::templates::role_playing_games::characters::{
    AddNewCharacterButton, OwnedCharactersBlock,
};
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink};
use crate::data::role_playing_games::characters::CharacterRow;
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use crate::errors::AppError;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;

pub mod characters;
pub mod games;

pub fn breadcrumb() -> BreadCrumb {
    BreadCrumb::only_home().plus_link(UrlLink::from("Jeux de rôle", "/role_playing_games"))
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    owned_characters_block: OwnedCharactersBlock,
}

impl HomePage {
    pub async fn get(
        app_state: &AppState,
        profile: &User,
        owned_characters: Vec<CharacterRow>,
        games: Vec<Game>,
    ) -> Result<Self, AppError> {
        Ok(Self {
            navigation_bar: NavigationBar::get(&app_state, &Some(profile.clone())),
            breadcrumb: BreadCrumb::only_home(),
            owned_characters_block: OwnedCharactersBlock {
                owned_characters,
                add_new_character_button: AddNewCharacterButton {
                    profile: Some(profile.clone()),
                    games,
                },
            },
        })
    }
}
