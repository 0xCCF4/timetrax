use crate::az_hash::AZHash;
use crate::data::activity::Activity;
use crate::data::blocker::Blocker;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;
use time::Duration;

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
    /// target work quota
    #[serde(with = "pretty_duration_serde")]
    pub work_quota: Duration,
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
            work_quota: time::Duration::ZERO,
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
    pub fn new(date: time::Date, work_quota: time::Duration) -> Self {
        Self {
            date,
            inner: DayInner {
                work_quota,
                activities: Vec::new(),
                blockers: Vec::new(),
            },
        }
    }
}

/// pretty print a duration
pub mod pretty_duration_serde {
    use serde::Deserialize;
    use std::sync::LazyLock;
    use time::Duration;

    static REGEX_PRETTY_DURATION: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^(?P<hours>\d{1,}?)h(ours?)?\s+(?P<minutes>\d{1,2}?)m(in(utes?)?)?\s+(?P<seconds>\d{1,2}?)s(sec(onds?)?)?$").unwrap()
    });

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let secs = duration.whole_seconds() % 60;
        let mins = duration.whole_minutes() % 60;
        let hours = duration.whole_hours();

        if secs < 0 || mins < 0 || hours < 0 {
            return Err(serde::ser::Error::custom(
                "Negative durations are not supported",
            ));
        }

        let s = format!("{:02}h {:02}m {:02}s", hours, mins, secs);

        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let captures = REGEX_PRETTY_DURATION
            .captures(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid duration format: {}", s)))?;

        let hours = match captures.name("hours").map(|m| m.as_str()) {
            None => 0,
            Some(h) => h.parse::<u32>().map_err(|e| {
                serde::de::Error::custom(format!("Invalid hours in duration: {}: {}", h, e))
            })?,
        };

        let minutes = match captures.name("minutes").map(|m| m.as_str()) {
            None => 0,
            Some(m) => m.parse::<u32>().map_err(|e| {
                serde::de::Error::custom(format!("Invalid minutes in duration: {}: {}", m, e))
            })?,
        };

        let seconds = match captures.name("seconds").map(|m| m.as_str()) {
            None => 0,
            Some(s) => s.parse::<u32>().map_err(|e| {
                serde::de::Error::custom(format!("Invalid seconds in duration: {}: {}", s, e))
            })?,
        };

        Ok(Duration::hours(hours as i64)
            + Duration::minutes(minutes as i64)
            + Duration::seconds(seconds as i64))
    }
}
