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
    async fn get_quest(&self, id: &str) -> Result<Option<Quest>, Error>;
    async fn get_input(&self, quest_id: &str, username: &str) -> Result<Option<String>, Error>;
    async fn verify_answer(
        &self,
        quest_id: &str,
        username: &str,
        answer: &str,
    ) -> Result<Option<bool>, Error>;
}
