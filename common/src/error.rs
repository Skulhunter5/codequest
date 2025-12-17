use rocket::{Response, response::Responder};

#[derive(Debug)]
pub enum Error {
    InvalidResponse,
    ServerUnreachable,
    IncoherentState,
    Sqlx(sqlx::Error),
    Nats(async_nats::Error),
    Json(serde_json::Error),
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

impl From<async_nats::ConnectError> for Error {
    fn from(error: async_nats::ConnectError) -> Self {
        Self::Nats(error.into())
    }
}

impl From<async_nats::jetstream::context::CreateStreamError> for Error {
    fn from(error: async_nats::jetstream::context::CreateStreamError) -> Self {
        Self::Nats(error.into())
    }
}

impl From<async_nats::jetstream::context::PublishError> for Error {
    fn from(error: async_nats::jetstream::context::PublishError) -> Self {
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

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        eprintln!("internal error: {:?}", self);
        Response::build()
            .status(rocket::http::Status::InternalServerError)
            .ok()
    }
}
