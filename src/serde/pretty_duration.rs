
use serde::Deserialize;
use std::sync::LazyLock;
use time::Duration;

static REGEX_PRETTY_DURATION: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^(?P<hours>\d{1,}?)h(ours?)?\s+(?P<minutes>\d{1,2}?)m(in(utes?)?)?\s+(?P<seconds>\d{1,2}?)s(sec(onds?)?)?$").unwrap()
});

/// Serialize a `Duration` as `"HHh MMm SSs"`.
///
/// # Errors
/// Returns a serialization error if the duration is negative.
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

    let s = format!("{hours:02}h {mins:02}m {secs:02}s");

    serializer.serialize_str(&s)
}

/// Deserialize a `Duration` from `"HHh MMm SSs"` (with flexible unit names).
///
/// # Errors
/// Returns a deserialization error if the string format is not recognized.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let captures = REGEX_PRETTY_DURATION
        .captures(&s)
        .ok_or_else(|| serde::de::Error::custom(format!("Invalid duration format: {s}")))?;

    let hours = match captures.name("hours").map(|m| m.as_str()) {
        None => 0,
        Some(h) => h.parse::<u32>().map_err(|e| {
            serde::de::Error::custom(format!("Invalid hours in duration: {h}: {e}"))
        })?,
    };

    let minutes = match captures.name("minutes").map(|m| m.as_str()) {
        None => 0,
        Some(m) => m.parse::<u32>().map_err(|e| {
            serde::de::Error::custom(format!("Invalid minutes in duration: {m}: {e}"))
        })?,
    };

    let seconds = match captures.name("seconds").map(|m| m.as_str()) {
        None => 0,
        Some(s) => s.parse::<u32>().map_err(|e| {
            serde::de::Error::custom(format!("Invalid seconds in duration: {s}: {e}"))
        })?,
    };

    Ok(Duration::hours(i64::from(hours))
        + Duration::minutes(i64::from(minutes))
        + Duration::seconds(i64::from(seconds)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrapper {
        #[serde(with = "super")]
        duration: Duration,
    }

    fn round(d: Duration) -> Duration {
        let json = serde_json::to_string(&Wrapper { duration: d }).unwrap();
        serde_json::from_str::<Wrapper>(&json).unwrap().duration
    }

    #[test]
    fn roundtrip_zero() {
        assert_eq!(round(Duration::ZERO), Duration::ZERO);
    }

    #[test]
    fn roundtrip_hours_only() {
        let d = Duration::hours(8);
        assert_eq!(round(d), d);
    }

    #[test]
    fn roundtrip_hours_minutes_seconds() {
        let d = Duration::hours(1) + Duration::minutes(30) + Duration::seconds(45);
        assert_eq!(round(d), d);
    }

    #[test]
    fn serialize_format() {
        let w = Wrapper { duration: Duration::hours(2) + Duration::minutes(5) + Duration::seconds(3) };
        let json = serde_json::to_string(&w).unwrap();
        assert_eq!(json, r#"{"duration":"02h 05m 03s"}"#);
    }

    #[test]
    fn deserialize_relaxed_unit_names() {
        // regex allows "hours", "min", "sec" variants
        let json = r#"{"duration":"02h 05m 03s"}"#;
        let w: Wrapper = serde_json::from_str(json).unwrap();
        assert_eq!(w.duration, Duration::hours(2) + Duration::minutes(5) + Duration::seconds(3));
    }

    #[test]
    fn deserialize_invalid_format_errors() {
        let json = r#"{"duration":"not-a-duration"}"#;
        assert!(serde_json::from_str::<Wrapper>(json).is_err());
    }

    #[test]
    fn serialize_negative_errors() {
        let w = Wrapper { duration: Duration::hours(-1) };
        assert!(serde_json::to_string(&w).is_err());
    }
}
