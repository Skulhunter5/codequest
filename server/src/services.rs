use std::{
    collections::HashMap,
    fs::File as StdFile,
    io,
    path::{Path, PathBuf},
};

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rocket::{
    async_trait,
    serde::json,
    tokio::{fs::File as TokioFile, io::AsyncWriteExt, sync::RwLock},
};

use crate::Quest;

// TODO: restrict valid usernames

#[async_trait]
pub trait UserService: Send + Sync {
    async fn verify_password(&self, username: &str, password: &str) -> bool;
    async fn add_user(&self, username: &str, password: &str) -> bool;
    async fn user_exists(&self, username: &str) -> bool;
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

    pub fn with(salt: SaltString, users: HashMap<String, String>) -> Self {
        Self {
            users: RwLock::new(users),
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

    async fn user_exists(&self, username: &str) -> bool {
        self.users.read().await.contains_key(username)
    }
}

pub struct FileUserService {
    path: PathBuf,
    in_memory_user_service: InMemoryUserService,
}

impl FileUserService {
    pub fn new<P: AsRef<Path>>(salt: SaltString, path: P) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let users = match StdFile::open(&path) {
            Ok(file) => {
                let mut reader =
                    rocket::serde::json::serde_json::Deserializer::from_reader(file).into_iter();
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

        let in_memory_user_service = InMemoryUserService::with(salt, users);

        Ok(Self {
            path,
            in_memory_user_service,
        })
    }

    async fn save(&self) -> Result<(), std::io::Error> {
        let mut file = TokioFile::create(&self.path).await?;

        let users = self.in_memory_user_service.users.read().await;
        let json_string = json::to_string(&*users)?;
        file.write_all(json_string.as_bytes()).await
    }
}

#[async_trait]
impl UserService for FileUserService {
    async fn verify_password(&self, username: &str, password: &str) -> bool {
        self.in_memory_user_service
            .verify_password(username, password)
            .await
    }

    async fn add_user(&self, username: &str, password: &str) -> bool {
        let created = self
            .in_memory_user_service
            .add_user(username, password)
            .await;
        if created {
            if let Err(e) = self.save().await {
                eprintln!("FileUserService: failed to write users to file: {}", e);
            }
        }
        return created;
    }

    async fn user_exists(&self, username: &str) -> bool {
        self.in_memory_user_service.user_exists(username).await
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
    async fn get_quests(&self) -> &[Quest];
    async fn get_quest(&self, id: &str) -> Option<&Quest> {
        self.get_quests().await.iter().find(|quest| quest.id == id)
    }
    async fn get_input(&self, quest: &Quest, username: &str) -> String;
}

pub struct ConstQuestService;

impl ConstQuestService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl QuestService for ConstQuestService {
    async fn get_quests(&self) -> &[Quest] {
        QUESTS
    }

    async fn get_input(&self, quest: &Quest, username: &str) -> String {
        format!(
            "[WIP] Input for quest '{}' for user '{}'",
            &quest.name, &username
        )
    }
}
