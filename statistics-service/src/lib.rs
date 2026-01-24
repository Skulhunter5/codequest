use std::sync::Arc;

use codequest_common::{
    Credentials, Error, UserId,
    event::{ProgressionEvent, QuestEvent},
    nats::NatsClient,
    services::{QuestService, StatisticsService},
    statistics::Metric,
};
use reqwest::{Client, StatusCode};
use rocket::async_trait;
use sqlx::{PgPool, postgres::PgPoolOptions};

async fn stat_plus_one(
    stat: &str,
    user_id: &UserId,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO statistics (user_id, metric_key, metric_value)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, metric_key) DO UPDATE SET
                    metric_value = statistics.metric_value + EXCLUDED.metric_value",
    )
    .bind(user_id)
    .bind(stat)
    .bind(1)
    .execute(pool)
    .await?;
    Ok(())
}

pub struct DatabaseStatisticsService {
    pool: PgPool,
}

impl DatabaseStatisticsService {
    pub async fn new<S: AsRef<str>>(
        address: S,
        db_name: S,
        credentials: Credentials,
        nats_address: impl AsRef<str>,
        quest_service: Arc<dyn QuestService>,
    ) -> Result<Self, Error> {
        let nats_address = nats_address.as_ref();

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(
                format!(
                    "postgres://{}:{}@{}/{}",
                    credentials.username,
                    credentials.password,
                    address.as_ref(),
                    db_name.as_ref()
                )
                .as_str(),
            )
            .await?;

        let _join_handle = {
            let pool = pool.clone();
            let nats_client = NatsClient::new(nats_address).await?;
            rocket::tokio::spawn(async move {
                println!("NATS event worker started: ProgressionEvents");
                let _x = nats_client
                    .consume::<ProgressionEvent>(
                        "PROGRESSION_EVENTS",
                        "statistics-service".to_owned(),
                        async move |event| {
                            match event {
                                ProgressionEvent::AnswerSubmitted {
                                    user_id,
                                    correct: _,
                                } => stat_plus_one("answers_submitted", &user_id, &pool).await?,
                                ProgressionEvent::QuestCompleted {
                                    user_id,
                                    quest_id: _,
                                } => stat_plus_one("quests_completed", &user_id, &pool).await?,
                            }
                            Ok(())
                        },
                    )
                    .await
                    .expect("NATS event worker crashed: ProgressionEvents");
            })
        };

        let (tx, mut rx) = rocket::tokio::sync::mpsc::unbounded_channel::<QuestEvent>();
        let _join_handle = {
            let nats_client = NatsClient::new(nats_address).await?;
            rocket::tokio::spawn(async move {
                println!("NATS event worker started: QuestEvents 1");
                let _x = nats_client
                    .consume::<QuestEvent>(
                        "QUEST_EVENTS",
                        "statistics-service".to_owned(),
                        async move |event| {
                            tx.send(event).unwrap();
                            Ok(())
                        },
                    )
                    .await
                    .expect("NATS event worker crashed: QuestEvents 1");
            })
        };
        let _join_handle = {
            let pool = pool.clone();
            rocket::tokio::spawn(async move {
                println!("NATS event worker started: QuestEvents 2");
                async move {
                    while let Some(event) = rx.recv().await {
                        match event {
                            QuestEvent::Created(quest_id) => {
                                if let Some(Some(author)) =
                                    quest_service.get_quest_author(&quest_id).await?
                                {
                                    stat_plus_one("quests_created", &author, &pool).await?;
                                }
                            }
                            QuestEvent::Modified(quest_id) => {
                                if let Some(Some(author)) =
                                    quest_service.get_quest_author(&quest_id).await?
                                {
                                    stat_plus_one("quests_modified", &author, &pool).await?;
                                }
                            }
                            QuestEvent::Deleted(quest_id) => {
                                if let Some(Some(author)) =
                                    quest_service.get_quest_author(&quest_id).await?
                                {
                                    stat_plus_one("quests_deleted", &author, &pool).await?;
                                }
                            }
                        }
                    }
                    Result::<(), Error>::Ok(())
                }
                .await
                .expect("NATS event worker crashed: QuestEvents 2");
            })
        };

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl StatisticsService for DatabaseStatisticsService {
    async fn get_user_metrics(&self, user_id: &UserId) -> Result<Vec<Metric>, Error> {
        let metrics = sqlx::query_as::<_, Metric>(
            "SELECT metric_key as key, metric_value::TEXT as value FROM statistics WHERE (user_id = $1)",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(metrics)
    }
}

pub struct BackendStatisticsService {
    address: String,
    client: Client,
}

impl BackendStatisticsService {
    pub fn new<S: AsRef<str>>(address: S) -> Self {
        Self {
            address: address.as_ref().to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl StatisticsService for BackendStatisticsService {
    async fn get_user_metrics(&self, user_id: &UserId) -> Result<Vec<Metric>, Error> {
        let response = self
            .client
            .get(format!("{}/{}", &self.address, user_id))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.json().await {
                Ok(metrics) => Ok(metrics),
                Err(_) => Err(Error::InvalidResponse),
            },
            _ => Err(Error::InvalidResponse),
        }
    }
}
