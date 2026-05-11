use crate::AppState;
use crate::app::templates::blood_bowl::games::GameCard;
use crate::app::templates::blood_bowl::games::GamesScheduleTable;
use crate::data::blood_bowl::games::GameSummary;
use crate::data::users::User;
use askama::Template;
use askama_web::WebTemplate;
use std::fmt;
use std::fmt::Formatter;

pub mod blood_bowl;
pub mod role_playing_games;

pub mod users;

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    google_connection_url: String,
    bb_playing_games: Vec<GameSummary>,
    bb_scheduled_games: Vec<GameSummary>,
}

impl HomePage {
    pub async fn get(app_state: AppState, profile: Option<User>) -> Self {
        let bb_playing_games = if let Ok(games) =
            crate::data::blood_bowl::games::select_all_playing(&app_state).await
        {
            games
        } else {
            Vec::new()
        };

        let bb_scheduled_games = if let Some(coach_id) = profile.clone().and_then(|coach| coach.id)
        {
            if let Ok(games) =
                crate::data::blood_bowl::games::select_scheduled_for_coach(&app_state, &coach_id)
                    .await
            {
                games
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            profile,
            google_connection_url: crate::auth::google::connection_url(app_state),
            bb_playing_games,
            bb_scheduled_games,
        }
    }
}

#[derive(Clone)]
pub struct UrlLink {
    pub name: String,
    pub url: String,
}

impl UrlLink {
    pub fn from(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "breadcrumb.html")]
pub struct BreadCrumb {
    url_links: Vec<UrlLink>,
}

impl BreadCrumb {
    pub fn only_home() -> Self {
        Self { url_links: vec![] }
    }

    pub fn plus_link(mut self, url_link: UrlLink) -> Self {
        self.url_links.push(url_link);
        self
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "navigation_bar.html")]
pub struct NavigationBar {
    profile: Option<User>,
    google_connection_url: String,
    is_admin: bool,
}

impl NavigationBar {
    pub fn get(app_state: &AppState, profile: &Option<User>) -> Self {
        let is_admin = match profile {
            Some(user) => user.is_admin(app_state),
            _ => false,
        };

        Self {
            profile: profile.clone(),
            google_connection_url: crate::auth::google::connection_url(app_state.clone()),
            is_admin,
        }
    }
}

pub enum AlertType {
    Primary,
    Success,
    Warning,
    Danger,
}

impl fmt::Display for AlertType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AlertType::Primary => write!(f, "primary"),
            AlertType::Success => write!(f, "success"),
            AlertType::Warning => write!(f, "warning"),
            AlertType::Danger => write!(f, "danger"),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "alert_message.html")]
pub struct AlertMessage {
    pub alert_type: AlertType,
    pub message: String,
}

pub fn month_to_fr(month_number: u32) -> String {
    match month_number {
        1 => "Janvier",
        2 => "Février",
        3 => "Mars",
        4 => "Avril",
        5 => "Mai",
        6 => "Juin",
        7 => "Juillet",
        8 => "Août",
        9 => "Septembre",
        10 => "Octobre",
        11 => "Novembre",
        12 => "Décembre",
        _ => "",
    }
    .to_string()
}
