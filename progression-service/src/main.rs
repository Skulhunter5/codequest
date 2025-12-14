use std::{fs::DirBuilder, sync::Arc};

use codequest_common::{
    Error, load_secret_key,
    services::{ProgressionService, QuestService},
};
use codequest_progression_service::InMemoryProgressionService;
use codequest_quest_service::BackendQuestService;
use rocket::{
    State, catchers,
    response::{content::RawText, status},
    routes,
};

pub const RUN_DIR: &'static str = "./run";
pub const USER_PROGRESS_FILE: &'static str = "./run/user-progress.json";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";

#[rocket::get("/<username>/<quest_id>")]
async fn has_user_completed_quest(
    username: &str,
    quest_id: &str,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<String, Error> {
    progression_service
        .has_user_completed_quest(username, quest_id)
        .await
        .map(|res| res.to_string())
}

#[rocket::post("/<username>/<quest_id>/answer", data = "<answer>")]
async fn submit_answer(
    quest_id: &str,
    username: &str,
    answer: &str,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(progression_service
        .submit_answer(username, quest_id, answer)
        .await?
        .ok_or_else(|| status::NotFound(RawText("")))
        .map(|answer_was_correct| answer_was_correct.to_string()))
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
        .merge(("port", 8003));

    let quest_service =
        Arc::new(BackendQuestService::new("http://localhost:8002/quest")) as Arc<dyn QuestService>;

    rocket::custom(&rocket_config)
        .register("/", catchers![catch_all])
        .mount(
            "/progression",
            routes![has_user_completed_quest, submit_answer],
        )
        .manage(
            Arc::new(InMemoryProgressionService::new(quest_service)) as Arc<dyn ProgressionService>
        )
        .launch()
        .await?;

    Ok(())
}
