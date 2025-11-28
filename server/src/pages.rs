use std::path::Path;

use rocket::{fs::NamedFile, http, response::Redirect};
use rocket_dyn_templates::Template;
use serde::Serialize;

use crate::AuthUser;

#[derive(Serialize)]
struct PageContext<'a, MainContext: Serialize> {
    user: Option<&'a str>,
    #[serde(flatten)]
    content: MainContext,
}

impl<'a> PageContext<'a, ()> {
    fn simple(user: &'a Option<AuthUser>) -> Self {
        Self {
            user: if let Some(AuthUser { username }) = &user {
                Some(username.as_str())
            } else {
                None
            },
            content: (),
        }
    }
}

impl<'a, MainContext: Serialize> PageContext<'a, MainContext> {
    fn new(user: &'a Option<AuthUser>, content: MainContext) -> Self {
        Self {
            user: if let Some(AuthUser { username }) = &user {
                Some(username.as_str())
            } else {
                None
            },
            content,
        }
    }
}

#[rocket::get("/")]
pub async fn index(user: Option<AuthUser>) -> Template {
    Template::render("index", PageContext::simple(&user))
}

#[rocket::get("/about")]
pub async fn about(user: Option<AuthUser>) -> Template {
    Template::render("about", PageContext::simple(&user))
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

#[derive(Serialize)]
struct QuestsPageContext<'a> {
    quests: Vec<QuestContext<'a>>,
}

#[derive(Serialize)]
struct QuestContext<'a> {
    name: &'a str,
    uri: String,
}

#[rocket::get("/quests")]
pub async fn quests(user: Option<AuthUser>) -> Template {
    Template::render(
        "quests",
        PageContext::new(
            &user,
            QuestsPageContext {
                quests: QUESTS
                    .iter()
                    .map(|quest| QuestContext::from(quest))
                    .collect::<Vec<_>>(),
            },
        ),
    )
}

#[derive(Serialize)]
struct QuestPageContext<'a> {
    quest: QuestContext<'a>,
}

impl<'a> From<&Quest<'a>> for QuestContext<'a> {
    fn from(quest: &Quest<'a>) -> Self {
        Self {
            name: quest.name,
            uri: format!("/quest/{}", &quest.id),
        }
    }
}

struct Quest<'a> {
    name: &'a str,
    id: &'a str,
}

static QUESTS: &[Quest] = &[
    Quest {
        name: "Quest 1",
        id: "quest-1",
    },
    Quest {
        name: "Quest 2",
        id: "quest-2",
    },
    Quest {
        name: "Quest 3",
        id: "quest-3",
    },
    Quest {
        name: "Quest 4",
        id: "quest-4",
    },
];

#[rocket::get("/quest/<id>")]
pub async fn quest(id: &str, user: Option<AuthUser>) -> Result<Template, http::Status> {
    if let Some(quest) = QUESTS.iter().find(|quest| quest.id == id) {
        Ok(Template::render(
            "quest",
            PageContext::new(
                &user,
                QuestPageContext {
                    quest: QuestContext::from(quest),
                },
            ),
        ))
    } else {
        Err(http::Status::NotFound)
    }
}
