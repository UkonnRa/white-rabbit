use chrono::Utc;

use crate::error::Result;
use crate::journal::command::{
    JournalCommand, JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate,
};
use crate::journal::event::{JournalCreated, JournalDeleted, JournalEvent, JournalUpdated};
use crate::journal::repository::JournalRepository;
use crate::journal::specification::JournalSpecification;
use crate::journal::{Journal, JournalId, JournalInput};
use shared::{Entity, ErrorKind, HandleResult, RepositorySession, UnitOfWork, WriteService};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct JournalService<R: JournalRepository> {
    pub repository: Arc<R>,
}

impl<R: JournalRepository> JournalService<R> {
    /// Create new [`Journal`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `name` must be unique both within the batch and across
    ///   all existing journals.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names within the batch or against existing journals.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — a name is empty or blank.
    async fn do_create(
        &self,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        commands: Vec<JournalCommandCreate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
        }

        let now = Utc::now();

        let mut seen_names = HashSet::new();
        for cmd in &commands {
            if !seen_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let spec = JournalSpecification::names(seen_names.iter().copied());
        if let Some(existing) = uow
            .find_one(self.repository.as_ref(), sess, &spec)
            .await
            .map_err(|e| e.convert())?
        {
            return Err(ErrorKind::duplicate_values(&existing.name)
                .with_resource_type(Journal::ENTITY_TYPE)
                .with_field("name")
                .convert());
        }

        let journals = commands
            .into_iter()
            .map(|cmd| {
                let input = JournalInput {
                    name: cmd.name,
                    description: cmd.description,
                    tags: cmd.tags,
                    created_at: Some(now),
                    ..Default::default()
                };
                input.try_into()
            })
            .collect::<Result<Vec<Journal>>>()?;

        for journal in journals {
            uow.add_event(JournalEvent::Created(JournalCreated {
                id: journal.id.clone(),
                name: journal.name.to_string(),
                description: journal.description.clone(),
                tags: journal.tags.iter().map(|t| t.to_string()).collect(),
                created_at: journal.created_at.unwrap_or(now),
            }));
            uow.register_new(journal);
        }

        Ok(())
    }

