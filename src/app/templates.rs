use crate::data::users::User;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use std::fmt;
use std::fmt::Formatter;

pub mod blood_bowl;
pub mod users;

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    profile: Option<User>,
    google_connection_url: String,
}

impl HomePage {
    pub fn get(app_state: AppState, profile: Option<User>) -> Self {
        Self {
            navigation_bar: NavigationBar::get(&app_state, &profile),
            profile,
            google_connection_url: crate::auth::google::connection_url(app_state),
        }
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
