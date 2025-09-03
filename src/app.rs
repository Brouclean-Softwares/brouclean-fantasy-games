use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::routing::get;
use axum::Router;

pub mod blood_bowl;

pub fn init_router() -> Router<AppState> {
    Router::new()
        .nest("/blood_bowl", blood_bowl::init_router())
        .route("/", get(home_page))
}

pub async fn home_page(
    State(app_state): State<AppState>,
    profile: Option<UserProfile>,
) -> HomePage {
    HomePage::from(app_state, profile)
}

#[derive(Template, WebTemplate)]
#[template(path = "home_page.html")]
pub struct HomePage {
    navigation_bar: NavigationBar,
    profile: Option<UserProfile>,
    google_connection_url: String,
}

impl HomePage {
    pub fn from(app_state: AppState, profile: Option<UserProfile>) -> Self {
        Self {
            navigation_bar: NavigationBar::from(&app_state, &profile),
            profile,
            google_connection_url: crate::auth::google::connection_url(app_state),
        }
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "navigation_bar.html")]
pub struct NavigationBar {
    profile: Option<UserProfile>,
    google_connection_url: String,
}

impl NavigationBar {
    pub fn from(app_state: &AppState, profile: &Option<UserProfile>) -> Self {
        Self {
            profile: profile.clone(),
            google_connection_url: crate::auth::google::connection_url(app_state.clone()),
        }
    }
}
