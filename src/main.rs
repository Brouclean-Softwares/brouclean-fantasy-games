use axum::{extract::FromRef, Extension, Router};
use axum_extra::extract::cookie::Key;
use oauth2::basic::BasicClient;
use reqwest::Client;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

pub mod app;
pub mod auth;
pub mod errors;

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    ctx: Client,
    key: Key,
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

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

    let google_oauth_id = secrets.get("GOOGLE_OAUTH_CLIENT_ID").unwrap();
    let google_oauth_secret = secrets.get("GOOGLE_OAUTH_CLIENT_SECRET").unwrap();
    let google_oauth_client =
        auth::google::build_oauth_client(google_oauth_id.clone(), google_oauth_secret);

    let ctx = Client::new();

    let state = AppState {
        db,
        ctx,
        key: Key::generate(),
    };

    let router = init_router(state, google_oauth_client, google_oauth_id);

    Ok(router.into())
}

fn init_router(state: AppState, oauth_client: BasicClient, oauth_id: String) -> Router {
    Router::new()
        .nest("/", app::init_router(oauth_id))
        .nest("/auth", auth::init_router())
        .layer(Extension(oauth_client))
        .with_state(state)
}
