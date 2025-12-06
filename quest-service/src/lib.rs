use std::{io, path::Path, sync::Arc};

use codequest_common::{Error, Quest, services::QuestService};
use reqwest::{Client, StatusCode};
use rocket::{async_trait, serde::json};

pub struct ConstQuestService {
    quests: Arc<[Quest]>,
}

impl ConstQuestService {
    pub fn new() -> Self {
        let quests = Arc::from(
            vec![
                Quest::new("Quest 1", "quest-1"),
                Quest::new("Quest 2", "quest-2"),
                Quest::new("Quest 3", "quest-3"),
                Quest::new("Quest 4", "quest-4"),
            ]
            .into_boxed_slice(),
        );
        Self { quests }
    }
}

#[async_trait]
impl QuestService for ConstQuestService {
    async fn get_quests(&self) -> Result<Arc<[Quest]>, Error> {
        Ok(self.quests.clone())
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
    }
}

pub struct FileQuestService {
    quests: Arc<[Quest]>,
}

impl FileQuestService {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let quests = json::from_str(std::fs::read_to_string(&path)?.as_str())?;
        Ok(Self { quests })
    }
}

#[async_trait]
impl QuestService for FileQuestService {
    async fn get_quests(&self) -> Result<Arc<[Quest]>, Error> {
        Ok(self.quests.clone())
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
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
    async fn get_quests(&self) -> Result<Arc<[Quest]>, Error> {
        let response = self
            .client
            .get(&self.address)
            .send()
            .await
            .map_err(|_| Error::ServerUnreachable)?;
        match response.status() {
            StatusCode::OK => match response.json::<Box<[Quest]>>().await {
                Ok(quests) => Ok(Arc::from(quests)),
                Err(_) => Err(Error::InvalidResponse),
            },
            _ => Err(Error::InvalidResponse),
        }
    }

    async fn get_quest(&self, id: &str) -> Result<Option<Quest>, Error> {
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
}
