use example_quests::{QuestContext, quest_context_generator};

fn main() {
    quest_context_generator(|_rng| {
        QuestContext::new("Input for Quest 2".to_owned(), "quest-2".to_owned())
    });
}
