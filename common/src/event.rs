use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{QuestId, UserId};

pub trait Event: Serialize + DeserializeOwned {
    fn get_subject(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserEvent {
    Created(UserId),
    Deleted(UserId),
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
    AnswerSubmitted { user_id: UserId, correct: bool },
    QuestCompleted { user_id: UserId, quest_id: QuestId },
}

impl Event for ProgressionEvent {
    fn get_subject(&self) -> &'static str {
        match self {
            Self::AnswerSubmitted { .. } => "progression.events.answer_submitted",
            Self::QuestCompleted { .. } => "progression.events.quest_completed",
        }
    }
}
