use std::sync::Arc;

use rocket::async_trait;

use crate::Quest;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn verify_password(&self, username: &str, password: &str) -> bool;
    async fn add_user(&self, username: &str, password: &str) -> bool;
    async fn user_exists(&self, username: &str) -> bool;
}

#[async_trait]
pub trait QuestService: Send + Sync {
    async fn get_quests(&self) -> Arc<[Quest]>;
    async fn get_quest(&self, id: &str) -> Option<Quest> {
        self.get_quests()
            .await
            .into_iter()
            .find(|quest| quest.id == id)
            .cloned()
    }
    async fn get_input(&self, quest_id: &str, username: &str) -> String;
}
