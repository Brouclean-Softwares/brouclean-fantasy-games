use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use blood_bowl_rs::translation::TranslatedName;
use oauth2::HttpClientError;
use oauth2::basic::BasicErrorResponse;
use reqwest::Error;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    SQL(#[from] sqlx::Error),
    Request(#[from] reqwest::Error),
    TokenError(#[from] oauth2::RequestTokenError<HttpClientError<Error>, BasicErrorResponse>),
    Unauthorized,
    OptionError,
    ParseIntError(#[from] std::num::TryFromIntError),
    ParseDateError(#[from] chrono::format::ParseError),
    JsonError(#[from] serde_json::Error),
    FromRequestPartsError(#[from] std::convert::Infallible),
    BloodBowlError(#[from] blood_bowl_rs::errors::Error),
    BloodBowlAppError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let response = match self {
            Self::SQL(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Request(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::TokenError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized!".to_string()),
            Self::OptionError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Attempted to get a non-none value but found none".to_string(),
            ),
            Self::ParseIntError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::ParseDateError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::JsonError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::FromRequestPartsError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::BloodBowlError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::BloodBowlAppError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        response.into_response()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::SQL(error) => write!(f, "Oups! Souci avec la base de données : {}", error),
            AppError::Request(error) => {
                write!(f, "Oups! Souci avec les appels internets : {}", error)
            }
            AppError::TokenError(error) => write!(f, "Oups! Souci de connexion : {}", error),
            AppError::Unauthorized => write!(f, "Pas le droit d'accéder à ce contenu"),
            AppError::OptionError => write!(f, "Oups! Souci avec une valeur inexistante"),
            AppError::ParseIntError(error) => write!(
                f,
                "Oups! Souci lors d'une conversion de données : {}",
                error
            ),
            AppError::ParseDateError(error) => {
                write!(f, "Oups! Souci lors d'une conversion de dates : {}", error)
            }
            AppError::JsonError(error) => write!(
                f,
                "Oups! Souci lors d'une conversion de données en Json : {}",
                error
            ),
            AppError::FromRequestPartsError(error) => write!(
                f,
                "Oups! Souci lors du déchiffrage de la requète web : {}",
                error
            ),
            AppError::BloodBowlError(error) => {
                write!(
                    f,
                    "Règles de blood bowl non respectées : {}",
                    error.name("fr")
                )
            }
            AppError::BloodBowlAppError(error) => {
                write!(f, "Règles de blood bowl non respectées : {}", error)
            }
        }
    }
}

impl AppError {
    pub fn log_and_redirect(&self, redirect: Redirect) -> Redirect {
        tracing::error!("{}", self);

        redirect
    }
}
