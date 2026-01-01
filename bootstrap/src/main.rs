use std::{env, time::Duration};

use async_nats::jetstream;
use codequest_common::{Error, load_or_generate_salt, load_or_generate_secret_key};
use dotenv::dotenv;

async fn ensure_stream(
    js: &jetstream::Context,
    name: &str,
    subjects: Vec<String>,
    max_age: Duration,
) -> Result<(), Error> {
    let cfg = jetstream::stream::Config {
        name: name.into(),
        subjects,
        max_age,
        storage: jetstream::stream::StorageType::File,
        retention: jetstream::stream::RetentionPolicy::Limits,
        ..Default::default()
    };

    match js.get_stream(name).await {
        Ok(_) => {
            println!("stream {name} already exists");
        }
        Err(_) => {
            js.create_stream(cfg).await?;
            println!("stream {name} created");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    if let Ok(salt_file) = env::var("SALT_FILE") {
        let _salt = load_or_generate_salt(salt_file);
        println!("Salt loaded or generated");
    }
    if let Ok(secret_key_file) = env::var("SECRET_KEY_FILE") {
        let _secret_key = load_or_generate_secret_key(secret_key_file)
            .expect("failed to load or generate secret key");
        println!("SecretKey loaded or generated");
    }

    let nats_address = env::var("NATS_ADDRESS").expect("NATS_ADDRESS not set");

    loop {
        match async_nats::connect(&nats_address).await {
            Ok(client) => {
                let js = jetstream::new(client);

                ensure_stream(
                    &js,
                    "USER_EVENTS",
                    vec!["user.events.*"]
                        .into_iter()
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>(),
                    Duration::from_secs(60 * 60 * 24 * 30),
                )
                .await?;
                ensure_stream(
                    &js,
                    "QUEST_EVENTS",
                    vec!["quest.events.*"]
                        .into_iter()
                        .map(|s| s.to_owned())
                        .collect::<Vec<_>>(),
                    Duration::from_secs(60 * 60 * 24 * 30),
                )
                .await?;

                println!("NATS JetStream bootstrap completed");
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
        }
    }

    Ok(())
}
