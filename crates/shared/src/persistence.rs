use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::EntityId;

pub trait Persistence: Eq + Serialize + DeserializeOwned + Clone + Send + Sync {
    type Id: EntityId;

    fn id(&self) -> &Self::Id;
    fn version(&self) -> usize;
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn last_modified_at(&self) -> Option<DateTime<Utc>>;
}
