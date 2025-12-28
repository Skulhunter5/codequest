use async_nats::jetstream;
use rocket::futures::TryStreamExt as _;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Error, Username};

pub trait Event: Serialize + DeserializeOwned {
    fn get_subject(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserEvent {
    Created(Username),
    Deleted(Username),
}

impl Event for UserEvent {
    fn get_subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "user.events.created",
            Self::Deleted(_) => "user.events.deleted",
        }
    }
}

pub struct NatsClient {
    js: jetstream::Context,
}

impl NatsClient {
    pub async fn new(address: impl AsRef<str>) -> Result<Self, Error> {
        let client = async_nats::connect(address.as_ref()).await?;
        let js = jetstream::new(client);

        Ok(Self { js })
    }

    pub async fn consume<E: Event>(
        self,
        stream_name: impl AsRef<str>,
        consumer_name: String,
        handler: impl AsyncFn(E) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let stream = self.js.get_stream(stream_name.as_ref()).await?;

        let consumer = stream
            .get_or_create_consumer(
                &consumer_name,
                jetstream::consumer::pull::Config {
                    durable_name: Some(consumer_name.clone()),
                    ..Default::default()
                },
            )
            .await?;

        let mut messages = consumer.messages().await?;

        while let Some(message) = messages.try_next().await? {
            let event: E = serde_json::from_slice(&message.payload)?;
            handler(event).await?;
            message.ack().await?;
        }

        Ok(())
    }

    pub async fn emit<E: Event>(&self, event: E) -> Result<(), Error> {
        let payload = serde_json::to_vec(&event)?;
        self.js.publish(event.get_subject(), payload.into()).await?;
        Ok(())
    }
}
