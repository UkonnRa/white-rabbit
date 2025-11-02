use chrono::{DateTime, Utc};
use std::fmt::Debug;
use std::hash::Hash;

pub trait Entity: Send + Sync + Debug + Hash + Eq {
    type Operator: Entity;

    fn id(&self) -> &str;
    fn version(&self) -> usize;
    fn created_at(&self) -> Option<DateTime<Utc>>;
    fn created_by(&self) -> Option<&Self::Operator>;
    fn last_modified_at(&self) -> Option<DateTime<Utc>>;
    fn last_modified_by(&self) -> Option<&Self::Operator>;
}
