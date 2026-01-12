use std::process::ExitStatus;

use rocket::{Response, http, response::Responder};

use crate::{QuestId, UserId};

#[derive(Debug)]
pub enum Error {
    InvalidResponse,
    ServerUnreachable,
    IncoherentState,
    Unauthorized,
    IO(std::io::Error),
    InvalidUsername(String),
    Reqwest(reqwest::Error),
    Sqlx(sqlx::Error),
    Nats(async_nats::Error),
    Json(serde_json::Error),
    QuestContextGeneratorFailed {
        quest: QuestId,
        user: UserId,
        exit_status: ExitStatus,
    },
    InvalidUuid(uuid::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(error: sqlx::migrate::MigrateError) -> Self {
        Self::Sqlx(error.into())
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Sqlx(error)
    }
}

impl<Kind> From<async_nats::error::Error<Kind>> for Error
where
    Kind: 'static + Clone + std::fmt::Debug + std::fmt::Display + PartialEq + Send + Sync,
{
    fn from(error: async_nats::error::Error<Kind>) -> Self {
        Self::Nats(error.into())
    }
}

impl From<async_nats::Error> for Error {
    fn from(error: async_nats::Error) -> Self {
        Self::Nats(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<uuid::Error> for Error {
    fn from(error: uuid::Error) -> Self {
        Self::InvalidUuid(error)
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        eprintln!("internal error: {:?}", self);
        Response::build()
            .status(match self {
                Self::InvalidUsername(_) => http::Status::BadRequest,
                Self::Unauthorized => http::Status::Unauthorized,
                _ => http::Status::InternalServerError,
            })
            .ok()
    }
}
