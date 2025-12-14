use rocket::{Response, response::Responder};

#[derive(Debug)]
pub enum Error {
    InvalidResponse,
    ServerUnreachable,
    IncoherentState,
    Sqlx(sqlx::Error),
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

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        eprintln!("internal error: {:?}", self);
        Response::build()
            .status(rocket::http::Status::InternalServerError)
            .ok()
    }
}
