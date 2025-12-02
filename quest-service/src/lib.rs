use std::sync::Arc;

use codequest_common::{Quest, services::QuestService};
use reqwest::{Client, StatusCode};
use rocket::async_trait;

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
    async fn get_quests(&self) -> Arc<[Quest]> {
        self.quests.clone()
    }

    async fn get_input(&self, quest_id: &str, username: &str) -> String {
        format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest_id, &username
        )
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
    async fn get_quests(&self) -> Arc<[Quest]> {
        let response = match self.client.get(&self.address).send().await {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to quest-service backend failed: {}", e);
                return Arc::new([]);
            }
        };
        if response.status() == StatusCode::OK {
            let quests: Box<[Quest]> = response.json().await.unwrap();
            return Arc::from(quests);
        }
        return Arc::new([]);
    }
    async fn get_quest(&self, id: &str) -> Option<Quest> {
        let response = match self
            .client
            .get(format!("{}/{}", &self.address, id))
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to quest-service backend failed: {}", e);
                return None;
            }
        };
        if response.status() == StatusCode::OK {
            let quest = response.json().await.unwrap();
            return Some(quest);
        }
        return None;
    }
    async fn get_input(&self, quest_id: &str, username: &str) -> String {
        let response = match self
            .client
            .get(format!("{}/{}/input/{}", &self.address, quest_id, username))
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to quest-service backend failed: {}", e);
                return String::new();
            }
        };
        if response.status() == StatusCode::OK {
            return response.text().await.unwrap();
        }
        return String::new();
    }
}
