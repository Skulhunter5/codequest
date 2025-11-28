use std::path::Path;

use rocket::{fs::NamedFile, response::Redirect};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::AuthUser;

#[derive(Serialize)]
struct PageContext<'a> {
    user: Option<&'a str>,
}

impl<'a> PageContext<'a> {
    fn new(user: &'a Option<AuthUser>) -> Self {
        Self {
            user: if let Some(AuthUser { username }) = &user {
                Some(username.as_str())
            } else {
                None
            },
        }
    }
}

#[rocket::get("/")]
pub async fn index(user: Option<AuthUser>) -> Template {
    Template::render("index", PageContext::new(&user))
}

#[rocket::get("/about")]
pub async fn about(user: Option<AuthUser>) -> Template {
    Template::render("about", PageContext::new(&user))
}

#[rocket::get("/signup")]
pub async fn signup(user: Option<AuthUser>) -> Result<Template, Redirect> {
    if user.is_some() {
        return Err(Redirect::to("/"));
    }
    Ok(Template::render("signup", ""))
}

#[rocket::get("/login")]
pub async fn login(user: Option<AuthUser>) -> Result<Template, Redirect> {
    if user.is_some() {
        return Err(Redirect::to("/"));
    }
    Ok(Template::render("login", ""))
}

#[rocket::get("/style.css")]
pub async fn stylesheet() -> Option<NamedFile> {
    let path = Path::new("static").join("style.css");
    NamedFile::open(path).await.ok()
}

#[rocket::get("/puzzles")]
pub async fn puzzles(user: Option<AuthUser>) -> Template {
    Template::render("puzzles", PageContext::new(&user))
}
