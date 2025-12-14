use std::{
    collections::HashMap,
    fs::File as StdFile,
    io,
    path::{Path, PathBuf},
};

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use codequest_common::{Error, services::UserService};
use reqwest::{Client, StatusCode};
use rocket::{
    async_trait,
    serde::json,
    tokio::{fs::File as TokioFile, io::AsyncWriteExt as _, sync::RwLock},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};

// TODO: restrict valid usernames
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

pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct DatabaseUserService {
    salt: SaltString,
    pool: PgPool,
}

impl DatabaseUserService {
    pub async fn new<S: AsRef<str>>(
        address: S,
        db_name: S,
        credentials: Credentials,
        salt: SaltString,
    ) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(
                format!(
                    "postgres://{}:{}@{}/{}",
                    credentials.username,
                    credentials.password,
                    address.as_ref(),
                    db_name.as_ref()
                )
                .as_str(),
            )
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { salt, pool })
    }

    fn hash_password(&self, password: &str) -> String {
        Argon2::default()
            .hash_password(password.as_bytes(), self.salt.as_salt())
            .unwrap()
            .to_string()
    }
}

#[async_trait]
impl UserService for DatabaseUserService {
    async fn verify_password(&self, username: &str, password: &str) -> bool {
        let password_hash = self.hash_password(password);
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE (username = $1 AND password_hash = $2))",
        )
        .bind(&username)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn add_user(&self, username: &str, password: &str) -> bool {
        let password_hash = self.hash_password(password);
        match sqlx::query("INSERT INTO users (username, password_hash) VALUES ($1, $2)")
            .bind(&username)
            .bind(&password_hash)
            .execute(&self.pool)
            .await
        {
            Ok(_) => true,
            Err(sqlx::Error::Database(db_error)) if db_error.constraint() == Some("users_pkey") => {
                false
            }
            e => panic!("DatabaseError: {:?}", e),
        }
    }

    async fn user_exists(&self, username: &str) -> bool {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(&username)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

pub struct BackendUserService {
    address: String,
    client: Client,
}

impl BackendUserService {
    pub fn new<S: AsRef<str>>(address: S) -> Self {
        Self {
            address: address.as_ref().to_owned(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserCredentials<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[async_trait]
impl UserService for BackendUserService {
    async fn verify_password(&self, username: &str, password: &str) -> bool {
        let credentials = UserCredentials { username, password };
        let response = match self
            .client
            .post(format!("{}/login", &self.address))
            .json(&credentials)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to user-service backend failed: {}", e);
                return false;
            }
        };
        if response.status() == StatusCode::NO_CONTENT {
            return true;
        }
        return false;
    }

    async fn add_user(&self, username: &str, password: &str) -> bool {
        let credentials = UserCredentials { username, password };
        let response = match self
            .client
            .post(&self.address)
            .json(&credentials)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to user-service backend failed: {}", e);
                return false;
            }
        };
        if response.status() == StatusCode::CREATED {
            return true;
        }
        return false;
    }

    async fn user_exists(&self, username: &str) -> bool {
        let response = match self
            .client
            .get(format!("{}/{}", &self.address, username))
            .json(&username)
            .send()
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("request to user-service backend failed: {}", e);
                return false;
            }
        };
        if response.status() == StatusCode::NO_CONTENT {
            return true;
        }
        return false;
    }
}
