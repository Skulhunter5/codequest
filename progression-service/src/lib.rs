use std::{
    collections::HashMap,
    fs::File as StdFile,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use codequest_common::{
    Credentials, Error, QuestId, UserId,
    event::{ProgressionEvent, UserEvent},
    nats::NatsClient,
    services::{ProgressionService, QuestService},
};
use reqwest::{Client, StatusCode};
use rocket::{
    async_trait,
    serde::json::{self, serde_json},
    tokio::{fs::File as TokioFile, io::AsyncWriteExt as _, sync::RwLock},
};
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct InMemoryProgressionService {
    user_progress: RwLock<HashMap<UserId, Vec<QuestId>>>,
    quest_service: Arc<dyn QuestService>,
}

impl InMemoryProgressionService {
    pub fn new(quest_service: Arc<dyn QuestService>) -> Self {
        let user_progress = RwLock::new(HashMap::new());
        Self {
            user_progress,
            quest_service,
        }
    }

    pub fn with(
        user_progress: HashMap<UserId, Vec<QuestId>>,
        quest_service: Arc<dyn QuestService>,
    ) -> Self {
        Self {
            user_progress: RwLock::new(user_progress),
            quest_service,
        }
    }
}

#[async_trait]
impl ProgressionService for InMemoryProgressionService {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error> {
        let users = self.user_progress.read().await;
        Ok(if let Some(completed_quests) = users.get(user_id) {
            completed_quests
                .iter()
                .find(|quest| *quest == quest_id)
                .is_some()
        } else {
            false
        })
    }

    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        if self.has_user_completed_quest(user_id, quest_id).await? {
            return Ok(None);
        }
        let res = self
            .quest_service
            .verify_answer(quest_id, user_id, answer)
            .await?;
        if Some(true) == res {
            let mut user_progress = self.user_progress.write().await;
            if let Some(completed_quests) = user_progress.get_mut(user_id) {
                completed_quests.push(quest_id.clone());
            } else {
                user_progress.insert(user_id.to_owned(), vec![quest_id.to_owned()]);
            }
        }

        return Ok(res);
    }
}

pub struct FileProgressionService {
    path: PathBuf,
    in_memory_progression_service: InMemoryProgressionService,
}

impl FileProgressionService {
    pub fn new<P: AsRef<Path>>(path: P, quest_service: Arc<dyn QuestService>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let users = match StdFile::open(&path) {
            Ok(file) => {
                let mut reader = serde_json::Deserializer::from_reader(file).into_iter();
                let users = match reader.next() {
                    Some(users) => users,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "invalid input: no json object in file",
                        ));
                    }
                }?;
                if reader.next().is_some() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "invalid input: too many json objects in file",
                    ));
                }

                users
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => HashMap::new(),
            Err(e) => return Err(e),
        };

        let in_memory_progression_service = InMemoryProgressionService::with(users, quest_service);

        Ok(Self {
            path,
            in_memory_progression_service,
        })
    }

    async fn save(&self) -> Result<(), std::io::Error> {
        let mut file = TokioFile::create(&self.path).await?;

        let user_progress = self
            .in_memory_progression_service
            .user_progress
            .read()
            .await;
        let json_string = json::to_string(&*user_progress)?;
        file.write_all(json_string.as_bytes()).await
    }
}

#[async_trait]
impl ProgressionService for FileProgressionService {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error> {
        self.in_memory_progression_service
            .has_user_completed_quest(user_id, quest_id)
            .await
    }

    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        let res = self
            .in_memory_progression_service
            .submit_answer(user_id, quest_id, answer)
            .await?;
        if Some(true) == res {
            if let Err(e) = self.save().await {
                eprintln!(
                    "FileUserService: failed to write user_progress to file: {}",
                    e
                );
            }
        }
        return Ok(res);
    }
}

pub struct DatabaseProgressionService {
    pool: PgPool,
    quest_service: Arc<dyn QuestService>,
}

