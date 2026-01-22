use std::{path::Path, sync::Arc};

use codequest_common::{
    Error, PartialQuestData, QuestData, QuestEntry, QuestId,
    services::{ProgressionService, QuestService, StatisticsService, UserService},
};
use rocket::{
    FromForm, State, form::Form, fs::NamedFile, http, response::Redirect, serde::json::Json,
};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};

use crate::account::AuthUser;

#[derive(Serialize)]
struct PageContext<'a, MainContext: Serialize> {
    user: Option<&'a str>,
    #[serde(flatten)]
    content: MainContext,
}

impl<'a> PageContext<'a, ()> {
    fn simple(user: &'a Option<AuthUser>) -> Self {
        Self {
            user: if let Some(AuthUser { username, .. }) = &user {
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
            user: if let Some(AuthUser { username, .. }) = &user {
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

impl<'a> From<&'a QuestEntry> for QuestContext<'a> {
    fn from(quest: &'a QuestEntry) -> Self {
        Self {
            name: &quest.name,
            uri: format!("/quests/{}", &quest.id),
        }
    }
}

#[rocket::get("/quests/<quest_id>")]
pub async fn quest(
    quest_id: QuestId,
    user: Option<AuthUser>,
    user_service: &State<Arc<dyn UserService>>,
    quest_service: &State<Arc<dyn QuestService>>,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<Result<Template, http::Status>, Error> {
    let Some(quest) = quest_service.get_quest(&quest_id).await? else {
        return Ok(Err(http::Status::NotFound));
    };

    let author = if let Some(author_id) = &quest.author {
        if let Some(author) = user_service.get_user(author_id).await? {
            Some(author.username)
        } else {
            None
        }
    } else {
        None
    };
    let (quest_completed, quest_answer) = if let Some(user) = &user {
        let quest_completed = progression_service
            .has_user_completed_quest(&user.id, &quest_id)
            .await?;
        let quest_answer = if quest_completed {
            quest_service.get_answer(&quest_id, &user.id).await?
        } else {
            None
        };
        (quest_completed, quest_answer)
    } else {
        (false, None)
    };

    Ok(Ok(Template::render(
        "quest",
        PageContext::new(
            &user,
            context! {
                quest: context! {
                    name: &quest.name,
                    id: &quest.id,
                    author,
                    text: &quest.text,
                    completed: quest_completed,
                    answer: quest_answer,
                },
            },
        ),
    )))
}

#[rocket::get("/quests/create")]
pub async fn create_quest_page(user: AuthUser) -> Template {
    Template::render("create-quest", PageContext::new(&Some(user), context! {}))
}

#[derive(FromForm)]
pub(crate) struct CreateQuestForm<'a> {
    name: &'a str,
    text: &'a str,
}

#[rocket::post("/quests", data = "<form>")]
pub async fn create_quest_form(
    form: Form<CreateQuestForm<'_>>,
    user: AuthUser,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Redirect, Error> {
    let author = Some(user.id);
    let official = false;
    let quest = QuestData::new(form.name, author, official, form.text.replace("\r\n", "\n"));
    quest_service
        .create_quest(quest)
        .await
        .map(|quest_id| Redirect::to(format!("/quests/{}", quest_id)))
}

#[rocket::get("/quests/<id>/edit")]
pub async fn edit_quest_page(
    id: QuestId,
    user: AuthUser,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<Template, http::Status>, Error> {
    if let Some(quest) = quest_service.get_quest(&id).await? {
        if let Some(author) = quest.author {
            if author == user.id {
                return Ok(Ok(Template::render(
                    "edit-quest",
                    PageContext::new(
                        &Some(user),
                        context! {
                            quest: context! {
                                id: &quest.id,
                                name: &quest.name,
                                text_json: rocket::serde::json::serde_json::to_string(&quest.text)?,
                            },
                        },
                    ),
                )));
            }
        }
        return Ok(Err(http::Status::Forbidden));
    } else {
        return Ok(Err(http::Status::NotFound));
    }
}

#[derive(Deserialize)]
pub(crate) struct ModifyQuestRequest<'a> {
    name: Option<&'a str>,
    text: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ModifyQuestResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ModifyQuestResponse {
    fn success(redirect: impl Into<String>) -> Self {
        Self {
            success: true,
            redirect: Some(redirect.into()),
            error: None,
        }
    }

    fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            redirect: None,
            error: Some(error.into()),
        }
    }
}

#[rocket::patch("/quests/<id>", data = "<request>")]
pub async fn modify_quest(
    id: QuestId,
    request: Json<ModifyQuestRequest<'_>>,
    user: AuthUser,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<(http::Status, Json<ModifyQuestResponse>), Error> {
    let Some(quest) = quest_service.get_quest(&id).await? else {
        return Ok((
            http::Status::NotFound,
            Json(ModifyQuestResponse::error("Quest doesn't exist.")),
        ));
    };
    if !quest.is_author(&user.id) {
        return Ok((
            http::Status::Forbidden,
            Json(ModifyQuestResponse::error(
                "You are not the author of this quest.",
            )),
        ));
    }

    let request = request.0;
    let mut quest_data = PartialQuestData::new();
    if let Some(name) = request.name {
        quest_data.set_name(name);
    }
    if let Some(text) = request.text {
        quest_data.set_text(text);
    }
    Ok(match quest_service.modify_quest(&id, quest_data).await? {
        true => (
            http::Status::Ok,
            Json(ModifyQuestResponse::success(format!("/quests/{}", id))),
        ),
        false => (
            http::Status::NotFound,
            Json(ModifyQuestResponse::error("Quest doesn't exist.")),
        ),
    })
}

#[rocket::get("/quests/<quest_id>/input")]
pub async fn quest_input(
    quest_id: QuestId,
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<String, http::Status> {
    if let Some(user) = user {
        match quest_service.get_input(&quest_id, &user.id).await {
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
    answer: &'a str,
}

#[rocket::post("/quests/<quest_id>/answer", data = "<form>")]
pub async fn quest_answer(
    form: Form<AnswerForm<'_>>,
    quest_id: QuestId,
    user: Option<AuthUser>,
    quest_service: &State<Arc<dyn QuestService>>,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<Result<Template, http::Status>, Error> {
    let quest = match quest_service.get_quest(&quest_id).await? {
        Some(quest) => quest,
        None => return Ok(Err(http::Status::NotFound)),
    };
    Ok(if let Some(user) = user {
        match progression_service
            .submit_answer(&user.id, &quest_id, &form.answer)
            .await?
        {
            Some(answer_was_correct) => Ok(Template::render(
                "answer",
                PageContext::new(
                    &Some(user),
                    context! {
                        answer_was_correct,
                        quest: context! {
                            name: &quest.name,
                            id: &quest.id,
                        },
                    },
                ),
            )),
            None => Err(http::Status::NotFound),
        }
    } else {
        Err(http::Status::Unauthorized)
    })
}

#[rocket::get("/account")]
pub async fn account(user: AuthUser) -> Result<Template, Error> {
    Ok(Template::render(
        "account",
        PageContext::new(&Some(user), context! {}),
    ))
}

#[rocket::get("/account/statistics")]
pub async fn account_statistics(
    user: AuthUser,
    statistics_service: &State<Arc<dyn StatisticsService>>,
) -> Result<Template, Error> {
    let metrics = statistics_service.get_user_metrics(&user.id).await?;
    let statistics = metrics
        .into_iter()
        .map(|metric| {
            context! {
                name: metric.get_display_name(),
                value: metric.value,
            }
        })
        .collect::<Vec<_>>();
    Ok(Template::render(
        "account-statistics",
        PageContext::new(
            &Some(user),
            context! {
                statistics,
            },
        ),
    ))
}
