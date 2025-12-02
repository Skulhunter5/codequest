use std::{fs::DirBuilder, sync::Arc};

use codequest_common::{
    load_secret_key,
    services::{QuestService, UserService},
};
use codequest_gateway::services::ConstQuestService;
use codequest_user_service::BackendUserService;
use rocket::routes;
use rocket_dyn_templates::Template;

mod auth;
mod pages;

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";
pub const USERS_FILE: &'static str = "./run/users.json";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";

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
        .merge(("port", 8000));

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
                pages::quest,
                pages::quest_input,
                auth::login,
                auth::signup,
                auth::logout,
            ],
        )
        .attach(Template::fairing())
        .manage(
            Arc::new(BackendUserService::new("http://localhost:8001/user")) as Arc<dyn UserService>,
        )
        .manage(Arc::new(ConstQuestService::new()) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
