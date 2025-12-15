use std::{io, path::Path};

use codequest_common::{Credentials, Error, Quest, QuestItem, services::QuestService};
use reqwest::{Client, StatusCode};
use rocket::{async_trait, serde::json};
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};

pub struct ConstQuestService {
    quests: Box<[Quest]>,
}

impl ConstQuestService {
    pub fn new() -> Self {
        let quests = vec![
            Quest::new(
                "quest-1",
                "Quest 1",
                "For this quest, you have to submit '1'",
            ),
            Quest::new(
                "quest-2",
                "Quest 2",
                "For this quest, you have to submit '2'",
            ),
            Quest::new(
                "quest-3",
                "Quest 3",
                "For this quest, you have to submit '3'",
            ),
            Quest::new(
                "quest-4",
                "Quest 4",
                "For this quest, you have to submit '4'",
            ),
        ]
        .into_boxed_slice();
        Self { quests }
    }
}

#[async_trait]
impl QuestService for ConstQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error> {
        Ok(self
            .quests
            .iter()
            .map(|quest| quest.item.clone())
            .collect::<Vec<QuestItem>>()
            .into_boxed_slice())
    }

    async fn get_quest(&self, quest_id: &str) -> Result<Option<Quest>, Error> {
        Ok(self
            .quests
            .iter()
            .find(|quest| quest.item.id == quest_id)
            .cloned())
    }

    async fn quest_exists(&self, quest_id: &str) -> Result<bool, Error> {
        Ok(self
            .quests
            .iter()
            .find(|quest| quest.item.id == quest_id)
            .is_some())
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
    }

    async fn get_answer(&self, quest_id: &str, _username: &str) -> Result<Option<String>, Error> {
        Ok(if self.quest_exists(&quest_id).await? {
            Some(quest_id.to_owned())
        } else {
            None
        })
    }
}

pub struct FileQuestService {
    quests: Box<[Quest]>,
}

impl FileQuestService {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let quests = json::from_str(std::fs::read_to_string(&path)?.as_str())?;
        println!(">> quests loaded: {:?}", &quests);
        Ok(Self { quests })
    }
}

#[async_trait]
impl QuestService for FileQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error> {
        Ok(self
            .quests
            .iter()
            .map(|quest| quest.item.clone())
            .collect::<Vec<QuestItem>>()
            .into_boxed_slice())
    }

    async fn get_quest(&self, quest_id: &str) -> Result<Option<Quest>, Error> {
        Ok(self
            .quests
            .iter()
            .find(|quest| quest.item.id == quest_id)
            .cloned())
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
    }

    async fn get_answer(&self, quest_id: &str, _username: &str) -> Result<Option<String>, Error> {
        Ok(if self.quest_exists(&quest_id).await? {
            Some(quest_id.to_owned())
        } else {
            None
        })
    }
}

pub struct DatabaseQuestService {
    pool: PgPool,
}

impl DatabaseQuestService {
    pub async fn new<S: AsRef<str>>(
        address: S,
        db_name: S,
        credentials: Credentials,
    ) -> Result<Self, Error> {
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

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl QuestService for DatabaseQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error> {
        Ok(
            sqlx::query_as::<_, QuestItem>("SELECT id, name FROM quests")
                .fetch_all(&self.pool)
                .await?
                .into_boxed_slice(),
        )
    }

    async fn get_quest(&self, quest_id: &str) -> Result<Option<Quest>, Error> {
        Ok(
            sqlx::query_as::<_, Quest>("SELECT id, name, description FROM quests WHERE id = $1")
                .bind(&quest_id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    async fn quest_exists(&self, quest_id: &str) -> Result<bool, Error> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM quests WHERE id = $1)")
                .bind(&quest_id)
                .fetch_one(&self.pool)
                .await
                .unwrap(),
        )
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
    }

    async fn get_answer(&self, quest_id: &str, _username: &str) -> Result<Option<String>, Error> {
        Ok(if self.quest_exists(&quest_id).await? {
            Some(quest_id.to_owned())
        } else {
            None
        })
    }
}

pub struct BackendQuestService {
    address: String,
    client: Client,
}

impl BackendQuestService {
    pub fn new<S: AsRef<str>>(address: S) -> Self {
        Self {
            address: address.as_ref().to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl QuestService for BackendQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error> {
        let response = self
            .client
            .get(&self.address)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;
        match response.status() {
            StatusCode::OK => response
                .json::<Box<[QuestItem]>>()
                .await
                .map_err(|_| Error::InvalidResponse),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn get_quest(&self, quest_id: &str) -> Result<Option<Quest>, Error> {
        let response = self
            .client
            .get(format!("{}/{}", &self.address, quest_id))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;
        match response.status() {
            StatusCode::OK => match response.json().await {
                Ok(quest) => Ok(Some(quest)),
                Err(_) => Err(Error::ServerUnreachable),
            },
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        let response = self
            .client
            .get(format!("{}/{}/input/{}", &self.address, quest_id, username))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.text().await {
                Ok(input) => Ok(Some(input)),
                Err(_) => Err(Error::InvalidResponse),
            },
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn get_answer(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        let response = self
            .client
            .get(format!(
                "{}/{}/answer/{}",
                &self.address, quest_id, username
            ))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.text().await {
                Ok(answer) => Ok(Some(answer)),
                Err(_) => Err(Error::InvalidResponse),
            },
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn verify_answer(
        &self,
        quest_id: &str,
        username: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        let response = self
            .client
            .post(format!(
                "{}/{}/answer/{}",
                &self.address, quest_id, username
            ))
            .body(answer.to_owned())
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.text().await {
                Ok(input) => Ok(Some(
                    input.parse::<bool>().map_err(|_| Error::InvalidResponse)?,
                )),
                Err(_) => Err(Error::InvalidResponse),
            },
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::InvalidResponse),
        }
    }
}
