use crate::data::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectInner {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Project {
    pub id: Uuid,
    #[serde(flatten)]
    pub inner: ProjectInner,
}

impl Project {
    pub fn identifier_matches<Q: Borrow<Identifier>>(&self, identifier: Q) -> bool {
        match identifier.borrow() {
            Identifier::Uuid(id) => &self.id == id,
            Identifier::ByName(name) => &self.inner.name == name,
        }
    }
}
