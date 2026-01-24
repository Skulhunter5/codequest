use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
pub struct Metric {
    key: String,
    pub value: String,
}

impl Metric {
    pub fn get_display_name(&self) -> &'static str {
        match self.key.as_str() {
            "answers_submitted" => "Total answers submitted",
            "quests_completed" => "Total quests completed",
            "quests_created" => "Total quests created",
            "quests_modified" => "Total quests modified",
            "quests_deleted" => "Total quests deleted",
            _ => "unknown_metric",
        }
    }
}
