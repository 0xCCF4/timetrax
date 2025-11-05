use crate::az_hash::AZHash;
use crate::data::identifier::Identifier;
use crate::data::interval::Interval;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Blocker
/// Add a constant time amount to the daily amount
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blocker {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// Optional name of the activity
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    /// Activity class, work, break, ...
    pub class: Identifier,
    /// Blocked time spend on the activity
    pub time: Interval,
    /// Projects worked on
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub projects: Vec<Identifier>,
}

impl AZHash for Blocker {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}
