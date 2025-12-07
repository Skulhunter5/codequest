use std::{io, path::Path, sync::Arc};

use codequest_common::{Error, Quest, QuestItem, services::QuestService};
use reqwest::{Client, StatusCode};
use rocket::{async_trait, serde::json};

pub struct ConstQuestService {
    quests: Arc<[Quest]>,
}

impl ConstQuestService {
    pub fn new() -> Self {
        let quests = Arc::from(
            vec![
                Quest::new(
                    "Quest 1",
                    "quest-1",
                    "For this quest, you have to submit '1'",
                ),
                Quest::new(
                    "Quest 2",
                    "quest-2",
                    "For this quest, you have to submit '2'",
                ),
                Quest::new(
                    "Quest 3",
                    "quest-3",
                    "For this quest, you have to submit '3'",
                ),
                Quest::new(
                    "Quest 4",
                    "quest-4",
                    "For this quest, you have to submit '4'",
                ),
            ]
            .into_boxed_slice(),
        );
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

    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error> {
        Ok(Some(format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )))
    }

    async fn submit_answer(
        &self,
        quest_id: &str,
        _username: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        Ok(if let Some(_) = self.get_quest(quest_id).await? {
            if answer == quest_id {
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        })
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

    async fn submit_answer(
        &self,
        quest_id: &str,
        _username: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        Ok(if let Some(_) = self.get_quest(quest_id).await? {
            if answer == quest_id {
                Some(true)
            } else {
                Some(false)
            }
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

    async fn submit_answer(
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