impl DatabaseProgressionService {
    pub async fn new<S: AsRef<str>>(
        quest_service: Arc<dyn QuestService>,
        address: S,
        db_name: S,
        credentials: Credentials,
        nats_address: impl AsRef<str>,
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
        let pool2 = pool.clone();

        let nats_client = NatsClient::new(nats_address).await?;

        let _join_handle = rocket::tokio::spawn(async move {
            println!("NATS garbage collector started");
            let pool = pool2;
            let _x = nats_client
                .consume::<UserEvent>(
                    "USER_EVENTS",
                    "progression-service".to_owned(),
                    async move |event| {
                        match event {
                            UserEvent::Deleted(user_id) => {
                                let _query_result =
                                    sqlx::query("DELETE FROM progression WHERE (user_id = $1)")
                                        .bind(&user_id)
                                        .execute(&pool)
                                        .await?;
                            }
                            UserEvent::Created(_) => (),
                        }
                        Ok(())
                    },
                )
                .await
                .expect("NATS garbage collector crashed");
        });

        sqlx::migrate!().run(&pool).await?;

        Ok(Self {
            pool,
            quest_service,
        })
    }
}

#[async_trait]
impl ProgressionService for DatabaseProgressionService {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM progression WHERE (quest_id = $1 AND user_id = $2))",
        )
        .bind(&quest_id)
        .bind(&user_id)
        .fetch_one(&self.pool)
        .await
        .unwrap())
    }

    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        if self.has_user_completed_quest(user_id, quest_id).await? {
            return Ok(None);
        }
        let res = self
            .quest_service
            .verify_answer(quest_id, user_id, answer)
            .await?;
        if Some(true) == res {
            match sqlx::query("INSERT INTO progression (quest_id, user_id) VALUES ($1, $2)")
                .bind(&quest_id)
                .bind(&user_id)
                .execute(&self.pool)
                .await
            {
                Ok(_) => (),
                Err(sqlx::Error::Database(db_error))
                    if db_error.constraint() == Some("progression_pkey") =>
                {
                    return Ok(None);
                }
                Err(e) => return Err(e.into()),
            }
        }

        return Ok(res);
    }
}

pub struct BackendProgressionService {
    address: String,
    client: Client,
}

impl BackendProgressionService {
    pub fn new<S: AsRef<str>>(address: S) -> Self {
        Self {
            address: address.as_ref().to_owned(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl ProgressionService for BackendProgressionService {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error> {
        let response = self
            .client
            .get(format!("{}/{}/{}", &self.address, user_id, quest_id))
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.text().await {
                Ok(user_has_completed_quest) => user_has_completed_quest
                    .parse::<bool>()
                    .map_err(|_| Error::InvalidResponse),
                Err(_) => Err(Error::InvalidResponse),
            },
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        let response = self
            .client
            .post(format!("{}/{}/{}/answer", &self.address, user_id, quest_id))
            .body(answer.to_owned())
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;

        match response.status() {
            StatusCode::OK => match response.text().await {
                Ok(answer_was_correct) => Ok(Some(
                    answer_was_correct
                        .parse::<bool>()
                        .map_err(|_| Error::InvalidResponse)?,
                )),
                Err(_) => Err(Error::InvalidResponse),
            },
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(Error::InvalidResponse),
        }
    }
}

pub struct ProgressionServiceNatsWrapper {
    progression_service: Arc<dyn ProgressionService>,
    nats_client: NatsClient,
}

impl ProgressionServiceNatsWrapper {
    pub async fn new(
        progression_service: Arc<dyn ProgressionService>,
        nats_address: impl AsRef<str>,
    ) -> Result<Self, Error> {
        let nats_client = NatsClient::new(nats_address).await?;
        Ok(Self {
            progression_service,
            nats_client,
        })
    }
}

#[async_trait]
impl ProgressionService for ProgressionServiceNatsWrapper {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error> {
        self.progression_service
            .has_user_completed_quest(user_id, quest_id)
            .await
    }

    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        let res = self
            .progression_service
            .submit_answer(user_id, quest_id, answer)
            .await?;
        if let Some(correct) = res {
            self.nats_client
                .emit(ProgressionEvent::AnswerSubmitted {
                    user_id: user_id.clone(),
                    correct,
                })
                .await?;
            if correct {
                self.nats_client
                    .emit(ProgressionEvent::QuestCompleted {
                        user_id: user_id.clone(),
                        quest_id: quest_id.clone(),
                    })
                    .await?
            }
        }
        return Ok(res);
    }
}
