use rocket::async_trait;

use crate::{Error, Quest, QuestId, QuestItem, User, UserId, Username, statistics::Metric};

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_user(&self, id: &UserId) -> Result<Option<User>, Error>;
    async fn login(&self, username: &Username, password: &str) -> Result<Option<UserId>, Error>;
    async fn create_user(
        &self,
        username: Username,
        password: &str,
    ) -> Result<Option<UserId>, Error>;
    async fn delete_user(&self, id: &UserId) -> Result<bool, Error>;
    async fn user_exists(&self, id: &UserId) -> Result<bool, Error>;

    async fn change_password(
        &self,
        id: &UserId,
        old_password: &str,
        new_password: &str,
    ) -> Result<bool, Error>;
}

#[async_trait]
pub trait QuestService: Send + Sync {
    async fn list_quests(&self) -> Result<Box<[QuestItem]>, Error>;
    async fn get_quest(&self, id: &QuestId) -> Result<Option<Quest>, Error>;

    async fn quest_exists(&self, id: &QuestId) -> Result<bool, Error> {
        Ok(self.get_quest(&id).await?.is_some())
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error>;
    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error>;

    async fn verify_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
        answer: &str,
    ) -> Result<Option<bool>, Error> {
        Ok(self
            .get_answer(quest_id, user_id)
            .await?
            .map(|correct_answer| answer == correct_answer))
    }
}

#[async_trait]
pub trait ProgressionService: Send + Sync {
    async fn has_user_completed_quest(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
    ) -> Result<bool, Error>;
    async fn submit_answer(
        &self,
        user_id: &UserId,
        quest_id: &QuestId,
        answer: &str,
    ) -> Result<Option<bool>, Error>;
}

#[async_trait]
pub trait StatisticsService: Send + Sync {
    async fn get_user_metrics(&self, user_id: &UserId) -> Result<Vec<Metric>, Error>;
}
