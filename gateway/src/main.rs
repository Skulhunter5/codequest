use std::{env, sync::Arc};

use codequest_common::{
    load_secret_key,
    services::{ProgressionService, QuestService, StatisticsService, UserService},
};
use codequest_progression_service::BackendProgressionService;
use codequest_quest_service::BackendQuestService;
use codequest_statistics_service::BackendStatisticsService;
use codequest_user_service::BackendUserService;
use dotenv::dotenv;
use rocket::routes;
use rocket_dyn_templates::Template;

mod account;
mod pages;

mod defaults {
    pub const SECRET_KEY_FILE: &'static str = "./secrets/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let secret_key = load_secret_key(
        env::var("SECRET_KEY_FILE").unwrap_or_else(|_| defaults::SECRET_KEY_FILE.to_owned()),
    )
    .expect("failed to load secret key");

    let port = env::var("GATEWAY_PORT")
        .map(|port| {
            port.parse::<u16>()
                .expect(format!("invalid GATEWAY_PORT: '{}'", port).as_str())
        })
        .unwrap_or(defaults::PORT);

    let rocket_config = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));

    let user_service_address =
        env::var("USER_SERVICE_ADDRESS").expect("USER_SERVICE_ADDRESS not set");
    let quest_service_address =
        env::var("QUEST_SERVICE_ADDRESS").expect("QUEST_SERVICE_ADDRESS not set");
    let progression_service_address =
        env::var("PROGRESSION_SERVICE_ADDRESS").expect("PROGRESSION_SERVICE_ADDRESS not set");
    let statistics_service_address =
        env::var("STATISTICS_SERVICE_ADDRESS").expect("STATISTICS_SERVICE_ADDRESS not set");

    let user_service = BackendUserService::new(user_service_address);
    let quest_service = BackendQuestService::new(quest_service_address);
    let progression_service = BackendProgressionService::new(progression_service_address);
    let statistics_service = BackendStatisticsService::new(statistics_service_address);

    rocket::custom(&rocket_config)
        .mount(
            "/",
            routes![
                pages::index,
                pages::about,
                pages::signup,
                pages::login,
                pages::stylesheet,
                pages::quests,
                pages::create_quest_page,
                pages::create_quest_form,
                pages::edit_quest_page,
                pages::modify_quest,
                pages::quest,
                pages::quest_input,
                pages::quest_answer,
                pages::account,
                pages::account_statistics,
            ],
        )
        .mount(
            "/account",
            routes![
                account::login,
                account::signup,
                account::logout,
                account::change_password,
                account::delete,
            ],
        )
        .attach(Template::fairing())
        .manage(Arc::new(user_service) as Arc<dyn UserService>)
        .manage(Arc::new(quest_service) as Arc<dyn QuestService>)
        .manage(Arc::new(progression_service) as Arc<dyn ProgressionService>)
        .manage(Arc::new(statistics_service) as Arc<dyn StatisticsService>)
        .launch()
        .await?;

    Ok(())
}
