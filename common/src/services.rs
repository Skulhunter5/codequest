use rocket::async_trait;

use crate::{Error, Quest, QuestId, QuestItem, Username, statistics::Metric};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn verify_password(&self, username: &Username, password: &str) -> Result<bool, Error>;
    async fn add_user(&self, username: Username, password: &str) -> Result<bool, Error>;
    async fn delete_user(&self, username: &Username) -> Result<bool, Error>;
    async fn change_password(
        &self,
        username: &Username,
        old_password: &str,
        new_password: &str,
    ) -> Result<bool, Error>;
    async fn user_exists(&self, username: &Username) -> Result<bool, Error>;
}

#[async_trait]
pub trait QuestService: Send + Sync {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error>;
    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error>;

    async fn quest_exists(&self, id: &QuestId) -> Result<bool, Error> {
        Ok(self.get_quest(&id).await?.is_some())
    }

    async fn get_input(&self, quest_id: &QuestId, username: &str) -> Result<Option<String>, Error>;
    async fn get_answer(&self, quest_id: &QuestId, username: &str)
    -> Result<Option<String>, Error>;

    async fn verify_answer(
        &self,
        quest_id: &QuestId,
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
    async fn has_user_completed_quest(
        &self,
        username: &str,
        quest_id: &QuestId,
    ) -> Result<bool, Error>;
    async fn submit_answer(
        &self,
        username: &str,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error>;
}

#[async_trait]
pub trait StatisticsService: Send + Sync {
    async fn get_user_metrics(&self, username: &Username) -> Result<Vec<Metric>, Error>;
}
