use serde::{Deserialize, Serialize};

/// Specified time interval, may be open-ended
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
    pub start: time::Time,
    pub end: Option<time::Time>,
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

    /// end of interval or if open-ended, end of day
    pub fn end_time_or_end_of_day(&self) -> time::Time {
        self.end.unwrap_or(time::Time::MAX)
    }
}
