
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
