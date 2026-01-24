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

    pub fn is_author(&self, user: &UserId) -> bool {
        let Some(author) = &self.author else {
            return false;
        };
        author == user
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestData {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<UserId>,
    pub official: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialQuestData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Option<UserId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub official: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl PartialQuestData {
    pub fn empty() -> Self {
        Self {
            name: None,
            author: None,
            official: None,
            text: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        return self;
    }

    pub fn with_author(mut self, author: Option<UserId>) -> Self {
        self.author = Some(author);
        return self;
    }

    pub fn with_official(mut self, official: bool) -> Self {
        self.official = Some(official);
        return self;
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        return self;
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    pub fn set_author(&mut self, author: Option<UserId>) {
        self.author = Some(author);
    }

    pub fn set_official(&mut self, official: bool) {
        self.official = Some(official);
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = Some(text.into());
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.author.is_none()
            && self.official.is_none()
            && self.text.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestDataFields {
    pub name: bool,
    pub author: bool,
    pub official: bool,
    pub text: bool,
}

impl QuestDataFields {
    pub fn none() -> Self {
        Self {
            name: false,
            author: false,
            official: false,
            text: false,
        }
    }

    pub fn name() -> Self {
        Self::none().and_name()
    }

    pub fn author() -> Self {
        Self::none().and_author()
    }

    pub fn official() -> Self {
        Self::none().and_author()
    }

    pub fn text() -> Self {
        Self::none().and_author()
    }

    pub fn and_name(mut self) -> Self {
        self.name = true;
        return self;
    }

    pub fn and_author(mut self) -> Self {
        self.author = true;
        return self;
    }

    pub fn and_official(mut self) -> Self {
        self.official = true;
        return self;
    }

    pub fn and_text(mut self) -> Self {
        self.text = true;
        return self;
    }
}
