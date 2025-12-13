use std::{path::Path, sync::Arc};

use codequest_common::{Error, QuestItem, services::QuestService};
use rocket::{FromForm, State, form::Form, fs::NamedFile, http, response::Redirect};
use rocket_dyn_templates::{Template, context};
use serde::Serialize;

use crate::auth::AuthUser;

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
pub async fn quests(
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Template, Error> {
    Ok(Template::render(
        "quests",
        PageContext::new(
            &user,
            QuestsPageContext {
                quests: quest_service
                    .list_quests()
                    .await?
                    .iter()
                    .map(|quest| QuestContext::from(quest))
                    .collect::<Vec<_>>(),
            },
        ),
    ))
}

impl<'a> From<&'a QuestItem> for QuestContext<'a> {
    fn from(quest: &'a QuestItem) -> Self {
        Self {
            name: &quest.name,
            uri: format!("/quest/{}", &quest.id),
        }
    }
}

#[rocket::get("/quest/<id>")]
pub async fn quest(
    id: &str,
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<Template, http::Status>, Error> {
    Ok(if let Some(quest) = quest_service.get_quest(id).await? {
        Ok(Template::render(
            "quest",
            PageContext::new(
                &user,
                context! {
                    quest: context! {
                        name: &quest.item.name,
                        id: &quest.item.id,
                        text: &quest.text,
                    },
                },
            ),
        ))
    } else {
        Err(http::Status::NotFound)
    })
}

#[rocket::get("/quest/<id>/input")]
pub async fn quest_input(
    id: &str,
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<String, http::Status> {
    if let Some(user) = user {
        match quest_service.get_input(&id, &user.username).await {
            Ok(Some(input)) => Ok(input),
            Ok(None) => Err(http::Status::NotFound),
            Err(_) => Err(http::Status::InternalServerError),
        }
    } else {
        Err(http::Status::Unauthorized)
    }
}

#[derive(FromForm)]
pub(crate) struct AnswerForm<'a> {
    pub(crate) answer: &'a str,
}

#[rocket::post("/quest/<quest_id>/answer", data = "<form>")]
pub async fn quest_answer(
    form: Form<AnswerForm<'_>>,
    quest_id: &str,
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<Template, http::Status>, Error> {
    let quest = match quest_service.get_quest(&quest_id).await? {
        Some(quest) => quest,
        None => return Ok(Err(http::Status::NotFound)),
    };
    Ok(if let Some(user) = user {
        match quest_service
            .verify_answer(&quest_id, &user.username, &form.answer)
            .await?
        {
            Some(answer_was_correct) => Ok(Template::render(
                "answer",
                context! {
                    answer_was_correct,
                    quest: context! {
                        name: &quest.item.name,
                        id: &quest.item.id,
                    },
                },
            )),
            None => Err(http::Status::NotFound),
        }
    } else {
        Err(http::Status::Unauthorized)
    })
}
