use std::collections::{BinaryHeap, LinkedList};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::az_hash::AZHash;

/// Specified time interval, may be open-ended
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
      pub  start: time::Time,
   pub     end: Option<time::Time>,

}

impl Interval {
    /// Duration if interval ended
    pub fn duration(&self) -> Option<time::Duration> {
                if let Some(end_time) = self.end {
                    Some(end_time - self.start)
                } else {
                    None
                }
    }

    /// Interval completed
    pub fn is_complete(&self) -> bool {
        self.end.is_some()
    }
}
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

/// Activity
/// Multiple activities of the same class may be worked on at the same time
/// example: 9-12 work(projectA) 10-12 work(projectB)
/// An activity of type break will interrupt running WORK activities
/// An activity of type EXCUSED will interrupt running BREAK/WORK
/// An activity of type HOLIDAY will interrupt running BREAK/WORK/EXCUSED
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Activity {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// created
    pub created_at: time::Time,
    /// modified
    pub modified_at: time::Time,
    /// Optional name of the activity
    pub name: Option<String>,
    /// Activity class, work, break, ...
    pub class: ActivityClass,
    /// Time spend on the activity
    pub time: Interval,
    /// Optional description
    pub description: Option<String>,
    /// Optional tags
    pub tags: Vec<String>,
    /// Projects worked on
    pub projects: Vec<String>,
}

impl AZHash for Activity {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}

/// Blocker
/// Add a constant time amount to the daily amount
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blocker {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// created
    pub created_at: time::Time,
    /// modified
    pub modified_at: time::Time,
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
    pub projects: Vec<String>,
}

impl AZHash for Blocker {
    fn az_hash(&self) -> String {
        self.id.az_hash()
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
    pub fn new(date: time::Date, work_quota: time::Duration) -> Self {
        Self {
            date, work_quota, activities: Vec::new(), blockers: Vec::new()
        }
    }

    pub fn calculate_activity_closure(&self) -> Vec<Activity> {
        let mut activities: Vec<Activity> = self.activities.clone();
        activities.sort_by(|a, b| b.time.start.cmp(&a.time.start));

        let mut closure: Vec<Activity> = Vec::with_capacity(activities.len() * 2 - 1);
        let mut activity_stack = BinaryHeap::with_capacity(activities.len());

        for activity in activities.into_iter() {

        }

        closure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_activity_classes() {
        assert!(ActivityClass::Work < ActivityClass::Break);
        assert!(ActivityClass::Break < ActivityClass::Excused);
        assert!(ActivityClass::Excused < ActivityClass::Holiday);
    }
}