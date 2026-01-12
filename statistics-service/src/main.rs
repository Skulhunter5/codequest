use std::{env, sync::Arc};

use codequest_common::{
    Credentials, Error, UserId, load_secret_key, services::StatisticsService, statistics::Metric,
};
use codequest_statistics_service::DatabaseStatisticsService;
use dotenv::dotenv;
use rocket::{State, catchers, routes, serde::json::Json};

mod defaults {
    pub const SECRET_KEY_FILE: &'static str = "./secrets/secret_key";
    pub const PORT: u16 = 8000;
}

#[rocket::get("/<user_id>")]
async fn user_metrics(
    user_id: UserId,
    statistics_service: &State<Arc<dyn StatisticsService>>,
) -> Result<Json<Vec<Metric>>, Error> {
    statistics_service
        .get_user_metrics(&user_id)
        .await
        .map(|metrics| Json(metrics))
}

#[rocket::catch(default)]
fn catch_all() -> &'static str {
    ""
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok();

    let db_credentials = {
        let username = env::var("DB_USERNAME_STATISTICS_SERVICE")
            .expect("DB_USERNAME_STATISTICS_SERVICE not set");
        let password = env::var("DB_PASSWORD_STATISTICS_SERVICE")
            .expect("DB_PASSWORD_STATISTICS_SERVICE not set");
        Credentials::new(username, password)
    };
    let db_name = env::var("POSTGRES_DB").expect("POSTGRES_DB not set");
    let db_address = env::var("DB_ADDRESS").expect("DB_ADDRESS not set");

    let nats_address = env::var("NATS_ADDRESS").expect("NATS_ADDRESS not set");

    let secret_key = load_secret_key(
        env::var("SECRET_KEY_FILE").unwrap_or_else(|_| defaults::SECRET_KEY_FILE.to_owned()),
    )
    .expect("failed to load secret key");

    let port = env::var("STATISTICS_SERVICE_PORT")
        .map(|port| {
            port.parse::<u16>()
                .expect(format!("invalid STATISTICS_SERVICE_PORT: '{}'", port).as_str())
        })
        .unwrap_or(defaults::PORT);

    let rocket_config = rocket::Config::figment()
        .merge(("secret_key", secret_key))
        .merge(("port", port));

    let statistics_service =
        DatabaseStatisticsService::new(&db_address, &db_name, db_credentials, nats_address)
            .await
            .expect("failed to start DatabaseStatisticsService");

    rocket::custom(&rocket_config)
        .register("/", catchers![catch_all])
        .mount("/statistics", routes![user_metrics])
        .manage(Arc::new(statistics_service) as Arc<dyn StatisticsService>)
        .launch()
        .await?;

    Ok(())
}