    /// Update existing [`Journal`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `id` must refer to an existing journal.
    /// - Each `id` must appear at most once in the batch.
    /// - The resulting `name`s must be unique:
    ///   - No two commands in the batch may produce the same name.
    ///   - No resulting name may collide with an existing journal that is
    ///     **not** part of this batch.
    ///
    /// Name swaps are allowed: if Journal A has name "X" and Journal B
    /// has name "Y", a batch that renames A→"Y" and B→"X" succeeds
    /// because both are in the batch — the conflict check excludes
    /// journals that are being updated.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(NotFound)`](shared::ErrorKind::NotFound)
    ///   — a journal ID does not exist.
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names within the batch or against existing journals
    ///   not being updated.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — a name is empty or blank.
    async fn do_update(
        &self,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        commands: Vec<JournalCommandUpdate>,
    ) -> Result<()> {
        if commands.is_empty() {
            return Ok(());
        }

        let now = Utc::now();

        let batch_ids: HashSet<_> = commands.iter().map(|cmd| &cmd.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Journal::ENTITY_TYPE)
                .with_field("id")
                .convert());
        }

        let id_vec: Vec<_> = commands.iter().map(|cmd| cmd.id.clone()).collect();
        let existing = uow
            .find_all_by_ids(self.repository.as_ref(), sess, &id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("id")
                    .with_detail(format!("journal {id} not found"))
                    .convert());
            }
        }

        let mut new_names = HashSet::new();
        for cmd in &commands {
            if !new_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let spec = JournalSpecification::names(new_names.iter().copied());
        let conflicts = uow
            .find_all(self.repository.as_ref(), sess, &spec, None)
            .await
            .map_err(|e| e.convert())?;

        for (conflict_id, conflict) in &conflicts {
            if !batch_ids.contains(conflict_id) {
                return Err(ErrorKind::duplicate_values(&conflict.name)
                    .with_resource_type(Journal::ENTITY_TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        let commands_by_id: HashMap<_, _> = commands
            .into_iter()
            .map(|cmd| (cmd.id.clone(), cmd))
            .collect();

        let journals = id_vec
            .iter()
            .map(|id| {
                let journal = &existing[id];
                let cmd = &commands_by_id[id];
                let input = JournalInput {
                    id: journal.id.clone(),
                    version: journal.version,
                    created_at: journal.created_at,
                    last_modified_at: Some(now),
                    archived_at: journal.archived_at,
                    name: if cmd.name.is_empty() {
                        journal.name.to_string()
                    } else {
                        cmd.name.clone()
                    },
                    description: cmd
                        .description
                        .clone()
                        .unwrap_or_else(|| journal.description.clone()),
                    tags: cmd
                        .tags
                        .clone()
                        .unwrap_or_else(|| journal.tags.iter().map(|t| t.to_string()).collect()),
                };
                input.try_into()
            })
            .collect::<Result<Vec<Journal>>>()?;

        for journal in journals {
            uow.add_event(JournalEvent::Updated(JournalUpdated {
                id: journal.id.clone(),
                name: Some(journal.name.to_string()),
                description: Some(journal.description.clone()),
                tags: Some(journal.tags.iter().map(|t| t.to_string()).collect()),
                last_modified_at: journal.last_modified_at.unwrap_or(now),
            }));
            uow.register_dirty(journal);
        }

        Ok(())
    }

    /// Delete [`Journal`]s by their IDs.
    ///
    /// Duplicate IDs are deduplicated automatically. IDs that do not match
    /// any existing journal are silently ignored — this prevents
    /// ID-guessing attacks from inferring which IDs exist via error
    /// responses.
    async fn do_delete(uow: &mut UnitOfWork, ids: HashSet<JournalId>) -> Result<()> {
        for id in ids {
            uow.add_event(JournalEvent::Deleted(JournalDeleted { id: id.clone() }));
            uow.register_deleted::<Journal>(id);
        }
        Ok(())
    }

    /// Execute a batch of create, update, and delete operations atomically.
    ///
    /// Operations are applied in the order: **delete → create → update**.
    /// This ordering ensures:
    /// - Deleted journals free their names for reuse by create/update.
    /// - Newly created journals are visible to the update name-conflict check.
    async fn do_batch(
        &self,
        sess: &R::Session,
        uow: &mut UnitOfWork,
        command: JournalCommandBatch,
    ) -> Result<()> {
        Self::do_delete(uow, command.delete).await?;
        self.do_create(sess, uow, command.create).await?;
        self.do_update(sess, uow, command.update).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<R: JournalRepository> WriteService<JournalCommand> for JournalService<R> {
    type Entity = Journal;
    type Event = JournalEvent;
    type Session = R::Session;
    type Error = crate::error::Error;

    async fn do_handle(
        &self,
        sess: &Self::Session,
        uow: &mut UnitOfWork,
        command: JournalCommand,
    ) -> Result<()> {
        match command {
            JournalCommand::Create(cmd) => self.do_create(sess, uow, vec![cmd]).await,
            JournalCommand::Update(cmd) => self.do_update(sess, uow, vec![cmd]).await,
            JournalCommand::Delete(ids) => Self::do_delete(uow, ids).await,
            JournalCommand::Batch(cmd) => self.do_batch(sess, uow, cmd).await,
        }
    }

    async fn handle(
        &self,
        sess: &mut Self::Session,
        command: JournalCommand,
    ) -> Result<HandleResult<Journal, JournalEvent>> {
        let mut uow = UnitOfWork::new();

        sess.begin().await.map_err(|e| e.convert())?;

        if let Err(e) = self.do_handle(sess, &mut uow, command).await {
            let _ = sess.rollback().await;
            return Err(e);
        }

        let events = uow.events::<JournalEvent>();
        let mut saved_entities = Vec::new();

        if let Some(cs) = uow.take_change_set::<Journal>() {
            if !cs.deleted.is_empty() {
                let ids: Vec<_> = cs.deleted.into_iter().collect();
                if let Err(e) = self.repository.delete_all_by_ids(sess, &ids).await {
                    let _ = sess.rollback().await;
                    return Err(e.convert());
                }
            }

            let to_save: Vec<_> = cs.new.into_values().chain(cs.dirty.into_values()).collect();
            if !to_save.is_empty() {
                match self.repository.save_all(sess, &to_save).await {
                    Ok(saved) => saved_entities = saved.into_values().collect(),
                    Err(e) => {
                        let _ = sess.rollback().await;
                        return Err(e.convert());
                    }
                }
            }
        }

        sess.commit().await.map_err(|e| e.convert())?;

        Ok(HandleResult {
            entities: saved_entities,
            events,
        })
    }
}
