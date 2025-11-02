use crate::data::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActivityClassInner {
    pub name: String,
    pub priority: i32,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActivityClass {
    pub id: Uuid,
    #[serde(flatten)]
    pub inner: ActivityClassInner,
}

impl ActivityClass {
    pub fn identifier_matches<Q: Borrow<Identifier>>(&self, identifier: Q) -> bool {
        match identifier.borrow() {
            Identifier::Uuid(id) => &self.id == id,
            Identifier::ByName(name) => &self.inner.name == name,
        }
    }
}
