use std::{
    fs::{self, DirBuilder},
    sync::Arc,
};

use argon2::password_hash::{SaltString, rand_core::OsRng};
use rocket::{
    Request, State, async_trait,
    form::{Form, FromForm},
    http::{self, Cookie, CookieJar},
    request::{FromRequest, Outcome},
    response::Redirect,
    routes,
    serde::json::Json,
};
use rocket_dyn_templates::Template;
use serde::Serialize;
use services::{ConstQuestService, InMemoryUserService, QuestService, UserService};

mod pages;
mod services;

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";

fn load_or_generate_salt() -> SaltString {
    if let Ok(salt) = fs::read_to_string(&SALT_FILE) {
        return SaltString::from_b64(&salt).expect("failed to create salt");
    }

    let salt = SaltString::generate(&mut OsRng);
    fs::write(&SALT_FILE, salt.as_str()).expect("failed to write salt to file");
    return salt;
}

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
            return Outcome::Success(AuthUser { username });
        }

        Outcome::Error((http::Status::Unauthorized, ()))
    }
}

#[rocket::get("/logout")]
async fn logout(jar: &CookieJar<'_>) -> Redirect {
    jar.remove_private("user_id");
    Redirect::to("/")
}

#[derive(FromForm)]
struct SignupForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct SignupResponse {
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
async fn signup(
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
struct LoginForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct LoginResponse {
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
async fn login(
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

pub struct Quest<'a> {
    pub name: &'a str,
    pub id: &'a str,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");
    let salt = load_or_generate_salt();

    // let rocket_config = rocket::Config::figment().merge(("template_dir", "static/"));
    // rocket::custom(&rocket_config)

    rocket::build()
        .mount(
            "/",
            routes![
                pages::index,
                pages::about,
                pages::signup,
                pages::login,
                pages::stylesheet,
                pages::quests,
                pages::quest,
                login,
                signup,
                logout,
            ],
        )
        .attach(Template::fairing())
        .manage(Arc::new(InMemoryUserService::new(salt)) as Arc<dyn UserService>)
        .manage(Arc::new(ConstQuestService::new()) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
