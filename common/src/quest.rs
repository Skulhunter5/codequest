use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{Error, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct QuestId(Uuid);

impl QuestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn try_parse(input: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Self(Uuid::try_parse(input.as_ref())?))
    }
}

impl std::fmt::Display for QuestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'r> rocket::request::FromParam<'r> for QuestId {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Uuid::parse_str(param).map(QuestId).map_err(|_| param)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct QuestEntry {
    pub id: QuestId,
    pub name: String,
    pub author: Option<UserId>,
    pub official: bool,
}

impl QuestEntry {
    pub fn new(name: impl Into<String>, author: Option<UserId>, official: bool) -> Self {
        Self {
            id: QuestId::new(),
            name: name.into(),
            author,
            official,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct Quest {
    pub id: QuestId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserId>,
    pub official: bool,
    #[sqlx(rename = "description")]
    pub text: String,
}

impl Quest {
    pub fn new(
        name: impl Into<String>,
        author: Option<UserId>,
        official: bool,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: QuestId::new(),
            name: name.into(),
            author,
            official,
            text: text.into(),
        }
    }

    pub fn to_entry(&self) -> QuestEntry {
        QuestEntry {
            id: self.id,
            name: self.name.clone(),
            author: self.author,
            official: self.official,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct QuestData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserId>,
    pub official: bool,
    #[sqlx(rename = "description")]
    pub text: String,
}

impl QuestData {
    pub fn new(
        name: impl Into<String>,
        author: Option<UserId>,
        official: bool,
        text: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            author,
            official,
            text: text.into(),
        }
    }
}
