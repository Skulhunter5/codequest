use std::{collections::HashMap, io::ErrorKind, path::PathBuf, sync::Arc};

use codequest_common::{Error, QuestId, UserId};
use rocket::{async_trait, tokio::sync::RwLock};
use tokio::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestContext {
    input: String,
    answer: String,
}

impl QuestContext {
    pub fn new(input: String, answer: String) -> Self {
        Self { input, answer }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContextKey {
    quest: QuestId,
    user: UserId,
}

impl ContextKey {
    pub fn new(quest: QuestId, user: UserId) -> Self {
        Self { quest, user }
    }
}

#[async_trait]
pub trait QuestContextProvider: Send + Sync {
    async fn get_context(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<QuestContext>, Error>;

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.get_context(quest_id, user_id)
            .await
            .map(|res| res.map(|context| context.input))
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        self.get_context(quest_id, user_id)
            .await
            .map(|res| res.map(|context| context.answer))
    }
}

pub struct QuestContextGenerator {
    generator_dir_path: PathBuf,
}

impl QuestContextGenerator {
    pub fn new(generator_dir_path: impl Into<PathBuf>) -> Self {
        Self {
            generator_dir_path: generator_dir_path.into(),
        }
    }
}

#[async_trait]
impl QuestContextProvider for QuestContextGenerator {
    async fn get_context(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<QuestContext>, Error> {
        let generator_path = self.generator_dir_path.join(quest_id.to_string());
        let result = match Command::new(generator_path)
            .arg(user_id.to_string())
            .output()
            .await
        {
            Ok(result) => result,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };

        if !result.status.success() {
            return Err(Error::QuestContextGeneratorFailed {
                quest: quest_id.clone(),
                user: user_id.clone(),
                exit_status: result.status,
            });
        }

        let output = String::from_utf8(result.stdout).map_err(|_| Error::InvalidResponse)?;
        let (input, answer) = output.split_once('\0').ok_or(Error::InvalidResponse)?;
        return Ok(Some(QuestContext::new(input.to_owned(), answer.to_owned())));
    }
}

pub struct InMemoryQuestContextCache {
    contexts: RwLock<HashMap<ContextKey, Option<QuestContext>>>,
    backend: Arc<dyn QuestContextProvider>,
}

impl InMemoryQuestContextCache {
    pub fn new(backend: Arc<dyn QuestContextProvider>) -> Self {
        Self {
            contexts: RwLock::new(HashMap::new()),
            backend,
        }
    }

    async fn fetch_and_cache(&self, key: ContextKey) -> Result<Option<QuestContext>, Error> {
        let context = self.backend.get_context(&key.quest, &key.user).await?;
        self.contexts.write().await.insert(key, context.clone());
        Ok(context)
    }
}

#[async_trait]
impl QuestContextProvider for InMemoryQuestContextCache {
    async fn get_context(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<QuestContext>, Error> {
        let key = ContextKey::new(*quest_id, *user_id);
        if let Some(context) = self.contexts.read().await.get(&key) {
            return Ok(context.clone());
        }

        self.fetch_and_cache(key).await
    }

    async fn get_input(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        let key = ContextKey::new(*quest_id, *user_id);
        if let Some(context) = self.contexts.read().await.get(&key) {
            return Ok(context.as_ref().map(|context| context.input.clone()));
        }

        Ok(self
            .fetch_and_cache(key)
            .await?
            .map(|context| context.input))
    }

    async fn get_answer(
        &self,
        quest_id: &QuestId,
        user_id: &UserId,
    ) -> Result<Option<String>, Error> {
        let key = ContextKey::new(*quest_id, *user_id);
        if let Some(context) = self.contexts.read().await.get(&key) {
            return Ok(context.as_ref().map(|context| context.answer.clone()));
        }

        Ok(self
            .fetch_and_cache(key)
            .await?
            .map(|context| context.answer))
    }
}
