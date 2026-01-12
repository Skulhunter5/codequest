use std::{env, sync::Arc};

use codequest_common::{
    Credentials, Error, User, UserId, load_salt, load_secret_key, services::UserService,
};
use codequest_user_service::{
    ChangePasswordRequest, CreateUserRequest, DatabaseUserService, LoginRequest,
    UserServiceNatsWrapper,
};
use dotenv::dotenv;
use rocket::{
    State, http,
    response::{content::RawJson, status},
    routes,
    serde::json::Json,
};

mod defaults {
    pub const SALT_FILE: &'static str = "./secrets/salt";
    pub const SECRET_KEY_FILE: &'static str = "./secrets/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::get("/<user_id>")]
async fn get_user(
    user_id: UserId,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<Result<Json<User>, status::NotFound<RawJson<&'static str>>>, Error> {
    Ok(if let Some(user) = user_service.get_user(&user_id).await? {
        Ok(Json(user))
    } else {
        Err(status::NotFound(RawJson("")))
    })
}

#[rocket::post("/", format = "json", data = "<request_data>")]
async fn create_user(
    request_data: Json<CreateUserRequest<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<Result<(http::Status, String), status::Conflict<&'static str>>, Error> {
    let request_data = request_data.0;
    Ok(
        if let Some(user_id) = user_service
            .create_user(request_data.username, request_data.password)
            .await?
        {
            Ok((http::Status::Created, user_id.to_string()))
        } else {
            Err(status::Conflict(""))
        },
    )
}

#[rocket::delete("/<user_id>")]
async fn delete_user(
    user_id: UserId,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<Result<status::NoContent, status::NotFound<()>>, Error> {
    Ok(if user_service.delete_user(&user_id).await? {
        Ok(status::NoContent)
    } else {
        Err(status::NotFound(()))
    })
}

#[rocket::post("/change-password", format = "json", data = "<request_data>")]
async fn change_password(
    request_data: Json<ChangePasswordRequest<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<String, Error> {
    user_service
        .change_password(
            &request_data.user_id,
            request_data.old_password,
            request_data.new_password,
        )
        .await
        .map(|password_was_changed| password_was_changed.to_string())
}

#[rocket::post("/login", format = "json", data = "<request_data>")]
async fn login(
    request_data: Json<LoginRequest<'_>>,
    user_service: &State<Arc<dyn UserService>>,
) -> Result<Result<(http::Status, String), status::Unauthorized<&'static str>>, Error> {
    Ok(
        match user_service
            .login(&request_data.username, request_data.password)
            .await?
        {
            Some(user_id) => Ok((http::Status::Ok, user_id.to_string())),
            None => Err(status::Unauthorized("")),
        },
    )
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

    let nats_address = env::var("NATS_ADDRESS").expect("NATS_ADDRESS not set");

    let salt_file = env::var("SALT_FILE").unwrap_or_else(|_| defaults::SALT_FILE.to_owned());
    let salt = load_salt(salt_file).expect("failed to load salt");

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
    let user_service = UserServiceNatsWrapper::new(Arc::new(user_service), nats_address)
        .await
        .expect("failed to start nats wrapper");

    rocket::custom(&rocket_config)
        .mount(
            "/user",
            routes![get_user, create_user, delete_user, change_password, login],
        )
        .manage(Arc::new(user_service) as Arc<dyn UserService>)
        .launch()
        .await?;

    Ok(())
}
