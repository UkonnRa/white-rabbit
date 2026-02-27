use super::*;
use crate::EntityId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
struct DummyId(String);
impl EntityId for DummyId {
    type Entity = DummyEntity;
    const TYPE_NAME: &'static str = "DummyId";
    fn value(&self) -> &str {
        &self.0
    }
    fn from_value(value: impl Into<String>) -> Self {
        DummyId(value.into())
    }
}
impl std::fmt::Display for DummyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DummyEntity {
    id: DummyId,
    counter: i32,
}

impl Entity for DummyEntity {
    type Id = DummyId;
    const ENTITY_TYPE: &'static str = "test::DummyEntity";
    fn id(&self) -> &DummyId {
        &self.id
    }
    fn version(&self) -> usize {
        0
    }
    fn created_at(&self) -> Option<DateTime<Utc>> {
        None
    }
    fn last_modified_at(&self) -> Option<DateTime<Utc>> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum DummyEvent {
    Incremented { amount: i32 },
    Reset,
}
impl DomainEvent for DummyEvent {
    fn event_type(&self) -> &'static str {
        match self {
            DummyEvent::Incremented { .. } => "test::event::Incremented",
            DummyEvent::Reset => "test::event::Reset",
        }
    }
}

impl AggregateRoot for DummyEntity {
    type Event = DummyEvent;
    fn apply(&mut self, event: &DummyEvent) {
        match event {
            DummyEvent::Incremented { amount } => self.counter += amount,
            DummyEvent::Reset => self.counter = 0,
        }
    }
}

#[test]
fn apply_increments_counter() {
    let mut entity = DummyEntity {
        id: DummyId("1".into()),
        counter: 0,
    };
    entity.apply(&DummyEvent::Incremented { amount: 5 });
    assert_eq!(entity.counter, 5);
}

#[test]
fn apply_is_deterministic() {
    let mut a = DummyEntity {
        id: DummyId("1".into()),
        counter: 10,
    };
    let mut b = a.clone();
    let event = DummyEvent::Incremented { amount: 3 };
    a.apply(&event);
    b.apply(&event);
    assert_eq!(a, b);
}

#[test]
fn apply_multi_event_fold() {
    let mut entity = DummyEntity {
        id: DummyId("1".into()),
        counter: 0,
    };
    let events = vec![
        DummyEvent::Incremented { amount: 10 },
        DummyEvent::Incremented { amount: 5 },
        DummyEvent::Reset,
        DummyEvent::Incremented { amount: 1 },
    ];
    for event in &events {
        entity.apply(event);
    }
    assert_eq!(entity.counter, 1);
}
