use log::error;
use serde::{Deserialize, Serialize};

/// Specified time interval, may be open-ended
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
    #[serde(with = "crate::serde::pretty_time")]
    pub start: time::Time,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        with = "crate::serde::pretty_time_option"
    )]
    pub end: Option<time::Time>,
}

impl Interval {
    /// Duration if interval ended
    #[must_use]
    pub fn duration(&self) -> Option<time::Duration> {
        self.end.map(|end_time| end_time - self.start)
    }

    /// Interval completed
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.end.is_some()
    }

    /// end of interval or if open-ended, end of day
    #[must_use]
    pub fn end_time_or_end_of_day(&self) -> time::Time {
        self.end.unwrap_or(time::Time::MAX)
    }

    /// create a new interval from now on
    #[must_use]
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

    /// create a new interval starting at the given time
    #[must_use]
    pub fn start_at(t: time::Time) -> Self {
        Self { start: t, end: None }
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

    /// complete this interval at the given time if it is open
    pub fn complete_at(&mut self, t: time::Time) {
        if self.end.is_none() {
            self.end = Some(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Duration, Time};

    fn t(h: u8, m: u8, s: u8) -> Time {
        Time::from_hms(h, m, s).unwrap()
    }

    #[test]
    fn start_at_sets_start_no_end() {
        let iv = Interval::start_at(t(9, 0, 0));
        assert_eq!(iv.start, t(9, 0, 0));
        assert!(iv.end.is_none());
    }

    #[test]
    fn is_complete_false_when_open() {
        let iv = Interval::start_at(t(9, 0, 0));
        assert!(!iv.is_complete());
    }

    #[test]
    fn is_complete_true_when_closed() {
        let iv = Interval { start: t(9, 0, 0), end: Some(t(17, 0, 0)) };
        assert!(iv.is_complete());
    }

    #[test]
    fn duration_none_for_open_interval() {
        let iv = Interval::start_at(t(9, 0, 0));
        assert!(iv.duration().is_none());
    }

    #[test]
    fn duration_some_for_closed_interval() {
        let iv = Interval { start: t(9, 0, 0), end: Some(t(17, 0, 0)) };
        assert_eq!(iv.duration(), Some(Duration::hours(8)));
    }

    #[test]
    fn duration_handles_sub_hour() {
        let iv = Interval { start: t(9, 0, 0), end: Some(t(9, 30, 0)) };
        assert_eq!(iv.duration(), Some(Duration::minutes(30)));
    }

    #[test]
    fn complete_at_sets_end_when_open() {
        let mut iv = Interval::start_at(t(9, 0, 0));
        iv.complete_at(t(17, 0, 0));
        assert_eq!(iv.end, Some(t(17, 0, 0)));
    }

    #[test]
    fn complete_at_noop_when_already_closed() {
        let mut iv = Interval { start: t(9, 0, 0), end: Some(t(12, 0, 0)) };
        iv.complete_at(t(17, 0, 0));
        // original end preserved
        assert_eq!(iv.end, Some(t(12, 0, 0)));
    }

    #[test]
    fn end_time_or_end_of_day_returns_end_when_set() {
        let iv = Interval { start: t(9, 0, 0), end: Some(t(17, 0, 0)) };
        assert_eq!(iv.end_time_or_end_of_day(), t(17, 0, 0));
    }

    #[test]
    fn end_time_or_end_of_day_returns_max_when_open() {
        let iv = Interval::start_at(t(9, 0, 0));
        assert_eq!(iv.end_time_or_end_of_day(), Time::MAX);
    }
}
