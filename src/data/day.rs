use crate::az_hash::AZHash;
use crate::data::activity::Activity;
use crate::data::blocker::Blocker;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// data structure for a single day
pub struct Day {
    /// date of the day
    pub date: time::Date,
    /// data
    #[serde(flatten)]
    pub inner: DayInner,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DayInner {
    /// blockers
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blockers: Vec<Blocker>,
    /// activities
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub activities: Vec<Activity>,
}

impl Default for DayInner {
    fn default() -> Self {
        Self {
            activities: Vec::new(),
            blockers: Vec::new(),
        }
    }
}

impl AZHash for Day {
    fn az_hash(&self) -> String {
        self.date.to_string().az_hash()
    }
}

impl Day {
    /// Create a new day
    pub fn new(date: time::Date) -> Self {
        Self {
            date,
            inner: DayInner {
                activities: Vec::new(),
                blockers: Vec::new(),
            },
        }
    }
}
