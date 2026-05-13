use crate::AppState;
use crate::app::templates::{BreadCrumb, NavigationBar, UrlLink, role_playing_games};
use crate::data::role_playing_games::games::Game;
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;

pub fn breadcrumb() -> BreadCrumb {
    role_playing_games::breadcrumb().plus_link(UrlLink::from("Jeux", "/role_playing_games/games"))
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/games/games_page.html")]
pub struct GamesPage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    profile: Option<User>,
    games: Vec<Game>,
}

impl GamesPage {
    pub fn get(app_state: AppState, profile: Option<User>, games: Vec<Game>) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: role_playing_games::breadcrumb(),
            profile,
            games,
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "role_playing_games/games/game_page.html")]
pub struct GamePage {
    navigation_bar: NavigationBar,
    breadcrumb: BreadCrumb,
    game: Game,
    deletable: bool,
    editable: bool,
    edit_mode: bool,
    field_edited: String,
}

impl GamePage {
    pub fn get(
        app_state: AppState,
        profile: Option<User>,
        game: Game,
        deletable: bool,
        editable: bool,
        edit_mode: bool,
        field_edited: Option<String>,
    ) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            breadcrumb: breadcrumb(),
            game,
            deletable,
            editable,
            edit_mode,
            field_edited: field_edited.unwrap_or_default(),
        }
    }
}
