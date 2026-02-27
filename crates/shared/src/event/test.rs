use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum TestEvent {
    Happened { value: i32 },
}

impl DomainEvent for TestEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TestEvent::Happened { .. } => "test::event::Happened",
        }
    }
}

#[test]
fn event_type_returns_expected_string() {
    let event = TestEvent::Happened { value: 42 };
    assert_eq!(event.event_type(), "test::event::Happened");
}

#[test]
fn event_serde_roundtrip() {
    let event = TestEvent::Happened { value: 42 };
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: TestEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deserialized);
}
