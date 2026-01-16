use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::hash::Hash;

use crate::EntityId;

pub trait Persistence: Eq + Hash + Serialize + DeserializeOwned + Clone + Send + Sync {
    type Id: EntityId;
    type OperatorId: EntityId;

    fn id(&self) -> &Self::Id;
    fn version(&self) -> usize;
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn created_by_id(&self) -> Option<&Self::OperatorId>;
    fn last_modified_at(&self) -> Option<DateTime<Utc>>;
    fn last_modified_by_id(&self) -> Option<&Self::OperatorId>;
}
