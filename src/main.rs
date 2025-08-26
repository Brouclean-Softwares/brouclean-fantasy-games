use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use oauth2::basic::BasicClient;
use reqwest::Client;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};

pub mod app;
pub mod auth;
pub mod errors;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://devapp:{secrets.DB_PASSWORD}@localhost:5432/brouclean_fantasy_games"
    )]
    db: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        db,
        http_requester: Client::new(),
        key: Key::generate(),
        google_oauth_client: auth::google::build_oauth_client(&secrets),
    };

    let router = init_router(state);

    Ok(router.into())
}

fn init_router(state: AppState) -> Router {
    Router::new()
        .nest("/", app::init_router())
        .nest("/auth", auth::init_router())
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .with_state(state)
}

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    http_requester: Client,
    key: Key,
    google_oauth_client: BasicClient,
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
