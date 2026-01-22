use std::{env, sync::Arc};

use codequest_common::{
    Credentials, Error, PartialQuestData, Quest, QuestData, QuestEntry, QuestId, UserId,
    load_secret_key, services::QuestService,
};
use codequest_quest_service::{
    DatabaseQuestService,
    quest_context::{InMemoryQuestContextCache, QuestContextGenerator},
};
use dotenv::dotenv;
use rocket::{
    State, catchers,
    response::{
        content::{RawJson, RawText},
        status,
    },
    routes,
    serde::json::Json,
};

mod defaults {
    pub const SECRET_KEY_FILE: &'static str = "./secrets/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::get("/")]
async fn list_quests(
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Json<Box<[QuestEntry]>>, Error> {
    quest_service.list_quests().await.map(|quests| Json(quests))
}

#[rocket::get("/<id>")]
async fn get_quest(
    id: QuestId,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<Json<Quest>, status::NotFound<RawJson<&'static str>>>, Error> {
    Ok(quest_service
        .get_quest(&id)
        .await?
        .map(|quest| Json(quest))
        .ok_or(status::NotFound(RawJson(""))))
}

#[rocket::get("/<quest_id>/input/<user_id>")]
async fn get_input(
    quest_id: QuestId,
    user_id: UserId,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(quest_service
        .get_input(&quest_id, &user_id)
        .await?
        .ok_or(status::NotFound(RawText(""))))
}

#[rocket::get("/<quest_id>/answer/<user_id>")]
async fn get_answer(
    quest_id: QuestId,
    user_id: UserId,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(quest_service
        .get_answer(&quest_id, &user_id)
        .await?
        .ok_or(status::NotFound(RawText(""))))
}

#[rocket::post("/<quest_id>/answer/<user_id>", data = "<answer>")]
async fn verify_answer(
    quest_id: QuestId,
    user_id: UserId,
    answer: &str,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(quest_service
        .verify_answer(&quest_id, &user_id, answer)
        .await?
        .map(|answer_was_correct| answer_was_correct.to_string())
        .ok_or(status::NotFound(RawText(""))))
}

#[rocket::post("/", data = "<data>")]
async fn create_quest(
    data: Json<QuestData>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<String, Error> {
    quest_service
        .create_quest(data.0)
        .await
        .map(|quest_id| quest_id.to_string())
}

#[rocket::put("/<id>", data = "<data>")]
async fn update_quest(
    id: QuestId,
    data: Json<QuestData>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<status::NoContent, status::NotFound<()>>, Error> {
    Ok(match quest_service.update_quest(&id, data.0).await? {
        true => Ok(status::NoContent),
        false => Err(status::NotFound(())),
    })
}

#[rocket::patch("/<id>", data = "<data>")]
async fn modify_quest(
    id: QuestId,
    data: Json<PartialQuestData>,
    quest_service: &State<Arc<dyn QuestService>>,
) -> Result<Result<status::NoContent, status::NotFound<()>>, Error> {
    Ok(match quest_service.modify_quest(&id, data.0).await? {
        true => Ok(status::NoContent),
        false => Err(status::NotFound(())),
    })
}

#[rocket::catch(default)]
fn catch_all() -> &'static str {
    ""
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let db_credentials = {
        let username =
            env::var("DB_USERNAME_QUEST_SERVICE").expect("DB_USERNAME_QUEST_SERVICE not set");
        let password =
            env::var("DB_PASSWORD_QUEST_SERVICE").expect("DB_PASSWORD_QUEST_SERVICE not set");
        Credentials::new(username, password)
    };
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS not set");

    let secret_key = load_secret_key(
        env::var("SECRET_KEY_FILE").unwrap_or_else(|_| defaults::SECRET_KEY_FILE.to_owned()),
    )
    .expect("failed to load secret key");

    let port = env::var("QUEST_SERVICE_PORT")
        .map(|port| {
            port.parse::<u16>()
                .expect(format!("invalid QUEST_SERVICE_PORT: '{}'", port).as_str())
        })
        .unwrap_or(defaults::PORT);

    let rocket_config = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));

    let quest_context_provider =
        InMemoryQuestContextCache::new(Arc::new(QuestContextGenerator::new("./quests/generators")));
    let quest_service = DatabaseQuestService::new(
        &db_address,
        &db_name,
        db_credentials,
        Arc::new(quest_context_provider),
    )
    .await
    .expect("failed to start DatabaseQuestService");

    rocket::custom(&rocket_config)
        .register("/", catchers![catch_all])
        .mount(
            "/quests",
            routes![
                list_quests,
                get_quest,
                get_input,
                get_answer,
                verify_answer,
                create_quest,
                update_quest,
                modify_quest,
            ],
        )
        .manage(Arc::new(quest_service) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
