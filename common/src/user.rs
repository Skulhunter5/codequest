use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::Error;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow, sqlx::Type,
)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'r> rocket::request::FromParam<'r> for UserId {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Uuid::parse_str(param).map(UserId).map_err(|_| param)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct Username(String);

impl Username {
    fn is_valid(value: impl AsRef<str>) -> bool {
        let value = value.as_ref();
        (1..=30).contains(&value.len())
            && value.chars().find(|c| !Self::is_valid_char(*c)).is_none()
    }

    fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }

    pub fn build(value: impl Into<String>) -> Result<Self, Error> {
        let value = value.into();
        if !Self::is_valid(&value) {
            return Err(Error::InvalidUsername(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_ref(&self) -> UsernameRef<'_> {
        UsernameRef(self.as_str())
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::build(value).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct UsernameRef<'a>(&'a str);

impl<'a> UsernameRef<'a> {
    pub fn build(value: &'a str) -> Result<Self, Error> {
        if !Username::is_valid(&value) {
            return Err(Error::InvalidUsername(value.to_owned()));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn to_owned(&self) -> Username {
        Username(self.0.to_owned())
    }
}

impl AsRef<str> for UsernameRef<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for UsernameRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: &str = Deserialize::deserialize(deserializer)?;
        Self::build(value).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for UsernameRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    id: UserId,
    username: Username,
}

impl User {
    pub fn new(id: UserId, username: Username) -> Self {
        Self { id, username }
    }
}
