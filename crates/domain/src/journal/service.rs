use crate::journal::command::{JournalCommandBatch, JournalCommandCreate, JournalCommandUpdate};
use crate::journal::repository::JournalRepository;
use crate::journal::specification::JournalSpecification;
use crate::journal::{JournalId, JournalInput};
use crate::{error::Result, journal::Journal};
use shared::ErrorKind;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct JournalService<R: JournalRepository> {
    pub repository: Arc<Mutex<R>>,
}

impl<R: JournalRepository> JournalService<R> {
    /// Create new [`Journal`]s in a batch.
    ///
    /// # Prerequisites
    ///
    /// - Each `name` must be unique both within the batch and across
    ///   all existing journals.
    ///
    /// # Returns
    ///
    /// The persisted [`Journal`] list as returned by the repository (with
    /// DB-assigned IDs; any client-provided IDs in the commands are ignored).
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::Shared(DuplicateValues)`](shared::ErrorKind::DuplicateValues)
    ///   — duplicate names within the batch or against existing journals.
    /// - [`ErrorKind::Shared(NonEmpty)`](shared::ErrorKind::NonEmpty)
    ///   — a name is empty or blank.
    pub async fn create(
        &self,
        commands: impl IntoIterator<Item = JournalCommandCreate>,
    ) -> Result<Vec<Journal>> {
        let mut repo = self.repository.lock().await;
        Self::do_create(&mut *repo, commands.into_iter().collect()).await
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
    /// This means **name swaps are allowed**: if Journal A has name "X" and
    /// Journal B has name "Y", a batch that renames A→"Y" and B→"X" succeeds
    /// because both A and B are in the batch — the conflict check excludes
    /// journals that are being updated.
    ///
    /// # Algorithm (name uniqueness)
    ///
    /// 1. Collect the set of **new names** from the batch.
    /// 2. Reject if any new name appears more than once in the batch.
    /// 3. Query existing journals whose names match any new name.
    /// 4. For each match, if its `id` is **not** in the batch → conflict.
    ///    (If it **is** in the batch, its name is about to change — no conflict.)
    ///
    /// # Returns
    ///
    /// The updated [`Journal`] list as returned by the repository.
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
    pub async fn update(
        &self,
        commands: impl IntoIterator<Item = JournalCommandUpdate>,
    ) -> Result<Vec<Journal>> {
        let mut repo = self.repository.lock().await;
        Self::do_update(&mut *repo, commands.into_iter().collect()).await
    }

    /// Delete [`Journal`]s by their IDs.
    ///
    /// Duplicate IDs are deduplicated automatically. IDs that do not match
    /// any existing journal are silently ignored — this is intentional to
    /// prevent ID-guessing attacks from inferring which IDs exist via error
    /// responses.
    ///
    /// # Returns
    ///
    /// The deleted [`Journal`] list as they were before deletion.
    /// Only journals that actually existed are included.
    pub async fn delete(
        &self,
        ids: impl IntoIterator<Item = impl Into<JournalId>>,
    ) -> Result<Vec<Journal>> {
        let mut repo = self.repository.lock().await;
        Self::do_delete(&mut *repo, ids.into_iter().map(Into::into).collect()).await
    }

    /// Execute a batch of create, update, and delete operations atomically.
    ///
    /// Operations are applied in the order: **delete → create → update**.
    /// This ordering ensures:
    /// - Deleted journals free their names for reuse by create/update.
    /// - Newly created journals are visible to the update name-conflict check.
    ///
    /// All three phases run under a single lock, so the batch is atomic
    /// with respect to concurrent callers.
    ///
    /// # Errors
    ///
    /// If any phase fails, the entire batch is aborted. Phases that already
    /// mutated the in-memory state are **not** rolled back (eventual consistency
    /// with a real database would use a transaction).
    ///
    /// # Returns
    ///
    /// The journals that remain in the database after the batch — i.e. created
    /// and updated journals, deduplicated by ID (if a journal is created then
    /// updated in the same batch, only the final state is returned).
    /// Deleted journals are not included.
    pub async fn batch(&self, command: JournalCommandBatch) -> Result<Vec<Journal>> {
        let mut repo = self.repository.lock().await;

        Self::do_delete(&mut *repo, command.delete).await?;
        let created = Self::do_create(&mut *repo, command.create).await?;
        let updated = Self::do_update(&mut *repo, command.update).await?;

        // Deduplicate by ID — updated version wins over created version
        let mut result: HashMap<_, _> = created.into_iter().map(|j| (j.id.clone(), j)).collect();
        for j in updated {
            result.insert(j.id.clone(), j);
        }

        Ok(result.into_values().collect())
    }

    // ── Internal implementations (operate on an already-locked repo) ──

    async fn do_create(repo: &mut R, commands: Vec<JournalCommandCreate>) -> Result<Vec<Journal>> {
        // 1. Reject duplicate names within the batch
        let mut seen_names = HashSet::new();
        for cmd in &commands {
            if !seen_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        // 2. Reject names that already exist in the repository
        let spec = JournalSpecification::names(seen_names.iter().copied());
        if let Some(existing) = repo.find_one(&spec).await.map_err(|e| e.convert())? {
            return Err(ErrorKind::duplicate_values(&existing.name)
                .with_resource_type(Journal::TYPE)
                .with_field("name")
                .convert());
        }

        // 3. Convert to domain entities and persist
        let journals = commands
            .into_iter()
            .map(|cmd| {
                let input = JournalInput {
                    name: cmd.name,
                    description: cmd.description,
                    tags: cmd.tags,
                    ..Default::default()
                };
                input.try_into()
            })
            .collect::<Result<Vec<Journal>>>()?;

        let saved = repo.save_all(&journals).await.map_err(|e| e.convert())?;
        Ok(saved.into_values().collect())
    }

    async fn do_update(repo: &mut R, commands: Vec<JournalCommandUpdate>) -> Result<Vec<Journal>> {
        // 1. Reject duplicate IDs within the batch
        let batch_ids: HashSet<_> = commands.iter().map(|cmd| &cmd.id).collect();
        if batch_ids.len() != commands.len() {
            return Err(ErrorKind::duplicate_values("id")
                .with_resource_type(Journal::TYPE)
                .with_field("id")
                .convert());
        }

        // 2. Fetch existing journals by ID
        let id_vec: Vec<_> = commands.iter().map(|cmd| cmd.id.clone()).collect();
        let existing = repo
            .find_all_by_ids(&id_vec)
            .await
            .map_err(|e| e.convert())?;

        for id in &id_vec {
            if !existing.contains_key(id) {
                return Err(ErrorKind::not_found()
                    .with_resource_type(Journal::TYPE)
                    .with_field("id")
                    .with_detail(format!("journal {id} not found"))
                    .convert());
            }
        }

        // 3. Reject duplicate new names within the batch
        let mut new_names = HashSet::new();
        for cmd in &commands {
            if !new_names.insert(&cmd.name) {
                return Err(ErrorKind::duplicate_values(&cmd.name)
                    .with_resource_type(Journal::TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        // 4. Check new names against existing journals NOT in this batch
        let spec = JournalSpecification::names(new_names.iter().copied());
        let conflicts = repo.find_all(&spec, None).await.map_err(|e| e.convert())?;

        for (conflict_id, conflict) in &conflicts {
            if !batch_ids.contains(conflict_id) {
                return Err(ErrorKind::duplicate_values(&conflict.name)
                    .with_resource_type(Journal::TYPE)
                    .with_field("name")
                    .convert());
            }
        }

        // 5. Apply updates and persist
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
                    last_modified_at: journal.last_modified_at,
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
            .collect::<Result<Vec<_>>>()?;

        let saved = repo.save_all(&journals).await.map_err(|e| e.convert())?;
        Ok(saved.into_values().collect())
    }

    async fn do_delete(repo: &mut R, ids: HashSet<JournalId>) -> Result<Vec<Journal>> {
        let ids: Vec<_> = ids.into_iter().collect();
        let deleted = repo
            .delete_all_by_ids(&ids)
            .await
            .map_err(|e| e.convert())?;
        Ok(deleted.into_values().collect())
    }
}
