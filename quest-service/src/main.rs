use std::{fs::DirBuilder, sync::Arc};

use codequest_common::{Quest, load_secret_key, services::QuestService};
use codequest_quest_service::ConstQuestService;
use rocket::{State, routes, serde::json::Json};

pub const RUN_DIR: &'static str = "./run";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";

#[rocket::get("/")]
async fn get_quests(quest_service: &State<Arc<dyn QuestService>>) -> Json<Arc<[Quest]>> {
    Json(quest_service.get_quests().await)
}

#[rocket::get("/<id>")]
async fn get_quest(id: &str, quest_service: &State<Arc<dyn QuestService>>) -> Json<Option<Quest>> {
    Json(quest_service.get_quest(id).await)
}

#[rocket::get("/<quest_id>/input/<username>")]
async fn get_input(
    quest_id: &str,
    username: &str,
    quest_service: &State<Arc<dyn QuestService>>,
) -> String {
    quest_service.get_input(quest_id, username).await
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
        .mount("/quest", routes![get_quests, get_quest, get_input])
        .manage(Arc::new(ConstQuestService::new()) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
