use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HalLink {
    pub href: String,
}

impl HalLink {
    pub fn new(href: impl Into<String>) -> Self {
        Self { href: href.into() }
    }
}

/// A single HAL+JSON resource.
///
/// `T` carries the domain fields; `_links` and `_embedded` are added around it.
#[derive(Debug, Clone, Serialize)]
pub struct HalResource<T: Serialize> {
    pub _links: HashMap<String, HalLink>,
    #[serde(flatten)]
    pub properties: T,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub _embedded: HashMap<String, serde_json::Value>,
}

impl<T: Serialize> HalResource<T> {
    pub fn new(self_href: impl Into<String>, properties: T) -> Self {
        let mut links = HashMap::new();
        links.insert("self".to_string(), HalLink::new(self_href));
        Self {
            _links: links,
            properties,
            _embedded: HashMap::new(),
        }
    }
}

/// A HAL+JSON collection wrapping multiple items under `_embedded`.
#[derive(Debug, Clone, Serialize)]
pub struct HalCollection<T: Serialize> {
    pub _links: HashMap<String, HalLink>,
    pub total: usize,
    pub _embedded: HashMap<String, Vec<HalResource<T>>>,
}

impl<T: Serialize> HalCollection<T> {
    pub fn new(
        self_href: impl Into<String>,
        embed_key: impl Into<String>,
        items: Vec<HalResource<T>>,
    ) -> Self {
        let total = items.len();
        let mut links = HashMap::new();
        links.insert("self".to_string(), HalLink::new(self_href));
        let mut embedded = HashMap::new();
        embedded.insert(embed_key.into(), items);
        Self {
            _links: links,
            total,
            _embedded: embedded,
        }
    }
}
