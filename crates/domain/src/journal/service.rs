use crate::journal::JournalInput;
use crate::journal::command::JournalCommandCreate;
use crate::journal::repository::JournalRepository;
use crate::{error::Result, journal::Journal};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct JournalService<R: JournalRepository> {
    pub repository: Arc<Mutex<R>>,
}

impl<R: JournalRepository> JournalService<R> {
    pub async fn create(&self, command: JournalCommandCreate) -> Result<Journal> {
        let input = JournalInput {
            name: command.name,
            description: command.description,
            tags: command.tags,
            ..Default::default()
        };
        let journal: Journal = input.try_into()?;
        let journal = self
            .repository
            .lock()
            .await
            .save(&journal)
            .await
            .map_err(|e| e.convert())?;
        Ok(journal)
    }
}
