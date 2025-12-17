use argon2::password_hash::{SaltString, rand_core::OsRng};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::FromRow;
use std::{fs, io, path::Path};

mod credentials;
mod error;
pub mod services;

pub use credentials::Credentials;
pub use error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct Quest {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub item: QuestItem,
    #[sqlx(rename = "description")]
    pub text: String,
}

impl Quest {
    pub fn new<S: AsRef<str>>(name: S, id: S, text: S) -> Self {
        Self {
            item: QuestItem::new(name, id),
            text: text.as_ref().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct QuestItem {
    pub id: String,
    pub name: String,
}

impl QuestItem {
    pub fn new<S: AsRef<str>>(id: S, name: S) -> Self {
        Self {
            id: id.as_ref().to_owned(),
            name: name.as_ref().to_owned(),
        }
    }
}

pub fn load_or_generate_salt<P: AsRef<Path>>(path: P) -> SaltString {
    if let Ok(salt) = fs::read_to_string(&path) {
        return SaltString::from_b64(&salt).expect("failed to create salt");
    }

    let salt = SaltString::generate(&mut OsRng);
    fs::write(path, salt.as_str()).expect("failed to write salt to file");
    return salt;
}

pub fn load_secret_key<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    pub fn build(value: String) -> Result<Self, Error> {
        if value.len() == 0 || value.chars().find(|c| !c.is_ascii_alphanumeric()).is_some() {
            return Err(Error::InvalidUsername(value));
        }
        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Username::build(value).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
