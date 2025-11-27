use std::{
    collections::HashMap,
    fs::{self, DirBuilder},
    path::Path,
    sync::Arc,
};

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use rocket::{
    Request, State, async_trait,
    form::{Form, FromForm},
    fs::NamedFile,
    http::{self, Cookie, CookieJar},
    request::{FromRequest, Outcome},
    response::Redirect,
    routes,
    tokio::sync::RwLock,
};
use rocket_dyn_templates::Template;
use serde::Serialize;

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

#[rocket::get("/")]
async fn index() -> Template {
    #[derive(Serialize)]
    struct IndexPageContext<'a> {
        username: &'a str,
        links: Vec<LinkContext<'a>>,
    }

    #[derive(Serialize)]
    struct LinkContext<'a> {
        name: &'a str,
        url: &'a str,
    }

    Template::render(
        "index",
        IndexPageContext {
            username: "Someone",
            links: vec![
                LinkContext {
                    name: "GitHub",
                    url: "https://www.github.com",
                },
                LinkContext {
                    name: "Google",
                    url: "https://www.google.com",
                },
            ],
        },
    )
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

#[derive(FromForm)]
struct SignupForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[rocket::post("/signup", data = "<form>")]
async fn signup(
    form: Form<SignupForm<'_>>,
    jar: &CookieJar<'_>,
    user_service: &State<Arc<dyn UserService>>,
) -> Redirect {
    let SignupForm { username, password } = *form;

    if user_service.add_user(username, password).await {
        jar.add_private(Cookie::new("user_id", username.to_owned()));
        Redirect::to("/")
    } else {
        Redirect::to("/signup?error")
    }
}

#[derive(FromForm)]
struct LoginForm<'a> {
    username: &'a str,
    password: &'a str,
}

#[rocket::post("/login", data = "<form>")]
async fn login(
    form: Form<LoginForm<'_>>,
    jar: &CookieJar<'_>,
    user_service: &State<Arc<dyn UserService>>,
) -> Redirect {
    let LoginForm { username, password } = *form;

    if user_service.verify_password(username, password).await {
        jar.add_private(Cookie::new("user_id", username.to_owned()));
        Redirect::to("/")
    } else {
        Redirect::to("/login?error")
    }
}

#[rocket::get("/about")]
async fn about() -> Option<NamedFile> {
    let path = Path::new("static").join("about.html");
    NamedFile::open(path).await.ok()
}

#[rocket::get("/signup")]
async fn signup_page(user: Option<AuthUser>) -> Result<Option<NamedFile>, Redirect> {
    if user.is_some() {
        return Err(Redirect::to("/"));
    }
    let path = Path::new("static").join("signup.html");
    Ok(NamedFile::open(path).await.ok())
}

#[rocket::get("/login")]
async fn login_page(user: Option<AuthUser>) -> Result<Option<NamedFile>, Redirect> {
    if user.is_some() {
        return Err(Redirect::to("/"));
    }
    let path = Path::new("static").join("login.html");
    Ok(NamedFile::open(path).await.ok())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AuthUser {
    username: String,
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
            routes![index, about, login_page, login, signup_page, signup],
        )
        .attach(Template::fairing())
        .manage(Arc::new(InMemoryUserService::new(salt)) as Arc<dyn UserService>)
        .launch()
        .await?;

    Ok(())
}
