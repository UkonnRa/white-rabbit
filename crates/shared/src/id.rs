use crate::Entity;
use std::fmt::Debug;
use std::hash::Hash;

pub trait EntityId: Clone + Debug + Eq + Hash + Send + Sync + 'static {
    type Entity: Entity;

    /// The type name for this ID (used in GlobalId conversion)
    const TYPE_NAME: &'static str;

    /// Get the underlying string value of this ID
    fn value(&self) -> &str;

    /// Create an ID from a string value
    fn from_value(value: impl Into<String>) -> Self;

    /// Generate a new random UUID-based ID
    fn generate() -> Self
    where
        Self: Sized,
    {
        Self::from_value(uuid::Uuid::now_v7().to_string())
    }
}

#[macro_export]
macro_rules! define_id {
    ($name:ident, $typ:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $crate::id::EntityId for $name {
            type Entity = $typ;

            const TYPE_NAME: &'static str = stringify!($typ);

            fn value(&self) -> &str {
                &self.0
            }

            fn from_value(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}
