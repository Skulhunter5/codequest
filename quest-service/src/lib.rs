use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use codequest_common::{
    Credentials, Error, PartialQuestData, Quest, QuestData, QuestEntry, QuestId, UserId,
    event::QuestEvent, nats::NatsClient, services::QuestService,
};
use reqwest::{Client, StatusCode};
use rocket::{async_trait, serde::json};
use sqlx::{PgPool, QueryBuilder, postgres::PgPoolOptions};
use tokio::{fs::File as TokioFile, io::AsyncWriteExt as _, sync::RwLock};

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

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        Ok(self
            .quests
            .iter()
            .find(|quest| quest.id == *id)
            .map(|quest| quest.author.clone()))
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

    async fn update_quest(&self, _id: &QuestId, _data: QuestData) -> Result<bool, Error> {
        Err(Error::Unsupported)
    }

    async fn modify_quest(&self, _id: &QuestId, _data: PartialQuestData) -> Result<bool, Error> {
        Err(Error::Unsupported)
    }
}

pub struct InMemoryQuestService {
    quests: RwLock<HashMap<QuestId, Quest>>,
}

impl InMemoryQuestService {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            quests: RwLock::new(HashMap::new()),
        })
    }

    pub fn with(quests: HashMap<QuestId, Quest>) -> Self {
        Self {
            quests: RwLock::new(quests),
        }
    }
}

#[async_trait]
impl QuestService for InMemoryQuestService {
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

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        Ok(self
            .quests
            .read()
            .await
            .get(id)
            .map(|quest| quest.author.clone()))
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

    async fn update_quest(&self, id: &QuestId, data: QuestData) -> Result<bool, Error> {
        let quests = &mut self.quests.write().await;
        let Some(quest) = quests.get_mut(id) else {
            return Ok(false);
        };
        quest.name = data.name;
        quest.author = data.author;
        quest.official = data.official;
        quest.text = data.text;
        Ok(true)
    }

    async fn modify_quest(&self, id: &QuestId, data: PartialQuestData) -> Result<bool, Error> {
        if data.is_empty() {
            return Err(Error::BadRequest);
        }

        let quests = &mut self.quests.write().await;
        let Some(quest) = quests.get_mut(id) else {
            return Ok(false);
        };
        if let Some(name) = data.name {
            quest.name = name;
        }
        if let Some(author) = data.author {
            quest.author = author;
        }
        if let Some(official) = data.official {
            quest.official = official;
        }
        if let Some(text) = data.text {
            quest.text = text;
        }
        Ok(true)
    }
}

pub struct FileQuestService {
    path: PathBuf,
    in_memory_quest_service: InMemoryQuestService,
}

impl FileQuestService {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let quests: HashMap<QuestId, Quest> =
            json::from_str(std::fs::read_to_string(&path)?.as_str())?;
        println!("{} quest(s) loaded", quests.len());

        let in_memory_quest_service = InMemoryQuestService::with(quests);

        Ok(Self {
            path,
            in_memory_quest_service,
        })
    }

    async fn save(&self) -> Result<(), std::io::Error> {
        let mut file = TokioFile::create(&self.path).await?;

        let quests = self.in_memory_quest_service.quests.read().await;
        let json_string = json::to_string(&*quests)?;
        file.write_all(json_string.as_bytes()).await
    }
}

