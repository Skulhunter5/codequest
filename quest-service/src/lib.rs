use std::{collections::HashMap, io, path::Path, sync::Arc};

use codequest_common::{
    Credentials, Error, Quest, QuestData, QuestEntry, QuestId, UserId, services::QuestService,
};
use reqwest::{Client, StatusCode};
use rocket::{async_trait, serde::json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::RwLock;

use crate::quest_context::QuestContextProvider;

pub mod quest_context;

pub struct ConstQuestService {
    quests: Box<[Quest]>,
}

impl ConstQuestService {
    pub fn new() -> Self {
        let quests = vec![
            Quest::new(
                "Quest 1",
                None,
                true,
                "For this quest, you have to submit '1'",
            ),
            Quest::new(
                "Quest 2",
                None,
                true,
                "For this quest, you have to submit '2'",
            ),
            Quest::new(
                "Quest 3",
                None,
                true,
                "For this quest, you have to submit '3'",
            ),
            Quest::new(
                "Quest 4",
                None,
                true,
                "For this quest, you have to submit '4'",
            ),
        ]
        .into_boxed_slice();
        Self { quests }
    }
}

#[async_trait]
impl QuestService for ConstQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        Ok(self
            .quests
            .iter()
            .map(|quest| quest.to_entry())
            .collect::<Vec<QuestEntry>>()
            .into_boxed_slice())
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        Ok(self.quests.iter().find(|quest| quest.id == *id).cloned())
    }

    async fn quest_exists(&self, id: &QuestId) -> Result<bool, Error> {
        Ok(self.quests.iter().find(|quest| quest.id == *id).is_some())
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &user_id
        )))
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        _user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        Ok(if self.quest_exists(&quest_id).await? {
            Some(quest_id.to_string())
        } else {
            None
        })
    }

    async fn create_quest(&self, _quest: QuestData) -> Result<QuestId, Error> {
        Err(Error::Unsupported)
    }
}

pub struct FileQuestService {
    quests: RwLock<HashMap<QuestId, Quest>>,
}

impl FileQuestService {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let quests = RwLock::new(json::from_str(std::fs::read_to_string(&path)?.as_str())?);
        println!(">> quests loaded: {:?}", &quests);
        Ok(Self { quests })
    }
}

#[async_trait]
impl QuestService for FileQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        Ok(self
            .quests
            .read()
            .await
            .values()
            .map(|quest| quest.to_entry())
            .collect::<Vec<QuestEntry>>()
            .into_boxed_slice())
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        Ok(self.quests.read().await.get(id).cloned())
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &user_id
        )))
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        _user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        Ok(if self.quest_exists(&quest_id).await? {
            Some(quest_id.to_string())
        } else {
            None
        })
    }

    async fn create_quest(&self, quest: QuestData) -> Result<QuestId, Error> {
        let quest = Quest::new(quest.name, quest.author, quest.official, quest.text);
        let id = quest.id;
        let old_value = self.quests.write().await.insert(id, quest);
        assert!(old_value.is_none());
        Ok(id)
    }
}

pub struct DatabaseQuestService {
    pool: PgPool,
    context_provider: Arc<dyn QuestContextProvider>,
}

impl DatabaseQuestService {
    pub async fn new<S: AsRef<str>>(
        address: S,
        db_name: S,
        credentials: Credentials,
        context_provider: Arc<dyn QuestContextProvider>,
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

        Ok(Self {
            pool,
            context_provider,
        })
    }
}

#[async_trait]
impl QuestService for DatabaseQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        Ok(
            sqlx::query_as::<_, QuestEntry>("SELECT id, name, author, official FROM quests")
                .fetch_all(&self.pool)
                .await?
                .into_boxed_slice(),
        )
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        Ok(sqlx::query_as::<_, Quest>(
            "SELECT id, name, description, author, official FROM quests WHERE id = $1",
        )
        .bind(&id)
        .fetch_optional(&self.pool)
        .await?)
    }

    async fn quest_exists(&self, id: &QuestId) -> Result<bool, Error> {
        Ok(
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM quests WHERE id = $1)")
                .bind(&id)
                .fetch_one(&self.pool)
                .await
                .unwrap(),
        )
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.context_provider.get_input(quest_id, user_id).await
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.context_provider.get_answer(quest_id, user_id).await
    }

    async fn create_quest(&self, quest: QuestData) -> Result<QuestId, Error> {
        let id = sqlx::query_scalar::<_, QuestId>(
            "INSERT INTO quests (name, description, author, official) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(quest.name)
        .bind(quest.text)
        .bind(quest.author)
        .bind(quest.official)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
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
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        let response = self
            .client
            .get(&self.address)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;
        match response.status() {
            StatusCode::OK => response
                .json::<Box<[QuestEntry]>>()
                .await
                .map_err(|_| Error::InvalidResponse),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        let response = self
            .client
            .get(format!("{}/{}", &self.address, id))
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

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        let response = self
            .client
            .get(format!("{}/{}/input/{}", &self.address, quest_id, user_id))
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

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        let response = self
            .client
            .get(format!("{}/{}/answer/{}", &self.address, quest_id, user_id))
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
        quest_id: &QuestId,
        user_id: &UserId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        let response = self
            .client
            .post(format!("{}/{}/answer/{}", &self.address, quest_id, user_id))
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

    async fn create_quest(&self, quest: QuestData) -> Result<QuestId, Error> {
        let response = self
            .client
            .post(format!("{}", &self.address))
            .json(&quest)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => Ok(QuestId::try_parse(response.text().await?)?),
            _ => Err(Error::InvalidResponse),
        }
    }
}
