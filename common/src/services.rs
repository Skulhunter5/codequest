use rocket::async_trait;

use crate::{Error, Quest, QuestItem};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn verify_password(&self, username: &str, password: &str) -> bool;
    async fn add_user(&self, username: &str, password: &str) -> bool;
    async fn user_exists(&self, username: &str) -> bool;
}

#[async_trait]
pub trait QuestService: Send + Sync {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error>;
    async fn get_quest(&self, quest_id: &str) -> Result<Option<Quest>, Error>;
    async fn quest_exists(&self, quest_id: &str) -> Result<bool, Error> {
        Ok(self.get_quest(&quest_id).await?.is_some())
    }
    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error>;
    async fn get_answer(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error>;
    async fn verify_answer(
        &self,
        quest_id: &str,
        username: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        Ok(self
            .get_answer(&quest_id, &username)
            .await?
            .map(|correct_answer| answer == correct_answer))
    }
}

#[async_trait]
pub trait ProgressionService: Send + Sync {
    async fn has_user_completed_quest(&self, username: &str, quest_id: &str)
    -> Result<bool, Error>;
    async fn submit_answer(
        &self,
        username: &str,
        quest_id: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error>;
}
