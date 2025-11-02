use crate::az_hash::AZHash;
use crate::data::day::ActivityClass;
use crate::data::interval::Interval;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Blocker
/// Add a constant time amount to the daily amount
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blocker {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// created
    #[serde(default = "time::UtcDateTime::now")]
    pub created_at: time::UtcDateTime,
    /// modified
    #[serde(default = "time::UtcDateTime::now")]
    pub modified_at: time::UtcDateTime,
    /// Optional name of the activity
    pub name: Option<String>,
    /// Activity class, work, break, ...
    pub class: ActivityClass,
    /// Blocked time spend on the activity
    pub time: Interval,
    /// Optional description
    pub description: Option<String>,
    /// Optional tags
    pub tags: Vec<String>,
    /// Projects worked on
    pub projects: Vec<Uuid>,
}

impl AZHash for Blocker {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}
