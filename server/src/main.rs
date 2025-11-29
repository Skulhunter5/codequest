use std::{
    fs::{self, DirBuilder},
    sync::Arc,
};

use argon2::password_hash::{SaltString, rand_core::OsRng};
use rocket::routes;
use rocket_dyn_templates::Template;
use services::{ConstQuestService, InMemoryUserService, QuestService, UserService};

mod auth;
mod pages;
mod services;

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";

fn load_or_generate_salt() -> SaltString {
    if let Ok(salt) = fs::read_to_string(&SALT_FILE) {
        return SaltString::from_b64(&salt).expect("failed to create salt");
    }

    let salt = SaltString::generate(&mut OsRng);
    fs::write(&SALT_FILE, salt.as_str()).expect("failed to write salt to file");
    return salt;
}

pub struct Quest<'a> {
    pub name: &'a str,
    pub id: &'a str,
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");
    let salt = load_or_generate_salt();

    // let rocket_config = rocket::Config::figment().merge(("template_dir", "static/"));
    // rocket::custom(&rocket_config)

    rocket::build()
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
                auth::login,
                auth::signup,
                auth::logout,
            ],
        )
        .attach(Template::fairing())
        .manage(Arc::new(InMemoryUserService::new(salt)) as Arc<dyn UserService>)
        .manage(Arc::new(ConstQuestService::new()) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
