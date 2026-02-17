use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::EntityId;

pub trait Entity: Send + Sync + Debug + Eq + Serialize + DeserializeOwned + Clone {
    type Id: EntityId;

    const ENTITY_TYPE: &'static str;

    fn id(&self) -> &Self::Id;
    fn version(&self) -> usize;
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn last_modified_at(&self) -> Option<DateTime<Utc>>;
}
