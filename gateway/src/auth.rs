use std::sync::Arc;

use codequest_common::services::UserService;
use rocket::{
    FromForm, Request, State, async_trait,
    form::Form,
    http::{self, Cookie, CookieJar},
    request::{FromRequest, Outcome},
    response::Redirect,
    serde::json::Json,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AuthUser {
    pub(crate) username: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = request.cookies();

        if let Some(cookie) = jar.get_private("user_id") {
            let username = cookie.value().to_owned();
            let user_service = match request.guard::<&State<Arc<dyn UserService>>>().await {
                Outcome::Success(user_service) => user_service,
                _ => return Outcome::Error((http::Status::InternalServerError, ())),
            };
            if user_service.user_exists(&username).await {
                return Outcome::Success(AuthUser { username });
            } else {
                jar.remove_private("user_id");
            }
        }

        Outcome::Error((http::Status::Unauthorized, ()))
    }
}

#[rocket::get("/logout")]
pub async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("user_id");
    Redirect::to("/")
}

#[derive(FromForm)]
pub struct SignupForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
pub struct SignupResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl SignupResponse {
    fn success(redirect: String) -> Self {
        Self {
            success: true,
            redirect: Some(redirect),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            redirect: None,
            error: Some(error),
        }
    }
}

#[rocket::post("/signup", data = "<form>")]
pub async fn signup(
    form: Form<SignupForm<'_>>,
    jar: &CookieJar<'_>,
    user_service: &State<Arc<dyn UserService>>,
) -> (http::Status, Json<SignupResponse>) {
    let SignupForm { username, password } = *form;

    if user_service.add_user(username, password).await {
        jar.add_private(Cookie::new("user_id", username.to_owned()));
        (
            http::Status::Ok,
            Json(SignupResponse::success("/".to_owned())),
        )
    } else {
        (
            http::Status::Unauthorized,
            Json(SignupResponse::error("Username already taken".to_owned())),
        )
    }
}

#[derive(FromForm)]
pub struct LoginForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
pub struct LoginResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl LoginResponse {
    fn success(redirect: String) -> Self {
        Self {
            success: true,
            redirect: Some(redirect),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            redirect: None,
            error: Some(error),
        }
    }
}

#[rocket::post("/login", data = "<form>")]
pub async fn login(
    form: Form<LoginForm<'_>>,
    jar: &CookieJar<'_>,
    user_service: &State<Arc<dyn UserService>>,
) -> (http::Status, Json<LoginResponse>) {
    let LoginForm { username, password } = *form;

    if user_service.verify_password(username, password).await {
        jar.add_private(Cookie::new("user_id", username.to_owned()));
        (
            http::Status::Ok,
            Json(LoginResponse::success("/".to_owned())),
        )
    } else {
        (
            http::Status::Unauthorized,
            Json(LoginResponse::error(
                "Invalid username or password".to_owned(),
            )),
        )
    }
}
