use std::{env, fs::DirBuilder, sync::Arc};

use codequest_common::{
    Credentials, Error, Username, load_or_generate_salt, load_secret_key, services::UserService,
};
use codequest_user_service::{DatabaseUserService, UserCredentials};
use dotenv::dotenv;
use rocket::{State, http, routes, serde::json::Json};

mod defaults {
    pub const RUN_DIR: &'static str = "./run";
    pub const SALT_FILE: &'static str = "./run/salt";
    pub const SECRET_KEY_FILE: &'static str = "./run/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::get("/<username>")]
async fn get_user(
    username: &str,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<(http::Status, &'static str), Error> {
    let username = Username::build(username)?;
    Ok(if user_service.user_exists(&username).await? {
        (http::Status::Ok, "")
    } else {
        (http::Status::NotFound, "")
    })
}

#[rocket::post("/", format = "json", data = "<credentials>")]
async fn add_user(
    credentials: Json<UserCredentials<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<(http::Status, &'static str), Error> {
    let username = Username::build(credentials.username)?;
    Ok(
        if user_service
            .add_user(username, credentials.password)
            .await?
        {
            (http::Status::Created, "")
        } else {
            (http::Status::Conflict, "")
        },
    )
}

#[rocket::post("/login", format = "json", data = "<credentials>")]
async fn verify_password(
    credentials: Json<UserCredentials<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<String, Error> {
    let username = Username::build(credentials.username)?;
    user_service
        .verify_password(&username, credentials.password)
        .await
        .map(|res| res.to_string())
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let db_credentials = {
        let username =
            env::var("DB_USERNAME_USER_SERVICE").expect("DB_USERNAME_USER_SERVICE not set");
        let password =
            env::var("DB_PASSWORD_USER_SERVICE").expect("DB_PASSWORD_USER_SERVICE not set");
        Credentials::new(username, password)
    };
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS not set");

    DirBuilder::new()
        .recursive(true)
        .create(defaults::RUN_DIR)
        .expect("failed to create run dir");

    let salt = load_or_generate_salt(
        env::var("SALT_FILE").unwrap_or_else(|_| defaults::SALT_FILE.to_owned()),
    );

    let secret_key = load_secret_key(
        env::var("SECRET_KEY_FILE").unwrap_or_else(|_| defaults::SECRET_KEY_FILE.to_owned()),
    )
    .expect("failed to load secret key");

    let port = env::var("USER_SERVICE_PORT")
        .map(|port| {
            port.parse::<u16>()
                .expect(format!("invalid USER_SERVICE_PORT: '{}'", port).as_str())
        })
        .unwrap_or(defaults::PORT);

    let rocket_config = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));

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
