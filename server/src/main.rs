use std::{
    fs::{self, DirBuilder},
    io,
    path::Path,
    sync::Arc,
};

use argon2::password_hash::{SaltString, rand_core::OsRng};
use code_quest::services::{ConstQuestService, FileUserService, QuestService, UserService};
use rocket::routes;
use rocket_dyn_templates::Template;

mod auth;
mod pages;

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";
pub const USERS_FILE: &'static str = "./run/users.json";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";

fn load_or_generate_salt<P: AsRef<Path>>(path: P) -> SaltString {
    if let Ok(salt) = fs::read_to_string(&path) {
        return SaltString::from_b64(&salt).expect("failed to create salt");
    }

    let salt = SaltString::generate(&mut OsRng);
    fs::write(path, salt.as_str()).expect("failed to write salt to file");
    return salt;
}

fn load_secret_key<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");
    let salt = load_or_generate_salt(&SALT_FILE);

    let rocket_config = rocket::Config::figment().merge((
        "secret_key",
        load_secret_key(&SECRET_KEY_FILE).expect("failed to load secret key"),
    ));

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
        .manage(Arc::new(
            FileUserService::new(salt, &USERS_FILE).expect("failed to start UserService"),
        ) as Arc<dyn UserService>)
        .manage(Arc::new(ConstQuestService::new()) as Arc<dyn QuestService>)
        .launch()
        .await?;

    Ok(())
}
