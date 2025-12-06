use std::{fs::DirBuilder, sync::Arc};

use codequest_common::{Error, Quest, load_secret_key, services::QuestService};
use codequest_quest_service::FileQuestService;
use rocket::{
    State, catchers,
    response::{
        content::{RawJson, RawText},
        status,
    },
    routes,
    serde::json::Json,
};

pub const RUN_DIR: &'static str = "./run";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";
pub const QUESTS_FILE: &'static str = "./run/quests.json";

#[rocket::get("/")]
async fn get_quests(
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Json<Arc<[Quest]>>, Error> {
    quest_service.get_quests().await.map(|quests| Json(quests))
}

#[rocket::get("/<id>")]
async fn get_quest(
    id: &str,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<Json<Quest>, status::NotFound<RawJson<&'static str>>>, Error> {
    Ok(quest_service
        .get_quest(id)
        .await?
        .map(|quest| Json(quest))
        .ok_or(status::NotFound(RawJson(""))))
}

#[rocket::get("/<quest_id>/input/<username>")]
async fn get_input(
    quest_id: &str,
    username: &str,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(quest_service
        .get_input(quest_id, username)
        .await?
        .ok_or(status::NotFound(RawText(""))))
}

#[rocket::catch(default)]
fn catch_all() -> &'static str {
    ""
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");

    let rocket_config = rocket::Config::figment()
        .merge((
            "secret_key",
            load_secret_key(&SECRET_KEY_FILE).expect("failed to load secret key"),
        ))
        .merge(("port", 8002));

    rocket::custom(&rocket_config)
        .register("/", catchers![catch_all])
        .mount("/quest", routes![get_quests, get_quest, get_input])
        .manage(
            Arc::new(FileQuestService::new(&QUESTS_FILE).expect("failed to start QuestService"))
                as Arc<dyn QuestService>,
        )
        .launch()
        .await?;

    Ok(())
}
