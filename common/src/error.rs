use rocket::{Response, response::Responder};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Error {
    InvalidResponse,
    ServerUnreachable,
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        eprintln!("internal error: {:?}", self);
        Response::build()
            .status(rocket::http::Status::InternalServerError)
            .ok()
    }
}
