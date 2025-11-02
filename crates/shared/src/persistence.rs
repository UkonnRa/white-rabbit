use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::hash::Hash;

pub trait Persistence: Eq + Hash + Serialize + DeserializeOwned + Clone + Send + Sync {
    fn id(&self) -> &str;
    fn version(&self) -> usize;
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn created_by(&self) -> Option<String>;
    fn last_modified_at(&self) -> Option<DateTime<Utc>>;
    fn last_modified_by(&self) -> Option<String>;
}
