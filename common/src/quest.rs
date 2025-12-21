use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow, sqlx::Type)]
#[sqlx(transparent)]
#[repr(transparent)]
pub struct QuestId(Uuid);

impl QuestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
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
pub struct QuestItem {
    pub id: QuestId,
    pub name: String,
}

impl QuestItem {
    pub fn new<S: AsRef<str>>(id: QuestId, name: S) -> Self {
        Self {
            id,
            name: name.as_ref().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, FromRow)]
pub struct Quest {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub item: QuestItem,
    #[sqlx(rename = "description")]
    pub text: String,
}

impl Quest {
    pub fn new<S: AsRef<str>>(name: QuestId, id: S, text: S) -> Self {
        Self {
            item: QuestItem::new(name, id),
            text: text.as_ref().to_owned(),
        }
    }
}
