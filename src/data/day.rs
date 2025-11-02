use crate::az_hash::AZHash;
use crate::data::activity::Activity;
use crate::data::blocker::Blocker;
use serde::{Deserialize, Serialize};

/// Activity class, defines the types of activities
/// the order defines the override order, see `Activity`
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ActivityClass {
    /// Time worked
    Work,
    /// Break time
    Break,
    /// Excused due to some reason (e.g. doctor's appointment), counted as work time
    Excused,
    /// Vacation days
    Holiday,
}

impl ActivityClass {
    pub fn lowest_priority() -> Self {
        ActivityClass::Work
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Time, UtcDateTime};

    #[test]
    fn test_priority_activity_classes() {
        assert!(ActivityClass::Work < ActivityClass::Break);
        assert!(ActivityClass::Break < ActivityClass::Excused);
        assert!(ActivityClass::Excused < ActivityClass::Holiday);
    }
}
