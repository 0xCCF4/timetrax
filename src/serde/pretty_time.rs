use serde::Deserialize;
use std::sync::LazyLock;
use time::format_description::BorrowedFormatItem;
use time::{Time, format_description};

static TIME_FORMAT: LazyLock<Vec<BorrowedFormatItem>> = LazyLock::new(|| {
    format_description::parse("[hour]:[minute]:[second]").expect("Failed to parse time format")
});

pub fn serialize<S>(time: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    time.format(&*TIME_FORMAT)
        .map_err(|e| serde::ser::Error::custom(format!("Failed to format time: {}", e)))
        .and_then(|s| serializer.serialize_str(&s))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Time::parse(&s, &*TIME_FORMAT)
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse time: {}", e)))
}