#[async_trait]
impl QuestService for FileQuestService {
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        self.in_memory_quest_service.list_quests().await
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        self.in_memory_quest_service.get_quest(id).await
    }

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        self.in_memory_quest_service.get_quest_author(id).await
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.in_memory_quest_service
            .get_input(quest_id, user_id)
            .await
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.in_memory_quest_service
            .get_answer(quest_id, user_id)
            .await
    }

    async fn create_quest(&self, quest: QuestData) -> Result<QuestId, Error> {
        let quest_id = self.in_memory_quest_service.create_quest(quest).await?;
        if let Err(e) = self.save().await {
            eprintln!("FileQuestService: failed to write quests to file: {}", e);
        }
        return Ok(quest_id);
    }

    async fn update_quest(&self, id: &QuestId, data: QuestData) -> Result<bool, Error> {
        let quest_updated = self.in_memory_quest_service.update_quest(id, data).await?;
        if quest_updated {
            if let Err(e) = self.save().await {
                eprintln!("FileQuestService: failed to write quests to file: {}", e);
            }
        }
        return Ok(quest_updated);
    }

    async fn modify_quest(&self, id: &QuestId, data: PartialQuestData) -> Result<bool, Error> {
        let quest_modified = self.in_memory_quest_service.modify_quest(id, data).await?;
        if quest_modified {
            if let Err(e) = self.save().await {
                eprintln!("FileQuestService: failed to write quests to file: {}", e);
            }
        }
        return Ok(quest_modified);
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

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        Ok(
            sqlx::query_scalar::<_, Option<UserId>>("SELECT author FROM quests WHERE id = $1")
                .bind(&id)
                .fetch_optional(&self.pool)
                .await?,
        )
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

    async fn update_quest(&self, id: &QuestId, data: QuestData) -> Result<bool, Error> {
        let res = sqlx::query(
            "UPDATE quests SET name = $2, author = $3, official = $4, description = $5 WHERE (id = $1)",
        )
        .bind(id)
        .bind(data.name)
        .bind(data.author)
        .bind(data.official)
        .bind(data.text)
        .execute(&self.pool)
        .await?;
        match res.rows_affected() {
            0 => Ok(false),
            1 => Ok(true),
            x => unreachable!(
                "SQL 'UPDATE quests' query is constrained by primary key (id) but multiple rows ({}) were affected",
                x
            ),
        }
    }

    async fn modify_quest(&self, id: &QuestId, data: PartialQuestData) -> Result<bool, Error> {
        if data.is_empty() {
            return Err(Error::BadRequest);
        }

        let mut query_builder = QueryBuilder::new("UPDATE quests SET ");
        let mut separated = query_builder.separated(", ");
        if let Some(name) = data.name {
            separated.push("name = ").push_bind_unseparated(name);
        }
        if let Some(author) = data.author {
            separated.push("author = ").push_bind_unseparated(author);
        }
        if let Some(official) = data.official {
            separated
                .push("official = ")
                .push_bind_unseparated(official);
        }
        if let Some(text) = data.text {
            separated.push("description = ").push_bind_unseparated(text);
        }
        query_builder.push(" WHERE id = ").push_bind(id);
        let query = query_builder.build();

        let res = query.execute(&self.pool).await?;
        match res.rows_affected() {
            0 => Ok(false),
            1 => Ok(true),
            x => unreachable!(
                "SQL 'UPDATE quests' query is constrained by primary key (id) but multiple rows ({}) were affected",
                x
            ),
        }
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

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        let response = self
            .client
            .get(format!("{}/{}/author", &self.address, id))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;
        match response.status() {
            StatusCode::OK => match response.json().await {
                Ok(author) => Ok(Some(author)),
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

    async fn update_quest(&self, id: &QuestId, data: QuestData) -> Result<bool, Error> {
        let response = self
            .client
            .put(format!("{}/{}", &self.address, id))
            .json(&data)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn modify_quest(&self, id: &QuestId, data: PartialQuestData) -> Result<bool, Error> {
        let response = self
            .client
            .patch(format!("{}/{}", &self.address, id))
            .json(&data)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            StatusCode::BAD_REQUEST => Err(Error::BadRequest),
            _ => Err(Error::InvalidResponse),
        }
    }
}

pub struct QuestServiceNatsWrapper {
    quest_service: Arc<dyn QuestService>,
    nats_client: NatsClient,
}

impl QuestServiceNatsWrapper {
    pub async fn new(
        quest_service: Arc<dyn QuestService>,
        nats_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let nats_client = NatsClient::new(nats_address).await?;
        Ok(Self {
            quest_service,
            nats_client,
        })
    }
}

#[async_trait]
impl QuestService for QuestServiceNatsWrapper {
    async fn list_quests(&self) -> Result<Box<[QuestEntry]>, Error> {
        self.quest_service.list_quests().await
    }

    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error> {
        self.quest_service.get_quest(id).await
    }

    async fn get_quest_author(&self, id: &QuestId) -> Result<Option<Option<UserId>>, Error> {
        self.quest_service.get_quest_author(id).await
    }

    async fn quest_exists(&self, id: &QuestId) -> Result<bool, Error> {
        self.quest_service.quest_exists(id).await
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.quest_service.get_input(quest_id, user_id).await
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.quest_service.get_answer(quest_id, user_id).await
    }

    async fn verify_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        self.quest_service
            .verify_answer(quest_id, user_id, answer)
            .await
    }

    async fn create_quest(&self, quest: QuestData) -> Result<QuestId, Error> {
        let quest_id = self.quest_service.create_quest(quest).await?;
        self.nats_client.emit(QuestEvent::Created(quest_id)).await?;
        return Ok(quest_id);
    }

    async fn update_quest(&self, id: &QuestId, data: QuestData) -> Result<bool, Error> {
        let quest_modified = self.quest_service.update_quest(id, data).await?;
        if quest_modified {
            self.nats_client
                .emit(QuestEvent::Modified(id.clone()))
                .await?;
        }
        return Ok(quest_modified);
    }

    async fn modify_quest(&self, id: &QuestId, data: PartialQuestData) -> Result<bool, Error> {
        let quest_modified = self.quest_service.modify_quest(id, data).await?;
        if quest_modified {
            self.nats_client
                .emit(QuestEvent::Modified(id.clone()))
                .await?;
        }
        return Ok(quest_modified);
    }
}
