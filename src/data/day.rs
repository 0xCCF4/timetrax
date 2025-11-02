use crate::az_hash::AZHash;
use crate::data::activity::Activity;
use crate::data::blocker::Blocker;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// data structure for a single day
pub struct Day {
    /// date of the day
    pub date: time::Date,
    /// target work quota
    pub work_quota: time::Duration,
    /// blockers
    pub blockers: Vec<Blocker>,
    /// activities
    pub activities: Vec<Activity>,
}

impl AZHash for Day {
    fn az_hash(&self) -> String {
        self.date.to_string().az_hash()
    }
}

impl Day {
    /// Create a new day
    pub fn new(date: time::Date, work_quota: time::Duration) -> Self {
        Self {
            date,
            work_quota,
            activities: Vec::new(),
            blockers: Vec::new(),
        }
    }
}
