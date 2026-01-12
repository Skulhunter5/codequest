use std::{env, sync::Arc};

use codequest_common::{
    Credentials, Error, QuestId, UserId, load_secret_key,
    services::{ProgressionService, QuestService},
};
use codequest_progression_service::{DatabaseProgressionService, ProgressionServiceNatsWrapper};
use codequest_quest_service::BackendQuestService;
use dotenv::dotenv;
use rocket::{
    State, catchers,
    response::{content::RawText, status},
    routes,
};

mod defaults {
    pub const SECRET_KEY_FILE: &'static str = "./secrets/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::get("/<user_id>/<quest_id>")]
async fn has_user_completed_quest(
    user_id: UserId,
    quest_id: QuestId,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<String, Error> {
    progression_service
        .has_user_completed_quest(&user_id, &quest_id)
        .await
        .map(|res| res.to_string())
}

#[rocket::post("/<user_id>/<quest_id>/answer", data = "<answer>")]
async fn submit_answer(
    quest_id: QuestId,
    user_id: UserId,
    answer: &str,
    progression_service: &State<Arc<dyn ProgressionService>>,
) -> Result<Result<String, status::NotFound<RawText<&'static str>>>, Error> {
    Ok(progression_service
        .submit_answer(&user_id, &quest_id, answer)
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
    dotenv().ok();

    let db_credentials = {
        let username = env::var("DB_USERNAME_PROGRESSION_SERVICE")
            .expect("DB_USERNAME_PROGRESSION_SERVICE not set");
        let password = env::var("DB_PASSWORD_PROGRESSION_SERVICE")
            .expect("DB_PASSWORD_PROGRESSION_SERVICE not set");
        Credentials::new(username, password)
    };
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS not set");

    let nats_address = env::var("NATS_ADDRESS").expect("NATS_ADDRESS not set");

    let secret_key = load_secret_key(
        env::var("SECRET_KEY_FILE").unwrap_or_else(|_| defaults::SECRET_KEY_FILE.to_owned()),
    )
    .expect("failed to load secret key");

    let port = env::var("PROGRESSION_SERVICE_PORT")
        .map(|port| {
            port.parse::<u16>()
                .expect(format!("invalid PROGRESSION_SERVICE_PORT: '{}'", port).as_str())
        })
        .unwrap_or(defaults::PORT);

    let rocket_config = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));

    let quest_service_address =
        env::var("QUEST_SERVICE_ADDRESS").expect("QUEST_SERVICE_ADDRESS not set");

    let quest_service =
        Arc::new(BackendQuestService::new(quest_service_address)) as Arc<dyn QuestService>;

    let progression_service = DatabaseProgressionService::new(
        quest_service,
        &db_address,
        &db_name,
        db_credentials,
        nats_address.clone(),
    )
    .await
    .expect("failed to start DatabaseProgressionService");
    let progression_service =
        ProgressionServiceNatsWrapper::new(Arc::new(progression_service), nats_address)
            .await
            .expect("failed to start nats wrapper");

    rocket::custom(&rocket_config)
        .register("/", catchers![catch_all])
        .mount(
            "/progression",
            routes![has_user_completed_quest, submit_answer],
        )
        .manage(Arc::new(progression_service) as Arc<dyn ProgressionService>)
        .launch()
        .await?;

    Ok(())
}
