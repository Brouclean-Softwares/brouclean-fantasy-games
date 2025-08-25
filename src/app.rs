use crate::auth::UserProfile;
use crate::AppState;
use askama::Template;
use askama_web::WebTemplate;
use axum::extract::State;
use axum::routing::get;
use axum::Router;

pub fn init_router() -> Router<AppState> {
    Router::new().route("/", get(home_page))
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
    profile: Option<UserProfile>,
    google_connection_url: String,
}

impl HomePage {
    pub fn from(app_state: AppState, profile: Option<UserProfile>) -> Self {
        Self {
            profile,
            google_connection_url: crate::auth::google::connection_url(app_state),
        }
    }
}
