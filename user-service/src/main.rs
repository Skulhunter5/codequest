use std::{env, fs::DirBuilder, sync::Arc};

use codequest_common::{
    Credentials, load_or_generate_salt, load_secret_key, services::UserService,
};
use codequest_user_service::{DatabaseUserService, UserCredentials};
use dotenv::dotenv;
use rocket::{State, http, routes, serde::json::Json};

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";
pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";

#[rocket::get("/<username>")]
async fn get_user(
    username: &str,
    user_service: &State<Arc<dyn UserService>>,
) -> (http::Status, &'static str) {
    if user_service.user_exists(username).await {
        (http::Status::NoContent, "")
    } else {
        (http::Status::NotFound, "")
    }
}

#[rocket::post("/", format = "json", data = "<credentials>")]
async fn add_user(
    credentials: Json<UserCredentials<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> (http::Status, &'static str) {
    if user_service
        .add_user(credentials.username, credentials.password)
        .await
    {
        (http::Status::Created, "")
    } else {
        (http::Status::Conflict, "")
    }
}

#[rocket::post("/login", format = "json", data = "<credentials>")]
async fn verify_password(
    credentials: Json<UserCredentials<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> (http::Status, &'static str) {
    if user_service
        .verify_password(credentials.username, credentials.password)
        .await
    {
        (http::Status::NoContent, "")
    } else {
        (http::Status::Unauthorized, "")
    }
}

// TODO: restrict valid usernames
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let db_credentials = {
        let username = env::var("POSTGRES_USER").expect("POSTGRES_USER not set");
        let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not set");
        Credentials::new(username, password)
    };
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS not set");

    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");
    let salt = load_or_generate_salt(&SALT_FILE);

    let rocket_config = rocket::Config::figment()
        .merge((
            "secret_key",
            load_secret_key(&SECRET_KEY_FILE).expect("failed to load secret key"),
        ))
        .merge(("port", 8001));

    let user_service = DatabaseUserService::new(&db_address, &db_name, db_credentials, salt)
        .await
        .expect("failed to start DatabaseUserService");

    rocket::custom(&rocket_config)
        .mount("/user", routes![get_user, add_user, verify_password])
        .manage(Arc::new(user_service) as Arc<dyn UserService>)
        .launch()
        .await?;

    Ok(())
}
