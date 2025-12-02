use codequest_common::{Quest, services::QuestService};
use rocket::async_trait;

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
