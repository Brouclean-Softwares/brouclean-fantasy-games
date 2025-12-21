use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use dotenv::dotenv;
use oauth2::basic::BasicClient;
use reqwest::Client;
use sqlx::PgPool;
use std::env;
use tower_http::services::{ServeDir, ServeFile};
use tracing::Level;

pub mod app;
pub mod auth;
pub mod data;
pub mod errors;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let tracing_level = match env::var("LOG_LEVEL") {
        Ok(log_level) => {
            if log_level.to_uppercase().eq("DEBUG") {
                Level::DEBUG
            } else if log_level.to_uppercase().eq("ERROR") {
                Level::ERROR
            } else if log_level.to_uppercase().eq("TRACE") {
                Level::TRACE
            } else if log_level.to_uppercase().eq("WARN") {
                Level::WARN
            } else {
                Level::INFO
            }
        }

        Err(_) => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .init();

    let app_url = env::var("APP_URL").expect("APP_URL must be set");

    let admin_email = env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL must be set");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPool::connect(&database_url).await.unwrap();

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let state = AppState {
        db,
        http_requester: Client::new(),
        key: Key::generate(),
        google_oauth_client: auth::google::build_oauth_client(),
        admin_email,
    };

    let router = init_router(state);

    tracing::info!("Application binding on : {}", app_url);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

fn init_router(state: AppState) -> Router {
    Router::new()
        .nest("/", app::init_router())
        .nest("/auth", auth::init_router())
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .nest_service(
            "/manifest.webmanifest",
            ServeFile::new("assets/manifest.webmanifest"),
        )
        .with_state(state)
}

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    http_requester: Client,
    key: Key,
    google_oauth_client: BasicClient,
    admin_email: String,
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
