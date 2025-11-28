use std::{
    collections::HashMap,
    fs::{self, DirBuilder},
    sync::Arc,
};

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use rocket::{
    Request, State, async_trait,
    form::{Form, FromForm},
    http::{self, Cookie, CookieJar},
    request::{FromRequest, Outcome},
    response::Redirect,
    routes,
    serde::json::Json,
    tokio::sync::RwLock,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

mod pages;

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

#[async_trait]
trait UserService: Send + Sync {
    async fn verify_password(&self, username: &str, password: &str) -> bool;
    async fn add_user(&self, username: &str, password: &str) -> bool;
}

pub struct InMemoryUserService {
    users: RwLock<HashMap<String, String>>,
    salt: SaltString,
}

impl InMemoryUserService {
    pub fn new(salt: SaltString) -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            salt,
        }
    }

    fn hash_password(&self, password: &str) -> String {
        Argon2::default()
            .hash_password(password.as_bytes(), self.salt.as_salt())
            .unwrap()
            .to_string()
    }
}

#[async_trait]
impl UserService for InMemoryUserService {
    async fn verify_password(&self, username: &str, password: &str) -> bool {
        if let Some(correct_hash) = self.users.read().await.get(username) {
            let hash = self.hash_password(password);
            return hash == *correct_hash;
        }
        return false;
    }

    async fn add_user(&self, username: &str, password: &str) -> bool {
        if self.users.read().await.contains_key(username) {
            return false;
        }

        let hash = self.hash_password(password);
        self.users.write().await.insert(username.to_owned(), hash);
        return true;
    }
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
        .launch()
        .await?;

    Ok(())
}
