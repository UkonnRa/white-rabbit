use crate::journal::JournalInput;
use crate::journal::command::JournalCommandCreate;
use crate::journal::repository::JournalRepository;
use crate::journal::specification::JournalSpecification;
use crate::{error::Result, journal::Journal};
use shared::ErrorKind;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct JournalService<R: JournalRepository> {
    pub repository: Arc<Mutex<R>>,
}

impl<R: JournalRepository> JournalService<R> {
    /// Create a new [`Journal`].
    ///
    /// # Prerequisites
    ///
    /// - The `name` must be unique across all existing journals.
    ///
    /// # Returns
    ///
    /// The persisted [`Journal`] as returned by the repository (with a
    /// DB-assigned ID; any client-provided ID in the command is ignored).
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — a journal with the same name already exists.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — the name is empty or blank.
    pub async fn create(&self, command: JournalCommandCreate) -> Result<Journal> {
        let mut repo = self.repository.lock().await;

        let spec = JournalSpecification::name(&command.name);
        if repo
            .find_one(&spec)
            .await
            .map_err(|e| e.convert())?
            .is_some()
        {
            return Err(ErrorKind::duplicate_values(&command.name)
                .with_resource_type(Journal::TYPE)
                .with_field("name")
                .convert());
        }

        let input = JournalInput {
            name: command.name,
            description: command.description,
            tags: command.tags,
            ..Default::default()
        };
        let journal: Journal = input.try_into()?;
        let journal = repo.save(&journal).await.map_err(|e| e.convert())?;
        Ok(journal)
    }
}
