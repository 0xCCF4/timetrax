use log::error;
use serde::{Deserialize, Serialize};

/// Specified time interval, may be open-ended
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
    pub start: time::Time,
    #[serde(skip_serializing_if = "Option::is_none", default)]
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

    /// create a new interval from now on
    pub fn start_now() -> Self {
        Self {
            start: time::OffsetDateTime::now_local()
                .unwrap_or_else(|e| {
                    error!("Failed to get local time: {e}. Falling back to UTC time.");
                    time::OffsetDateTime::now_utc()
                })
                .time(),
            end: None,
        }
    }

    /// complete this interval if it is open
    pub fn complete_now(&mut self) {
        if self.end.is_none() {
            self.end = Some(
                time::OffsetDateTime::now_local()
                    .unwrap_or_else(|e| {
                        error!("Failed to get local time: {e}. Falling back to UTC time.");
                        time::OffsetDateTime::now_utc()
                    })
                    .time(),
            );
        }
    }
}
