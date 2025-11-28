use std::collections::HashMap;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rocket::{async_trait, tokio::sync::RwLock};

use crate::Quest;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn verify_password(&self, username: &str, password: &str) -> bool;
    async fn add_user(&self, username: &str, password: &str) -> bool;
}

pub struct InMemoryUserService {
    users: RwLock<HashMap<String, String>>,
    salt: SaltString,
}

impl InMemoryUserService {
    pub fn new(salt: SaltString) -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
            salt,
        }
    }

    fn hash_password(&self, password: &str) -> String {
        Argon2::default()
            .hash_password(password.as_bytes(), self.salt.as_salt())
            .unwrap()
            .to_string()
    }
}

#[async_trait]
impl UserService for InMemoryUserService {
    async fn verify_password(&self, username: &str, password: &str) -> bool {
        if let Some(correct_hash) = self.users.read().await.get(username) {
            let hash = self.hash_password(password);
            return hash == *correct_hash;
        }
        return false;
    }

    async fn add_user(&self, username: &str, password: &str) -> bool {
        if self.users.read().await.contains_key(username) {
            return false;
        }

        let hash = self.hash_password(password);
        self.users.write().await.insert(username.to_owned(), hash);
        return true;
    }
}

static QUESTS: &[Quest] = &[
    Quest {
        name: "Quest 1",
        id: "quest-1",
    },
    Quest {
        name: "Quest 2",
        id: "quest-2",
    },
    Quest {
        name: "Quest 3",
        id: "quest-3",
    },
    Quest {
        name: "Quest 4",
        id: "quest-4",
    },
];

#[async_trait]
pub trait QuestService: Send + Sync {
    fn get_quests(&self) -> &[Quest];
    fn get_quest(&self, id: &str) -> Option<&Quest> {
        self.get_quests().iter().find(|quest| quest.id == id)
    }
}

pub struct ConstQuestService;

impl ConstQuestService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QuestService for ConstQuestService {
    fn get_quests(&self) -> &[Quest] {
        QUESTS
    }
}
