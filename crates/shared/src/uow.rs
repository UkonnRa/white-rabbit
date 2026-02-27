use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use crate::entity::Entity;
use crate::error::Result;
use crate::id::Id;
use crate::repository::ReadRepository;
use crate::specification::{Specification, SpecificationEvaluator, SpecificationExpression};

// ── EntityChangeSet ──────────────────────────────────────────────

/// Tracks pending mutations for a single entity type.
pub struct EntityChangeSet<E: Entity> {
    pub new: HashMap<Id<E>, E>,
    pub dirty: HashMap<Id<E>, E>,
    pub deleted: HashSet<Id<E>>,
}

impl<E: Entity> Default for EntityChangeSet<E> {
    fn default() -> Self {
        Self {
            new: HashMap::new(),
            dirty: HashMap::new(),
            deleted: HashSet::new(),
        }
    }
}

impl<E: Entity> EntityChangeSet<E> {
    pub fn is_empty(&self) -> bool {
        self.new.is_empty() && self.dirty.is_empty() && self.deleted.is_empty()
    }
}

// ── Type-erased wrapper ──────────────────────────────────────────

trait AnyChangeSet: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<E: Entity + 'static> AnyChangeSet for EntityChangeSet<E> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// ── UnitOfWork ───────────────────────────────────────────────────

/// Generic, type-erased Unit of Work that tracks in-memory entity
/// mutations and buffered domain events for one command execution.
///
/// The UoW does **not** own a session or repository. Those are passed
/// to query and flush methods by the caller.
pub struct UnitOfWork {
    change_sets: HashMap<TypeId, Box<dyn AnyChangeSet>>,
    events: Vec<Box<dyn Any + Send + Sync>>,
}

impl Default for UnitOfWork {
    fn default() -> Self {
        Self::new()
    }
}

impl UnitOfWork {
    pub fn new() -> Self {
        Self {
            change_sets: HashMap::new(),
            events: Vec::new(),
        }
    }

    // ── Change-set access ────────────────────────────────────────

    fn change_set_mut<E: Entity + 'static>(&mut self) -> &mut EntityChangeSet<E> {
        self.change_sets
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(EntityChangeSet::<E>::default()))
            .as_any_mut()
            .downcast_mut::<EntityChangeSet<E>>()
            .expect("TypeId mismatch in UnitOfWork change_set")
    }

    fn change_set<E: Entity + 'static>(&self) -> Option<&EntityChangeSet<E>> {
        self.change_sets.get(&TypeId::of::<E>()).map(|cs| {
            cs.as_any()
                .downcast_ref::<EntityChangeSet<E>>()
                .expect("TypeId mismatch in UnitOfWork change_set")
        })
    }

    // ── Registration ─────────────────────────────────────────────

    /// Stage a newly created entity.
    pub fn register_new<E: Entity + 'static>(&mut self, entity: E) {
        let id = entity.id().clone();
        self.change_set_mut::<E>().new.insert(id, entity);
    }

    /// Stage a modified entity.
    pub fn register_dirty<E: Entity + 'static>(&mut self, entity: E) {
        let id = entity.id().clone();
        self.change_set_mut::<E>().dirty.insert(id, entity);
    }

    /// Stage a deletion by entity ID.
    pub fn register_deleted<E: Entity + 'static>(&mut self, id: Id<E>) {
        self.change_set_mut::<E>().deleted.insert(id);
    }

    /// Buffer a domain event.
    pub fn add_event<Ev: Send + Sync + 'static>(&mut self, event: Ev) {
        self.events.push(Box::new(event));
    }

    // ── Extraction ───────────────────────────────────────────────

    /// Remove and return the change set for entity type `E`.
    pub fn take_change_set<E: Entity + 'static>(&mut self) -> Option<EntityChangeSet<E>> {
        self.change_sets.remove(&TypeId::of::<E>()).map(|boxed| {
            let raw = Box::into_raw(boxed);
            // SAFETY: we inserted this as EntityChangeSet<E> keyed by TypeId::of::<E>().
            unsafe {
                let concrete = Box::from_raw(raw as *mut EntityChangeSet<E>);
                *concrete
            }
        })
    }

    /// Collect all staged entities (new + dirty) of a given type.
    pub fn entities<E: Entity + Clone + 'static>(&self) -> Vec<E> {
        match self.change_set::<E>() {
            Some(cs) => cs.new.values().chain(cs.dirty.values()).cloned().collect(),
            None => Vec::new(),
        }
    }

    /// Collect all buffered events of a given type.
    pub fn events<Ev: Clone + 'static>(&self) -> Vec<Ev> {
        self.events
            .iter()
            .filter_map(|e| e.downcast_ref::<Ev>().cloned())
            .collect()
    }

    // ── Overlay queries ──────────────────────────────────────────

    /// Find all entities matching `spec`, merging DB results with the
    /// in-memory overlay (tombstones removed, new/dirty merged).
    pub async fn find_all<E, S, R>(
        &self,
        repo: &R,
        sess: &R::Session,
        spec: &SpecificationExpression<S>,
        limit: Option<usize>,
    ) -> Result<HashMap<Id<E>, E>>
    where
        E: Entity + 'static,
        S: Specification + SpecificationEvaluator<E>,
        R: ReadRepository<SpecificationExpression<S>, Entity = E>,
    {
        let mut result = repo.find_all(sess, spec, limit).await?;
        if let Some(cs) = self.change_set::<E>() {
            result.retain(|id, _| !cs.deleted.contains(id));
            for (id, entity) in &cs.dirty {
                if spec.matches(entity) {
                    result.insert(id.clone(), entity.clone());
                }
            }
            for (id, entity) in &cs.new {
                if spec.matches(entity) {
                    result.insert(id.clone(), entity.clone());
                }
            }
        }
        Ok(result)
    }

    /// Find the first entity matching `spec` (overlay-aware).
    pub async fn find_one<E, S, R>(
        &self,
        repo: &R,
        sess: &R::Session,
        spec: &SpecificationExpression<S>,
    ) -> Result<Option<E>>
    where
        E: Entity + 'static,
        S: Specification + SpecificationEvaluator<E>,
        R: ReadRepository<SpecificationExpression<S>, Entity = E>,
    {
        Ok(self
            .find_all(repo, sess, spec, Some(1))
            .await?
            .into_values()
            .next())
    }

    /// Find entities by ID, merging DB results with the overlay.
    pub async fn find_all_by_ids<E, S, R>(
        &self,
        repo: &R,
        sess: &R::Session,
        ids: &[Id<E>],
    ) -> Result<HashMap<Id<E>, E>>
    where
        E: Entity + 'static,
        S: Specification,
        R: ReadRepository<SpecificationExpression<S>, Entity = E>,
    {
        let mut result = repo.find_all_by_ids(sess, ids).await?;
        if let Some(cs) = self.change_set::<E>() {
            result.retain(|id, _| !cs.deleted.contains(id));
            for id in ids {
                if let Some(entity) = cs.dirty.get(id) {
                    result.insert(id.clone(), entity.clone());
                }
                if let Some(entity) = cs.new.get(id) {
                    result.insert(id.clone(), entity.clone());
                }
            }
        }
        Ok(result)
    }
}
