use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{QuestId, Username};

pub trait Event: Serialize + DeserializeOwned {
    fn get_subject(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserEvent {
    Created(Username),
    Deleted(Username),
}

impl Event for UserEvent {
    fn get_subject(&self) -> &'static str {
        match self {
            Self::Created(_) => "user.events.created",
            Self::Deleted(_) => "user.events.deleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressionEvent {
    AnswerSubmitted {
        username: Username,
        correct: bool,
    },
    QuestCompleted {
        username: Username,
        quest_id: QuestId,
    },
}

impl Event for ProgressionEvent {
    fn get_subject(&self) -> &'static str {
        match self {
            Self::AnswerSubmitted { .. } => "progression.events.answer_submitted",
            Self::QuestCompleted { .. } => "progression.events.quest_completed",
        }
    }
}
